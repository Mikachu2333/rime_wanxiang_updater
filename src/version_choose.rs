/// 版本选择和比较工具

use std::cmp::Ordering;

/// 版本信息结构
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
    pre_release: Option<String>,
}

impl Version {
    /// 从字符串解析版本号
    pub fn parse(version_str: &str) -> Option<Self> {
        let version_str = version_str.trim_start_matches('v');
        let parts: Vec<&str> = version_str.split('-').collect();

        let version_part = parts[0];
        let pre_release = if parts.len() > 1 {
            Some(parts[1..].join("-"))
        } else {
            None
        };

        let version_nums: Vec<&str> = version_part.split('.').collect();
        if version_nums.len() < 3 {
            return None;
        }

        Some(Version {
            major: version_nums[0].parse().ok()?,
            minor: version_nums[1].parse().ok()?,
            patch: version_nums[2].parse().ok()?,
            pre_release,
        })
    }

    /// 检查是否为预发布版本
    pub fn is_prerelease(&self) -> bool {
        self.pre_release.is_some()
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            other => return other,
        }

        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            other => return other,
        }

        match self.patch.cmp(&other.patch) {
            Ordering::Equal => {}
            other => return other,
        }

        // 预发布版本比正式版本小
        match (&self.pre_release, &other.pre_release) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(a), Some(b)) => a.cmp(b),
        }
    }
}

/// 选择最新的稳定版本
pub fn choose_latest_stable(versions: &[String]) -> Option<String> {
    let mut parsed_versions: Vec<(String, Version)> = versions
        .iter()
        .filter_map(|v| Version::parse(v).map(|parsed| (v.clone(), parsed)))
        .filter(|(_, version)| !version.is_prerelease())
        .collect();

    parsed_versions.sort_by(|(_, a), (_, b)| b.cmp(a));
    parsed_versions.first().map(|(original, _)| original.clone())
}

/// 选择最新版本（包括预发布版本）
pub fn choose_latest(versions: &[String]) -> Option<String> {
    let mut parsed_versions: Vec<(String, Version)> = versions
        .iter()
        .filter_map(|v| Version::parse(v).map(|parsed| (v.clone(), parsed)))
        .collect();

    parsed_versions.sort_by(|(_, a), (_, b)| b.cmp(a));
    parsed_versions.first().map(|(original, _)| original.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let v1 = Version::parse("1.2.3").unwrap();
        assert_eq!(v1.major, 1);
        assert_eq!(v1.minor, 2);
        assert_eq!(v1.patch, 3);
        assert_eq!(v1.pre_release, None);

        let v2 = Version::parse("v2.0.0-beta.1").unwrap();
        assert_eq!(v2.major, 2);
        assert_eq!(v2.minor, 0);
        assert_eq!(v2.patch, 0);
        assert_eq!(v2.pre_release, Some("beta.1".to_string()));
    }

    #[test]
    fn test_version_comparison() {
        let v1 = Version::parse("1.0.0").unwrap();
        let v2 = Version::parse("1.0.1").unwrap();
        let v3 = Version::parse("1.1.0").unwrap();
        let v4 = Version::parse("2.0.0-beta").unwrap();
        let v5 = Version::parse("2.0.0").unwrap();

        assert!(v2 > v1);
        assert!(v3 > v2);
        assert!(v5 > v4);
        assert!(v4 > v3);
    }
}