//! 内核二进制下载与完整性校验。
//! 纯函数(URL 拼接、.dgst 解析、哈希计算)可单测;真实下载/校验由 Executor 执行(可远程)。

use crate::executor::Cmd;
use sha2::{Digest, Sha256};

/// 下载规格:URL + 期望 sha256(可选)。
#[derive(Debug, Clone, PartialEq)]
pub struct DownloadSpec {
    pub url: String,
    pub dest: String,
    /// 期望的 sha256 hex,下载后比对;为空表示尚未获取(需运行时从官方 .dgst 拉取)。
    pub expected_sha256: String,
}

/// 构造 Xray-core 下载规格。
pub fn xray(version: &str, dest: &str, expected_sha256: &str) -> DownloadSpec {
    DownloadSpec {
        url: format!(
            "https://github.com/XTLS/Xray-core/releases/download/v{ver}/Xray-linux-64.zip",
            ver = version
        ),
        dest: dest.to_string(),
        expected_sha256: expected_sha256.to_string(),
    }
}

/// 构造 sing-box 下载规格。
pub fn singbox(version: &str, dest: &str, expected_sha256: &str) -> DownloadSpec {
    DownloadSpec {
        url: format!(
            "https://github.com/SagerNet/sing-box/releases/download/v{ver}/sing-box-{ver}-linux-amd64.tar.gz",
            ver = version
        ),
        dest: dest.to_string(),
        expected_sha256: expected_sha256.to_string(),
    }
}

/// Xray-core 官方校验文件(.dgst)的 URL(与 zip 同目录,含 SHA2-256=)。
pub fn xray_dgst_url(version: &str) -> String {
    format!(
        "https://github.com/XTLS/Xray-core/releases/download/v{ver}/Xray-linux-64.zip.dgst",
        ver = version
    )
}

/// 从 Xray .dgst 文件内容解析 SHA2-256 hex。
/// .dgst 格式(每行 `ALGO= hex`):
/// ```text
/// MD5= ...
/// SHA1= ...
/// SHA2-256= 23cd9af9...
/// SHA2-512= ...
/// ```
pub fn parse_dgst_sha256(dgst: &str) -> Option<String> {
    for line in dgst.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("SHA2-256=") {
            let h = rest.trim().to_lowercase();
            if !h.is_empty() {
                return Some(h);
            }
        }
    }
    None
}

/// 生成"下载 .dgst + 校验本地文件 sha256"的 shell 命令(供 Executor 执行,可远程)。
/// 逻辑:拉取官方 .dgst → 提取 SHA2-256 → 与本地文件 sha256sum 比对;不符即 exit 1 中止。
/// 官方校验值与二进制同源,可防传输损坏/CDN 投毒/中间人替换(不防源站被攻破)。
pub fn verify_cmd(dgst_url: &str, local_file: &str) -> Cmd {
    let script = format!(
        "set -e; \
         expected=$(curl -fsSL '{dgst}' | grep '^SHA2-256=' | sed 's/^SHA2-256=//' | tr -d '[:space:]' | tr 'A-F' 'a-f'); \
         if [ -z \"$expected\" ]; then echo 'cannot fetch expected sha256 from dgst' >&2; exit 1; fi; \
         actual=$(sha256sum '{file}' | awk '{{print $1}}'); \
         if [ \"$expected\" != \"$actual\" ]; then echo \"sha256 mismatch: expected=$expected actual=$actual\" >&2; exit 1; fi; \
         echo \"sha256 verified: $actual\"",
        dgst = dgst_url,
        file = local_file
    );
    Cmd::new("sh").args(["-c", &script])
}

/// 计算 sha256 hex(纯 Rust,用于本地校验/单测)。
pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// 比对下载哈希是否匹配预期。空 expected 视为"未校验" → 返回 false(拒未验证兜底)。
pub fn verify_hash(actual: &str, expected: &str) -> bool {
    if expected.is_empty() || actual.is_empty() {
        return false;
    }
    actual.eq_ignore_ascii_case(expected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xray_url_targets_version() {
        let d = xray("1.8.0", "/tmp/x.zip", "");
        assert!(d.url.contains("Xray-core/releases/download/v1.8.0"));
        assert!(d.url.ends_with("Xray-linux-64.zip"));
        assert_eq!(d.dest, "/tmp/x.zip");
    }

    #[test]
    fn singbox_url_targets_version() {
        let d = singbox("1.9.0", "/tmp/s.tar.gz", "");
        assert!(d.url.contains("sing-box-1.9.0-linux-amd64.tar.gz"));
    }

    #[test]
    fn xray_dgst_url_matches_zip() {
        let u = xray_dgst_url("1.8.23");
        assert!(u.ends_with("Xray-linux-64.zip.dgst"));
        assert!(u.contains("download/v1.8.23"));
    }

    #[test]
    fn parse_dgst_extracts_sha256() {
        let dgst = "MD5= ee4e2ff7\nSHA1= b55b06e7\nSHA2-256= 23CD9AF937744D97\nSHA2-512= e8bc40a0";
        assert_eq!(
            parse_dgst_sha256(dgst),
            Some("23cd9af937744d97".to_string())
        );
    }

    #[test]
    fn parse_dgst_none_when_absent() {
        assert_eq!(parse_dgst_sha256("MD5= abc\nSHA1= def"), None);
    }

    #[test]
    fn verify_cmd_contains_dgst_and_file() {
        let c = verify_cmd(
            "https://example.com/Xray-linux-64.zip.dgst",
            "/tmp/xray.zip",
        );
        assert_eq!(c.program, "sh");
        let disp = c.display();
        assert!(disp.contains("Xray-linux-64.zip.dgst"));
        assert!(disp.contains("/tmp/xray.zip"));
        assert!(disp.contains("sha256sum"));
        assert!(disp.contains("mismatch"));
    }

    #[test]
    fn sha256_hex_known_vector() {
        // echo -n "" | sha256sum → e3b0c442...
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        // "abc"
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn verify_hash_empty_expected_now_fails() {
        // 拒未验证兜底:空 expected 不再放行
        assert!(!verify_hash("abc", ""));
        assert!(!verify_hash("", "abc"));
    }

    #[test]
    fn verify_hash_match_case_insensitive() {
        assert!(verify_hash("ABCD", "abcd"));
    }

    #[test]
    fn verify_hash_mismatch_fails() {
        assert!(!verify_hash("deadbeef", "cafe"));
    }
}
