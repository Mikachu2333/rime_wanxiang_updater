use crate::types::UpdateConfig;
use ini::Ini;
use std::path::PathBuf;

/// 读取配置文件
pub fn read_config(config_path: &PathBuf) -> UpdateConfig {
    let mut config = UpdateConfig::default();

    if !config_path.exists() {
        println!("配置文件不存在，使用默认配置并创建配置文件");
        write_default_config(config_path, &config);
        return config;
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
            }

            // 读取 [options] 节
            if let Some(options) = ini.section(Some("options")) {
                if let Some(check_interval) = options.get("check_interval_hours") {
                    if let Ok(hours) = check_interval.parse::<u32>() {
                        config.check_interval_hours = hours;
                    }
                }
                if let Some(auto_update) = options.get("auto_update") {
                    config.auto_update = auto_update.trim().to_lowercase() == "true";
                }
                if let Some(backup) = options.get("backup_before_update") {
                    config.backup_before_update = backup.trim().to_lowercase() == "true";
                }
                if let Some(cookies) = options.get("github_cookies") {
                    config.github_cookies = cookies.trim_matches('"').to_string();
                }
            }

            println!("✅ 配置文件读取成功");
        }
        Err(e) => {
            eprintln!("读取配置文件失败: {}, 使用默认配置", e);
            write_default_config(config_path, &config);
        }
    }

    config
}

fn write_default_config(config_path: &PathBuf, config: &UpdateConfig) {
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
# schema_name: 方案文件的文件名，用于匹配 Release 中的资产文件
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

[options]
# 更新选项配置

# 检查更新间隔（小时）- 避免过于频繁的更新检查
check_interval_hours = {}

# 是否启用自动更新 - false 表示仅检查不自动下载
auto_update = {}

# 更新前是否备份 - 建议保持启用以防止数据丢失
backup_before_update = {}

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
        config.schema_name,
        config.dict_name,
        config.dict_tag,
        config.model_tag,
        config.model_file_name,
        config.check_interval_hours,
        config.auto_update,
        config.backup_before_update,
        config.github_cookies
    );

    if let Err(e) = std::fs::write(config_path, ini_content) {
        eprintln!("写入默认配置文件失败: {}", e);
    } else {
        println!("✅ 已创建默认配置文件: {:?}", config_path);
    }
}
