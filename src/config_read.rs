use crate::types::UpdateConfig;
use ini::Ini;
use std::{
    io::{self, Write},
    path::PathBuf,
};

/// 初始化配置时的交互式方案选择
pub fn init_config_with_selection(config_path: &PathBuf) -> UpdateConfig {
    let mut config = UpdateConfig::default();

    println!("\n=== 万象输入法方案配置向导 ===");

    // 选择方案版本
    config.schema_type = select_schema_type();

    // 如果选择增强版，需要选择具体方案
    if config.schema_type == "pro" {
        config.schema_key = select_schema_key();
        config.schema_name = format!("rime-wanxiang-{}.zip", config.schema_key);
        config.dict_name = format!("9-{}-dicts.zip", config.schema_key);
    } else {
        config.schema_key = "".to_string();
        config.schema_name = "rime-wanxiang-base.zip".to_string();
        config.dict_name = "9-base-dicts.zip".to_string();
    }

    // 显示选择结果
    println!("\n✅ 配置完成：");
    println!(
        "  方案版本: {}",
        if config.schema_type == "base" {
            "基础版"
        } else {
            "增强版"
        }
    );
    if config.schema_type == "pro" {
        println!("  辅助码: {}", get_schema_display_name(&config.schema_key));
    }
    println!("  方案文件: {}", config.schema_name);
    println!("  词库文件: {}", config.dict_name);

    // 写入配置文件
    write_default_config(config_path, &config);

    config
}

/// 选择方案版本类型
fn select_schema_type() -> String {
    loop {
        println!("\n请选择万象方案版本：");
        println!("[1] 万象基础版 - 适合大多数用户");
        println!("[2] 万象增强版 - 支持各种辅助码");

        print!("请输入选择 (1-2): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            match input.trim() {
                "1" => {
                    println!("✅ 已选择：万象基础版");
                    return "base".to_string();
                }
                "2" => {
                    println!("✅ 已选择：万象增强版");
                    return "pro".to_string();
                }
                _ => {
                    println!("❌ 输入无效，请重新选择");
                    continue;
                }
            }
        }
    }
}

/// 选择具体的辅助码方案
fn select_schema_key() -> String {
    loop {
        println!("\n请选择辅助码方案：");
        println!("[1] 墨奇码 (moqi)");
        println!("[2] 小鹤双拼 (flypy)");
        println!("[3] 自然码 (zrm)");
        println!("[4] 简单鹤 (jdh)");
        println!("[5] 虎码 (tiger)");
        println!("[6] 五笔 (wubi)");
        println!("[7] 汉心码 (hanxin)");

        print!("请输入选择 (1-7): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            let choice = match input.trim() {
                "1" => Some("moqi"),
                "2" => Some("flypy"),
                "3" => Some("zrm"),
                "4" => Some("jdh"),
                "5" => Some("tiger"),
                "6" => Some("wubi"),
                "7" => Some("hanxin"),
                _ => None,
            };

            if let Some(key) = choice {
                println!("✅ 已选择：{}", get_schema_display_name(key));
                return key.to_string();
            } else {
                println!("❌ 输入无效，请重新选择");
            }
        }
    }
}

/// 获取方案显示名称
fn get_schema_display_name(key: &str) -> &'static str {
    match key {
        "moqi" => "墨奇码",
        "flypy" => "小鹤双拼",
        "zrm" => "自然码",
        "jdh" => "简单鹤",
        "tiger" => "虎码",
        "wubi" => "五笔",
        "hanxin" => "汉心码",
        _ => "未知方案",
    }
}

/// 读取配置文件
pub fn read_config(config_path: &PathBuf) -> UpdateConfig {
    let mut config = UpdateConfig::default();

    if !config_path.exists() {
        println!("配置文件不存在，启动配置向导");
        return init_config_with_selection(config_path);
    }

    match Ini::load_from_file(config_path) {
        Ok(ini) => {
            // 读取 [general] 节
            if let Some(general) = ini.section(Some("general")) {
                if let Some(mirror) = general.get("mirror") {
                    config.mirror = mirror.trim_matches('"').to_string();
                }
            }

            // 读取 [repositories] 节
            if let Some(repos) = ini.section(Some("repositories")) {
                if let Some(schema_repo) = repos.get("schema_repo") {
                    config.schema_repo = schema_repo.trim_matches('"').to_string();
                }
                if let Some(dict_repo) = repos.get("dict_repo") {
                    config.dict_repo = dict_repo.trim_matches('"').to_string();
                }
                if let Some(model_repo) = repos.get("model_repo") {
                    config.model_repo = model_repo.trim_matches('"').to_string();
                }
                if let Some(self_repo) = repos.get("self_repo") {
                    config.self_repo = self_repo.trim_matches('"').to_string();
                }
            }

            // 读取 [files] 节
            if let Some(files) = ini.section(Some("files")) {
                if let Some(schema_type) = files.get("schema_type") {
                    config.schema_type = schema_type.trim_matches('"').to_string();
                }
                if let Some(schema_key) = files.get("schema_key") {
                    config.schema_key = schema_key.trim_matches('"').to_string();
                }
                if let Some(schema_name) = files.get("schema_name") {
                    config.schema_name = schema_name.trim_matches('"').to_string();
                }
                if let Some(dict_name) = files.get("dict_name") {
                    config.dict_name = dict_name.trim_matches('"').to_string();
                }
                if let Some(dict_tag) = files.get("dict_tag") {
                    config.dict_tag = dict_tag.trim_matches('"').to_string();
                }
                if let Some(model_tag) = files.get("model_tag") {
                    config.model_tag = model_tag.trim_matches('"').to_string();
                }
                if let Some(model_file_name) = files.get("model_file_name") {
                    config.model_file_name = model_file_name.trim_matches('"').to_string();
                }
                if let Some(github_cookies) = files.get("github_cookies") {
                    let cookies = github_cookies.trim_matches('"').to_string();
                    config.github_cookies = if cookies.is_empty() {
                        None
                    } else {
                        Some(cookies)
                    };
                }
            }

            // 检查是否需要重新配置方案
            if config.schema_type.is_empty() || config.schema_name.is_empty() {
                println!("检测到配置不完整，启动方案选择向导");
                return init_config_with_selection(config_path);
            }

            println!("✅ 配置文件读取成功");
        }
        Err(e) => {
            eprintln!("读取配置文件失败: {}, 启动配置向导", e);
            return init_config_with_selection(config_path);
        }
    }

    config
}

fn write_default_config(config_path: &PathBuf, config: &UpdateConfig) {
    // 确保配置文件目录存在
    if let Some(parent) = config_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!("创建配置文件目录失败: {}", e);
            return;
        }
    }

    let ini_content = format!(
        r#"# 万象词库更新器配置文件
# 
# 万象项目主页：https://github.com/amzxyz/rime_wanxiang
# 灵感来源：https://github.com/expoli/rime-wanxiang-update-tools
#
# 本项目主页：https://github.com/Mikachu2333/rime_wanxiang_updater

[general]
# 镜像网站配置 - 用于加速 GitHub 文件下载
# 如果留空则直接使用 GitHub 原始链接
# 可选镜像站示例：github.sagolu.top, gh-proxy.com, github.chenc.dev
# 更多镜像站请参考：https://github.akams.cn/
mirror = "{}"

[repositories]
# GitHub 仓库配置 - 格式为 "用户名/仓库名"
# 方案仓库：存放输入法配置方案的仓库
schema_repo = "{}"

# 词库仓库：存放词典文件的仓库
dict_repo = "{}"

# 模型仓库：存放语言模型文件的仓库
model_repo = "{}"

# 程序自身更新仓库：用于检查更新器程序本身的新版本
self_repo = "{}"

[files]
# 文件匹配规则 - 用于从 GitHub Releases 中识别和下载对应文件

# 方案相关文件配置
# schema_type: 方案版本类型 (base=基础版, pro=增强版)
schema_type = "{}"
# schema_key: 增强版的方案键值 (moqi, flypy, zrm, jdh, tiger, wubi, hanxin)
# 仅在 schema_type = "pro" 时生效
schema_key = "{}"
# schema_name: 方案文件的文件名，程序会根据上述配置自动生成
schema_name = "{}"

# 词典相关文件配置
# dict_name: 词典文件名称，用于匹配下载的词典文件
dict_name = "{}"
# dict_tag: 词典 Release 标签，指定从哪个 Release 下载词典
dict_tag = "{}"

# 语言模型相关文件配置
# model_tag: 模型 Release 标签，指定从哪个 Release 下载模型
model_tag = "{}"
# model_file_name: 具体的模型文件名
model_file_name = "{}"

# GitHub Cookies 配置（可选）
# 用于访问私有仓库或提高 API 访问限制
# 如果遇到 API 限制或需要访问私有仓库时才需要配置
# 格式示例: "session=xxx; _octo=xxx; logged_in=yes"
# 获取方法：浏览器登录 GitHub 后，在开发者工具中查看 Cookie
github_cookies = "{}"
"#,
        config.mirror,
        config.schema_repo,
        config.dict_repo,
        config.model_repo,
        config.self_repo,
        config.schema_type,
        config.schema_key,
        config.schema_name,
        config.dict_name,
        config.dict_tag,
        config.model_tag,
        config.model_file_name,
        config.github_cookies.as_deref().unwrap_or("")
    );

    if let Err(e) = std::fs::write(config_path, ini_content) {
        eprintln!("写入默认配置文件失败: {}", e);
    } else {
        println!("✅ 已创建默认配置文件: {:?}", config_path);
    }
}
