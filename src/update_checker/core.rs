use crate::types::{UpdateConfig, UpdateInfo};
use std::{collections::HashMap, fs, path::PathBuf};

use super::{github_client::GitHubClient, file_operations::FileOperations, weasel_manager::WeaselManager};

pub struct UpdateChecker {
    pub curl_path: PathBuf,
    pub config: UpdateConfig,
    pub cache_dir: PathBuf,
    github_client: GitHubClient,
    file_ops: FileOperations,
    weasel_mgr: WeaselManager,
}

impl UpdateChecker {
    pub fn new(weasel_path: &PathBuf, config: UpdateConfig, user_path: &PathBuf) -> Self {
        let curl_path = weasel_path.join("curl.exe");
        let cache_dir = user_path.join("UpdateCache");

        if !curl_path.exists() {
            panic!("未找到 curl.exe: {:?}", curl_path);
        }

        fs::create_dir_all(&cache_dir).expect("无法创建缓存目录");

        Self {
            curl_path: curl_path.clone(),
            config: config.clone(),
            cache_dir: cache_dir.clone(),
            github_client: GitHubClient::new(curl_path.clone(), config.clone()),
            file_ops: FileOperations::new(weasel_path.clone()),
            weasel_mgr: WeaselManager::new(weasel_path.clone()),
        }
    }

    /// 检查所有更新
    pub fn check_all_updates(&self) -> Result<HashMap<String, UpdateInfo>, Box<dyn std::error::Error>> {
        let mut updates = HashMap::new();

        // 检查方案更新
        if let Some(scheme_info) = self.github_client.check_scheme_update()? {
            let cache_path = self.cache_dir.join("scheme_info.json");
            if self.should_update(&scheme_info, &cache_path) {
                updates.insert("scheme".to_string(), scheme_info);
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

    /// 检查是否需要更新
    fn should_update(&self, remote_info: &UpdateInfo, local_cache_path: &PathBuf) -> bool {
        if !local_cache_path.exists() {
            return true;
        }

        if let Ok(content) = fs::read_to_string(local_cache_path) {
            if let Ok(local_info) = serde_json::from_str::<UpdateInfo>(&content) {
                return remote_info.update_time != local_info.update_time
                    || remote_info.tag != local_info.tag;
            }
        }

        true
    }

    /// 保存更新信息到本地缓存
    pub fn save_update_info(&self, info: &UpdateInfo, cache_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let json_content = serde_json::to_string_pretty(info)?;
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(cache_path, json_content)?;
        Ok(())
    }

    // 委托给其他模块的方法
    pub fn download_file(&self, url: &str, save_path: &PathBuf) -> bool {
        self.file_ops.download_file(&self.curl_path, url, save_path)
    }

    pub fn verify_sha256(&self, file_path: &PathBuf, expected_hash: &str) -> bool {
        self.file_ops.verify_sha256(file_path, expected_hash)
    }

    pub fn extract_zip(&self, zip_path: &PathBuf, extract_path: &PathBuf) -> bool {
        self.file_ops.extract_zip(zip_path, extract_path)
    }

    pub fn deploy_weasel(&self, weasel_path: &PathBuf) -> bool {
        self.weasel_mgr.deploy(weasel_path)
    }
}