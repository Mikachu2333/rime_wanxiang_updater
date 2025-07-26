use crate::types::{UpdateConfig, UpdateInfo};
use std::{fs, path::PathBuf, process::Command};

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
        if !self.config.dict_enabled {
            return Ok(None);
        }
        
        self.check_repo_update(&self.config.dict_repo, "dict")
    }

    /// 检查方案更新
    pub fn check_scheme_update(&self) -> Result<Option<UpdateInfo>, Box<dyn std::error::Error>> {
        if !self.config.scheme_enabled {
            return Ok(None);
        }
        
        self.check_repo_update(&self.config.scheme_repo, "scheme")
    }

    /// 检查模型更新
    pub fn check_model_update(&self) -> Result<Option<UpdateInfo>, Box<dyn std::error::Error>> {
        if !self.config.model_enabled {
            return Ok(None);
        }
        
        self.check_repo_update(&self.config.model_repo, "model")
    }

    /// 检查程序自身更新
    pub fn check_self_update(&self) -> Result<Option<UpdateInfo>, Box<dyn std::error::Error>> {
        if !self.config.self_update_enabled {
            return Ok(None);
        }
        
        self.check_repo_update(&self.config.self_repo, "self")
    }

    /// 通用的仓库更新检查方法
    fn check_repo_update(&self, repo: &str, component_type: &str) -> Result<Option<UpdateInfo>, Box<dyn std::error::Error>> {
        let api_url = format!("https://api.github.com/repos/{}/releases/latest", repo);
        let temp_file = std::env::temp_dir().join(format!("{}_latest.json", component_type));
        
        // 使用curl获取最新发布信息
        let output = Command::new(&self.curl_path)
            .args(&[
                "-s", "-L", 
                "-H", "Accept: application/vnd.github.v3+json",
                "-H", "User-Agent: RimeWanxiangUpdater/1.0",
                "-o", temp_file.to_str().unwrap(),
                &api_url
            ])
            .output()?;

        if !output.status.success() {
            return Err(format!("获取{}更新信息失败", component_type).into());
        }

        // 读取并解析JSON响应
        let content = fs::read_to_string(&temp_file)?;
        let release: serde_json::Value = serde_json::from_str(&content)?;
        
        // 清理临时文件
        let _ = fs::remove_file(&temp_file);

        // 解析发布信息
        if let Some(assets) = release["assets"].as_array() {
            // 根据组件类型选择合适的资产
            let asset = match component_type {
                "dict" => assets.iter().find(|a| {
                    let name = a["name"].as_str().unwrap_or("");
                    name.contains("dict") || name.ends_with(".zip")
                }),
                "scheme" => assets.iter().find(|a| {
                    let name = a["name"].as_str().unwrap_or("");
                    name.contains("scheme") || name.ends_with(".zip")
                }),
                "model" => assets.iter().find(|a| {
                    let name = a["name"].as_str().unwrap_or("");
                    name.contains("model") || name.ends_with(".dat")
                }),
                "self" => assets.iter().find(|a| {
                    let name = a["name"].as_str().unwrap_or("");
                    name.ends_with(".exe")
                }),
                _ => assets.first(),
            };

            if let Some(asset) = asset {
                let update_info = UpdateInfo {
                    tag: release["tag_name"].as_str().unwrap_or("").to_string(),
                    url: asset["browser_download_url"].as_str().unwrap_or("").to_string(),
                    file_name: asset["name"].as_str().unwrap_or("").to_string(),
                    file_size: asset["size"].as_u64().unwrap_or(0),
                    update_time: release["published_at"].as_str().unwrap_or("").to_string(),
                    description: release["body"].as_str().unwrap_or("").to_string(),
                    sha256: String::new(), // GitHub API不直接提供SHA256
                };
                
                return Ok(Some(update_info));
            }
        }

        Ok(None)
    }
}