use crate::types::UpdateConfig;
use crate::url_utils::{parse_github_repo, sanitize_repo_url};

pub fn read_config(config_path: &std::path::Path) -> UpdateConfig {
    let mut config = UpdateConfig::default();

    if let Ok(content) = std::fs::read_to_string(config_path) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"'); // 移除引号

                match key {
                    "mirror" => config.mirror = value.to_string(),
                    "repo_url" => {
                        config.repo_url = sanitize_repo_url(value);
                        if let Some((owner, repo)) = parse_github_repo(&config.repo_url) {
                            config.owner = owner;
                            config.repo = repo;
                        }
                    }
                    "dict_releases_tag" => config.dict_releases_tag = value.to_string(),
                    "model_name" => config.model_name = value.to_string(),
                    "model_tag" => config.model_tag = value.to_string(),
                    "model_file_name" => config.model_file_name = value.to_string(),
                    "github_cookies" => config.github_cookies = value.to_string(), // 新增cookies解析
                    _ => {}
                }
            }
        }
    }

    config
}
