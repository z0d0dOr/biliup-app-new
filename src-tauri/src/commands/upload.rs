use std::sync::Arc;

use crate::{
    AppData,
    models::{TemplateConfig, UploadTask, VideoInfo},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;
use tracing::{error, info};

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadProgress {
    pub task_id: String,
    pub progress: f64,
    pub status: String,
    pub message: String,
}

fn get_bilibili_csrf(bilibili: &biliup::bilibili::BiliBili) -> Result<String, String> {
    let csrf = bilibili
        .login_info
        .cookie_info
        .get("cookies")
        .and_then(|c| c.as_array())
        .ok_or("Cookie error")?
        .iter()
        .filter_map(|c| c.as_object())
        .find(|c| c["name"] == "bili_jct")
        .ok_or("JCT error")?
        .get("value")
        .and_then(|v| v.as_str())
        .ok_or("CSRF error")?;
    Ok(csrf.to_string())
}

async fn log_edit_by_web_http_debug(
    bilibili: &biliup::bilibili::BiliBili,
    studio: &biliup::bilibili::Studio,
) {
    let csrf = match get_bilibili_csrf(bilibili) {
        Ok(v) => v,
        Err(e) => {
            error!("edit_by_web debug: get csrf failed: {}", e);
            return;
        }
    };

    let ts = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(v) => v.as_millis(),
        Err(e) => {
            error!("edit_by_web debug: system time error: {}", e);
            return;
        }
    };

    let url = format!("https://member.bilibili.com/x/vu/web/edit?t={ts}&csrf={csrf}");
    match bilibili.client.post(&url).json(studio).send().await {
        Ok(resp) => {
            let status = resp.status();
            let content_type = resp
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();
            match resp.bytes().await {
                Ok(bytes) => {
                    let raw_body = String::from_utf8_lossy(&bytes);
                    error!(
                        "edit_by_web decode debug: status={} content_type={} body_len={}",
                        status,
                        content_type,
                        bytes.len()
                    );
                    error!("edit_by_web decode debug raw body: {}", raw_body);
                }
                Err(e) => {
                    error!(
                        "edit_by_web decode debug: status={} content_type={} read body failed: {}",
                        status, content_type, e
                    );
                }
            }
        }
        Err(e) => {
            error!("edit_by_web decode debug request failed: {}", e);
        }
    }
}

/// 创建上传任务
#[tauri::command]
pub async fn create_upload_task(
    app: AppHandle,
    uid: u64,
    template: String,
    video: VideoInfo,
) -> Result<(), String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let mut app_data = app_lock.lock().await;
    let user = app_data
        .clients
        .lock()
        .await
        .get(&uid)
        .ok_or("用户未登录或不存在")?
        .user
        .clone();
    let config_copy = Arc::clone(&app_data.config);
    let clients_copy = Arc::clone(&app_data.clients);
    let upload_service = &mut app_data.upload_service;

    upload_service
        .create_task(&user, &template, &video, config_copy, clients_copy)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// 开始上传
#[tauri::command]
pub async fn start_upload(app: AppHandle, task_id: String) -> Result<bool, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let mut app_data = app_lock.lock().await;
    let upload_service = &mut app_data.upload_service;

    upload_service
        .start_upload(&task_id)
        .await
        .map_err(|e| e.to_string())
}

/// 暂停上传
#[tauri::command]
pub async fn pause_upload(app: AppHandle, task_id: String) -> Result<bool, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let mut app_data = app_lock.lock().await;
    let upload_service = &mut app_data.upload_service;

    upload_service
        .pause_upload(&task_id)
        .await
        .map_err(|e| e.to_string())
}

/// 取消上传
#[tauri::command]
pub async fn cancel_upload(app: AppHandle, task_id: String) -> Result<bool, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let mut app_data = app_lock.lock().await;
    let upload_service = &mut app_data.upload_service;

    upload_service
        .cancel_upload(&task_id)
        .await
        .map_err(|e| e.to_string())
}

/// 获取上传队列
#[tauri::command]
pub async fn get_upload_queue(app: AppHandle) -> Result<Vec<UploadTask>, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let mut app_data = app_lock.lock().await;
    let upload_service = &mut app_data.upload_service;
    upload_service
        .get_upload_queue()
        .await
        .map_err(|e| e.to_string())
}

/// 重新上传失败的任务
#[tauri::command]
pub async fn retry_upload(app: AppHandle, task_id: String) -> Result<bool, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let mut app_data = app_lock.lock().await;
    let upload_service = &mut app_data.upload_service;

    upload_service
        .retry_upload(&task_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn submit(app: AppHandle, uid: u64, form: TemplateConfig) -> Result<Value, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let app_data = app_lock.lock().await;

    if form.aid.is_none() {
        // 将前端表单转换为B站API需要的格式
        let bilibili_form = form.into_bilibili_form();
        let studio = bilibili_form.try_into_studio().map_err(|e| e.to_string())?;

        #[cfg(debug_assertions)]
        {
            use tracing::debug;

            let json_content = serde_json::to_string_pretty(&studio).unwrap();
            debug!("转换后的B站提交表单: {uid}\n{}", json_content);
        }

        let proxy = app_data
            .config
            .lock()
            .await
            .config
            .get(&uid)
            .and_then(|c| c.proxy.clone());

        match app_data
            .clients
            .lock()
            .await
            .get(&uid)
            .ok_or("用户未登录或不存在")?
            .bilibili
            .submit_by_app(&studio, proxy.as_deref())
            .await
        {
            Ok(resp) => {
                info!("添加稿件成功：{resp}");
                Ok(resp.data.ok_or("返回值错误").map_err(|e| e.to_string())?)
            }
            Err(e) => Err(e.to_string()),
        }
    } else {
        let bilibili_form = form.into_bilibili_form();
        let studio = bilibili_form.try_into_studio().map_err(|e| e.to_string())?;
        let bilibili = app_data
            .clients
            .lock()
            .await
            .get(&uid)
            .ok_or("用户未登录或不存在")?
            .bilibili
            .clone();
        match bilibili.edit_by_web(&studio).await {
            Ok(resp) => {
                info!("编辑稿件成功：{resp}");
                Ok(resp["data"].clone())
            }
            Err(e) => {
                let err_text = e.to_string();
                if err_text.contains("error decoding response body") {
                    log_edit_by_web_http_debug(&bilibili, &studio).await;
                }
                Err(err_text)
            }
        }
    }
}
