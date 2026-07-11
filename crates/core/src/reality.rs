//! Reality 密钥对生成(对齐 v2ray-agent 的 `xray x25519`)。
//! 纯函数拼命令,经 Executor 执行拿公钥;私钥落盘由调用方管理(不入库)。

use crate::executor::{Cmd, Executor};
use crate::Error;

/// 构造 `xray x25519` 命令(纯函数)。
/// xray 二进制路径可配,默认 /usr/local/bin/xray。
pub fn x25519_cmd(xray_bin: &str) -> Cmd {
    Cmd::new(xray_bin).args(["x25519"])
}

/// 解析 xray x25519 输出,提取 Public key。
/// 输出形如:
///   Private key: ...
///   Public key: ...
pub fn parse_public_key(output: &str) -> Result<String, Error> {
    for line in output.lines() {
        if let Some(rest) = line.trim().strip_prefix("Public key:") {
            return Ok(rest.trim().to_string());
        }
    }
    Err(Error::Render("xray x25519 输出缺少 Public key".into()))
}

/// 生成 Reality 公钥(经 Executor)。
/// xray_bin 指向已安装的 xray 二进制。
pub fn generate_public_key(xray_bin: &str, ex: &dyn Executor) -> Result<String, Error> {
    let out = ex.run(&x25519_cmd(xray_bin))?;
    if !out.ok() {
        return Err(Error::Render(format!("xray x25519 failed: {}", out.stderr)));
    }
    parse_public_key(&out.stdout)
}

/// 生成短 ID(16 进制,长度 1-16)。用系统随机数,纯本地无需 Executor。
pub fn generate_short_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    // 取一个 8 字节随机感的 16 进制串(简单实现,无需外部依赖)
    format!("{:016x}", n.wrapping_mul(0x9E3779B97F4A7C15))
        .chars()
        .take(8)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::{ExecOutput, FakeExecutor};

    #[test]
    fn x25519_cmd_targets_binary() {
        let c = x25519_cmd("/usr/local/bin/xray");
        assert_eq!(c.program, "/usr/local/bin/xray");
        assert!(c.args.contains(&"x25519".to_string()));
    }

    #[test]
    fn parse_public_key_extracts() {
        let out = "Private key: abc\nPublic key: def123\n";
        assert_eq!(parse_public_key(out).unwrap(), "def123");
    }

    #[test]
    fn parse_public_key_missing_errors() {
        assert!(parse_public_key("no key here").is_err());
    }

    #[test]
    fn generate_public_key_via_executor() {
        let ex = FakeExecutor::new().expect(
            "/usr/local/bin/xray",
            ExecOutput::success("Private key: priv\nPublic key: pubkey123\n"),
        );
        assert_eq!(
            generate_public_key("/usr/local/bin/xray", &ex).unwrap(),
            "pubkey123"
        );
    }

    #[test]
    fn generate_public_key_failure_propagates() {
        let ex = FakeExecutor::new().expect(
            "/usr/local/bin/xray",
            ExecOutput::failure(1, "cmd not found"),
        );
        assert!(generate_public_key("/usr/local/bin/xray", &ex).is_err());
    }

    #[test]
    fn short_id_is_hex_8() {
        let s = generate_short_id();
        assert_eq!(s.len(), 8);
        assert!(s.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
