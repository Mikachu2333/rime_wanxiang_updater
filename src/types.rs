use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 用户路径配置
#[derive(Debug, Clone)]
pub struct UserPath {
    pub weasel: PathBuf,
    pub user: PathBuf,
    pub config: PathBuf,
    pub curl: PathBuf,
    pub zip: PathBuf,
}

/// 更新配置
#[derive(Debug, Clone, serde::Deserialize)]
pub struct UpdateConfig {
    pub schema_repo: String,
    pub dict_repo: String,
    pub model_repo: String,
    pub self_repo: String,
    pub mirror: String,
    pub schema_name: String,
    pub dict_name: String,
    pub dict_tag: String,
    pub model_tag: String,
    pub model_file_name: String,
    pub check_interval_hours: u32,
    pub auto_update: bool,
    pub backup_before_update: bool,
    pub github_cookies: String,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        let config = Self {
            schema_repo: "amzxyz/rime_wanxiang".to_string(),
            dict_repo: "amzxyz/rime_wanxiang".to_string(),
            model_repo: "amzxyz/RIME-LMDG".to_string(),
            self_repo: "Mikachu2333/rime_wanxiang_updater".to_string(),
            mirror: "".to_string(),
            schema_name: "rime-wanxiang-base.zip".to_string(),
            dict_name: "9-base-dicts.zip".to_string(),
            dict_tag: "dict-nightly".to_string(),
            model_tag: "LTS".to_string(),
            model_file_name: "wanxiang-lts-zh-hans.gram".to_string(),
            check_interval_hours: 24,
            auto_update: false,
            backup_before_update: true,
            github_cookies: "".to_string(),
        };
        println!("默认配置 - model_repo: {}", config.model_repo);
        config
    }
}

/// 更新信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateInfo {
    pub tag: String,
    pub file_name: String,
    pub file_size: u64,
    pub url: String,
    pub sha256: Option<String>,
    pub update_time: String,
    pub description: String,
}
