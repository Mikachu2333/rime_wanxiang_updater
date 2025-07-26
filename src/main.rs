use std::{fs, os::windows::process::CommandExt, path::PathBuf};

mod config_read;
mod github_api;
mod path_get;
mod types;
mod update_checker;
mod url_utils;
mod version_choose;

use config_read::get_config;
use types::UpdateInfo;
use update_checker::UpdateChecker;

const PROCESS_ID: &str = "3A5583B7F6A5CF24D2E7C8650277DBB4";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let instance = Box::new(single_instance::SingleInstance::new(PROCESS_ID).unwrap());
    if !instance.is_single() {
        let _ = std::process::Command::new("mshta")
            .raw_arg("\"javascript:var sh=new ActiveXObject('WScript.Shell'); sh.Popup('检测到程序已在运行',0,'错误',16);close()\"").spawn();
        panic!("程序已在运行!")
    };

    println!(
        r#"
=== 万象词库更新器 v{} ===

万象项目主页：https://github.com/amzxyz/rime_wanxiang
灵感来源：https://github.com/expoli/rime-wanxiang-update-tools

本项目主页：？
"#,
        env!("CARGO_PKG_VERSION")
    );

    let paths = path_get::get_path();
    let config = get_config(&paths.config);

    println!("小狼毫路径: {:?}", paths.weasel);
    println!("用户目录: {:?}", paths.user);
    println!("配置文件: {:?}", paths.config);

    // 创建更新检查器
    let checker = UpdateChecker::new(&paths.weasel, config, &paths.user);

    // 检查所有更新
    println!("\n正在检查更新...");
    match checker.check_all_updates() {
        Ok(updates) => {
            let mut has_updates = false;
            if updates.is_empty() {
                println!("所有组件都是最新版本！");
            } else {
                println!("发现 {} 个更新:", updates.len());
                for (component, info) in &updates {
                    println!("  {} - {} ({})", component, info.tag, info.update_time);
                    println!(
                        "    文件: {} ({})",
                        info.file_name,
                        format_file_size(info.file_size)
                    );
                    if !info.description.is_empty() {
                        println!(
                            "    描述: {}",
                            info.description.lines().next().unwrap_or("")
                        );
                    }
                    println!();

                    // 处理各个组件的更新
                    match component.as_str() {
                        "scheme" => {
                            if perform_update(&checker, info, &paths.user, "方案") {
                                has_updates = true;
                            }
                        }
                        "dict" => {
                            if perform_update(&checker, info, &paths.user.join("dicts"), "词库") {
                                has_updates = true;
                            }
                        }
                        "model" => {
                            let model_path = paths.user.join(&info.file_name);
                            if download_and_replace(&checker, info, &model_path) {
                                has_updates = true;
                            }
                        }
                        _ => {}
                    }
                }

                // 保存更新信息到缓存
                for (component, info) in &updates {
                    let cache_path = PathBuf::from(format!("cache/{}_info.json", component));
                    if let Err(e) = checker.save_update_info(info, &cache_path) {
                        eprintln!("保存 {} 更新信息失败: {}", component, e);
                    }
                }
            }

            if has_updates {
                println!("\n正在重新部署...");
                if checker.deploy_weasel(&paths.weasel) {
                    println!("✅ 更新完成!");
                } else {
                    println!("❌ 部署失败，请手动重新部署");
                }
            }
        }
        Err(e) => {
            eprintln!("检查更新时出错: {}", e);
        }
    }

    Ok(())
}

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

fn perform_update(
    checker: &UpdateChecker,
    update: &UpdateInfo,
    extract_path: &PathBuf,
    update_type: &str,
) -> bool {
    let download_path = checker.cache_dir.join(&update.file_name);

    // 下载文件
    if !checker.download_file(&update.url, &download_path) {
        return false;
    }

    // 验证哈希
    if !checker.verify_sha256(&download_path, &update.sha256) {
        return false;
    }

    // 解压文件
    fs::create_dir_all(extract_path).expect("无法创建解压目录");

    if checker.extract_zip(&download_path, extract_path) {
        println!("✅ {} 更新成功", update_type);
        true
    } else {
        println!("❌ {} 更新失败", update_type);
        false
    }
}

fn download_and_replace(
    checker: &UpdateChecker,
    update: &UpdateInfo,
    target_path: &PathBuf,
) -> bool {
    let download_path = checker.cache_dir.join(&update.file_name);

    // 下载文件
    if !checker.download_file(&update.url, &download_path) {
        return false;
    }

    // 验证哈希
    if !checker.verify_sha256(&download_path, &update.sha256) {
        return false;
    }

    // 替换文件
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).expect("无法创建目标目录");
    }

    if fs::copy(&download_path, target_path).is_ok() {
        println!("✅ 模型文件更新成功");
        true
    } else {
        println!("❌ 模型文件更新失败");
        false
    }
}
