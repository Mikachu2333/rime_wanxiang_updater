use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// ===== 核心业务类型 =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub tag: String,
    pub file_name: String,
    pub file_size: u64,
    pub url: String,
    pub sha3_256: Option<String>,
    pub update_time: String,
    pub description: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct UpdateConfig {
    pub schema_repo: String,
    pub dict_repo: String,
    pub model_repo: String,
    pub self_repo: String,
    pub mirror: String,
    pub schema_type: String,
    pub schema_key: String,
    pub schema_name: String,
    pub dict_name: String,
    pub dict_tag: String,
    pub model_tag: String,
    pub model_file_name: String,
    pub github_cookies: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UserPath {
    pub user: PathBuf,
    pub weasel: PathBuf,
    pub config: PathBuf,
    pub curl: PathBuf,
    pub zip: PathBuf,
}

// ===== GitHub API 相关类型 =====

/// GitHub Release 响应结构
#[derive(Debug, Clone, serde::Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub published_at: String,
    pub body: Option<String>,
    pub assets: Vec<GitHubAsset>,
}

/// GitHub Asset 响应结构
#[derive(Debug, Clone, serde::Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub size: u64,
    pub browser_download_url: String,
    #[serde(rename = "sha3-256")]
    pub sha3_256: Option<String>,
}

/// GitHub API 错误响应结构
#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
pub struct GitHubApiError {
    pub message: String,
    pub documentation_url: Option<String>,
}

// ===== 实现默认值 =====

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            schema_repo: "amzxyz/rime_wanxiang".to_string(),
            dict_repo: "amzxyz/rime_wanxiang".to_string(),
            model_repo: "amzxyz/RIME-LMDG".to_string(),
            self_repo: "Mikachu2333/rime_wanxiang_updater".to_string(),
            mirror: "".to_string(),
            schema_type: "base".to_string(),
            schema_key: "".to_string(),
            schema_name: "rime-wanxiang-base.zip".to_string(),
            dict_name: "9-base-dicts.zip".to_string(),
            dict_tag: "dict-nightly".to_string(),
            model_tag: "LTS".to_string(),
            model_file_name: "wanxiang-lts-zh-hans.gram".to_string(),
            github_cookies: None,
        }
    }
}

pub fn compare_version(remote_info: String, local_info: String) -> bool {
    dbg!(&remote_info, &local_info);
    let remote_each = remote_info
        .splitn(3, '.')
        .map(|x| {
            let filtered: String = x.chars().filter(|c| c.is_ascii_digit()).collect();
            if filtered.is_empty() {
                0
            } else {
                filtered.parse::<u16>().unwrap_or(0)
            }
        })
        .collect::<Vec<u16>>();
    let local_each = local_info
        .splitn(3, '.')
        .map(|x| {
            let filtered: String = x.chars().filter(|c| c.is_ascii_digit()).collect();
            if filtered.is_empty() {
                0
            } else {
                filtered.parse::<u16>().unwrap_or(0)
            }
        })
        .collect::<Vec<u16>>();
    let mut result = false;
    dbg!(&local_each, &remote_each);
    for i in 0..remote_each.len() {
        if remote_each[i] > local_each[i] {
            result = result || true;
        } else {
            result = result || false;
        }
    }
    result
}
