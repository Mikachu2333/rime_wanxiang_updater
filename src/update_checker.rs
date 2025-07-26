mod core;

use crate::github_api::GitHubRelease;
use crate::types::{UpdateConfig, UpdateInfo};
use crate::url_utils::{
    apply_mirror_to_download_url, build_releases_api_url, build_releases_tag_api_url,
    sanitize_mirror_domain,
};
use serde_json::Value;
use std::collections::HashMap;
use std::{fs, path::PathBuf, process::Command};

pub struct UpdateChecker {
    pub curl_path: PathBuf,
    pub config: UpdateConfig,
    pub cache_dir: PathBuf,
}

impl UpdateChecker {
    pub fn new(weasel_path: &PathBuf, config: UpdateConfig, user_path: &PathBuf) -> Self {
        let curl_path = weasel_path.join("curl.exe");
        let cache_dir = user_path.join("UpdateCache");

        if !curl_path.exists() {
            panic!("未找到 curl.exe: {:?}", curl_path);
        }

        fs::create_dir_all(&cache_dir).expect("无法创建缓存目录");

        // 使用 sanitize_mirror_domain 清理镜像配置
        let sanitized_config = UpdateConfig {
            mirror: sanitize_mirror_domain(&config.mirror),
            ..config
        };

        Self {
            curl_path,
            config: sanitized_config,
            cache_dir,
        }
    }

    /// 检查字典更新
    pub fn check_dict_update(&self) -> Option<UpdateInfo> {
        // 使用 build_releases_tag_api_url 构建API URL
        let url = build_releases_tag_api_url(
            &self.config.owner,
            &self.config.repo,
            &self.config.dict_releases_tag,
        );

        let response_json = self.fetch_json(&url)?;
        let release: Value = serde_json::from_str(&response_json).ok()?;
        self.parse_release_info(release, "dict")
    }

    /// 检查模型更新
    pub fn check_model_update(&self) -> Option<UpdateInfo> {
        // 使用 build_releases_tag_api_url 构建API URL
        let url = build_releases_tag_api_url(
            &self.config.owner,
            &self.config.repo,
            &self.config.model_tag,
        );

        let response_json = self.fetch_json(&url)?;
        let release: Value = serde_json::from_str(&response_json).ok()?;
        self.parse_release_info(release, "model")
    }

    /// 检查程序自身更新
    pub fn check_self_update(&self) -> Option<UpdateInfo> {
        let url = "https://api.github.com/repos/Mikachu2333/rime_wanxiang_updater/releases/latest";

        let response_json = self.fetch_json(&url)?;
        let release: Value = serde_json::from_str(&response_json).ok()?;

        // 检查是否为稳定版本（不含预发布标记）
        if release["prerelease"].as_bool().unwrap_or(true) {
            return None;
        }

        let tag_name = release["tag_name"].as_str().unwrap_or("").to_string();
        let published_at = release["published_at"].as_str().unwrap_or("");
        let description = release["body"].as_str().unwrap_or("").to_string();

        // 比较版本号
        let current_version = env!("CARGO_PKG_VERSION");
        if !self.is_newer_version(&tag_name, current_version) {
            return None;
        }

        // 查找 Windows 可执行文件
        if let Some(assets) = release["assets"].as_array() {
            for asset in assets {
                let file_name = asset["name"].as_str().unwrap_or("");

                // 寻找 Windows 可执行文件
                if file_name.ends_with(".exe") || file_name.contains("windows") {
                    let download_url = if self.config.mirror.is_empty() {
                        asset["browser_download_url"]
                            .as_str()
                            .unwrap_or("")
                            .to_string()
                    } else {
                        format!(
                            "https://{}/{}",
                            self.config.mirror,
                            asset["browser_download_url"].as_str().unwrap_or("")
                        )
                    };

                    return Some(UpdateInfo {
                        url: download_url,
                        update_time: published_at.to_string(),
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

    /// 比较版本号是否更新
    fn is_newer_version(&self, remote_version: &str, current_version: &str) -> bool {
        // 移除版本号前的 'v' 前缀
        let remote_version = remote_version.trim_start_matches('v');
        let current_version = current_version.trim_start_matches('v');

        // 简单的版本比较（假设格式为 x.y.z）
        let remote_parts: Vec<u32> = remote_version
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        let current_parts: Vec<u32> = current_version
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();

        // 补齐版本号位数
        let max_len = remote_parts.len().max(current_parts.len());
        let mut remote_normalized = remote_parts;
        let mut current_normalized = current_parts;

        remote_normalized.resize(max_len, 0);
        current_normalized.resize(max_len, 0);

        remote_normalized > current_normalized
    }

    /// 检查方案更新（同步方法，使用curl）
    pub fn check_scheme_update(&self, scheme_file: &str) -> Option<UpdateInfo> {
        let api_url = build_releases_api_url(&self.config.owner, &self.config.repo);
        let releases_json = self.fetch_json(&api_url)?;

        let releases: Vec<GitHubRelease> = match serde_json::from_str(&releases_json) {
            Ok(releases) => releases,
            Err(e) => {
                eprintln!("解析方案更新JSON失败: {}", e);
                return None;
            }
        };

        self.find_asset_in_releases(&releases, scheme_file)
    }

    /// 从curl获取JSON数据（支持cookies）
    fn fetch_json(&self, url: &str) -> Option<String> {
        println!("正在请求: {}", url);

        let mut command = Command::new(&self.curl_path);

        // 基本curl参数
        command
            .arg("-s") // 静默模式
            .arg("-L") // 跟随重定向
            .arg("--fail") // 失败时返回错误
            .arg("--max-time")
            .arg("30") // 30秒超时
            .arg("-H")
            .arg("User-Agent: rime_wanxiang_updater/1.0")
            .arg("-H")
            .arg("Accept: application/vnd.github+json");

        // 如果配置了cookies，添加cookie参数
        if !self.config.github_cookies.is_empty() {
            println!("使用GitHub Cookies进行请求");
            command
                .arg("-H")
                .arg(format!("Cookie: {}", self.config.github_cookies));
        }

        // 添加URL参数
        command.arg(url);

        let output = match command.output() {
            Ok(output) => output,
            Err(e) => {
                println!("执行curl失败: {}", e);
                return None;
            }
        };

        if output.status.success() {
            let response = String::from_utf8_lossy(&output.stdout).to_string();
            if response.trim().is_empty() {
                println!("警告: 收到空响应");
                return None;
            }
            Some(response)
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            println!("curl请求失败: {}", error);

            // 如果使用了cookies但请求失败，提示用户检查cookies
            if !self.config.github_cookies.is_empty() {
                println!("提示: 请检查GitHub Cookies是否有效");
            }

            None
        }
    }

    /// 在发布列表中查找资源文件
    fn find_asset_in_releases(
        &self,
        releases: &[GitHubRelease],
        target_file: &str,
    ) -> Option<UpdateInfo> {
        // 查找最新的非预发布版本
        for release in releases {
            if release.draft || release.prerelease {
                continue;
            }

            if let Some(update_info) = self.find_asset_in_release(release, target_file) {
                return Some(update_info);
            }
        }
        None
    }

    /// 在单个发布中查找资源文件
    fn find_asset_in_release(
        &self,
        release: &GitHubRelease,
        target_file: &str,
    ) -> Option<UpdateInfo> {
        for asset in &release.assets {
            if asset.name == target_file {
                let url =
                    apply_mirror_to_download_url(&self.config.mirror, &asset.browser_download_url);
                let formatted_size = Self::format_file_size(asset.size);

                println!("找到文件: {} ({})", asset.name, formatted_size);

                return Some(UpdateInfo {
                    url,
                    update_time: asset.updated_at.clone(),
                    tag: release.tag_name.clone(),
                    sha256: String::new(), // GitHub API 不直接提供 SHA256
                    description: release.body.clone().unwrap_or_else(|| {
                        format!("Release {} - {}", release.tag_name, release.name)
                    }),
                    file_size: asset.size,
                    file_name: asset.name.clone(),
                });
            }
        }
        None
    }

    /// 检查是否需要更新（统一的检查逻辑）
    pub fn should_update(&self, remote_info: &UpdateInfo, local_cache_path: &PathBuf) -> bool {
        if !local_cache_path.exists() {
            return true;
        }

        if let Ok(content) = fs::read_to_string(local_cache_path) {
            if let Ok(local_info) = serde_json::from_str::<UpdateInfo>(&content) {
                // 比较更新时间和标签
                return remote_info.update_time != local_info.update_time
                    || remote_info.tag != local_info.tag;
            }
        }

        true
    }

    /// 保存更新信息到本地缓存
    pub fn save_update_info(
        &self,
        info: &UpdateInfo,
        cache_path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json_content = serde_json::to_string_pretty(info)?;
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(cache_path, json_content)?;
        Ok(())
    }

    /// 执行完整的更新检查（处理不同组件的同步更新）
    pub fn check_all_updates(
        &self,
    ) -> Result<HashMap<String, UpdateInfo>, Box<dyn std::error::Error>> {
        let mut updates = HashMap::new();

        // 检查字典更新
        println!("检查字典更新...");
        if let Some(dict_info) = self.check_dict_update() {
            let cache_path = self.cache_dir.join("dict_info.json");
            if self.should_update(&dict_info, &cache_path) {
                println!(
                    "发现字典更新: {} ({})",
                    dict_info.tag, dict_info.update_time
                );
                updates.insert("dict".to_string(), dict_info);
            } else {
                println!("字典已是最新版本");
            }
        } else {
            println!("未找到字典更新信息");
        }

        // 检查模型更新
        println!("检查模型更新...");
        if let Some(model_info) = self.check_model_update() {
            let cache_path = self.cache_dir.join("model_info.json");
            if self.should_update(&model_info, &cache_path) {
                println!(
                    "发现模型更新: {} ({})",
                    model_info.tag, model_info.update_time
                );
                updates.insert("model".to_string(), model_info);
            } else {
                println!("模型已是最新版本");
            }
        } else {
            println!("未找到模型更新信息");
        }

        // 检查程序自身更新
        println!("检查程序更新...");
        if let Some(self_info) = self.check_self_update() {
            let cache_path = self.cache_dir.join("self_info.json");
            if self.should_update(&self_info, &cache_path) {
                println!(
                    "发现程序更新: {} ({})",
                    self_info.tag, self_info.update_time
                );
                updates.insert("self".to_string(), self_info);
            } else {
                println!("程序已是最新版本");
            }
        } else {
            println!("程序已是最新版本");
        }

        // 同步检查方案更新（如果需要的话）
        println!("检查方案更新...");
        if let Some(scheme_info) = self.check_scheme_update(&self.config.model_file_name) {
            let cache_path = self.cache_dir.join("scheme_info.json");
            if self.should_update(&scheme_info, &cache_path) {
                println!(
                    "发现方案更新: {} ({})",
                    scheme_info.tag, scheme_info.update_time
                );
                updates.insert("scheme".to_string(), scheme_info);
            } else {
                println!("方案已是最新版本");
            }
        } else {
            println!("未找到方案更新信息");
        }

        Ok(updates)
    }

    /// 格式化文件大小
    fn format_file_size(size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = size as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.1} {}", size, UNITS[unit_index])
    }

    /// 下载文件
    pub fn download_file(&self, url: &str, save_path: &PathBuf) -> bool {
        println!("正在下载: {}", url);

        let output = Command::new(&self.curl_path)
            .args([
                "-L",
                "-o",
                save_path.to_str().unwrap(),
                "--create-dirs",
                "--progress-bar",
                url,
            ])
            .output()
            .expect("curl 下载失败");

        if output.status.success() {
            println!("下载完成: {:?}", save_path);
            true
        } else {
            eprintln!("下载失败: {}", String::from_utf8_lossy(&output.stderr));
            false
        }
    }

    /// 验证SHA256哈希
    pub fn verify_sha256(&self, file_path: &PathBuf, expected_hash: &str) -> bool {
        if expected_hash.is_empty() {
            println!("跳过哈希验证（未提供期望值）");
            return true;
        }

        let output = Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    "Get-FileHash -Path '{}' -Algorithm SHA256 | Select-Object -ExpandProperty Hash",
                    file_path.display()
                ),
            ])
            .output()
            .expect("PowerShell 哈希计算失败");

        if output.status.success() {
            let calculated_hash = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_lowercase();
            let expected_hash = expected_hash.to_lowercase();

            if calculated_hash == expected_hash {
                println!("✅ 文件哈希验证成功");
                true
            } else {
                println!("❌ 文件哈希验证失败");
                println!("期望: {}", expected_hash);
                println!("实际: {}", calculated_hash);
                false
            }
        } else {
            eprintln!("哈希计算失败: {}", String::from_utf8_lossy(&output.stderr));
            false
        }
    }

    /// 解压ZIP文件
    pub fn extract_zip(&self, zip_path: &PathBuf, extract_to: &PathBuf) -> bool {
        let output = Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                    zip_path.display(),
                    extract_to.display()
                ),
            ])
            .output()
            .expect("PowerShell 解压失败");

        if output.status.success() {
            println!("✅ 解压完成: {:?} -> {:?}", zip_path, extract_to);
            true
        } else {
            eprintln!("❌ 解压失败: {}", String::from_utf8_lossy(&output.stderr));
            false
        }
    }

    /// 终止小狼毫进程
    #[allow(dead_code)]
    pub fn terminate_weasel_processes(&self, weasel_path: &PathBuf) {
        println!("正在终止小狼毫进程...");

        let server_exe = weasel_path.join("WeaselServer.exe");
        if server_exe.exists() {
            let _ = Command::new(server_exe).arg("/q").output();
        }

        std::thread::sleep(std::time::Duration::from_millis(500));

        let _ = Command::new("taskkill")
            .args(["/IM", "WeaselServer.exe", "/F"])
            .output();

        let _ = Command::new("taskkill")
            .args(["/IM", "WeaselDeployer.exe", "/F"])
            .output();

        println!("进程终止完成");
    }

    /// 部署小狼毫
    pub fn deploy_weasel(&self, weasel_path: &PathBuf) -> bool {
        println!("正在部署小狼毫...");

        let server_exe = weasel_path.join("WeaselServer.exe");
        if server_exe.exists() {
            let _ = Command::new(&server_exe).spawn();
            std::thread::sleep(std::time::Duration::from_secs(2));
        }

        let deployer_exe = weasel_path.join("WeaselDeployer.exe");
        if deployer_exe.exists() {
            let output = Command::new(deployer_exe)
                .arg("/deploy")
                .output()
                .expect("部署器执行失败");

            if output.status.success() {
                println!("✅ 部署成功");
                true
            } else {
                eprintln!("❌ 部署失败: {}", String::from_utf8_lossy(&output.stderr));
                false
            }
        } else {
            eprintln!("❌ 未找到部署器");
            false
        }
    }

    /// 解析发布信息（统一的同步方法）
    fn parse_release_info(&self, release: Value, update_type: &str) -> Option<UpdateInfo> {
        let tag_name = release["tag_name"].as_str().unwrap_or("").to_string();
        let published_at = release["published_at"].as_str().unwrap_or("");
        let description = release["body"].as_str().unwrap_or("").to_string();

        let update_time = published_at.to_string();

        if let Some(assets) = release["assets"].as_array() {
            for asset in assets {
                let file_name = asset["name"].as_str().unwrap_or("");

                let matches = if update_type == "model" {
                    file_name == self.config.model_file_name
                } else if update_type == "dict" {
                    file_name.contains("dict") || file_name.ends_with(".zip")
                } else {
                    false
                };

                if matches {
                    let original_url = asset["browser_download_url"]
                        .as_str()
                        .unwrap_or("")
                        .to_string();

                    // 使用 apply_mirror_to_download_url 应用镜像
                    let download_url = if self.config.mirror.is_empty() {
                        original_url
                    } else {
                        apply_mirror_to_download_url(&self.config.mirror, &original_url)
                    };

                    return Some(UpdateInfo {
                        url: download_url,
                        update_time,
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
}
