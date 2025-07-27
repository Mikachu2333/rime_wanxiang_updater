use crate::types::{UpdateConfig, UpdateInfo, UserPath};
use std::{collections::HashMap, fs, path::PathBuf};

use super::{
    file_operations::FileOperations, github_client::GitHubClient, weasel_manager::WeaselManager,
};

pub struct UpdateChecker {
    pub cache_dir: PathBuf,
    github_client: GitHubClient,
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
    pub fn download_file(&self, url: &str, save_path: &PathBuf) -> bool {
        self.file_ops
            .download_file(&self.github_client.curl_path, url, save_path)
    }

    pub fn verify_sha256(&self, file_path: &PathBuf, expected_hash: &str) -> bool {
        self.file_ops.verify_sha256(file_path, expected_hash)
    }

    pub fn extract_zip(&self, zip_path: &PathBuf, extract_path: &PathBuf) -> bool {
        self.file_ops.extract_zip(zip_path, extract_path)
    }

    pub fn deploy_weasel(&self) -> bool {
        self.weasel_mgr.deploy()
    }
}
