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
#[derive(Debug, Clone)]
pub struct UpdateConfig {
    // [general] 节
    pub mirror: String,

    // [repositories] 节
    pub scheme_repo: String,
    pub scheme_branch: String,
    pub dict_repo: String,
    pub dict_branch: String,
    pub model_repo: String,
    pub model_branch: String,
    pub self_repo: String,
    pub self_branch: String,

    // [files] 节
    pub scheme_name: String,
    pub scheme_tag: String,
    pub dict_name: String,
    pub dict_tag: String,
    pub model_name: String,
    pub model_tag: String,
    pub model_file_name: String,

    // [options] 节
    pub check_interval_hours: u32,
    pub auto_update: bool,
    pub backup_before_update: bool,
    pub github_cookies: String,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            // [general] 节
            mirror: "".to_string(),

            // [repositories] 节
            scheme_repo: "amzxyz/rime_wanxiang".to_string(),
            scheme_branch: "main".to_string(),
            dict_repo: "amzxyz/rime_wanxiang".to_string(),
            dict_branch: "main".to_string(),
            model_repo: "amzxyz/rime_wanxiang".to_string(),
            model_branch: "main".to_string(),
            self_repo: "Mikachu2333/rime_wanxiang_updater".to_string(),
            self_branch: "main".to_string(),

            // [files] 节
            scheme_name: "base".to_string(),
            scheme_tag: "rime-wanxiang-base.zip".to_string(),
            dict_name: "9-base-dicts.zip".to_string(),
            dict_tag: "dict-nightly".to_string(),
            model_name: "RIME-LMDG".to_string(),
            model_tag: "LTS".to_string(),
            model_file_name: "wanxiang-lts-zh-hans.gram".to_string(),

            // [options] 节
            check_interval_hours: 24,
            auto_update: false,
            backup_before_update: true,
            github_cookies: "".to_string(),
        }
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
