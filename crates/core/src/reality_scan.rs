//! Reality SNI 扫描(对齐 v2ray-agent 的 RealiTLScanner)。
//! 下载扫描器 → 对本机公网 IP 扫可用 SNI → 写结果供 Reality 用。
//! 命令拼装纯函数,经 Executor 执行。

use crate::executor::{Cmd, Executor};
use crate::spec::Spec;
use crate::Error;
use std::path::Path;

/// 构造下载命令(GitHub release 的 linux-64 二进制)。base_dir 推导落地目录(支持普通用户)。
/// 构造下载命令(GitHub release 的 linux-64 二进制)。config 推导落地目录(支持普通用户)。
pub fn download_cmd(config: &Path) -> Cmd {
    let base_dir = Spec::base_dir(config);
    let dir = base_dir.join("reality_scan").to_string_lossy().to_string();
    let url =
        "https://github.com/XTLS/RealiTLScanner/releases/download/latest/RealiTLScanner-linux-64";
    Cmd::new("wget").args(["-c", "-q", "-P", &dir, url])
}

/// 构造扫描命令(对公网 IP 扫 SNI)。
pub fn scan_cmd(public_ip: &str, config: &Path) -> Cmd {
    let base_dir = Spec::base_dir(config);
    let bin = base_dir
        .join("reality_scan")
        .join("RealiTLScanner-linux-64")
        .to_string_lossy()
        .to_string();
    Cmd::new(bin).args(["-addr", public_ip])
}

/// 解析扫描输出,提取可用 SNI(每行一个域名)。
pub fn parse_results(output: &str) -> Vec<String> {
    output
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.to_string())
        .collect()
}

/// 下载扫描器(经 Executor)。
pub fn download(config: &Path, ex: &dyn Executor) -> Result<(), Error> {
    let out = ex.run(&download_cmd(config))?;
    if out.ok() {
        Ok(())
    } else {
        Err(Error::Render(format!(
            "RealiTLScanner download failed: {}",
            out.stderr
        )))
    }
}

/// 扫描可用 SNI(经 Executor),返回域名列表。
pub fn scan(public_ip: &str, config: &Path, ex: &dyn Executor) -> Result<Vec<String>, Error> {
    let out = ex.run(&scan_cmd(public_ip, config))?;
    if out.ok() {
        Ok(parse_results(&out.stdout))
    } else {
        Err(Error::Render(format!(
            "RealiTLScanner scan failed: {}",
            out.stderr
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::{ExecOutput, FakeExecutor};
    use std::path::Path;

    #[test]
    fn download_cmd_targets_release() {
        let c = download_cmd(Path::new("/etc/vagent/spec.toml"));
        assert_eq!(c.program, "wget");
        let d = c.display();
        assert!(d.contains("RealiTLScanner-linux-64"));
        assert!(d.contains("/etc/vagent/reality_scan"));
    }

    #[test]
    fn scan_cmd_passes_addr() {
        let c = scan_cmd("1.2.3.4", Path::new("/etc/vagent/spec.toml"));
        assert!(c.display().contains("-addr 1.2.3.4"));
    }

    #[test]
    fn parse_results_skips_comments() {
        let out = "# header\nwww.a.com\n# comment\nmail.b.com\n\n";
        let r = parse_results(out);
        assert_eq!(r, vec!["www.a.com", "mail.b.com"]);
    }

    #[test]
    fn download_failure_propagates() {
        let ex = FakeExecutor::new().expect("wget", ExecOutput::failure(1, "404"));
        assert!(download(Path::new("/etc/vagent/spec.toml"), &ex).is_err());
    }

    #[test]
    fn scan_via_executor_returns_domains() {
        let ex = FakeExecutor::new().expect(
            "/etc/vagent/reality_scan/RealiTLScanner-linux-64",
            ExecOutput::success("www.x.com\napi.y.com\n"),
        );
        let r = scan("9.9.9.9", Path::new("/etc/vagent/spec.toml"), &ex).unwrap();
        assert_eq!(r, vec!["www.x.com", "api.y.com"]);
    }
}
