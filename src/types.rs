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
    pub scheme_repo: String,
    pub scheme_branch: String,
    pub dict_repo: String,
    pub dict_branch: String,
    pub model_repo: String,
    pub model_branch: String,
    pub self_repo: String,
    pub self_branch: String,
    #[serde(default)]
    pub mirror: String,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            scheme_repo: "amzxyz/rime_wanxiang".to_string(),
            scheme_branch: "main".to_string(),
            dict_repo: "amzxyz/rime_wanxiang".to_string(),
            dict_branch: "main".to_string(),
            model_repo: "amzxyz/rime_wanxiang".to_string(),
            model_branch: "main".to_string(),
            self_repo: "Mikachu2333/rime_wanxiang_updater".to_string(),
            self_branch: "main".to_string(),
            mirror: "".to_string(),
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
