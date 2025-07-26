use crate::types::{UpdateConfig, Paths};
use rust_embed::Embed;
use std::{env, fs, path::PathBuf, process::Command};

const CONF_FILENAME: &str = "updater_conf.ini";

/// 获取所有必要的路径
pub fn get_path() -> Paths {
    let weasel_path = find_weasel_path();
    let user_path = find_user_path(&weasel_path);
    let config_path = user_path.join("updater_config.json");

    Paths {
        weasel: weasel_path,
        user: user_path,
        config: config_path,
    }
}

/// 查找小狼毫安装路径
fn find_weasel_path() -> PathBuf {
    // 优先级顺序查找
    let possible_paths = vec![
        // 环境变量
        env::var("WEASEL_ROOT").ok().map(PathBuf::from),
        // 程序文件目录
        Some(PathBuf::from(r"C:\Program Files (x86)\Rime\weasel")),
        Some(PathBuf::from(r"C:\Program Files\Rime\weasel")),
        // 当前目录（便携版）
        Some(env::current_dir().unwrap_or_default()),
    ];

    for path in possible_paths.into_iter().flatten() {
        if path.join("WeaselDeployer.exe").exists() && path.join("curl.exe").exists() {
            return path;
        }
    }

    // 如果都找不到，返回默认路径并提示用户
    eprintln!("警告: 未找到小狼毫安装目录，请确保:");
    eprintln!("1. 小狼毫已正确安装");
    eprintln!("2. 或设置 WEASEL_ROOT 环境变量");
    eprintln!("3. 或将程序放在小狼毫目录中");

    PathBuf::from(r"C:\Program Files (x86)\Rime\weasel")
}

/// 查找用户数据目录
fn find_user_path(weasel_path: &PathBuf) -> PathBuf {
    // 优先级顺序查找
    let possible_paths = vec![
        // 环境变量
        env::var("RIME_USER_DIR").ok().map(PathBuf::from),
        // AppData目录
        env::var("APPDATA").ok().map(|p| PathBuf::from(p).join("Rime")),
        // 小狼毫目录下的User子目录（便携版）
        Some(weasel_path.join("User")),
        // 默认用户目录
        env::var("USERPROFILE").ok().map(|p| PathBuf::from(p).join("AppData").join("Roaming").join("Rime")),
    ];

    for path in possible_paths.into_iter().flatten() {
        if path.exists() || path.parent().map_or(false, |p| p.exists()) {
            // 确保目录存在
            if let Err(e) = fs::create_dir_all(&path) {
                eprintln!("创建用户目录失败: {}", e);
                continue;
            }
            return path;
        }
    }

    // 如果都找不到，使用默认路径
    let default_path = env::var("APPDATA")
        .map(|p| PathBuf::from(p).join("Rime"))
        .unwrap_or_else(|_| PathBuf::from("./User"));

    if let Err(e) = fs::create_dir_all(&default_path) {
        eprintln!("创建默认用户目录失败: {}", e);
    }

    default_path
}

fn config_exist(config_path: &PathBuf) {
    dbg!(&config_path);
    if !config_path.exists() {
        #[derive(Embed)]
        #[folder = "res/"]
        struct Asset;
        let ini_res = Asset::get(&CONF_FILENAME).expect("Error read embedded INI res file.");
        let _ = std::fs::write(&config_path, ini_res.data.as_ref()).expect("Error write INI file.");
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
