//! 伪装站（nginx SNI 反代）管理命令。
//! 对标 v2ray-agent 的「8. 伪装站管理」：在自有域名上建 Reality 伪站，
//! 把 SNI 反代到真实站点（流量特征伪装）。渲染逻辑在 core::render::nginx。

use std::path::{Path, PathBuf};
use vagent_core::render;

/// 生成伪装站 nginx 配置（SNI 反代 server block）。
/// 直接消费 core 已实现的 render::nginx::render，不在命令层重复渲染逻辑。
pub fn run(config: &Path) -> anyhow::Result<()> {
    let spec = vagent_core::load_spec(config)?;
    let block = render::nginx::render(&spec)?;
    // 写出到 spec 同目录的 nginx 片段（由部署时 include 进主配置）。
    let out = config
        .parent()
        .map(|p| p.join("nginx-sni-proxy.conf"))
        .unwrap_or_else(|| PathBuf::from("nginx-sni-proxy.conf"));
    std::fs::write(&out, &block)?;
    println!("已生成伪装站配置: {}", out.display());
    println!("（部署时将此文件 include 进 nginx 主配置，并配置证书后 reload）");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use vagent_core::Spec;

    #[test]
    fn run_generates_sni_proxy_conf() {
        let dir = tempfile::tempdir().unwrap();
        let spec_path = dir.path().join("spec.toml");
        let spec = Spec::default_for("v.example.com");
        std::fs::write(&spec_path, toml::to_string(&spec).unwrap()).unwrap();

        // 命令层只调 render::nginx::render，不应崩溃且产出含域名的 server block
        run(&spec_path).unwrap();

        let out = dir.path().join("nginx-sni-proxy.conf");
        let cfg = std::fs::read_to_string(&out).unwrap();
        assert!(cfg.contains("v.example.com"));
        assert!(cfg.contains("proxy_pass https://v.example.com:443"));
    }
}
