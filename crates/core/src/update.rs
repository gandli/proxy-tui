//! 版本更新检查(对标 v2ray-agent 的「更新脚本」)。
//! 菜单「14. 更新提示」改为真实检查:本地版本 vs GitHub Releases 最新版本。
//!
//! HTTP 请求经 `Executor` 抽象(调系统 `curl`),与 core 其他系统副作用一致 ——
//! 测试用 `FakeExecutor` 注入固定响应,不依赖真实网络,无新供应链依赖。

use crate::executor::{Cmd, Executor};
use anyhow::{anyhow, Result};
use serde::Deserialize;

/// 版本来源抽象(便于测试注入,不依赖真实网络)。
/// `ex` 用于执行 HTTP 请求(系统 curl),测试可传 `FakeExecutor`。
pub trait VersionSource {
    fn latest(&self, ex: &dyn Executor) -> Result<String>;
}

/// 经 `curl` 查 GitHub Releases 最新版本(默认实现,走 Executor,可测)。
pub struct GitHubReleases {
    pub repo: String, // 形如 "gandli/vagent"
}

impl VersionSource for GitHubReleases {
    fn latest(&self, ex: &dyn Executor) -> Result<String> {
        let url = format!("https://api.github.com/repos/{}/releases/latest", self.repo);
        // 用系统 curl(GitHub API 需 User-Agent,否则 403)
        let cmd = Cmd::new("curl").args(["-fsSL", "-H", "User-Agent: vagent", &url]);
        let out = ex
            .run(&cmd)
            .map_err(|e| anyhow!("查询 GitHub Releases 失败: {e}"))?;
        let body: ReleaseResp =
            serde_json::from_str(&out.stdout).map_err(|e| anyhow!("解析 GitHub 响应失败: {e}"))?;
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
pub fn check_update(local: &str, src: &dyn VersionSource, ex: &dyn Executor) -> UpdateStatus {
    match src.latest(ex) {
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
    use crate::executor::{ExecOutput, FakeExecutor};

    struct ConstSource(String);
    impl VersionSource for ConstSource {
        fn latest(&self, _ex: &dyn Executor) -> Result<String> {
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
        assert!(is_newer("0.1", "0.1.0"));
    }

    #[test]
    fn check_update_reports_newer() {
        let ex = FakeExecutor::new();
        let st = check_update("0.1.0", &ConstSource("0.2.0".into()), &ex);
        match st {
            UpdateStatus::NewerAvailable { version } => assert_eq!(version, "0.2.0"),
            _ => panic!("应为 NewerAvailable"),
        }
    }

    #[test]
    fn check_update_reports_uptodate() {
        let ex = FakeExecutor::new();
        let st = check_update("0.1.0", &ConstSource("0.1.0".into()), &ex);
        assert!(matches!(st, UpdateStatus::UpToDate));
    }

    #[test]
    fn github_releases_parses_tag_via_fake_executor() {
        // 用 FakeExecutor 模拟 curl 返回 GitHub Releases JSON
        let ex = FakeExecutor::new().expect(
            "curl",
            ExecOutput::success(r#"{"tag_name":"v0.2.0","name":"0.2.0"}"#),
        );
        let src = GitHubReleases {
            repo: "gandli/vagent".into(),
        };
        let v = src.latest(&ex).expect("应解析出版本");
        assert_eq!(v, "0.2.0");
    }
}
