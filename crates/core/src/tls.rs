//! TLS 自动(仅非 Reality 协议需要)。
//! 本模块只拼装 acme.sh 命令 + 续期判断,不真跑。证书落 /etc/vagent/certs/。

use crate::executor::{Cmd, Executor};
use crate::Error;

pub const CERT_DIR: &str = "/etc/vagent/certs";

/// 构造签发命令(acme.sh,standalone 模式)。
pub fn issue_cmd(domain: &str) -> Cmd {
    Cmd::new("acme.sh").args([
        "--issue",
        "-d",
        domain,
        "--standalone",
        "-k",
        "ec-256",
        "--cert-file",
        &format!("{CERT_DIR}/{domain}.cer"),
        "--key-file",
        &format!("{CERT_DIR}/{domain}.key"),
    ])
}

/// 构造续期命令。
pub fn renew_cmd() -> Cmd {
    Cmd::new("acme.sh").args(["--cron", "--home", "/root/.acme.sh"])
}

/// 执行签发(经 Executor)。
pub fn issue(domain: &str, ex: &dyn Executor) -> Result<(), Error> {
    let out = ex.run(&issue_cmd(domain))?;
    if out.ok() {
        Ok(())
    } else {
        Err(Error::Render(format!(
            "acme.sh issue failed: {}",
            out.stderr
        )))
    }
}

/// 判断证书是否临近到期(占位:<30 天需续)。真实实现解析 x509 有效期。
pub fn needs_renew(_cert_path: &str) -> Result<bool, Error> {
    // MVP 占位:真实逻辑用 openssl x509 -enddate 解析。
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::{ExecOutput, FakeExecutor};

    #[test]
    fn issue_cmd_targets_domain() {
        let c = issue_cmd("v.example.com");
        assert_eq!(c.program, "acme.sh");
        assert!(c.display().contains("-d v.example.com"));
        assert!(c.display().contains("/etc/vagent/certs/v.example.com.cer"));
    }

    #[test]
    fn issue_failure_propagates() {
        let ex = FakeExecutor::new().expect("acme.sh", ExecOutput::failure(1, "dnserr"));
        assert!(issue("x.com", &ex).is_err());
    }

    #[test]
    fn issue_success_ok() {
        let ex = FakeExecutor::new().expect("acme.sh", ExecOutput::success("issued"));
        assert!(issue("x.com", &ex).is_ok());
    }
}
