/// 清理镜像域名格式，移除协议前缀和路径后缀
pub fn sanitize_mirror_domain(input: &str) -> String {
    let mut domain = input.trim().to_string();

    if domain.starts_with("https://") {
        domain = domain.strip_prefix("https://").unwrap().to_string();
    } else if domain.starts_with("http://") {
        domain = domain.strip_prefix("http://").unwrap().to_string();
    }

    if let Some(slash_pos) = domain.find('/') {
        domain = domain[..slash_pos].to_string();
    }

    if domain.is_empty() {
        println!("警告: mirror 配置为空，使用默认值 gh-proxy.com");
        return "gh-proxy.com".to_string();
    }

    if domain.contains(' ') || domain.contains('\t') {
        println!("警告: mirror 配置包含非法字符，使用默认值 gh-proxy.com");
        return "gh-proxy.com".to_string();
    }

    if input != domain {
        println!("Mirror 配置已清理: '{}' -> '{}'", input, domain);
    }
    domain
}

/// 清理仓库URL格式
pub fn sanitize_repo_url(input: &str) -> String {
    let mut url = input.trim().to_string();

    while url.ends_with('/') {
        url.pop();
    }

    if !url.starts_with("http") {
        if url.contains('/') && !url.contains("github.com") {
            url = format!("https://github.com/{}", url);
        }
    }
    dbg!(&url);
    url
}

/// 从GitHub URL解析出 owner 和 repo 名称
pub fn parse_github_repo(repo_url: &str) -> Option<(String, String)> {
    let url = repo_url.trim();

    if let Some(github_part) = url.strip_prefix("https://github.com/") {
        let parts: Vec<&str> = github_part.split('/').collect();
        if parts.len() >= 2 && !parts[0].is_empty() && !parts[1].is_empty() {
            return Some((parts[0].to_string(), parts[1].to_string()));
        }
    }

    if !url.contains("://") && url.contains('/') {
        let parts: Vec<&str> = url.split('/').collect();
        if parts.len() >= 2 && !parts[0].is_empty() && !parts[1].is_empty() {
            return Some((parts[0].to_string(), parts[1].to_string()));
        }
    }

    None
}

/// 构建GitHub Releases API URL
pub fn build_releases_api_url(owner: &str, repo: &str) -> String {
    format!("https://api.github.com/repos/{}/{}/releases", owner, repo)
}

/// 构建GitHub Releases Tag API URL
#[allow(dead_code)]
pub fn build_releases_tag_api_url(owner: &str, repo: &str, tag: &str) -> String {
    format!(
        "https://api.github.com/repos/{}/{}/releases/tags/{}",
        owner, repo, tag
    )
}

/// 应用镜像前缀到下载URL
pub fn apply_mirror_to_download_url(mirror: &str, url: &str) -> String {
    if mirror.is_empty() {
        url.to_string()
    } else {
        format!("https://{}/{}", mirror, url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_mirror_domain() {
        assert_eq!(
            sanitize_mirror_domain("https://gh-proxy.com"),
            "gh-proxy.com"
        );
        assert_eq!(
            sanitize_mirror_domain("http://gh-proxy.com"),
            "gh-proxy.com"
        );
        assert_eq!(
            sanitize_mirror_domain("https://gh-proxy.com/"),
            "gh-proxy.com"
        );
        assert_eq!(sanitize_mirror_domain("gh-proxy.com"), "gh-proxy.com");
        assert_eq!(sanitize_mirror_domain(""), "gh-proxy.com");
    }

    #[test]
    fn test_sanitize_repo_url() {
        assert_eq!(
            sanitize_repo_url("https://github.com/amzxyz/rime_wanxiang/"),
            "https://github.com/amzxyz/rime_wanxiang"
        );

        assert_eq!(
            sanitize_repo_url("amzxyz/rime_wanxiang"),
            "https://github.com/amzxyz/rime_wanxiang"
        );

        assert_eq!(
            sanitize_repo_url("https://github.com/amzxyz/rime_wanxiang"),
            "https://github.com/amzxyz/rime_wanxiang"
        );
    }

    #[test]
    fn test_parse_github_repo() {
        assert_eq!(
            parse_github_repo("https://github.com/amzxyz/rime_wanxiang"),
            Some(("amzxyz".to_string(), "rime_wanxiang".to_string()))
        );

        assert_eq!(
            parse_github_repo("https://github.com/amzxyz/rime_wanxiang/"),
            Some(("amzxyz".to_string(), "rime_wanxiang".to_string()))
        );

        assert_eq!(
            parse_github_repo("amzxyz/rime_wanxiang"),
            Some(("amzxyz".to_string(), "rime_wanxiang".to_string()))
        );

        assert_eq!(parse_github_repo("invalid"), None);
        assert_eq!(parse_github_repo(""), None);
        assert_eq!(parse_github_repo("https://github.com/"), None);
    }

    #[test]
    fn test_build_api_urls() {
        assert_eq!(
            build_releases_api_url("amzxyz", "rime_wanxiang"),
            "https://api.github.com/repos/amzxyz/rime_wanxiang/releases"
        );

        assert_eq!(
            build_releases_tag_api_url("amzxyz", "RIME-LMDG", "LTS"),
            "https://api.github.com/repos/amzxyz/RIME-LMDG/releases/tags/LTS"
        );
    }

    #[test]
    fn test_apply_mirror_to_download_url() {
        let original_url =
            "https://github.com/amzxyz/rime_wanxiang/releases/download/test/file.zip";

        assert_eq!(
            apply_mirror_to_download_url("gh-proxy.com", original_url),
            "https://gh-proxy.com/https://github.com/amzxyz/rime_wanxiang/releases/download/test/file.zip"
        );

        assert_eq!(
            apply_mirror_to_download_url("github.com", original_url),
            original_url
        );

        assert_eq!(apply_mirror_to_download_url("", original_url), original_url);
    }
}
