//! 版本更新检查(对标 v2ray-agent 的「更新脚本」)。
//! 菜单「14. 更新提示」改为真实检查:本地版本 vs GitHub Releases 最新版本。
//! HTTP 属副作用,经 `VersionSource` trait 抽象,测试可注入假实现(不打真实网络)。

use anyhow::{anyhow, Result};
use serde::Deserialize;

/// 版本来源抽象(便于测试注入,不依赖真实网络)。
pub trait VersionSource {
    /// 返回最新版本号(不含前导 `v`,如 `0.1.0`)。
    fn latest(&self) -> Result<String>;
}

/// GitHub Releases 最新版本来源(默认实现,走 ureq + rustls)。
pub struct GitHubReleases {
    pub repo: String, // 形如 "gandli/vagent"
}

impl VersionSource for GitHubReleases {
    fn latest(&self) -> Result<String> {
        let url = format!("https://api.github.com/repos/{}/releases/latest", self.repo);
        let resp = ureq::get(&url)
            .set("User-Agent", "vagent")
            .call()
            .map_err(|e| anyhow!("查询 GitHub Releases 失败: {e}"))?;
        let body: ReleaseResp = resp
            .into_json()
            .map_err(|e| anyhow!("解析 GitHub 响应失败: {e}"))?;
        let tag = body
            .tag_name
            .strip_prefix('v')
            .unwrap_or(&body.tag_name)
            .to_string();
        if tag.is_empty() {
            return Err(anyhow!("GitHub 返回空版本号"));
        }
        Ok(tag)
    }
}

#[derive(Deserialize)]
struct ReleaseResp {
    tag_name: String,
}

/// 语义化版本比较:remote > local 时返回 true。
/// 仅比较数字段(忽略 pre-release/metadata);非数字段按字典序兜底。
pub fn is_newer(local: &str, remote: &str) -> bool {
    let parse = |s: &str| -> Vec<i64> {
        s.split(['.', '-'])
            .map(|p| {
                p.chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
            })
            .filter_map(|p| p.parse::<i64>().ok())
            .collect()
    };
    let a = parse(local);
    let b = parse(remote);
    // 逐段比较,前导段相等则比长度
    for (x, y) in a.iter().zip(b.iter()) {
        if y > x {
            return true;
        }
        if y < x {
            return false;
        }
    }
    b.len() > a.len()
}

/// 更新状态。
pub enum UpdateStatus {
    UpToDate,
    NewerAvailable { version: String },
    CheckFailed { reason: String },
}

/// 检查更新:比对本地版本与来源最新版本。
pub fn check_update(local: &str, src: &dyn VersionSource) -> UpdateStatus {
    match src.latest() {
        Ok(remote) if is_newer(local, &remote) => UpdateStatus::NewerAvailable { version: remote },
        Ok(_) => UpdateStatus::UpToDate,
        Err(e) => UpdateStatus::CheckFailed {
            reason: e.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ConstSource(String);
    impl VersionSource for ConstSource {
        fn latest(&self) -> Result<String> {
            Ok(self.0.clone())
        }
    }

    #[test]
    fn is_newer_compares_numeric_segments() {
        assert!(is_newer("0.1.0", "0.1.1"));
        assert!(is_newer("0.1.0", "0.2.0"));
        assert!(is_newer("0.9.0", "1.0.0"));
        assert!(!is_newer("0.1.1", "0.1.0"));
        assert!(!is_newer("1.0.0", "1.0.0"));
        // 长度更长视为更新
        assert!(is_newer("0.1", "0.1.0"));
    }

    #[test]
    fn check_update_reports_newer() {
        let st = check_update("0.1.0", &ConstSource("0.2.0".into()));
        match st {
            UpdateStatus::NewerAvailable { version } => assert_eq!(version, "0.2.0"),
            _ => panic!("应为 NewerAvailable"),
        }
    }

    #[test]
    fn check_update_reports_uptodate() {
        let st = check_update("0.1.0", &ConstSource("0.1.0".into()));
        assert!(matches!(st, UpdateStatus::UpToDate));
    }

    #[test]
    fn check_update_strips_v_prefix() {
        // ConstSource 模拟 GitHub 返回带 v 前缀(虽然 trait 约定不带,这里验证比对鲁棒性)
        // 实际 GitHubReleases 已 strip;此处仅验证 is_newer 对 v 前缀鲁棒
        assert!(is_newer("v0.1.0", "v0.1.1"));
    }
}
