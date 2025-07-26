use crate::types::UpdateConfig;
use std::{fs, path::PathBuf};

/// 读取配置文件
pub fn read_config(config_path: &PathBuf) -> UpdateConfig {
    if config_path.exists() {
        match fs::read_to_string(config_path) {
            Ok(content) => {
                match parse_ini_config(&content) {
                    Ok(config) => {
                        println!("✅ 配置文件加载成功");
                        return config;
                    }
                    Err(e) => {
                        eprintln!("⚠️ 配置文件格式错误: {}", e);
                        eprintln!("   将使用默认配置");
                    }
                }
            }
            Err(e) => {
                eprintln!("⚠️ 读取配置文件失败: {}", e);
                eprintln!("   将使用默认配置");
            }
        }
    } else {
        println!("ℹ️ 配置文件不存在，将创建默认配置");
    }

    // 使用默认配置并保存
    let default_config = UpdateConfig::default();
    save_config(&default_config, config_path);
    default_config
}

/// 解析INI配置文件
fn parse_ini_config(content: &str) -> Result<UpdateConfig, String> {
    let mut config = UpdateConfig::default();
    
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') || line.starts_with('[') {
            continue;
        }
        
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            
            match key {
                "scheme_repo" => config.scheme_repo = value.to_string(),
                "scheme_branch" => config.scheme_branch = value.to_string(),
                "dict_repo" => config.dict_repo = value.to_string(),
                "dict_branch" => config.dict_branch = value.to_string(),
                "model_repo" => config.model_repo = value.to_string(),
                "model_branch" => config.model_branch = value.to_string(),
                "self_repo" => config.self_repo = value.to_string(),
                "self_branch" => config.self_branch = value.to_string(),
                _ => {} // 忽略未知配置项
            }
        }
    }
    
    Ok(config)
}

/// 保存配置文件（INI格式）
pub fn save_config(config: &UpdateConfig, config_path: &PathBuf) {
    let content = format!(
        r#"# 万象词库更新器配置文件
# 配置各组件的GitHub仓库信息

[repositories]
scheme_repo={}
scheme_branch={}
dict_repo={}
dict_branch={}
model_repo={}
model_branch={}
self_repo={}
self_branch={}
"#,
        config.scheme_repo,
        config.scheme_branch,
        config.dict_repo,
        config.dict_branch,
        config.model_repo,
        config.model_branch,
        config.self_repo,
        config.self_branch
    );

    if let Some(parent) = config_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("创建配置目录失败: {}", e);
            return;
        }
    }

    if let Err(e) = fs::write(config_path, content) {
        eprintln!("保存配置文件失败: {}", e);
    } else {
        println!("✅ 配置文件已保存到: {:?}", config_path);
    }
}
