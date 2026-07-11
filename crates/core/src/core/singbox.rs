//! sing-box 实现(MVP 占位:render 仅 direct 出站,install/reload 命令同构)。

use crate::core::{ProxyCore, Rendered};
use crate::executor::Cmd;
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
        Cmd::new("curl").args([
            "-L",
            "-o",
            "/tmp/singbox.tar.gz",
            &format!(
                "https://github.com/SagerNet/sing-box/releases/download/v{ver}/sing-box-{ver}-linux-amd64.tar.gz",
                ver = version
            ),
        ])
    }

    fn reload_cmd(&self) -> Cmd {
        Cmd::new("systemctl").args(["restart", "vagent-singbox"])
    }
}
