use serde::Deserialize;
use std::path::Path;
use std::process::Command;
use tokio::task;
use tracing::info;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CropOptions {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatermarkOptions {
    pub image_path: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoEditRequest {
    pub input_path: String,
    pub output_path: String,
    pub crop: Option<CropOptions>,
    pub watermark: Option<WatermarkOptions>,
}

#[tauri::command]
pub async fn edit_video_with_watermark(request: VideoEditRequest) -> Result<String, String> {
    if request.input_path.trim().is_empty() {
        return Err("输入视频路径不能为空".to_string());
    }
    if request.output_path.trim().is_empty() {
        return Err("输出视频路径不能为空".to_string());
    }

    if !Path::new(&request.input_path).exists() {
        return Err(format!("输入视频不存在: {}", request.input_path));
    }

    let has_crop = request
        .crop
        .as_ref()
        .is_some_and(|crop| crop.width > 0 && crop.height > 0);

    let has_watermark = request
        .watermark
        .as_ref()
        .is_some_and(|watermark| !watermark.image_path.trim().is_empty());

    if !has_crop && !has_watermark {
        return Err("请至少启用裁剪或图片水印".to_string());
    }

    if let Some(watermark) = request.watermark.as_ref() {
        if has_watermark && !Path::new(&watermark.image_path).exists() {
            return Err(format!("水印图片不存在: {}", watermark.image_path));
        }
        if watermark.width == 0 || watermark.height == 0 {
            return Err("水印尺寸必须大于 0".to_string());
        }
    }

    let mut args: Vec<String> = vec![
        "-y".to_string(),
        "-hide_banner".to_string(),
        "-i".to_string(),
        request.input_path.clone(),
    ];

    if has_watermark {
        let watermark_path = request
            .watermark
            .as_ref()
            .map(|w| w.image_path.clone())
            .ok_or_else(|| "水印路径缺失".to_string())?;
        args.push("-i".to_string());
        args.push(watermark_path);
    }

    let mut filter_parts: Vec<String> = Vec::new();
    let mut base_label = "[0:v]".to_string();

    if let Some(crop) = request
        .crop
        .as_ref()
        .filter(|c| c.width > 0 && c.height > 0)
    {
        filter_parts.push(format!(
            "[0:v]crop={}:{}:{}:{}[base]",
            crop.width, crop.height, crop.x, crop.y
        ));
        base_label = "[base]".to_string();
    }

    let output_label = if let Some(watermark) = request
        .watermark
        .as_ref()
        .filter(|w| !w.image_path.trim().is_empty())
    {
        filter_parts.push(format!(
            "[1:v]scale={}:{}[wm]",
            watermark.width, watermark.height
        ));
        filter_parts.push(format!(
            "{base_label}[wm]overlay={}:{}[vout]",
            watermark.x, watermark.y
        ));
        "[vout]".to_string()
    } else {
        base_label
    };

    if !filter_parts.is_empty() {
        args.push("-filter_complex".to_string());
        args.push(filter_parts.join(";"));
        args.push("-map".to_string());
        args.push(output_label);
    }

    args.extend_from_slice(&[
        "-map".to_string(),
        "0:a?".to_string(),
        "-c:v".to_string(),
        "libx264".to_string(),
        "-preset".to_string(),
        "veryfast".to_string(),
        "-crf".to_string(),
        "23".to_string(),
        "-c:a".to_string(),
        "copy".to_string(),
        "-movflags".to_string(),
        "+faststart".to_string(),
        request.output_path.clone(),
    ]);

    info!("执行视频编辑命令: ffmpeg {}", args.join(" "));

    let output = task::spawn_blocking(move || Command::new("ffmpeg").args(&args).output())
        .await
        .map_err(|e| format!("执行视频编辑任务失败: {e}"))?
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "未找到 ffmpeg，请先安装 ffmpeg 并确保其在系统 PATH 中".to_string()
            } else {
                format!("调用 ffmpeg 失败: {e}")
            }
        })?;

    if !output.status.success() {
        let err_text = String::from_utf8_lossy(&output.stderr).to_string();
        let concise = err_text
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .unwrap_or("视频导出失败");
        return Err(format!("视频导出失败: {concise}"));
    }

    Ok(request.output_path)
}
