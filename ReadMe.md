# RIME 万象更新器

一个自动检查和更新 RIME 万象输入法组件的工具。

## 功能特性

- 🔍 自动检查字典、模型和方案文件的更新
- 📦 支持 GitHub 镜像加速下载
- 💾 本地缓存机制，避免重复检查
- ⚡ 使用 curl 进行网络请求，无需额外运行时
- 🔧 可配置的更新源和文件

## 快速开始

1. **配置文件**：编辑 `config.txt`

   ```ini
   # GitHub 镜像域名 (可选)
   mirror=gh-proxy.com
   
   # 仓库 URL
   repo_url=https://github.com/amzxyz/rime_wanxiang
   
   # 更新标签
   dict_releases_tag=dict-nightly
   model_tag=LTS
   model_file_name=wanxiang-lts-zh-hans.gram
   ```

2. **运行程序**：

   ```bash
   cargo run
   ```

## 组件说明

程序会检查以下组件的更新：

- **字典文件**：基于 `dict_releases_tag` 标签检查
- **语言模型**：基于 `model_tag` 标签检查  
- **方案文件**：从最新发布中查找

每个组件都有独立的更新时间追踪，互不影响。

## 依赖项

- `serde` - JSON 序列化/反序列化
- 系统要求：curl.exe (通过小狼毫安装包提供)

## 构建

```bash
cargo build --release
```

编译后的程序无需额外的异步运行时，体积小巧。
