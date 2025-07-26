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
        println!("🔍 检查方案更新...");
        let api_url = format!(
            "https://api.github.com/repos/{}/releases/latest",
            self.config.scheme_repo
        );
        if let Some(release_info) = self.fetch_release_info(&api_url)? {
            // 查找方案相关的资产
            if let Some(asset) = self.find_scheme_asset(&release_info.assets) {
                println!("✅ 找到方案资产: {}", asset.name);
                return Ok(Some(UpdateInfo {
                    tag: release_info.tag_name,
                    file_name: asset.name.clone(),
                    file_size: asset.size,
                    url: self.convert_to_mirror_url(&asset.browser_download_url),
                    sha256: None,
                    update_time: release_info.published_at,
                    description: release_info.body.unwrap_or_default(),
                }));
            } else {
                println!("❌ 未找到方案相关的资产文件");
            }
        } else {
            println!("❌ 方案更新检查失败");
        }
        Ok(None)
    }

    /// 检查字典更新
    pub fn check_dict_update(&self) -> Result<Option<UpdateInfo>, Box<dyn std::error::Error>> {
        println!("🔍 检查词库更新...");
        let api_url = format!(
            "https://api.github.com/repos/{}/releases/tags/dict-nightly",
            self.config.dict_repo
        );
        if let Some(release_info) = self.fetch_release_info(&api_url)? {
            // 查找字典相关的资产
            if let Some(asset) = self.find_dict_asset(&release_info.assets) {
                println!("✅ 找到词库资产: {}", asset.name);
                return Ok(Some(UpdateInfo {
                    tag: release_info.tag_name,
                    file_name: asset.name.clone(),
                    file_size: asset.size,
                    url: self.convert_to_mirror_url(&asset.browser_download_url),
                    sha256: None,
                    update_time: release_info.published_at,
                    description: release_info.body.unwrap_or_default(),
                }));
            } else {
                println!("❌ 未找到词库相关的资产文件");
            }
        } else {
            println!("❌ 词库更新检查失败");
        }
        Ok(None)
    }

    /// 检查模型更新
    pub fn check_model_update(&self) -> Result<Option<UpdateInfo>, Box<dyn std::error::Error>> {
        println!("🔍 检查模型更新...");
        let api_url = format!(
            "https://api.github.com/repos/{}/releases/tags/LTS",
            self.config.model_repo
        );
        if let Some(release_info) = self.fetch_release_info(&api_url)? {
            // 查找模型相关的资产
            if let Some(asset) = self.find_model_asset(&release_info.assets) {
                println!("✅ 找到模型资产: {}", asset.name);
                return Ok(Some(UpdateInfo {
                    tag: release_info.tag_name,
                    file_name: asset.name.clone(),
                    file_size: asset.size,
                    url: self.convert_to_mirror_url(&asset.browser_download_url),
                    sha256: None,
                    update_time: release_info.published_at,
                    description: release_info.body.unwrap_or_default(),
                }));
            } else {
                println!("❌ 未找到模型相关的资产文件");
            }
        } else {
            println!("❌ 模型更新检查失败");
        }
        Ok(None)
    }

    /// 检查程序自身更新
    pub fn check_self_update(&self) -> Result<Option<UpdateInfo>, Box<dyn std::error::Error>> {
        println!("🔍 检查程序自身更新...");
        let api_url = format!(
            "https://api.github.com/repos/{}/releases/latest",
            self.config.self_repo
        );
        if let Some(release_info) = self.fetch_release_info(&api_url)? {
            // 查找程序相关的资产
            if let Some(asset) = self.find_self_asset(&release_info.assets) {
                // 检查版本是否比当前版本更新
                let current_version = env!("CARGO_PKG_VERSION");
                let remote_version = release_info.tag_name.trim_start_matches('v');

                println!(
                    "当前版本: {}, 远程版本: {}",
                    current_version, remote_version
                );

                // 简单的版本比较：如果版本字符串不同，则认为有更新
                if remote_version != current_version {
                    println!("✅ 找到程序更新: {}", asset.name);
                    return Ok(Some(UpdateInfo {
                        tag: release_info.tag_name,
                        file_name: asset.name.clone(),
                        file_size: asset.size,
                        url: self.convert_to_mirror_url(&asset.browser_download_url),
                        sha256: None,
                        update_time: release_info.published_at,
                        description: release_info.body.unwrap_or_default(),
                    }));
                } else {
                    println!("✅ 程序已是最新版本");
                }
            } else {
                println!("❌ 未找到程序相关的资产文件");
            }
        } else {
            println!("❌ 程序更新检查失败");
        }
        Ok(None)
    }

    /// 将 GitHub 下载链接转换为镜像站链接
    fn convert_to_mirror_url(&self, github_url: &str) -> String {
        // 检查镜像站配置是否不为空
        if !self.config.mirror.is_empty() {
            // 构建完整的镜像站 URL
            let mirror_url = if self.config.mirror.starts_with("http") {
                self.config.mirror.clone()
            } else {
                format!("https://{}", self.config.mirror)
            };
            // 将 GitHub 链接转换为镜像站链接
            return github_url.replace("https://github.com", &mirror_url);
        }
        github_url.to_string()
    }

    /// 获取GitHub Release信息
    fn fetch_release_info(
        &self,
        api_url: &str,
    ) -> Result<Option<GitHubRelease>, Box<dyn std::error::Error>> {
        println!("正在请求 API: {}", api_url);

        let output = Command::new(&self.curl_path)
            .args(&[
                "-s",
                "-H",
                "Accept: application/vnd.github.v3+json",
                "-H",
                "User-Agent: rime_wanxiang_updater",
                api_url,
            ])
            .output()?;

        if output.status.success() {
            let response = String::from_utf8(output.stdout)?;

            // 检查是否是 API 错误响应
            if response.contains("\"message\"") && response.contains("\"documentation_url\"") {
                // 这可能是 GitHub API 错误响应
                eprintln!("❌ GitHub API 请求失败!");
                eprintln!("请求 URL: {}", api_url);
                eprintln!("完整响应内容: {}", response);

                if let Ok(error) = serde_json::from_str::<GitHubApiError>(&response) {
                    eprintln!("错误消息: {}", error.message);
                    if let Some(doc_url) = &error.documentation_url {
                        eprintln!("文档地址: {}", doc_url);
                    }
                }
                return Ok(None);
            }

            match serde_json::from_str::<GitHubRelease>(&response) {
                Ok(release) => {
                    println!(
                        "✅ 成功解析 Release: {} ({})",
                        release.tag_name, release.published_at
                    );
                    Ok(Some(release))
                }
                Err(e) => {
                    eprintln!("❌ 解析GitHub Release响应失败!");
                    eprintln!("请求 URL: {}", api_url);
                    eprintln!("解析错误: {}", e);
                    eprintln!("完整响应内容: {}", response);
                    Ok(None)
                }
            }
        } else {
            eprintln!("❌ curl 请求失败!");
            eprintln!("请求 URL: {}", api_url);
            eprintln!("错误信息: {}", String::from_utf8_lossy(&output.stderr));
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
            dbg!(&name);
            
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
