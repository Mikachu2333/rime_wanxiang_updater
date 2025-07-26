use crate::url_utils::{sanitize_repo_url, parse_github_repo};
use crate::types::UpdateConfig;
use std::{fs, path::PathBuf};

pub fn get_config(config_path: &PathBuf) -> UpdateConfig {
    if !config_path.exists() {
        println!("配置文件不存在，使用默认配置");
        return UpdateConfig::default();
    }

    match fs::read_to_string(config_path) {
        Ok(content) => parse_config(&content),
        Err(e) => {
            eprintln!("读取配置文件失败: {}, 使用默认配置", e);
            UpdateConfig::default()
        }
    }
}

fn parse_config(content: &str) -> UpdateConfig {
    let mut config = UpdateConfig::default();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "mirror" => config.mirror = value.to_string(),
                "repo_url" => {
                    // 使用 sanitize_repo_url 清理仓库URL
                    config.repo_url = sanitize_repo_url(value);

                    // 使用 parse_github_repo 解析 owner 和 repo
                    if let Some((owner, repo)) = parse_github_repo(&config.repo_url) {
                        config.owner = owner;
                        config.repo = repo;
                    }
                }
                "dict_releases_tag" => config.dict_releases_tag = value.to_string(),
                "model_name" => config.model_name = value.to_string(),
                "model_tag" => config.model_tag = value.to_string(),
                "model_file_name" => config.model_file_name = value.to_string(),
                _ => {}
            }
        }
    }

    config
}
