use crate::types::UserPath;
use rust_embed::Embed;
use std::{path::PathBuf, process::Command};

const CONF_FILENAME: &str = "updater_conf.ini";

pub fn get_path() -> UserPath {
    let (user_path, config_path) = get_user_path();
    let (weasel_path, curl_path, zip_path) = get_exe_path();
    let temp = UserPath {
        zip: zip_path,
        curl: curl_path,
        weasel: weasel_path,
        user: user_path,
        config: config_path,
    };
    dbg!(&temp);
    temp
}

fn get_user_path() -> (PathBuf, PathBuf) {
    let output = Command::new("powershell")
        .args([
            "-Command",
            "Get-ItemProperty",
            "-Path",
            "'Registry::HKEY_CURRENT_USER\\Software\\Rime\\Weasel'",
        ])
        .output()
        .expect("未安装 PowerShell 或调用失败");

    if output.status.success() {
        let user = parse_user_path(
            String::from_utf8(output.stdout).expect("Failed to convert output to string"),
        );
        let config = format!("{}/{}", user, CONF_FILENAME);
        let packed_config = PathBuf::from(config);
        config_exist(&packed_config);

        (PathBuf::from(user), packed_config)
    } else {
        panic!(
            "Failed to get Rime user directory: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn get_exe_path() -> (PathBuf, PathBuf, PathBuf) {
    let output = Command::new("powershell")
        .args([
            "-Command",
            "Get-ItemProperty",
            "-Path",
            "'Registry::HKEY_LOCAL_MACHINE\\SOFTWARE\\WOW6432Node\\Rime\\Weasel'",
        ])
        .output()
        .expect("未安装 PowerShell 或调用失败");

    if output.status.success() {
        let exe = parse_exe_path(
            String::from_utf8(output.stdout).expect("Failed to convert output to string"),
        );
        let curl = format!("{}/curl.exe", &exe);
        let zip = format!("{}/7z.exe", &exe);
        (PathBuf::from(exe), PathBuf::from(curl), PathBuf::from(zip))
    } else {
        panic!(
            "Failed to get Weasel directory: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn parse_user_path(output: String) -> String {
    let mut result = String::new();
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
            result = user_path_str.to_string();
            break;
        }
    }
    result
}

fn parse_exe_path(output: String) -> String {
    let mut result = String::new();
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
            result = exe_path_str.to_string();
            break;
        }
    }
    result
}

fn config_exist(config_path: &PathBuf) {
    dbg!(&config_path);
    if !config_path.exists() {
        #[derive(Embed)]
        #[folder = "res/"]
        struct Asset;
        let ini_res = Asset::get(CONF_FILENAME).expect("Error reading embedded INI resource file");
        let _ = std::fs::write(config_path, ini_res.data.as_ref()).expect("Error writing INI file");
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
