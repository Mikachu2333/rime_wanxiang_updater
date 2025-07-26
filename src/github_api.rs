use serde::{Deserialize, Serialize};

/// GitHub API交互模块

/// GitHub发布信息结构
#[derive(Debug, Deserialize, Serialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub published_at: String,
    pub body: String,
    pub assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitHubApiError {
    pub message: String,
    pub documentation_url: Option<String>,
}

/// 获取GitHub仓库的最新发布信息
pub fn get_latest_release(_repo: &str) -> Result<GitHubRelease, Box<dyn std::error::Error>> {
    // 这个函数将被GitHubClient使用
    // 实际的实现在update_checker/github_client.rs中
    Err("此函数已被重构到update_checker模块中".into())
}
