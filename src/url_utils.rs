/// URL工具函数
///
/// 解析GitHub仓库URL，提取用户名和仓库名
pub fn parse_github_repo(url: &str) -> Option<(String, String)> {
    // 支持多种格式：
    // - https://github.com/user/repo
    // - github.com/user/repo
    // - user/repo

    let url = url.trim().trim_end_matches('/');

    // 移除协议和域名前缀
    let path = if url.contains("github.com/") {
        url.split("github.com/").nth(1)?
    } else {
        url
    };

    // 分割用户名和仓库名
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 2 {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}

/// 清理和规范化仓库URL
pub fn sanitize_repo_url(url: &str) -> String {
    if let Some((user, repo)) = parse_github_repo(url) {
        format!("{}/{}", user, repo)
    } else {
        url.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_repo() {
        assert_eq!(
            parse_github_repo("https://github.com/user/repo"),
            Some(("user".to_string(), "repo".to_string()))
        );

        assert_eq!(
            parse_github_repo("github.com/user/repo"),
            Some(("user".to_string(), "repo".to_string()))
        );

        assert_eq!(
            parse_github_repo("user/repo"),
            Some(("user".to_string(), "repo".to_string()))
        );

        assert_eq!(parse_github_repo("invalid"), None);
    }

    #[test]
    fn test_sanitize_repo_url() {
        assert_eq!(
            sanitize_repo_url("https://github.com/user/repo"),
            "user/repo"
        );

        assert_eq!(sanitize_repo_url("invalid"), "invalid");
    }
}
