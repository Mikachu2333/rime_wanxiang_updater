/// 万象词库更新器主程序
/// 
/// 功能：
/// - 检查并更新万象输入法方案
/// - 检查并更新词库文件  
/// - 检查并更新模型文件
/// - 支持程序自身更新
/// - 支持单实例运行
/// - 支持自动重新部署小狼毫
use std::{fs, os::windows::process::CommandExt, path::PathBuf};

mod config_read;
mod github_api;
mod path_get;
mod types;
mod update_checker;
mod url_utils;
mod version_choose;

use config_read::read_config;
use types::UpdateInfo;
use update_checker::UpdateChecker;

const PROCESS_ID: &str = "3A5583B7F6A5CF24D2E7C8650277DBB4";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let instance = single_instance::SingleInstance::new(PROCESS_ID)?;
    if !instance.is_single() {
        let _ = std::process::Command::new("mshta")
            .raw_arg("\"javascript:var sh=new ActiveXObject('WScript.Shell'); sh.Popup('检测到程序已在运行',0,'错误',16);close()\"").spawn();
        eprintln!("❌ 程序已在运行！");
        std::process::exit(1);
    }

    println!(
        r#"
=== 万象词库更新器 v{} ===

万象项目主页：https://github.com/amzxyz/rime_wanxiang
灵感来源：https://github.com/expoli/rime-wanxiang-update-tools

本项目主页：https://github.com/Mikachu2333/rime_wanxiang_updater
"#,
        env!("CARGO_PKG_VERSION")
    );

    let paths = match path_get::get_path() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("获取路径失败: {}", e);
            std::process::exit(1);
        }
    };
    let config = read_config(&paths.config);

    println!("小狼毫路径: {:?}", paths.weasel);
    println!("用户目录: {:?}", paths.user);
    println!("配置文件: {:?}", paths.config);
    println!("cURL路径: {:?}", paths.curl);
    println!("7z路径: {:?}", paths.zip);

    // 创建更新检查器
    let checker = UpdateChecker::new(&paths, config);

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
                        "self" => {
                            println!("发现程序更新，正在准备自动更新...");
                            if perform_self_update(&checker, info) {
                                println!("✅ 程序将在更新后重新启动");
                                return Ok(()); // 程序退出，让批处理脚本接管
                            } else {
                                println!("❌ 自动更新失败，请手动下载更新:");
                                println!("  下载地址: {}", info.url);
                                println!("  更新说明: {}", info.description.lines().next().unwrap_or(""));
                            }
                        }
                        _ => {
                            eprintln!("⚠️ 未知的组件类型: {}", component);
                        }
                    }
                }

                // 保存更新信息到缓存
                for (component, info) in &updates {
                    let cache_path = checker.cache_dir.join(format!("{}_info.json", component));
                    if let Err(e) = checker.save_update_info(info, &cache_path) {
                        eprintln!("保存 {} 更新信息失败: {}", component, e);
                    }
                }
            }

            if has_updates {
                println!("\n正在重新部署...");
                if checker.deploy_weasel() {
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
        eprintln!("❌ {} 下载失败", update_type);
        return false;
    }

    // 验证哈希 (如果有哈希值的话)
    if let Some(ref expected_hash) = update.sha256 {
        if !checker.verify_sha256(&download_path, expected_hash) {
            eprintln!("❌ {} 文件完整性验证失败", update_type);
            return false;
        }
    }

    // 解压文件
    if !checker.extract_zip(&download_path, extract_path) {
        eprintln!("❌ {} 解压失败", update_type);
        return false;
    }

    println!("✅ {} 更新成功", update_type);
    true
}

fn download_and_replace(
    checker: &UpdateChecker,
    update: &UpdateInfo,
    target_path: &PathBuf,
) -> bool {
    let download_path = checker.cache_dir.join(&update.file_name);

    // 下载文件
    if !checker.download_file(&update.url, &download_path) {
        eprintln!("❌ 模型文件下载失败");
        return false;
    }

    // 验证哈希
    if let Some(ref expected_hash) = update.sha256 {
        if !checker.verify_sha256(&download_path, expected_hash) {
            eprintln!("❌ 模型文件完整性验证失败");
            return false;
        }
    }

    // 替换文件
    if let Err(e) = fs::copy(&download_path, target_path) {
        eprintln!("❌ 替换模型文件失败: {}", e);
        return false;
    }

    println!("✅ 模型更新成功");
    true
}

fn perform_self_update(checker: &UpdateChecker, update: &UpdateInfo) -> bool {
    let download_path = checker.cache_dir.join(&update.file_name);

    // 下载新版本
    if !checker.download_file(&update.url, &download_path) {
        eprintln!("❌ 程序更新文件下载失败");
        return false;
    }

    // 验证哈希
    if let Some(ref expected_hash) = update.sha256 {
        if !checker.verify_sha256(&download_path, expected_hash) {
            eprintln!("❌ 程序更新文件完整性验证失败");
            return false;
        }
    }

    // 创建自更新脚本
    let script_content = format!(
        r#"@echo off
timeout /t 3 /nobreak >nul
copy /y "{}" "{}"
if %errorlevel% equ 0 (
    echo 更新成功，正在重新启动程序...
    start "" "{}"
) else (
    echo 更新失败！
    pause
)
del "%~f0""#,
        download_path.display(),
        std::env::current_exe().unwrap().display(),
        std::env::current_exe().unwrap().display()
    );

    let script_path = checker.cache_dir.join("update.bat");
    if fs::write(&script_path, script_content).is_ok() {
        // 启动更新脚本
        let _ = std::process::Command::new(&script_path).spawn();
        return true;
    }

    false
}
