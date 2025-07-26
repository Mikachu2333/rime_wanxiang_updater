use crate::types::{UpdateConfig, UpdateInfo};
use crate::url_utils::{build_releases_tag_api_url, apply_mirror_to_download_url};
use serde_json::Value;
use std::{path::PathBuf, process::Command};

pub struct GitHubClient {
    curl_path: PathBuf,
    config: UpdateConfig,
}

impl GitHubClient {
    pub fn new(curl_path: PathBuf, config: UpdateConfig) -> Self {
        Self { curl_path, config }
    }

    /// 检查字典更新
    pub fn check_dict_update(&self) -> Result<Option<UpdateInfo>, Box<dyn std::error::Error>> {
        let url = build_releases_tag_api_url(
            &self.config.owner,
            &self.config.repo,
            &self.config.dict_releases_tag,
        );

        let response_json = self.fetch_json(&url)?;
        let release: Value = serde_json::from_str(&response_json)?;
        Ok(self.parse_release_info(release, "dict"))
    }

    /// 检查模型更新
    pub fn check_model_update(&self) -> Result<Option<UpdateInfo>, Box<dyn std::error::Error>> {
        let url = build_releases_tag_api_url(
            &self.config.owner,
            &self.config.repo,
            &self.config.model_tag,
        );

        let response_json = self.fetch_json(&url)?;
        let release: Value = serde_json::from_str(&response_json)?;
        Ok(self.parse_release_info(release, "model"))
    }

    /// 检查程序自身更新
    pub fn check_self_update(&self) -> Result<Option<UpdateInfo>, Box<dyn std::error::Error>> {
        let url = "https://api.github.com/repos/Mikachu2333/rime_wanxiang_updater/releases/latest";
        
        let response_json = self.fetch_json(&url)?;
        let release: Value = serde_json::from_str(&response_json)?;

        // 检查是否为稳定版本
        if release["prerelease"].as_bool().unwrap_or(true) {
            return Ok(None);
        }

        let tag_name = release["tag_name"].as_str().unwrap_or("");
        let current_version = env!("CARGO_PKG_VERSION");
        
        if !self.is_newer_version(tag_name, current_version) {
            return Ok(None);
        }

        Ok(self.parse_release_info(release, "self"))
    }

    /// 从curl获取JSON数据
    fn fetch_json(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        println!("正在请求: {}", url);

        let mut command = Command::new(&self.curl_path);
        
        command
            .arg("-s")
            .arg("-L")
            .arg("--fail")
            .arg("--max-time")
            .arg("30")
            .arg("-H")
            .arg("User-Agent: rime_wanxiang_updater/1.0")
            .arg("-H")
            .arg("Accept: application/vnd.github+json");

        if !self.config.github_cookies.is_empty() {
            println!("使用GitHub Cookies进行请求");
            command
                .arg("-H")
                .arg(format!("Cookie: {}", self.config.github_cookies));
        }

        command.arg(url);

        let output = command.output()?;

        if output.status.success() {
            let response = String::from_utf8_lossy(&output.stdout).to_string();
            if response.trim().is_empty() {
                return Err("收到空响应".into());
            }
            Ok(response)
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(format!("curl请求失败: {}", error).into())
        }
    }

    /// 解析发布信息
    fn parse_release_info(&self, release: Value, update_type: &str) -> Option<UpdateInfo> {
        let tag_name = release["tag_name"].as_str().unwrap_or("").to_string();
        let published_at = release["published_at"].as_str().unwrap_or("").to_string();
        let description = release["body"].as_str().unwrap_or("").to_string();

        if let Some(assets) = release["assets"].as_array() {
            for asset in assets {
                let file_name = asset["name"].as_str().unwrap_or("");
                
                let matches = match update_type {
                    "model" => file_name == self.config.model_file_name,
                    "dict" => file_name.contains("dict") || file_name.ends_with(".zip"),
                    "self" => file_name.ends_with(".exe") || file_name.contains("windows"),
                    _ => false,
                };

                if matches {
                    let original_url = asset["browser_download_url"].as_str().unwrap_or("");
                    let download_url = if self.config.mirror.is_empty() {
                        original_url.to_string()
                    } else {
                        apply_mirror_to_download_url(&self.config.mirror, original_url)
                    };

                    return Some(UpdateInfo {
                        url: download_url,
                        update_time: published_at,
                        tag: tag_name,
                        sha256: String::new(),
                        description,
                        file_size: asset["size"].as_u64().unwrap_or(0),
                        file_name: file_name.to_string(),
                    });
                }
            }
        }

        None
    }

    /// 版本比较
    fn is_newer_version(&self, remote_version: &str, current_version: &str) -> bool {
        let remote_version = remote_version.trim_start_matches('v');
        let current_version = current_version.trim_start_matches('v');

        let remote_parts: Vec<u32> = remote_version
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        let current_parts: Vec<u32> = current_version
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();

        let max_len = remote_parts.len().max(current_parts.len());
        let mut remote_normalized = remote_parts;
        let mut current_normalized = current_parts;

        remote_normalized.resize(max_len, 0);
        current_normalized.resize(max_len, 0);

        remote_normalized > current_normalized
    }
}