# RIME 万象更新器

一个用于自动检查和更新 RIME 万象输入法组件的工具程序。

## 主要功能

- 自动检查万象输入法方案、词库、模型文件的更新
- 支持程序自身更新
- 支持 GitHub 镜像站加速下载
- 本地缓存和版本比较机制
- 自动重新部署小狼毫输入法
- 单实例运行保护

## 快速开始

首次运行程序会启动配置向导，选择方案版本（基础版/增强版）和辅助码类型。

也可以手动编辑配置文件 `updater_conf.ini`：

```ini
[general]
mirror = "gh-proxy.com"

[repositories]
schema_repo = "amzxyz/rime_wanxiang"
dict_repo = "amzxyz/rime_wanxiang"
model_repo = "amzxyz/RIME-LMDG"
self_repo = "Mikachu2333/rime_wanxiang_updater"

[files]
schema_type = "base"
dict_tag = "dict-nightly"
model_tag = "LTS"
model_file_name = "wanxiang-lts-zh-hans.gram"
```

运行程序：

```powershell
cargo run
```

## 系统要求

- Windows 系统（Win10以上）
- 已安装小狼毫输入法（提供 curl.exe 和 7z.exe）
- PowerShell 支持

## 构建

```bash
cargo build --release
```
