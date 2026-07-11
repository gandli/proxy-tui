use vagent_core::core::{ProxyCore, XrayCore};
use vagent_core::executor::RealExecutor;

/// 安装内核二进制(MVP 默认 Xray)。
/// 经 RealExecutor 真下载;sha256 校验留待后续版本补全。
pub fn run(core: &str, version: &str) -> anyhow::Result<()> {
    match core {
        "xray" => {
            XrayCore.install(version, &RealExecutor)?;
            println!("xray v{version} 安装命令已执行");
        }
        "singbox" => {
            println!("singbox 安装将在 Phase 2 实现");
        }
        other => {
            eprintln!("未知内核: {other}");
            std::process::exit(1);
        }
    }
    Ok(())
}
