use std::path::PathBuf;

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

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct UpdateInfo {
    pub url: String,
    pub update_time: String,
    pub tag: String,
    pub sha256: String,
    pub description: String,
    pub file_size: u64,
    pub file_name: String,
}

impl Default for UpdateInfo {
    fn default() -> Self {
        UpdateInfo {
            url: String::new(),
            update_time: String::new(),
            tag: String::new(),
            sha256: String::new(),
            description: String::new(),
            file_size: 0,
            file_name: String::new(),
        }
    }
}

#[derive(Debug)]
pub struct UpdateConfig {
    pub mirror: String,
    pub repo_url: String,
    pub owner: String,
    pub repo: String,
    pub dict_releases_tag: String,
    pub model_name: String,
    pub model_tag: String,
    pub model_file_name: String,
    pub github_cookies: String,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        UpdateConfig {
            mirror: "gh-proxy.com".to_string(),
            repo_url: "https://github.com/amzxyz/rime_wanxiang".to_string(),
            owner: "amzxyz".to_string(),
            repo: "rime_wanxiang".to_string(),
            dict_releases_tag: "dict-nightly".to_string(),
            model_name: "RIME-LMDG".to_string(),
            model_tag: "LTS".to_string(),
            model_file_name: "wanxiang-lts-zh-hans.gram".to_string(),
            github_cookies: String::new(),
        }
    }
}
