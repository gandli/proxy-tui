//! nginx 安装与 reload(占 443 反代本机 xray/sing-box)。
//! root VPS 标准路径:nginx 以 root 持有 443,xray 绑高位端口(8443),由 nginx 反代进来。
//! 命令经 RealExecutor 真实执行(apt/apk + systemctl reload),需在 root 环境。

use vagent_core::executor::{Cmd, Executor, RealExecutor};
use vagent_core::systemd::{self, InitSystem};

/// 检测包管理器:Debian/Ubuntu → apt;Alpine → apk。
fn detect_pkg() -> &'static str {
    if std::path::Path::new("/etc/os-release").exists() {
        let content = std::fs::read_to_string("/etc/os-release").unwrap_or_default();
        if content.contains("ID=alpine") {
            return "apk";
        }
    }
    "apt"
}

/// 安装 nginx(系统包),并 enable。
pub fn install() -> anyhow::Result<()> {
    let pkg = detect_pkg();
    let ex = RealExecutor;
    let install_cmd = match pkg {
        "apk" => Cmd::new("apk").args(["add", "nginx"]),
        _ => Cmd::new("apt-get").args(["install", "-y", "nginx"]),
    };
    println!("安装 nginx ({pkg}) ...");
    let out = ex.run(&install_cmd)?;
    if !out.ok() {
        return Err(anyhow::anyhow!("nginx 安装失败: {}", out.stderr));
    }
    // enable + 启动(按 init 系统)
    let init = if std::path::Path::new("/sbin/openrc").exists() {
        InitSystem::Openrc
    } else {
        InitSystem::Systemd
    };
    if init == InitSystem::Systemd {
        let _ = ex.run(&Cmd::new("systemctl").args(["enable", "nginx"]));
        let _ = ex.run(&Cmd::new("systemctl").args(["start", "nginx"]));
    }
    println!("nginx 安装完成。用菜单\"nginx 管理\"生成反代配置后 reload。");
    Ok(())
}

/// reload nginx(应用新配置)。
pub fn reload() -> anyhow::Result<()> {
    let ex = RealExecutor;
    // 优先 nginx -s reload(无需 root 若 nginx 以当前用户跑);
    // 否则 systemctl reload(需 root,标准路径)。
    if systemd::is_root() {
        let out = ex.run(&Cmd::new("systemctl").args(["reload", "nginx"]))?;
        if out.ok() {
            println!("nginx reloaded (systemctl).");
            return Ok(());
        }
    }
    let out = ex.run(&Cmd::new("nginx").args(["-s", "reload"]))?;
    if out.ok() {
        println!("nginx reloaded (nginx -s reload).");
        Ok(())
    } else {
        Err(anyhow::anyhow!("nginx reload 失败: {}", out.stderr))
    }
}
