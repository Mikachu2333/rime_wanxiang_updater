use crate::types::UserPath;
use rust_embed::Embed;
use std::{path::PathBuf, process::Command};

const CONF_FILENAME: &str = "updater_conf.ini";

pub fn get_path() -> UserPath {
    let user_path = get_user_path();
    let config = get_config_path(&user_path);
    config.canonicalize().unwrap();
    UserPath {
        weasel: get_exe_path(),
        user: get_user_path(),
        config: config,
    }
}

fn get_user_path() -> PathBuf {
    let output = Command::new("powershell")
        .args([
            "-Command",
            "Get-ItemProperty",
            "-Path",
            "'Registry::HKEY_CURRENT_USER\\Software\\Rime\\Weasel'",
        ])
        .output()
        .expect("未安装 PowerShell 或 调用失败");

    if output.status.success() {
        parse_user_path(
            String::from_utf8(output.stdout).expect("Failed to convert output to string."),
        )
    } else {
        panic!(
            "Failed to get Rime user directory: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn get_exe_path() -> PathBuf {
    let output = Command::new("powershell")
        .args([
            "-Command",
            "Get-ItemProperty",
            "-Path",
            "'Registry::HKEY_LOCAL_MACHINE\\SOFTWARE\\WOW6432Node\\Rime\\Weasel'",
        ])
        .output()
        .expect("未安装 PowerShell 或 调用失败");

    if output.status.success() {
        parse_exe_path(
            String::from_utf8(output.stdout).expect("Failed to convert output to string."),
        )
    } else {
        panic!(
            "Failed to get Weasel directory: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn parse_user_path(output: String) -> PathBuf {
    let mut result = PathBuf::new();
    for line in output.lines() {
        if line.contains("RimeUserDir") {
            let user_path_str = line
                .split_once(':')
                .map(|(_, v)| {
                    let binding = v
                        .trim()
                        .replace("\\\\", "/")
                        .replace("\\", "/")
                        .replace("//", "/");
                    format!("{}", binding.trim_end_matches('/'))
                })
                .unwrap();
            dbg!(&user_path_str);
            let temp = PathBuf::from(user_path_str);
            temp.canonicalize().unwrap();
            result = temp;
            break;
        }
    }
    result
}

fn parse_exe_path(output: String) -> PathBuf {
    let mut result = PathBuf::new();
    for line in output.lines() {
        if line.contains("WeaselRoot") {
            let exe_path_str = line
                .split_once(':')
                .map(|(_, v)| {
                    let binding = v
                        .trim()
                        .replace("\\\\", "/")
                        .replace("\\", "/")
                        .replace("//", "/");
                    format!("{}", binding.trim_end_matches('/'))
                })
                .unwrap();
            dbg!(&exe_path_str);
            let temp = PathBuf::from(exe_path_str);
            temp.canonicalize().unwrap();
            result = temp;
            break;
        }
    }
    result
}

fn get_config_path(user: &PathBuf) -> PathBuf {
    let user_config = user.join(CONF_FILENAME);
    dbg!(&user_config);
    if user_config.exists() {
        user_config
    } else {
        #[derive(Embed)]
        #[folder = "res/"]
        struct Asset;
        let ini_res = Asset::get(&CONF_FILENAME).expect("Error read embedded INI res file.");
        let _ = std::fs::write(&user_config, ini_res.data.as_ref()).expect("Error write INI file.");

        user_config
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_path_correct() {
        get_path();
    }
}
