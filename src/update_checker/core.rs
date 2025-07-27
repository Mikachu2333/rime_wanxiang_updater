use crate::file_checker;
use crate::types::{UpdateConfig, UpdateInfo, UserPath, VERSION};
use std::{collections::HashMap, fs, path::PathBuf};

use super::{
    file_operations::FileOperations, github_client::GitHubClient, weasel_manager::WeaselManager,
};

pub struct UpdateChecker {
    pub cache_dir: PathBuf,
    pub github_client: GitHubClient,
    file_ops: FileOperations,
    weasel_mgr: WeaselManager,
}

impl UpdateChecker {
    pub fn new(paths: &UserPath, config: UpdateConfig) -> Self {
        let cache_dir = paths.user.join("UpdateCache");

        if !paths.curl.exists() {
            panic!("未找到 curl.exe: {:?}\n请确保小狼毫已正确安装", paths.curl);
        }

        if !paths.zip.exists() {
            panic!("未找到 7z.exe: {:?}\n请确保小狼毫已正确安装", paths.zip);
        }

        if let Err(e) = fs::create_dir_all(&cache_dir) {
            panic!("无法创建缓存目录 {:?}: {}", cache_dir, e);
        }

        Self {
            cache_dir: cache_dir.clone(),
            github_client: GitHubClient::new(&paths.curl, config.clone()),
            file_ops: FileOperations::new(&paths.zip),
            weasel_mgr: WeaselManager::new(&paths.weasel),
        }
    }

    /// 检查所有更新
    pub fn check_all_updates(
        &self,
    ) -> Result<HashMap<String, UpdateInfo>, Box<dyn std::error::Error>> {
        let mut updates = HashMap::new();

        // 检查方案更新
        if let Some(schema_info) = self.github_client.check_schema_update()? {
            let cache_path = self.cache_dir.join("schema_info.json");
            if self.should_update(&schema_info, &cache_path) {
                updates.insert("schema".to_string(), schema_info);
            }
        }

        // 检查字典更新
        if let Some(dict_info) = self.github_client.check_dict_update()? {
            let cache_path = self.cache_dir.join("dict_info.json");
            if self.should_update(&dict_info, &cache_path) {
                updates.insert("dict".to_string(), dict_info);
            }
        }

        // 检查模型更新
        if let Some(model_info) = self.github_client.check_model_update()? {
            let cache_path = self.cache_dir.join("model_info.json");
            if self.should_update(&model_info, &cache_path) {
                updates.insert("model".to_string(), model_info);
            }
        }

        // 检查程序更新
        if let Some(self_info) = self.github_client.check_self_update()? {
            let cache_path = self.cache_dir.join("self_info.json");
            if self.should_update(&self_info, &cache_path) {
                updates.insert("self".to_string(), self_info);
            }
        }

        Ok(updates)
    }

    /// 检查是否需要更新 - 同时检查JSON缓存和实际文件是否存在
    fn should_update(&self, remote_info: &UpdateInfo, local_cache_path: &PathBuf) -> bool {
        // 如果缓存信息不存在，需要更新
        if !local_cache_path.exists() {
            return true;
        }

        // 检查对应的文件是否存在于缓存目录中
        let cached_file_path = self.cache_dir.join(&remote_info.file_name);
        if !cached_file_path.exists() {
            println!("缓存文件不存在，需要重新下载: {:?}", cached_file_path);
            return true;
        }

        // 读取并比较缓存的更新信息
        if let Ok(content) = fs::read_to_string(local_cache_path) {
            if let Ok(local_info) = serde_json::from_str::<UpdateInfo>(&content) {
                // 使用compare_version函数进行版本比较
                let needs_update = self.compare_version(remote_info.tag.clone(), local_info.tag.clone());

                if needs_update {
                    println!(
                        "发现新版本，需要更新: {} -> {}",
                        local_info.tag, remote_info.tag
                    );
                }

                return needs_update;
            }
        }

        // 无法解析缓存信息，默认需要更新
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

    // 委托给其他模块的方法
    pub fn download_file(
        &self,
        url: &str,
        save_path: &PathBuf,
        expected_sha3_256: Option<&str>,
        cookies: Option<String>,
    ) -> bool {
        // 如果文件已存在，先校验完整性
        if save_path.exists() {
            // 检查文件大小，如果太小说明下载失败
            if let Ok(metadata) = std::fs::metadata(save_path) {
                if metadata.len() < 1000 {
                    println!("🔍 发现不完整的文件 ({}字节)，重新下载...", metadata.len());
                    if let Err(e) = std::fs::remove_file(save_path) {
                        eprintln!("警告：删除不完整文件失败: {}", e);
                    }
                } else if let Some(expected_hash) = expected_sha3_256 {
                    println!("🔍 检查本地文件完整性...");
                    if self.verify_sha3_256(save_path, expected_hash) {
                        println!("✅ 本地文件校验通过，跳过下载");
                        return true;
                    } else {
                        println!("❌ 本地文件校验失败，重新下载...");
                        // 删除损坏的文件
                        if let Err(e) = std::fs::remove_file(save_path) {
                            eprintln!("警告：删除损坏文件失败: {}", e);
                        }
                    }
                } else {
                    println!("⚠️ 未提供校验和，但检测到本地文件，验证文件完整性...");
                    // 即使没有哈希，也要检查文件是否是有效的 ZIP 文件
                    if self.verify_zip_integrity(save_path) {
                        println!("✅ 本地文件格式校验通过，跳过下载");
                        return true;
                    } else {
                        println!("❌ 本地文件格式校验失败，重新下载...");
                        if let Err(e) = std::fs::remove_file(save_path) {
                            eprintln!("警告：删除损坏文件失败: {}", e);
                        }
                    }
                }
            }
        }

        // 执行下载
        let download_success =
            self.file_ops
                .download_file(&self.github_client.curl_path, url, save_path, cookies);

        // 下载完成后再次校验
        if download_success {
            if let Some(expected_hash) = expected_sha3_256 {
                println!("🔍 校验下载的文件...");
                if !self.verify_sha3_256(save_path, expected_hash) {
                    eprintln!("❌ 下载文件校验失败");
                    return false;
                }
                println!("✅ 下载文件校验通过");
            } else {
                // 没有哈希时，至少验证是否为有效 ZIP
                if !self.verify_zip_integrity(save_path) {
                    eprintln!("❌ 下载文件格式校验失败");
                    return false;
                }
                println!("✅ 下载文件格式校验通过");
            }
        }

        download_success
    }

    pub fn verify_sha3_256(&self, file_path: &PathBuf, expected_hash: &str) -> bool {
        match file_checker::verify_sha3_256(file_path, expected_hash) {
            Ok(is_valid) => is_valid,
            Err(e) => {
                eprintln!("校验失败: {}", e);
                false
            }
        }
    }

    /// 验证 ZIP 文件完整性（基本格式检查）
    fn verify_zip_integrity(&self, file_path: &PathBuf) -> bool {
        use std::fs::File;
        use std::io::Read;

        // 检查文件是否以 ZIP 魔法字节开头
        if let Ok(mut file) = File::open(file_path) {
            let mut buffer = [0; 4];
            if file.read_exact(&mut buffer).is_ok() {
                // ZIP 文件的魔法字节：PK\x03\x04 或 PK\x05\x06 或 PK\x07\x08
                return buffer[0] == 0x50
                    && buffer[1] == 0x4B
                    && (buffer[2] == 0x03 || buffer[2] == 0x05 || buffer[2] == 0x07);
            }
        }
        false
    }

    pub fn extract_zip(&self, zip_path: &PathBuf, extract_path: &PathBuf) -> bool {
        self.file_ops.extract_zip(zip_path, extract_path)
    }

    pub fn deploy_weasel(&self) -> bool {
        self.weasel_mgr.deploy()
    }

    fn compare_version(&self, remote_info: String, local_info: String) -> bool {
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
}
