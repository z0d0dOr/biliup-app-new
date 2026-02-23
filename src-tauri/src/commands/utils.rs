use biliup::bilibili::BiliBili;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::{fs::File, io::Read, path::Path};
use tauri::Manager;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use crate::utils::crypto::encode_base64;
use crate::utils::file_utils::{self, FileEntry};
use crate::{AppData, models::TemplateConfig};

#[tauri::command]
pub async fn get_current_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

/// 获取文件大小
#[tauri::command]
pub async fn get_file_size(file_path: String) -> Result<u64, String> {
    let path = Path::new(&file_path);
    file_utils::get_file_size(path).map_err(|e| format!("获取文件大小失败: {e}"))
}

/// 递归读取目录
#[tauri::command]
pub async fn read_dir_recursive(
    dir_path: String,
    include_subdirs: bool,
    max_depth: Option<u32>,
) -> Result<Vec<FileEntry>, String> {
    let path = Path::new(&dir_path);
    file_utils::read_dir_recursive(path, include_subdirs, max_depth)
        .map_err(|e| format!("读取目录失败: {e}"))
}

/// 上传封面并进行返回url
#[tauri::command]
pub async fn upload_cover(app: tauri::AppHandle, uid: u64, file: String) -> Result<String, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let app_data = app_lock.lock().await;

    let mut cover_file = File::open(file).map_err(|e| format!("打开文件失败: {e}"))?;
    let mut cover_buf = vec![];

    cover_file
        .read_to_end(&mut cover_buf)
        .map_err(|e| format!("读取文件失败: {e}"))?;

    match app_data
        .clients
        .lock()
        .await
        .get(&uid)
        .ok_or("用户未登录或不存在")?
        .bilibili
        .cover_up(&cover_buf)
        .await
    {
        Ok(url) => {
            info!("封面上传成功: {}", url);
            Ok(url)
        }
        Err(e) => Err(e.to_string()),
    }
}

/// 下载封面并进行base64编码
#[tauri::command]
pub async fn download_cover(
    app: tauri::AppHandle,
    uid: u64,
    url: String,
) -> Result<String, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let app_data = app_lock.lock().await;

    match app_data
        .clients
        .lock()
        .await
        .get(&uid)
        .ok_or("用户未登录或不存在")?
        .bilibili
        .client
        .get(&url)
        .send()
        .await
    {
        Ok(res) => {
            let bytes = res.bytes().await.map_err(|e| e.to_string())?;
            Ok(encode_base64(&bytes))
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn get_type_list(app: tauri::AppHandle, uid: u64) -> Result<Value, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let app_data = app_lock.lock().await;

    match app_data
        .clients
        .lock()
        .await
        .get(&uid)
        .ok_or("用户未登录或不存在")?
        .bilibili
        .archive_pre()
        .await
    {
        Ok(res) => {
            debug!("获取分区列表成功: {}", res);
            Ok(res["data"]["typelist"].clone())
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn get_topic_list(app: tauri::AppHandle, uid: u64) -> Result<Value, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let app_data = app_lock.lock().await;

    match app_data
        .clients
        .lock()
        .await
        .get(&uid)
        .ok_or("用户未登录或不存在")?
        .bilibili
        .client
        .get("https://member.bilibili.com/x/vupre/web/topic/type?pn=0&ps=999")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Value>()
        .await
    {
        Ok(res) => {
            debug!("获取话题列表成功: {}", res);
            Ok(res["data"]["topics"].clone())
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn search_topics(
    app: tauri::AppHandle,
    uid: u64,
    query: String,
) -> Result<Value, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let app_data = app_lock.lock().await;

    match app_data.clients.lock().await.get(&uid).ok_or("用户未登录或不存在")?
            .bilibili
            .client
            .get(format!("https://member.bilibili.com/x/vupre/web/topic/search?keywords={query}&page_size=50&offset=0&t={}", chrono::Utc::now().timestamp()))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Value>()
            .await
        {
            Ok(res) => {
                debug!("搜索话题成功: {}", res);
                Ok(res["data"]["result"]["topics"].clone())
            },
            Err(e) => Err(e.to_string()),
        }
}

#[tauri::command]
pub async fn get_season_list(app: tauri::AppHandle, uid: u64) -> Result<Value, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let app_data = app_lock.lock().await;

    match app_data.clients.lock().await.get(&uid).ok_or("用户未登录或不存在")?
            .bilibili
            .client
            .get(format!("https://member.bilibili.com/x2/creative/web/seasons?pn=1&ps=50&order=desc&sort=mtime&filter=1&t={}", chrono::Utc::now().timestamp()))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Value>()
            .await
        {
            Ok(res) => {
                debug!("获取合集列表成功: {}", res);
                let mut season_vec = Vec::new();

                let seasons = res["data"]["seasons"].as_array()
                    .ok_or("用户没有创建合集").unwrap_or(&season_vec).to_owned();
                for season in &seasons {
                    let season_id = season["season"]["id"].as_u64().unwrap_or(0);
                    let season_title = season["season"]["title"].as_str().unwrap_or("").to_string();
                    let mut sections_vec = Vec::new();

                    if let Some(sections) = season["sections"]["sections"].as_array() {
                        for section in sections {
                            let section_id = section["id"].as_u64().unwrap_or(0);
                            let section_title = section["title"]
                                .as_str()
                                .unwrap_or(&season_title)
                                .to_string();

                            sections_vec.push(serde_json::json!({
                                "section_id": if section_id != 0 { Some(section_id) } else { None },
                                "title": section_title,
                            }));
                        }
                    }

                    let default_section_id = sections_vec
                        .first()
                        .and_then(|item| item["section_id"].as_u64())
                        .unwrap_or(0);

                    season_vec.push(serde_json::json!({
                        "season_id": if season_id != 0 { Some(season_id) } else { None },
                        "section_id": if default_section_id != 0 { Some(default_section_id) } else { None },
                        "title": season_title,
                        "sections": sections_vec,
                    }));
                }

                Ok(serde_json::json!({
                    "seasons": season_vec,
                }))
            },
            Err(e) => Err(e.to_string()),
        }
}

fn json_u64_candidates(value: &Value, candidates: &[&str]) -> Option<u64> {
    for key in candidates {
        if let Some(v) = value.get(*key) {
            if let Some(n) = v.as_u64() {
                return Some(n);
            }
            if let Some(n) = v.as_i64() && n > 0 {
                return Some(n as u64);
            }
            if let Some(s) = v.as_str() && let Ok(n) = s.parse::<u64>() {
                return Some(n);
            }
        }
    }
    None
}

fn normalize_timestamp_seconds(ts: u64) -> Option<u64> {
    if ts == 0 {
        return None;
    }
    // 13位毫秒时间戳 -> 秒
    if ts >= 1_000_000_000_000 {
        return Some(ts / 1000);
    }
    Some(ts)
}

fn parse_timestamp_like_value(value: &Value) -> Option<u64> {
    if let Some(n) = value.as_u64() {
        return normalize_timestamp_seconds(n);
    }
    if let Some(n) = value.as_i64() {
        if n > 0 {
            return normalize_timestamp_seconds(n as u64);
        }
    }
    if let Some(s) = value.as_str() {
        let s = s.trim();
        if s.is_empty() {
            return None;
        }
        if let Ok(n) = s.parse::<u64>() {
            return normalize_timestamp_seconds(n);
        }
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
            return Some(dt.timestamp().max(0) as u64);
        }
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
            return Some(dt.and_utc().timestamp().max(0) as u64);
        }
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y/%m/%d %H:%M:%S") {
            return Some(dt.and_utc().timestamp().max(0) as u64);
        }
    }
    None
}

fn json_str_candidates(value: &Value, candidates: &[&str]) -> Option<String> {
    for key in candidates {
        if let Some(v) = value.get(*key).and_then(|v| v.as_str()) && !v.is_empty() {
            return Some(v.to_string());
        }
    }
    None
}

fn recursive_find_timestamp_by_keys(value: &Value, candidates: &[&str], depth: usize) -> Option<u64> {
    if depth == 0 {
        return None;
    }

    match value {
        Value::Object(map) => {
            for key in candidates {
                if let Some(v) = map.get(*key)
                    && let Some(ts) = parse_timestamp_like_value(v)
                {
                    return Some(ts);
                }
            }
            for (_, child) in map {
                if let Some(ts) = recursive_find_timestamp_by_keys(child, candidates, depth - 1) {
                    return Some(ts);
                }
            }
            None
        }
        Value::Array(arr) => {
            for child in arr {
                if let Some(ts) = recursive_find_timestamp_by_keys(child, candidates, depth - 1) {
                    return Some(ts);
                }
            }
            None
        }
        _ => None,
    }
}

fn recursive_find_string_by_keys(value: &Value, candidates: &[&str], depth: usize) -> Option<String> {
    if depth == 0 {
        return None;
    }

    match value {
        Value::Object(map) => {
            for key in candidates {
                if let Some(v) = map.get(*key).and_then(|v| v.as_str()) {
                    let s = v.trim();
                    if !s.is_empty() {
                        return Some(s.to_string());
                    }
                }
            }
            for (_, child) in map {
                if let Some(s) = recursive_find_string_by_keys(child, candidates, depth - 1) {
                    return Some(s);
                }
            }
            None
        }
        Value::Array(arr) => {
            for child in arr {
                if let Some(s) = recursive_find_string_by_keys(child, candidates, depth - 1) {
                    return Some(s);
                }
            }
            None
        }
        _ => None,
    }
}

fn extract_episode_scheduled_dtime(episode: &Value) -> Option<u64> {
    let direct = json_u64_candidates(
        episode,
        &[
            "dtime",
            "delay_time",
            "delay_pub_time",
            "delay_pubtime",
        ],
    );

    let nested = episode
        .get("arc")
        .and_then(|arc| {
            json_u64_candidates(
                arc,
                &[
                    "dtime",
                    "delay_time",
                    "delay_pub_time",
                    "delay_pubtime",
                ],
            )
        })
        .or_else(|| {
            episode.get("publish").and_then(|publish| {
                json_u64_candidates(
                    publish,
                    &[
                        "dtime",
                        "delay_time",
                        "delay_pub_time",
                        "delay_pubtime",
                    ],
                )
            })
        });

    direct
        .or(nested)
        .and_then(normalize_timestamp_seconds)
        .or_else(|| {
            recursive_find_timestamp_by_keys(
                episode,
                &[
                    "dtime",
                    "delay_time",
                    "delay_pub_time",
                    "delay_pubtime",
                ],
                4,
            )
        })
        .or_else(|| {
            // 兜底：有些接口使用 ptime 字段但出现在其他嵌套对象，做一次浅层递归扫描
            for obj_key in ["arc", "archive", "publish", "meta"] {
                if let Some(obj) = episode.get(obj_key) {
                    if let Some(ts) = json_u64_candidates(
                        obj,
                        &["dtime", "delay_time", "delay_pub_time", "delay_pubtime"],
                    ) {
                        return normalize_timestamp_seconds(ts);
                    }
                }
            }
            None
        })
}

fn extract_episode_published_time(episode: &Value) -> Option<u64> {
    let direct = json_u64_candidates(
        episode,
        &["publish_time", "pub_time", "pubtime", "pubdate"],
    );

    let nested = episode
        .get("arc")
        .and_then(|arc| {
            json_u64_candidates(arc, &["publish_time", "pub_time", "pubtime", "pubdate"])
        })
        .or_else(|| {
            episode.get("publish").and_then(|publish| {
                json_u64_candidates(
                    publish,
                    &["publish_time", "pub_time", "pubtime", "pubdate"],
                )
            })
        });

    direct
        .or(nested)
        .and_then(normalize_timestamp_seconds)
        .or_else(|| {
            recursive_find_timestamp_by_keys(
                episode,
                &["publish_time", "pub_time", "pubtime", "pubdate"],
                4,
            )
        })
}

async fn fetch_archive_view_times_by_aid(
    client: &reqwest::Client,
    access_key: &str,
    aid: u64,
) -> Option<(Option<u64>, Option<u64>)> {
    if aid == 0 || access_key.is_empty() {
        return None;
    }

    let url = format!(
        "https://member.bilibili.com/x/client/archive/view?access_key={}&aid={}",
        access_key, aid
    );

    let res = client.get(url).send().await.ok()?.json::<Value>().await.ok()?;
    let data = res.get("data").unwrap_or(&res);
    let archive = data.get("archive").unwrap_or(data);
    Some((
        extract_episode_scheduled_dtime(archive).or_else(|| extract_episode_scheduled_dtime(data)),
        extract_episode_published_time(archive).or_else(|| extract_episode_published_time(data)),
    ))
}

async fn fetch_vupre_archive_view_times_by_aid(
    client: &reqwest::Client,
    aid: u64,
) -> Option<(Option<u64>, Option<u64>)> {
    if aid == 0 {
        return None;
    }

    let url = format!(
        "https://member.bilibili.com/x/vupre/web/archive/view?aid={}&t={}",
        aid,
        chrono::Utc::now().timestamp()
    );

    let res = client.get(url).send().await.ok()?.json::<Value>().await.ok()?;
    let data = res.get("data").unwrap_or(&res);
    let archive = data.get("archive").unwrap_or(data);

    let scheduled = extract_episode_scheduled_dtime(archive)
        .or_else(|| extract_episode_scheduled_dtime(data))
        .or_else(|| {
            recursive_find_timestamp_by_keys(
                data,
                &["dtime", "delay_time", "delay_pub_time", "delay_pubtime"],
                5,
            )
        });
    let published = extract_episode_published_time(archive)
        .or_else(|| extract_episode_published_time(data))
        .or_else(|| {
            recursive_find_timestamp_by_keys(
                data,
                &["publish_time", "pub_time", "pubtime", "pubdate"],
                5,
            )
        });

    Some((scheduled, published))
}

fn extract_web_archives_items(page_res: &Value) -> Vec<Value> {
    let data = page_res.get("data").unwrap_or(page_res);
    for key in ["arc_audits", "archives", "list", "items"] {
        if let Some(arr) = data.get(key).and_then(|v| v.as_array()) {
            return arr.clone();
        }
    }
    if let Some(obj) = data.as_object() {
        for (_, v) in obj {
            if let Some(arr) = v.as_array() {
                if arr.first().is_some_and(|item| item.is_object()) {
                    return arr.clone();
                }
            }
        }
    }
    Vec::new()
}

fn extract_season_title_from_archive_item(item: &Value) -> String {
    json_str_candidates(
        item,
        &[
            "season_title",
            "season_name",
            "seasonTitle",
            "collection_title",
            "series_title",
            "channel_name",
        ],
    )
    .or_else(|| {
        item.get("season")
            .and_then(|s| json_str_candidates(s, &["title", "name"]))
    })
    .or_else(|| {
        item.get("season_info")
            .and_then(|s| json_str_candidates(s, &["title", "name"]))
    })
    .or_else(|| {
        item.get("Archive")
            .and_then(|a| json_str_candidates(a, &["season_title", "season_name"]))
    })
    .or_else(|| {
        item.get("archive")
            .and_then(|a| json_str_candidates(a, &["season_title", "season_name"]))
    })
    .or_else(|| {
        recursive_find_string_by_keys(
            item,
            &[
                "season_title",
                "season_name",
                "seasonTitle",
                "collection_title",
                "series_title",
                "channel_name",
            ],
            4,
        )
    })
    .unwrap_or_default()
}

fn extract_season_id_from_archive_item(item: &Value) -> Option<u64> {
    json_u64_candidates(item, &["season_id", "seasonId"])
        .or_else(|| {
            item.get("season")
                .and_then(|s| json_u64_candidates(s, &["id", "season_id", "seasonId"]))
        })
        .or_else(|| {
            item.get("season_info")
                .and_then(|s| json_u64_candidates(s, &["id", "season_id", "seasonId"]))
        })
        .or_else(|| {
            item.get("archive")
                .and_then(|a| json_u64_candidates(a, &["season_id", "seasonId"]))
        })
        .or_else(|| {
            item.get("Archive")
                .and_then(|a| json_u64_candidates(a, &["season_id", "seasonId"]))
        })
}

async fn fetch_season_title_map(client: &reqwest::Client) -> HashMap<u64, String> {
    let mut result = HashMap::new();
    let mut pn = 1_u32;
    let ps = 50_u32;

    loop {
        let url = format!(
            "https://member.bilibili.com/x2/creative/web/seasons?pn={pn}&ps={ps}&order=desc&sort=mtime&filter=1&t={}",
            chrono::Utc::now().timestamp()
        );
        let Ok(resp) = client.get(url).send().await else { break };
        let Ok(res) = resp.json::<Value>().await else { break };

        let seasons = res["data"]["seasons"].as_array().cloned().unwrap_or_default();
        if seasons.is_empty() {
            break;
        }

        for season in &seasons {
            let sid = season["season"]["id"].as_u64().unwrap_or(0);
            let title = season["season"]["title"].as_str().unwrap_or("").to_string();
            if sid > 0 && !title.is_empty() {
                result.insert(sid, title);
            }
        }

        if seasons.len() < ps as usize {
            break;
        }
        pn += 1;
    }

    result
}

async fn fetch_video_season_id_by_aid(client: &reqwest::Client, aid: u64) -> Option<u64> {
    if aid == 0 {
        return None;
    }
    let url = format!(
        "https://member.bilibili.com/x2/creative/web/season/aid?id={}&t={}",
        aid,
        chrono::Utc::now().timestamp()
    );
    let res = client.get(url).send().await.ok()?.json::<Value>().await.ok()?;
    res.get("data")?.get("id")?.as_u64().filter(|n| *n > 0)
}

#[tauri::command]
pub async fn get_web_archives_pubing_for_publish_time(
    app: tauri::AppHandle,
    uid: u64,
) -> Result<Value, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let app_data = app_lock.lock().await;

    let clients = app_data.clients.lock().await;
    let client = clients
        .get(&uid)
        .ok_or("用户未登录或不存在")?
        .bilibili
        .client
        .clone();
    let access_key = clients
        .get(&uid)
        .map(|c| c.bilibili.login_info.token_info.access_token.clone())
        .unwrap_or_default();
    drop(clients);

    let season_title_map = fetch_season_title_map(&client).await;

    let mut all_items: Vec<Value> = Vec::new();
    let mut pn = 1_u32;
    let ps = 50_u32;

    loop {
        let url = format!(
            "https://member.bilibili.com/x/web/archives?status=is_pubing&pn={pn}&ps={ps}&coop=1&interactive=1"
        );
        let res = client
            .get(url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Value>()
            .await
            .map_err(|e| e.to_string())?;

        let mut page_items = extract_web_archives_items(&res);
        if page_items.is_empty() {
            break;
        }
        let page_len = page_items.len();
        all_items.append(&mut page_items);

        if page_len < ps as usize {
            break;
        }
        pn += 1;
    }

    let mut videos = Vec::new();
    let mut seen_aid = HashSet::<u64>::new();
    let now_ts = chrono::Utc::now().timestamp().max(0) as u64;

    for item in all_items {
        let aid = json_u64_candidates(&item, &["aid"])
            .or_else(|| item.get("archive").and_then(|a| json_u64_candidates(a, &["aid"])))
            .or_else(|| item.get("Archive").and_then(|a| json_u64_candidates(a, &["aid"])))
            .or_else(|| recursive_find_timestamp_by_keys(&item, &["aid"], 4))
            .unwrap_or(0);
        if aid != 0 && !seen_aid.insert(aid) {
            continue;
        }

        let bvid = json_str_candidates(&item, &["bvid"])
            .or_else(|| item.get("archive").and_then(|a| json_str_candidates(a, &["bvid"])))
            .or_else(|| item.get("Archive").and_then(|a| json_str_candidates(a, &["bvid"])))
            .or_else(|| recursive_find_string_by_keys(&item, &["bvid"], 4))
            .unwrap_or_default();
        let title = json_str_candidates(&item, &["title"])
            .or_else(|| item.get("archive").and_then(|a| json_str_candidates(a, &["title"])))
            .or_else(|| item.get("Archive").and_then(|a| json_str_candidates(a, &["title"])))
            .or_else(|| recursive_find_string_by_keys(&item, &["title", "archive_title"], 4))
            .unwrap_or_default();

        let state = item["state"]
            .as_i64()
            .or_else(|| item.get("state").and_then(|v| v.as_u64().map(|n| n as i64)))
            .unwrap_or(-1);
        let archive_state = item["archiveState"]
            .as_i64()
            .or_else(|| item.get("archiveState").and_then(|v| v.as_u64().map(|n| n as i64)))
            .or_else(|| {
                item.get("archive")
                    .and_then(|a| a.get("state"))
                    .and_then(|v| v.as_i64().or_else(|| v.as_u64().map(|n| n as i64)))
            })
            .unwrap_or(-1);

        let status_text = json_str_candidates(
            &item,
            &["status", "status_desc", "archive_state_desc", "state_desc", "state_text"],
        )
        .or_else(|| {
            item.get("archive").and_then(|a| {
                json_str_candidates(
                    a,
                    &["status", "status_desc", "archive_state_desc", "state_desc", "state_text"],
                )
            })
        })
        .unwrap_or_default();

        let mut dtime = extract_episode_scheduled_dtime(&item);
        let mut published_at = extract_episode_published_time(&item);

        if (dtime.is_none() || published_at.is_none()) && aid != 0 {
            if let Some((detail_dtime, detail_published_at)) =
                fetch_archive_view_times_by_aid(&client, &access_key, aid).await
            {
                if dtime.is_none() {
                    dtime = detail_dtime;
                }
                if published_at.is_none() {
                    published_at = detail_published_at;
                }
            }
        }

        if (dtime.is_none() || published_at.is_none()) && aid != 0 {
            if let Some((detail_dtime, detail_published_at)) =
                fetch_vupre_archive_view_times_by_aid(&client, aid).await
            {
                if dtime.is_none() {
                    dtime = detail_dtime;
                }
                if published_at.is_none() {
                    published_at = detail_published_at;
                }
            }
        }

        let has_future_schedule = dtime.is_some_and(|ts| ts > now_ts);
        let is_published = if has_future_schedule {
            false
        } else {
            published_at.is_some_and(|ts| ts > 0 && ts <= now_ts)
        };

        let mut season_title = String::new();
        if let Some(season_id) = extract_season_id_from_archive_item(&item) {
            season_title = season_title_map
                .get(&season_id)
                .cloned()
                .unwrap_or_else(|| format!("合集#{season_id}"));
        }
        if season_title.is_empty() {
            season_title = extract_season_title_from_archive_item(&item);
        }
        if season_title.is_empty() && aid != 0 {
            if let Some(season_id) = fetch_video_season_id_by_aid(&client, aid).await {
                season_title = season_title_map
                    .get(&season_id)
                    .cloned()
                    .unwrap_or_else(|| format!("合集#{season_id}"));
            }
        }
        if season_title.is_empty() {
            season_title = "未加入合集".to_string();
        }

        videos.push(json!({
            "id": json_u64_candidates(&item, &["id"]).unwrap_or(aid),
            "aid": aid,
            "bvid": bvid,
            "title": title,
            "season_title": season_title,
            "state": state,
            "archive_state": archive_state,
            "status_text": status_text,
            "published_at": published_at,
            "is_published": is_published,
            // 该接口本身已经是 status=is_pubing，这里信任接口结果，避免误判过滤
            "is_in_progress": true,
            "dtime": dtime,
            "raw": item
        }));
    }

    let season_count = videos
        .iter()
        .filter_map(|v| v.get("season_title").and_then(|x| x.as_str()))
        .map(|s| s.to_string())
        .collect::<HashSet<String>>()
        .len();

    Ok(json!({
        "videos": videos,
        "season_count": season_count,
        "source": "x/web/archives?status=is_pubing"
    }))
}

#[tauri::command]
pub async fn get_season_videos_for_publish_time(
    app: tauri::AppHandle,
    uid: u64,
    season_id: u64,
) -> Result<Value, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let app_data = app_lock.lock().await;

    let clients = app_data.clients.lock().await;
    let client = clients
        .get(&uid)
        .ok_or("用户未登录或不存在")?
        .bilibili
        .client
        .clone();
    let access_key = clients
        .get(&uid)
        .map(|c| c.bilibili.login_info.token_info.access_token.clone())
        .unwrap_or_default();
    drop(clients);

    let mut season_title = String::new();
    let mut sections: Vec<(u64, String)> = Vec::new();
    let mut pn = 1_u32;
    let ps = 50_u32;

    loop {
        let url = format!(
            "https://member.bilibili.com/x2/creative/web/seasons?pn={pn}&ps={ps}&order=desc&sort=mtime&filter=1&t={}",
            chrono::Utc::now().timestamp()
        );
        let res = client
            .get(url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Value>()
            .await
            .map_err(|e| e.to_string())?;

        let page_seasons = res["data"]["seasons"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        if page_seasons.is_empty() {
            break;
        }

        if let Some(target) = page_seasons
            .iter()
            .find(|s| s["season"]["id"].as_u64() == Some(season_id))
        {
            season_title = target["season"]["title"]
                .as_str()
                .unwrap_or("")
                .to_string();

            if let Some(section_arr) = target["sections"]["sections"].as_array() {
                for section in section_arr {
                    let sid = section["id"].as_u64().unwrap_or(0);
                    if sid == 0 {
                        continue;
                    }
                    let title = section["title"]
                        .as_str()
                        .unwrap_or(&season_title)
                        .to_string();
                    sections.push((sid, title));
                }
            }

            // 无 sections 时回退为合集本身（部分账号/接口返回结构可能不同）
            if sections.is_empty() {
                let fallback_section_id = target["season"]["id"].as_u64().unwrap_or(0);
                if fallback_section_id != 0 {
                    sections.push((fallback_section_id, season_title.clone()));
                }
            }
            break;
        }

        if page_seasons.len() < ps as usize {
            break;
        }
        pn += 1;
    }

    if sections.is_empty() {
        return Ok(json!({
            "season_id": season_id,
            "season_title": season_title,
            "videos": []
        }));
    }

    let mut videos = Vec::new();
    let mut seen_aid = HashSet::<u64>::new();

    for (section_id, section_title) in sections {
        let url = format!(
            "https://member.bilibili.com/x2/creative/web/season/section?id={}&t={}",
            section_id,
            chrono::Utc::now().timestamp()
        );

        let res = client
            .get(url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Value>()
            .await
            .map_err(|e| e.to_string())?;

        let episodes = res["data"]["episodes"].as_array().cloned().unwrap_or_default();
        for episode in episodes {
            let aid = json_u64_candidates(&episode, &["aid"])
                .or_else(|| episode.get("arc").and_then(|arc| json_u64_candidates(arc, &["aid"])))
                .unwrap_or(0);
            if aid != 0 && !seen_aid.insert(aid) {
                continue;
            }

            let bvid = json_str_candidates(&episode, &["bvid"])
                .or_else(|| {
                    episode
                        .get("arc")
                        .and_then(|arc| json_str_candidates(arc, &["bvid"]))
                })
                .unwrap_or_default();
            let title = json_str_candidates(&episode, &["title"])
                .or_else(|| {
                    episode
                        .get("arc")
                        .and_then(|arc| json_str_candidates(arc, &["title"]))
                })
                .unwrap_or_default();

            let state = episode["state"]
                .as_i64()
                .or_else(|| episode.get("state").and_then(|v| v.as_u64().map(|n| n as i64)))
                .unwrap_or(-1);
            let archive_state = episode["archiveState"]
                .as_i64()
                .or_else(|| {
                    episode
                        .get("archiveState")
                        .and_then(|v| v.as_u64().map(|n| n as i64))
                })
                .unwrap_or(-1);

            let mut dtime = extract_episode_scheduled_dtime(&episode);
            let mut published_at = extract_episode_published_time(&episode);

            // 创作中心合集列表经常不返回 dtime，按 aid 二次查询稿件详情补全
            if (dtime.is_none() || published_at.is_none()) && aid != 0 {
                if let Some((detail_dtime, detail_published_at)) =
                    fetch_archive_view_times_by_aid(&client, &access_key, aid).await
                {
                    if dtime.is_none() {
                        dtime = detail_dtime;
                    }
                    if published_at.is_none() {
                        published_at = detail_published_at;
                    }
                }
            }
            // 某些稿件在 x/client/archive/view 中也不返回 dtime，继续尝试 vupre 详情接口
            if (dtime.is_none() || published_at.is_none()) && aid != 0 {
                if let Some((detail_dtime, detail_published_at)) =
                    fetch_vupre_archive_view_times_by_aid(&client, aid).await
                {
                    if dtime.is_none() {
                        dtime = detail_dtime;
                    }
                    if published_at.is_none() {
                        published_at = detail_published_at;
                    }
                }
            }

            let now_ts = chrono::Utc::now().timestamp().max(0) as u64;
            let has_future_schedule = dtime.is_some_and(|ts| ts > now_ts);
            let is_published = if has_future_schedule {
                false
            } else {
                published_at.is_some_and(|ts| ts > 0 && ts <= now_ts)
            };

            // 创作中心返回中，0 常见于处理中/进行中状态；保守地同时接受任一字段为 0
            let is_in_progress = (state == 0 || archive_state == 0) && !is_published;

            videos.push(json!({
                "id": json_u64_candidates(&episode, &["id"]).unwrap_or(aid),
                "aid": aid,
                "bvid": bvid,
                "title": title,
                "season_id": season_id,
                "season_title": season_title,
                "section_id": section_id,
                "section_title": section_title,
                "state": state,
                "archive_state": archive_state,
                "published_at": published_at,
                "is_published": is_published,
                "is_in_progress": is_in_progress,
                "dtime": dtime,
                "raw": episode
            }));
        }
    }

    Ok(json!({
        "season_id": season_id,
        "season_title": season_title,
        "videos": videos
    }))
}

#[tauri::command]
pub async fn get_video_detail(
    app: tauri::AppHandle,
    uid: u64,
    video_id: String,
) -> Result<TemplateConfig, String> {
    let vid = biliup::uploader::bilibili::Vid::from_str(&video_id)
        .map_err(|e| format!("解析视频 ID 失败: {e}"))?;

    let app_lock = app.state::<Mutex<AppData>>();
    let app_data = app_lock.lock().await;

    let true_desc = match app_data.clients.lock().await.get(&uid) {
        Some(client) => {
            match client
                .bilibili
                .client
                .get(format!(
                    "https://api.bilibili.com/x/web-interface/view?{vid}",
                ))
                .send()
                .await
            {
                Ok(response) => match response.json::<Value>().await {
                    Ok(res) => res["data"]["desc"].as_str().unwrap_or("").to_string(),
                    Err(e) => {
                        error!("解析稿件描述响应失败: {:?}", e);
                        "".to_string()
                    }
                },
                Err(e) => {
                    error!("获取稿件描述请求失败: {:?}", e);
                    "".to_string()
                }
            }
        }
        None => {
            error!("用户未登录或不存在，无法获取稿件描述");
            "".to_string()
        }
    };

    let proxy: Option<String> = app_data
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
        .video_data(&vid, proxy.as_deref())
        .await
    {
        Ok(res) => {
            let mut template_config =
                TemplateConfig::from_bilibili_res(res).map_err(|e| e.to_string())?;
            if !true_desc.is_empty() {
                template_config.desc = true_desc;
            }
            Ok(template_config)
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn get_video_season(app: tauri::AppHandle, uid: u64, aid: u64) -> Result<u64, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let app_data = app_lock.lock().await;

    match app_data
        .clients
        .lock()
        .await
        .get(&uid)
        .ok_or("用户未登录或不存在")?
        .bilibili
        .client
        .get(format!(
            "https://member.bilibili.com/x2/creative/web/season/aid?id={}&t={}",
            aid,
            chrono::Utc::now().timestamp()
        ))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Value>()
        .await
    {
        Ok(res) => {
            debug!("获取稿件合集信息成功: {}", res);
            Ok(res["data"]["id"].as_u64().unwrap_or(0))
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn switch_season(
    app: tauri::AppHandle,
    uid: u64,
    aid: u64,
    cid: u64,
    season_id: u64,
    section_id: u64,
    title: String,
    add: bool,
) -> Result<bool, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let app_data = app_lock.lock().await;

    fn get_csrf(bilibili: &BiliBili) -> Result<String, String> {
        let csrf = bilibili
            .login_info
            .cookie_info
            .get("cookies")
            .and_then(|c| c.as_array())
            .ok_or("Cookie错误")?
            .iter()
            .filter_map(|c| c.as_object())
            .find(|c| c["name"] == "bili_jct")
            .ok_or("JCT错误")?
            .get("value")
            .and_then(|v| v.as_str())
            .ok_or("CSRF错误")?;
        Ok(csrf.to_string())
    }

    let csrf = get_csrf(
        &app_data
            .clients
            .lock()
            .await
            .get(&uid)
            .ok_or("用户未登录或不存在")?
            .bilibili,
    )
    .map_err(|e| e.to_string())?;

    if add {
        match app_data
        .clients
        .lock()
        .await
        .get(&uid)
        .ok_or("用户未登录或不存在")?
        .bilibili
        .client
        .post(format!(
            "https://member.bilibili.com/x2/creative/web/season/section/episodes/add?t={}&csrf={}",
            chrono::Utc::now().timestamp(),
            csrf
        ))
        .json(&json!({
            "episodes": [
                {
                    "title": title,
                    "aid": aid,
                    "cid": cid
                }
            ],
            "sectionId": section_id,
            "csrf": csrf
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Value>()
        .await
        {
            Ok(res) => {
                debug!("设置合集成功：{res}");
                Ok(true)
            },
            Err(e) => Err(e.to_string()),
        }
    } else {
        match app_data
            .clients
            .lock()
            .await
            .get(&uid)
            .ok_or("用户未登录或不存在")?
            .bilibili
            .client
            .post(format!(
                "https://member.bilibili.com/x2/creative/web/season/switch?t={}&csrf={}",
                chrono::Utc::now().timestamp(),
                csrf
            ))
            .json(&json!({
                "season_id": if season_id != 0 { Some(season_id) } else { None },
                "section_id": if section_id != 0 { Some(section_id) } else { None },
                "title": title,
                "aid": aid,
                "cid": cid,
                "csrf": csrf
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Value>()
            .await
        {
            Ok(res) => {
                debug!("修改合集成功：{res}");
                if res["code"].as_i64() != Some(0) {
                    return Err(serde_json::to_string(&res).unwrap_or("未知错误".to_string()));
                }
                Ok(true)
            }
            Err(e) => Err(e.to_string()),
        }
    }
}

/// 导出日志
#[tauri::command]
pub async fn export_logs() -> Result<String, String> {
    use std::fs;
    use std::io::Write;
    use zip::ZipWriter;

    let log_dir = crate::utils::get_log_path().map_err(|e| format!("获取日志目录失败: {e}"))?;

    // 创建临时zip文件
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let zip_path = log_dir.join(format!("logs_export_{timestamp}.zip"));

    let zip_file = fs::File::create(&zip_path).map_err(|e| format!("创建ZIP文件失败: {e}"))?;
    let mut zip = ZipWriter::new(zip_file);
    let options: zip::write::FileOptions<'_, ()> = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    // 添加日志文件
    if let Ok(entries) = fs::read_dir(&log_dir) {
        for entry in entries.flatten() {
            if let Some(extension) = entry.path().extension() {
                if extension == "log" {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if let Ok(content) = fs::read(entry.path()) {
                        zip.start_file(&file_name, options)
                            .map_err(|e| format!("创建ZIP条目失败: {e}"))?;
                        zip.write_all(&content)
                            .map_err(|e| format!("写入ZIP文件失败: {e}"))?;
                    }
                }
            }
        }
    }

    zip.finish().map_err(|e| format!("完成ZIP文件失败: {e}"))?;

    Ok(zip_path.to_string_lossy().to_string())
}

/// 检查更新
#[tauri::command]
pub async fn check_update() -> Result<Option<String>, String> {
    use reqwest;
    use serde_json::Value;

    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/repos/biliup/biliup-app-new/releases/latest")
        .header("User-Agent", "biliup-app")
        .send()
        .await
        .map_err(|e| format!("网络请求失败: {e}"))?;

    let release_info: Value = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {e}"))?;

    let latest_tag = release_info["tag_name"]
        .as_str()
        .ok_or("无法获取最新版本标签")?;

    info!("最新版本：{latest_tag}");
    // 解析版本号 (格式: app-va.b.c)
    let latest_version = latest_tag.strip_prefix("app-v").ok_or("版本标签格式错误")?;

    let current_version = env!("CARGO_PKG_VERSION");

    if is_newer_version(latest_version, current_version)? {
        Ok(Some(latest_tag.to_string()))
    } else {
        Ok(None)
    }
}

/// 比较版本号
fn is_newer_version(latest: &str, current: &str) -> Result<bool, String> {
    let parse_version = |v: &str| -> Result<Vec<u32>, String> {
        v.split('.')
            .map(|part| {
                part.parse::<u32>()
                    .map_err(|_| "版本号格式错误".to_string())
            })
            .collect()
    };

    let latest_parts = parse_version(latest)?;
    let current_parts = parse_version(current)?;

    for (latest_part, current_part) in latest_parts.iter().zip(current_parts.iter()) {
        if latest_part > current_part {
            return Ok(true);
        } else if latest_part < current_part {
            return Ok(false);
        }
    }

    // 如果前面的部分都相等，比较长度
    Ok(latest_parts.len() > current_parts.len())
}

/// 检查更新
#[tauri::command]
pub async fn console_log(
    _app: tauri::AppHandle,
    level: String,
    messages: Vec<String>,
) -> Result<(), String> {
    let message = messages.join(" ");
    match level.as_str() {
        "log" => info!("Webconsole: {}", message),
        "error" => error!("Webconsole: {}", message),
        "warn" => warn!("Webconsole: {}", message),
        _ => info!("Webconsole: {}", message),
    }
    Ok(())
}
