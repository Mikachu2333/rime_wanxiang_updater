use crate::types::UpdateConfig;
use crate::url_utils::{parse_github_repo, sanitize_mirror_domain, sanitize_repo_url};
use std::{fs, path::PathBuf};

pub fn get_config(config_path: &PathBuf) -> UpdateConfig {
    if !config_path.exists() {
        panic!("配置文件不存在: {:?}", config_path);
    }

    let content = fs::read_to_string(config_path)
        .unwrap_or_else(|_| panic!("无法读取配置文件: {:?}", config_path));

    let mut config = UpdateConfig::default();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"');

            match key {
                "mirror" => config.mirror = sanitize_mirror_domain(value),
                "repo_url" => config.repo_url = sanitize_repo_url(value),
                "dict_releases_tag" => config.dict_releases_tag = value.to_string(),
                "model_name" => config.model_name = value.to_string(),
                "model_tag" => config.model_tag = value.to_string(),
                "model_file_name" => config.model_file_name = value.to_string(),
                _ => {}
            }
        }
    }

    // 从 repo_url 解析出 owner 和 repo 名称
    if let Some((owner, repo)) = parse_github_repo(&config.repo_url) {
        config.owner = owner;
        config.repo = repo;
    } else {
        println!(
            "警告: 无法解析 repo_url '{}', 使用默认仓库",
            config.repo_url
        );
        config.owner = "amzxyz".to_string();
        config.repo = "rime_wanxiang".to_string();
    }

    config
}
