use crate::types::{UpdateConfig, UpdateInfo};
use std::{path::PathBuf, process::Command};

pub struct GitHubClient {
    pub curl_path: PathBuf,
    config: UpdateConfig,
}

impl GitHubClient {
    pub fn new(curl_path: &PathBuf, config: UpdateConfig) -> Self {
        Self {
            curl_path: curl_path.clone(),
            config,
        }
    }

    /// 检查方案更新
    pub fn check_scheme_update(&self) -> Result<Option<UpdateInfo>, Box<dyn std::error::Error>> {
        let api_url = format!("https://api.github.com/repos/{}/releases/latest", self.config.scheme_repo);
        if let Some(release_info) = self.fetch_release_info(&api_url)? {
            // 查找方案相关的资产
            if let Some(asset) = self.find_scheme_asset(&release_info.assets) {
                return Ok(Some(UpdateInfo {
                    tag: release_info.tag_name,
                    file_name: asset.name.clone(),
                    file_size: asset.size,
                    url: asset.browser_download_url.clone(),
                    sha256: None,
                    update_time: release_info.published_at,
                    description: release_info.body.unwrap_or_default(),
                }));
            }
        }
        Ok(None)
    }

    /// 检查字典更新
    pub fn check_dict_update(&self) -> Result<Option<UpdateInfo>, Box<dyn std::error::Error>> {
        let api_url = format!("https://api.github.com/repos/{}/releases/latest", self.config.dict_repo);
        if let Some(release_info) = self.fetch_release_info(&api_url)? {
            // 查找字典相关的资产
            if let Some(asset) = self.find_dict_asset(&release_info.assets) {
                return Ok(Some(UpdateInfo {
                    tag: release_info.tag_name,
                    file_name: asset.name.clone(),
                    file_size: asset.size,
                    url: asset.browser_download_url.clone(),
                    sha256: None,
                    update_time: release_info.published_at,
                    description: release_info.body.unwrap_or_default(),
                }));
            }
        }
        Ok(None)
    }

    /// 检查模型更新
    pub fn check_model_update(&self) -> Result<Option<UpdateInfo>, Box<dyn std::error::Error>> {
        let api_url = format!("https://api.github.com/repos/{}/releases/latest", self.config.model_repo);
        if let Some(release_info) = self.fetch_release_info(&api_url)? {
            // 查找模型相关的资产
            if let Some(asset) = self.find_model_asset(&release_info.assets) {
                return Ok(Some(UpdateInfo {
                    tag: release_info.tag_name,
                    file_name: asset.name.clone(),
                    file_size: asset.size,
                    url: asset.browser_download_url.clone(),
                    sha256: None,
                    update_time: release_info.published_at,
                    description: release_info.body.unwrap_or_default(),
                }));
            }
        }
        Ok(None)
    }

    /// 检查程序自身更新
    pub fn check_self_update(&self) -> Result<Option<UpdateInfo>, Box<dyn std::error::Error>> {
        let api_url = format!("https://api.github.com/repos/{}/releases/latest", self.config.self_repo);
        if let Some(release_info) = self.fetch_release_info(&api_url)? {
            // 查找程序相关的资产
            if let Some(asset) = self.find_self_asset(&release_info.assets) {
                // 检查版本是否比当前版本更新
                let current_version = env!("CARGO_PKG_VERSION");
                let remote_version = release_info.tag_name.trim_start_matches('v');
                
                // 简单的版本比较：如果版本字符串不同，则认为有更新
                if remote_version != current_version {
                    return Ok(Some(UpdateInfo {
                        tag: release_info.tag_name,
                        file_name: asset.name.clone(),
                        file_size: asset.size,
                        url: asset.browser_download_url.clone(),
                        sha256: None,
                        update_time: release_info.published_at,
                        description: release_info.body.unwrap_or_default(),
                    }));
                }
            }
        }
        Ok(None)
    }

    /// 获取GitHub Release信息
    fn fetch_release_info(&self, api_url: &str) -> Result<Option<GitHubRelease>, Box<dyn std::error::Error>> {
        let output = Command::new(&self.curl_path)
            .args(&[
                "-s",
                "-H", "Accept: application/vnd.github.v3+json",
                "-H", "User-Agent: rime_wanxiang_updater",
                api_url
            ])
            .output()?;

        if output.status.success() {
            let response = String::from_utf8(output.stdout)?;
            
            // 检查是否是 API 错误响应
            if response.contains("\"message\"") && response.contains("\"documentation_url\"") {
                // 这可能是 GitHub API 错误响应
                if let Ok(error) = serde_json::from_str::<GitHubApiError>(&response) {
                    eprintln!("GitHub API 错误: {}", error.message);
                    return Ok(None);
                }
            }
            
            let release: GitHubRelease = serde_json::from_str(&response)?;
            Ok(Some(release))
        } else {
            eprintln!("获取GitHub Release信息失败: {}", String::from_utf8_lossy(&output.stderr));
            Ok(None)
        }
    }

    /// 查找方案相关的资产文件
    fn find_scheme_asset<'a>(&self, assets: &'a [GitHubAsset]) -> Option<&'a GitHubAsset> {
        for asset in assets {
            let name = asset.name.to_lowercase();
            if name.contains("scheme") || name.contains("方案") {
                return Some(asset);
            }
        }
        None
    }

    /// 查找字典相关的资产文件
    fn find_dict_asset<'a>(&self, assets: &'a [GitHubAsset]) -> Option<&'a GitHubAsset> {
        for asset in assets {
            let name = asset.name.to_lowercase();
            if name.contains("dict") || name.contains("词库") || name.contains("dictionary") {
                return Some(asset);
            }
        }
        None
    }

    /// 查找模型相关的资产文件
    fn find_model_asset<'a>(&self, assets: &'a [GitHubAsset]) -> Option<&'a GitHubAsset> {
        for asset in assets {
            let name = asset.name.to_lowercase();
            if name.contains("model") || name.contains("模型") || name.contains(".gram") {
                return Some(asset);
            }
        }
        None
    }

    /// 查找程序相关的资产文件
    fn find_self_asset<'a>(&self, assets: &'a [GitHubAsset]) -> Option<&'a GitHubAsset> {
        for asset in assets {
            let name = asset.name.to_lowercase();
            if name.contains("updater") || name.ends_with(".exe") {
                return Some(asset);
            }
        }
        None
    }
}

/// GitHub Release 响应结构
#[derive(serde::Deserialize)]
struct GitHubRelease {
    tag_name: String,
    published_at: String,
    body: Option<String>,
    assets: Vec<GitHubAsset>,
}

/// GitHub Asset 响应结构
#[derive(serde::Deserialize)]
struct GitHubAsset {
    name: String,
    size: u64,
    browser_download_url: String,
}

/// GitHub API 错误响应结构
#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct GitHubApiError {
    message: String,
    documentation_url: Option<String>,
}