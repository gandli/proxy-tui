//! Xray-core 实现。

use crate::core::{ProxyCore, Rendered};
use crate::executor::Cmd;
use crate::render::xray;
use crate::spec::Spec;
use crate::Error;

pub struct XrayCore;

impl ProxyCore for XrayCore {
    fn id(&self) -> &'static str {
        "xray"
    }

    fn render(&self, spec: &Spec) -> Result<Rendered, Error> {
        Ok(Rendered {
            path: "/etc/vagent/cores/xray/config.json".to_string(),
            content: xray::render_string(spec)?,
        })
    }

    fn install_cmd(&self, version: &str) -> Cmd {
        // MVP:自管下载 + 校验(实际 sha256 校验在 install 流程中由调用方处理)
        Cmd::new("curl").args([
            "-L",
            "-o",
            "/tmp/xray.zip",
            &format!(
                "https://github.com/XTLS/Xray-core/releases/download/v{ver}/Xray-linux-64.zip",
                ver = version
            ),
        ])
    }

    fn reload_cmd(&self) -> Cmd {
        Cmd::new("systemctl").args(["restart", "vagent-xray"])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::FakeExecutor;
    use crate::spec::Spec;

    #[test]
    fn render_path_is_isolated() {
        let r = XrayCore.render(&Spec::default_for("x.com")).unwrap();
        assert_eq!(r.path, "/etc/vagent/cores/xray/config.json");
        assert!(r.content.contains("freedom"));
    }

    #[test]
    fn install_cmd_targets_release_zip() {
        let c = XrayCore.install_cmd("1.8.0");
        assert_eq!(c.program, "curl");
        assert!(c.display().contains("Xray-core/releases/download/v1.8.0"));
    }

    #[test]
    fn install_failure_propagates() {
        let ex = FakeExecutor::new().expect("curl", crate::executor::ExecOutput::failure(1, "404"));
        let r = XrayCore.install("1.8.0", &ex);
        assert!(r.is_err());
    }

    #[test]
    fn reload_via_executor() {
        let ex = FakeExecutor::new().expect("systemctl", crate::executor::ExecOutput::success(""));
        XrayCore.reload(&ex).unwrap();
    }
}
