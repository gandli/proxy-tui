//! sing-box 实现:render + 完整安装(下载→解压→放置,三步走 Executor)。

use crate::core::{ProxyCore, Rendered};
use crate::executor::{Cmd, Executor};
use crate::render::singbox;
use crate::spec::Spec;
use crate::Error;
use std::path::Path;

pub struct SingboxCore;

impl ProxyCore for SingboxCore {
    fn id(&self) -> &'static str {
        "singbox"
    }

    fn render(&self, spec: &Spec, config: &Path) -> Result<Rendered, Error> {
        let base_dir = Spec::base_dir(config);
        let path = base_dir.join("cores").join("singbox").join("config.json");
        Ok(Rendered {
            path: path.to_string_lossy().to_string(),
            content: singbox::render_string(spec, &base_dir)?,
        })
    }

    fn install_cmd(&self, version: &str) -> Cmd {
        // 下载 sing-box 发行包(资产名无 v 前缀)
        let url = format!(
            "https://github.com/SagerNet/sing-box/releases/download/v{ver}/sing-box-{ver}-linux-amd64.tar.gz",
            ver = version
        );
        Cmd::new("curl").args(["-fsSL", "-o", "/tmp/sing-box.tar.gz", &url])
    }

    fn reload_cmd(&self) -> Cmd {
        Cmd::new("systemctl").args(["restart", "vagent-singbox"])
    }

    /// 重写安装:下载 → 解压 → 放置(三步走 Executor)。
    fn install(&self, version: &str, ex: &dyn Executor) -> Result<(), Error> {
        let out = ex.run(&self.install_cmd(version))?;
        if !out.ok() {
            return Err(Error::Render(format!("sing-box download failed: {}", out.stderr)));
        }
        let out = ex.run(&Cmd::new("tar").args(["xzf", "/tmp/sing-box.tar.gz"]))?;
        if !out.ok() {
            return Err(Error::Render(format!("sing-box extract failed: {}", out.stderr)));
        }
        let dest = if crate::systemd::is_root() {
            "/usr/local/bin".to_string()
        } else {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            std::path::PathBuf::from(home)
                .join(".local")
                .join("bin")
                .to_string_lossy()
                .to_string()
        };
        let place = Cmd::new("sh").args([
            "-c",
            &format!("mkdir -p {d} && mv sing-box-*/sing-box {d}/sing-box", d = dest),
        ]);
        let out = ex.run(&place)?;
        if !out.ok() {
            return Err(Error::Render(format!("sing-box place failed: {}", out.stderr)));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::{take_history, ExecOutput, FakeExecutor};

    #[test]
    fn install_cmd_targets_release_tarball() {
        let c = SingboxCore.install_cmd("1.10.0");
        assert_eq!(c.program, "curl");
        let d = c.display();
        assert!(d.contains("sing-box-1.10.0-linux-amd64.tar.gz"));
        // 资产名无 v 前缀
        assert!(!d.contains("sing-box-v1.10.0"));
    }

    #[test]
    fn install_runs_three_steps_via_executor() {
        let ex = FakeExecutor::new()
            .expect("curl", ExecOutput::success(""))
            .expect("tar", ExecOutput::success(""))
            .expect("sh", ExecOutput::success(""));
        assert!(SingboxCore.install("1.10.0", &ex).is_ok());
        let h = take_history();
        assert!(h.iter().any(|c| c.program == "curl"));
        assert!(h.iter().any(|c| c.program == "tar"));
        assert!(h.iter().any(|c| c.program == "sh"));
    }

    #[test]
    fn install_failure_propagates() {
        let ex = FakeExecutor::new().expect("curl", ExecOutput::failure(1, "404"));
        assert!(SingboxCore.install("1.10.0", &ex).is_err());
    }
}
