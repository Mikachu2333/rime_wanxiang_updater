use crate::types::UpdateConfig;
use std::{fs, path::PathBuf};

/// 读取配置文件
pub fn read_config(config_path: &PathBuf) -> UpdateConfig {
    if config_path.exists() {
        match fs::read_to_string(config_path) {
            Ok(content) => {
                match serde_json::from_str::<UpdateConfig>(&content) {
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

/// 保存配置文件
pub fn save_config(config: &UpdateConfig, config_path: &PathBuf) {
    match serde_json::to_string_pretty(config) {
        Ok(content) => {
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
        Err(e) => {
            eprintln!("序列化配置失败: {}", e);
        }
    }
}
