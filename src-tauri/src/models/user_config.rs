use anyhow::Result;
use biliup::credential::LoginInfo;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub uid: u64,
    pub name: String,
    pub cookie: LoginInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Subtitle {
    #[serde(default)]
    pub open: u8,
    #[serde(default)]
    pub lan: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VideoInfo {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub cid: u64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub filename: String,
    #[serde(default)]
    pub desc: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub finished_at: u64,
    #[serde(default)]
    pub encoding_status: i64,
    #[serde(default)]
    pub status_desc: String,
    #[serde(default)]
    pub group_key: String,
    #[serde(default)]
    pub group_role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    #[serde(default)]
    pub copyright: u8, // 1: 自制, 2: ת
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub tid: u32, // 分区ID
    #[serde(default)]
    pub cover: String, // 封面URL
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub title_prefix: String,
    #[serde(default)]
    pub desc: String,
    #[serde(default)]
    pub desc_v2: Option<String>,
    #[serde(default)]
    pub dynamic: String, // ˿̬
    #[serde(default)]
    pub subtitle: Subtitle,
    #[serde(default)]
    pub tag: String, // ŷָıǩ
    #[serde(default)]
    pub videos: Vec<VideoInfo>,
    #[serde(default)]
    pub dtime: Option<u32>, // ʱʱ
    #[serde(default)]
    pub open_subtitle: bool,
    #[serde(default)]
    pub interactive: u8,
    #[serde(default)]
    pub mission_id: Option<u32>,
    #[serde(default)]
    pub topic_id: Option<u32>,
    #[serde(default)]
    pub season_id: Option<u64>,
    #[serde(default)]
    pub section_id: Option<u64>,
    #[serde(default)]
    pub dolby: u8,
    #[serde(default)]
    pub lossless_music: u8, // Hi-Res无损音质
    #[serde(default)]
    pub no_reprint: u8,
    #[serde(default)]
    pub open_elec: u8,
    #[serde(default)]
    pub aid: Option<u64>,
    #[serde(default)]
    pub up_selection_reply: u8,
    #[serde(default)]
    pub up_close_reply: u8,
    #[serde(default)]
    pub up_close_danmu: u8,
    #[serde(default)]
    pub atomic_int: u32,
    #[serde(default)]
    pub is_only_self: u8,
    #[serde(default)]
    pub watermark: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub user: UserInfo,
    #[serde(default)]
    pub line: Option<String>,
    #[serde(default)]
    pub proxy: Option<String>,
    #[serde(default)]
    pub limit: u32,
    #[serde(default)]
    pub watermark: u8,
    #[serde(default)]
    pub auto_edit: u8,
    #[serde(default)]
    pub templates: HashMap<String, TemplateConfig>, // ƥconfig.jsonе"templates"ֶ
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigRoot {
    #[serde(default)]
    pub max_curr: u32,
    #[serde(default)]
    pub auto_upload: bool,
    #[serde(default)]
    pub auto_start: bool,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default)]
    pub translation_api_url: String,
    #[serde(default)]
    pub translation_api_key: String,
    #[serde(default)]
    pub translation_model: String,
    #[serde(default = "default_translation_prompt")]
    pub translation_prompt: String,
    #[serde(default)]
    pub translation_auto: bool,
    #[serde(default)]
    pub config: HashMap<u64, UserConfig>,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_translation_prompt() -> String {
    "You are a professional video title translator. Translate the input title into concise, natural Simplified Chinese. Keep product names, proper nouns, and abbreviations accurate. Output only the translated title without explanation or quotes.".to_string()
}

impl ConfigRoot {
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let json_content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&json_content)?)
    }

    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        // ȷĿ¼
        if let Some(parent) = path.parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent)?;
        }

        let json_content = serde_json::to_string_pretty(self)?;
        fs::write(path, json_content)?;
        Ok(())
    }

    pub fn new_user_config(
        &mut self,
        uid: u64,
        username: String,
        cookie: LoginInfo,
        proxy: Option<String>,
    ) -> &Self {
        let user_info = UserInfo {
            uid,
            name: username,
            cookie,
        };
        let user_config = UserConfig {
            user: user_info,
            line: None,
            proxy,
            limit: 0,
            watermark: 0,
            auto_edit: 0,
            templates: HashMap::new(),
        };
        self.config.insert(uid, user_config);

        self
    }

    pub fn add_user_config(&mut self, config: UserConfig) -> &Self {
        let uid = config.user.uid;
        self.config.insert(uid, config);

        self
    }

    pub fn remove_user_config(&mut self, uid: u64) -> Result<&Self> {
        if self.config.remove(&uid).is_some() {
            Ok(self)
        } else {
            Err(anyhow::anyhow!("ûò"))
        }
    }

    pub fn save_user_config(
        &mut self,
        uid: u64,
        line: Option<String>,
        proxy: Option<String>,
        limit: u32,
        watermark: u8,
        auto_edit: u8,
    ) -> Result<&Self> {
        if let Some(user_config) = self.config.get_mut(&uid) {
            info!(
                "UID {} ûø: line={:?}, proxy={:?}, limit={}, watermark={}, auto_edit={}",
                uid, line, proxy, limit, watermark, auto_edit
            );
            user_config.line = line;
            user_config.proxy = proxy;
            user_config.limit = limit;
            user_config.watermark = watermark;
            user_config.auto_edit = auto_edit;
            Ok(self)
        } else {
            Err(anyhow::anyhow!("ûò"))
        }
    }

    pub fn save_global_config(
        &mut self,
        max_curr: u32,
        auto_start: bool,
        auto_upload: bool,
        log_level: String,
        translation_api_url: String,
        translation_api_key: String,
        translation_model: String,
        translation_prompt: String,
        translation_auto: bool,
    ) -> &Self {
        info!(
            "ȫ: max_curr={}, auto_start={}, auto_upload={}, log_level={}",
            max_curr, auto_start, auto_upload, log_level
        );
        self.max_curr = max_curr;
        self.auto_start = auto_start;
        self.auto_upload = auto_upload;
        self.log_level = log_level;
        self.translation_api_url = translation_api_url;
        self.translation_api_key = translation_api_key;
        self.translation_model = translation_model;
        self.translation_prompt = if translation_prompt.trim().is_empty() {
            default_translation_prompt()
        } else {
            translation_prompt
        };
        self.translation_auto = translation_auto;

        self
    }

    pub fn add_user_template(
        &mut self,
        uid: u64,
        template_name: &str,
        template: TemplateConfig,
    ) -> TemplateConfig {
        if let Some(user_config) = self.config.get_mut(&uid) {
            if let Some(old) = user_config
                .templates
                .insert(template_name.to_owned(), template.clone())
            {
                #[cfg(debug_assertions)]
                self.compare_templates(&old, &template);
            }
        }

        template
    }

    pub fn delete_user_template(
        &mut self,
        uid: u64,
        template_name: &str,
    ) -> Option<TemplateConfig> {
        if let Some(user_config) = self.config.get_mut(&uid) {
            user_config.templates.remove(template_name)
        } else {
            None
        }
    }

    pub fn default() -> Self {
        Self {
            max_curr: 1,
            auto_start: true,
            auto_upload: true,
            log_level: default_log_level(),
            translation_api_url: String::new(),
            translation_api_key: String::new(),
            translation_model: String::new(),
            translation_prompt: default_translation_prompt(),
            translation_auto: false,
            config: HashMap::new(),
        }
    }

    #[cfg(debug_assertions)]
    fn compare_templates(&self, old: &TemplateConfig, new: &TemplateConfig) {
        macro_rules! compare_field {
            ($field:ident, $old:expr, $new:expr) => {
                if $old.$field != $new.$field {
                    debug!(
                        "Field '{}' updated: {:?} -> {:?}",
                        stringify!($field),
                        $old.$field,
                        $new.$field
                    );
                }
            };
        }

        fn compare_subtitle(old: &Subtitle, new: &Subtitle) {
            compare_field!(open, old, new);
            compare_field!(lan, old, new);
        }

        fn compare_video_info_vec(old: &Vec<VideoInfo>, new: &Vec<VideoInfo>) {
            // ӡ VideoInfo
            for new_video in new {
                if !old.contains(new_video) {
                    debug!("ƵϢ: {:?}", new_video);
                }
            }
            // ӡɾ VideoInfo
            for old_video in old {
                if !new.contains(old_video) {
                    debug!("ɾƵϢ: {:?}", old_video);
                }
            }
        }

        fn compare_template_fields(old: &TemplateConfig, new: &TemplateConfig) {
            compare_field!(copyright, old, new);
            compare_field!(source, old, new);
            compare_field!(tid, old, new);
            compare_field!(cover, old, new);
            compare_field!(title, old, new);
            compare_field!(title_prefix, old, new);
            compare_field!(desc, old, new);
            compare_field!(desc_v2, old, new);
            compare_field!(dynamic, old, new);
            compare_subtitle(&old.subtitle, &new.subtitle);
            compare_field!(tag, old, new);
            compare_video_info_vec(&old.videos, &new.videos);
            compare_field!(dtime, old, new);
            compare_field!(open_subtitle, old, new);
            compare_field!(interactive, old, new);
            compare_field!(mission_id, old, new);
            compare_field!(dolby, old, new);
            compare_field!(lossless_music, old, new);
            compare_field!(no_reprint, old, new);
            compare_field!(open_elec, old, new);
            compare_field!(aid, old, new);
            compare_field!(up_selection_reply, old, new);
            compare_field!(up_close_reply, old, new);
            compare_field!(up_close_danmu, old, new);
            compare_field!(atomic_int, old, new);
        }

        compare_template_fields(old, new);
    }
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            copyright: 1, // Ĭ
            source: String::new(),
            tid: 0,
            cover: String::new(),
            title: String::new(),
            title_prefix: String::new(),
            desc: String::new(),
            desc_v2: None,
            dynamic: String::new(),
            subtitle: Subtitle::default(),
            tag: String::new(),
            videos: Vec::new(),
            dtime: None,
            open_subtitle: false,
            interactive: 0,
            mission_id: None,
            topic_id: None,
            season_id: None,
            section_id: None,
            dolby: 0,
            lossless_music: 0,
            no_reprint: 0,
            open_elec: 0,
            aid: None,
            up_selection_reply: 0,
            up_close_reply: 0,
            up_close_danmu: 0,
            atomic_int: 0,
            is_only_self: 0,
            watermark: 0,
        }
    }
}
