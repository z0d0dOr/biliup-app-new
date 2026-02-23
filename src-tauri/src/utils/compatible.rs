use crate::{
    models::{ConfigRoot, Subtitle, TemplateConfig, UserConfig, UserInfo, VideoInfo},
    services::validate_cookie_in_old_config,
    utils::{get_config_json_path, get_config_yaml_path, get_old_cookie_file_path},
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use tracing::{info, warn};

/// ɰ汾config.yamlĽṹ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyUserAccount {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyUser {
    pub account: LegacyUserAccount,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyVideoInfo {
    pub title: String,
    pub filename: String,
    pub desc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyStreamerConfig {
    pub copyright: u8,
    pub source: String,
    pub tid: u32,
    pub cover: String,
    pub title: String,
    pub desc_format_id: u32,
    pub desc: String,
    pub desc_v2: Option<String>,
    pub dynamic: String,
    pub subtitle: Subtitle,
    pub tag: String,
    pub videos: Vec<LegacyVideoInfo>,
    pub dtime: Option<u32>,
    pub open_subtitle: bool,
    pub interactive: u8,
    pub mission_id: Option<u32>,
    pub dolby: u8,
    pub lossless_music: u8,
    pub no_reprint: u8,
    pub open_elec: u8,
    pub aid: Option<u64>,
    pub up_selection_reply: bool,
    pub up_close_reply: bool,
    pub up_close_danmu: bool,
    #[serde(rename = "atomicInt")]
    pub atomic_int: u32,
    pub changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyConfig {
    pub user: LegacyUser,
    pub line: Option<String>,
    pub limit: u32,
    pub streamers: HashMap<String, LegacyStreamerConfig>,
}

/// ת
pub struct CompatibilityConverter;

impl CompatibilityConverter {
    /// ǷҪģʽ
    /// 如果config.jsonڵconfig.yamlڣҪģʽ
    pub fn should_convert_old_config() -> Result<bool> {
        let json_path = get_config_json_path()?;
        let yaml_path = get_config_yaml_path()?;

        let json_exists = json_path.exists();
        let yaml_exists = yaml_path.exists();

        // ֻеJSONڵYAMLʱŽģʽ
        Ok(!json_exists && yaml_exists)
    }

    /// ʱļԼת
    /// ҪģʽԶתYAML到JSON
    pub async fn startup_with_compatibility() -> Result<bool> {
        if Self::should_convert_old_config()? {
            info!("⵽ɰļʼת...");

            let yaml_path = get_config_yaml_path()?;
            let json_path = get_config_json_path()?;

            // ȡYAMLļ
            let yaml_content = fs::read_to_string(&yaml_path)?;

            // תΪJSONʽ
            let json_content = Self::convert_yaml_to_json(&yaml_content).await?;

            // дJSONļ
            fs::write(&json_path, &json_content)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Ӿɰconfig.yamlת°config.jsonʽ
    pub async fn convert_yaml_to_json(yaml_content: &str) -> Result<String> {
        // ɰYAML
        let legacy_config: LegacyConfig = serde_yaml::from_str(yaml_content)?;

        // תΪ°ʽ
        let user_config = Self::convert_legacy_to_user_config(legacy_config).await?;

        // лΪJSON
        let json_content = serde_json::to_string_pretty(&user_config)?;
        Ok(json_content)
    }

    /// ɰתΪUserConfigʽ
    async fn convert_legacy_to_user_config(legacy: LegacyConfig) -> Result<ConfigRoot> {
        let mut template = HashMap::new();

        // תÿstreamerΪtemplate
        for (streamer_name, streamer_config) in legacy.streamers {
            let template_config = TemplateConfig {
                copyright: streamer_config.copyright,
                source: streamer_config.source,
                tid: streamer_config.tid,
                cover: streamer_config.cover,
                title: streamer_config.title,
                title_prefix: String::new(),
                desc: streamer_config.desc,
                desc_v2: streamer_config.desc_v2,
                dynamic: streamer_config.dynamic,
                subtitle: streamer_config.subtitle,
                tag: streamer_config.tag,
                videos: {
                    let mut vids = Vec::new();
                    for v in streamer_config.videos {
                        vids.push(VideoInfo {
                            title: v.title,
                            id: v.filename.clone(), // õid和filenameͬ
                            cid: 0,
                            filename: v.filename,
                            desc: v.desc,
                            path: String::new(), // ɰûpathֶΣ
                            finished_at: 0,      // ɰûfinished_atֶΣĬ0
                            encoding_status: 0,  // ɰûencoding_statusֶΣĬ0
                            status_desc: String::new(), // ɰûstatus_descֶΣ
                            group_key: String::new(), // ɰûзֶ
                            group_role: String::new(), // ɰûзֶ
                        });
                    }
                    vids
                },
                dtime: streamer_config.dtime,
                open_subtitle: streamer_config.open_subtitle,
                interactive: streamer_config.interactive,
                mission_id: streamer_config.mission_id,
                topic_id: None,   // ɰûtopic_id
                season_id: None,  // ɰûseason_id
                section_id: None, // ɰûsection_id
                is_only_self: 0,  // ɰûis_only_self
                dolby: streamer_config.dolby,
                lossless_music: streamer_config.lossless_music,
                no_reprint: streamer_config.no_reprint,
                open_elec: streamer_config.open_elec,
                aid: streamer_config.aid,
                up_selection_reply: if streamer_config.up_selection_reply {
                    1
                } else {
                    0
                },
                up_close_reply: if streamer_config.up_close_reply { 1 } else { 0 },
                up_close_danmu: if streamer_config.up_close_danmu { 1 } else { 0 },
                atomic_int: streamer_config.atomic_int,
                watermark: 0, // ĬϹر
            };

            template.insert(streamer_name, template_config);
        }

        // ȡcookie
        let cookie_path = get_old_cookie_file_path()?;
        let (bilibili, user) = validate_cookie_in_old_config(&cookie_path)
            .await
            .map_err(|e| {
                warn!("Cookie֤ʧ: {}", e);
                e
            })?;

        let user_config = UserConfig {
            user: UserInfo {
                uid: user.uid,
                name: user.username,
                cookie: bilibili.login_info,
            },
            proxy: None, // ɰûд
            line: legacy.line,
            limit: legacy.limit,
            watermark: 0,
            auto_edit: 0,
            templates: template,
        };

        let mut config_root = ConfigRoot::default();
        config_root.add_user_config(user_config);

        Ok(config_root)
    }
}
