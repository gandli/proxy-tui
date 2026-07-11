//! 内核二进制下载:拼 release URL + 预期 sha256 校验(纯函数部分可单测)。
//! 真实下载/校验在 VPS 执行;这里产出结构,供 Executor 执行。

/// 下载规格:URL + 期望 sha256(可选)。
#[derive(Debug, Clone, PartialEq)]
pub struct DownloadSpec {
    pub url: String,
    pub dest: String,
    /// 期望的 sha256 hex,下载后比对;为空表示跳过校验。
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

/// 计算 sha256(用于校验已下载文件)。真实实现读文件分块哈希。
pub fn sha256_hex(_data: &[u8]) -> String {
    // MVP 占位:真实实现用 sha2::Sha256 分块
    String::new()
}

/// 比对下载哈希是否匹配预期。
pub fn verify_hash(actual: &str, expected: &str) -> bool {
    if expected.is_empty() {
        return true; // 未配置预期值,跳过
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
    fn verify_hash_empty_expected_passes() {
        assert!(verify_hash("abc", ""));
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
