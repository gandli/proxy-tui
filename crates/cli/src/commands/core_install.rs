use vagent_core::core::{ProxyCore, SingboxCore, XrayCore};
use vagent_core::executor::RealExecutor;

/// 安装内核二进制。经 RealExecutor 真下载(三步走:下载→解压→放置)。
pub fn run(core: &str, version: &str) -> anyhow::Result<()> {
    match core {
        "xray" => {
            XrayCore.install(version, &RealExecutor)?;
            println!("xray v{version} 安装完成(/usr/local/bin/xray 或 ~/.local/bin/xray)");
        }
        "singbox" => {
            SingboxCore.install(version, &RealExecutor)?;
            println!("sing-box v{version} 安装完成(/usr/local/bin 或 ~/.local/bin)");
        }
        other => {
            eprintln!("未知内核: {other}");
            std::process::exit(1);
        }
    }
    Ok(())
}
