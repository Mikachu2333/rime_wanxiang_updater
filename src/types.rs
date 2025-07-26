use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    // 字典更新配置
    pub dict_enabled: bool,
    pub dict_repo: String,

    // 方案更新配置
    pub scheme_enabled: bool,
    pub scheme_repo: String,

    // 模型更新配置
    pub model_enabled: bool,
    pub model_repo: String,

    // 程序自更新配置
    pub self_update_enabled: bool,
    pub self_repo: String,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            dict_enabled: true,
            dict_repo: "amzxyz/rime_wanxiang".to_string(),
            scheme_enabled: true,
            scheme_repo: "amzxyz/rime_wanxiang".to_string(),
            model_enabled: true,
            model_repo: "amzxyz/rime_wanxiang".to_string(),
            self_update_enabled: true,
            self_repo: "Mikachu2333/rime_wanxiang_updater".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub tag: String,
    pub url: String,
    pub file_name: String,
    pub file_size: u64,
    pub update_time: String,
    pub description: String,
    pub sha256: String,
}

#[derive(Debug)]
pub struct UserPath {
    pub weasel: PathBuf,
    pub user: PathBuf,
    pub config: PathBuf,
}

impl Default for UserPath {
    fn default() -> Self {
        UserPath {
            weasel: PathBuf::new(),
            user: PathBuf::new(),
            config: PathBuf::new(),
        }
    }
}
