use crate::types::UserPath;
use rust_embed::Embed;
use std::{os::windows::process::CommandExt, path::PathBuf, process::Command};

const CONF_FILENAME: &str = "updater_conf.ini";

pub fn get_path() -> Result<UserPath, Box<dyn std::error::Error>> {
    let (user_path, config_path) = get_user_path()?;
    let (weasel_path, curl_path, zip_path) = get_exe_path()?;
    Ok(UserPath {
        zip: zip_path,
        curl: curl_path,
        weasel: weasel_path,
        user: user_path,
        config: config_path,
    })
}

fn get_user_path() -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error>> {
    let output = Command::new("powershell")
        .args([
            "-Command",
            "Get-ItemProperty",
            "-Path",
            "'Registry::HKEY_CURRENT_USER\\Software\\Rime\\Weasel'",
        ])
        .creation_flags(0x08000000)
        .output()
        .expect("未安装 PowerShell 或调用失败");

    if output.status.success() {
        let user = parse_user_path(
            String::from_utf8(output.stdout).expect("Failed to convert output to string"),
        );
        let config = format!("{}/{}", user, CONF_FILENAME);
        let packed_config = PathBuf::from(config);
        config_exist(&packed_config);

        Ok((PathBuf::from(user), packed_config))
    } else {
        Err(format!(
            "Failed to get Rime user directory: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into())
    }
}

fn get_exe_path() -> Result<(PathBuf, PathBuf, PathBuf), Box<dyn std::error::Error>> {
    let output = Command::new("powershell")
        .args([
            "-Command",
            "Get-ItemProperty",
            "-Path",
            "'Registry::HKEY_LOCAL_MACHINE\\SOFTWARE\\WOW6432Node\\Rime\\Weasel'",
        ])
        .creation_flags(0x08000000)
        .output()
        .expect("未安装 PowerShell 或调用失败");

    if output.status.success() {
        let exe = parse_exe_path(
            String::from_utf8(output.stdout).expect("Failed to convert output to string"),
        );
        let curl = format!("{}/curl.exe", &exe);
        let zip = format!("{}/7z.exe", &exe);
        Ok((PathBuf::from(exe), PathBuf::from(curl), PathBuf::from(zip)))
    } else {
        Err(format!(
            "Failed to get Weasel directory: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into())
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
            result = exe_path_str.to_string();
            break;
        }
    }
    result
}

fn config_exist(config_path: &PathBuf) {
    if !config_path.exists() {
        #[derive(Embed)]
        #[folder = "res/"]
        struct Asset;
        let ini_res = Asset::get(CONF_FILENAME).expect("Error reading embedded INI resource file");
        if let Err(e) = std::fs::write(config_path, ini_res.data.as_ref()) {
            eprintln!("写入配置文件失败: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_correct() {
        match get_path() {
            Ok(_) => println!("路径获取成功"),
            Err(e) => println!("路径获取失败: {}", e),
        }
    }
}
