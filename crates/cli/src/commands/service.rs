//! 服务单元管理:生成并安装 systemd / openrc 单元。
//! init 默认 systemd;Alpine 用 openrc。路径从 config 推导(root-optional)。

use std::str::FromStr;
use vagent_core::spec::Spec;
use vagent_core::systemd::{self, InitSystem};

/// 二进制安装路径:root /usr/local/bin,普通用户 ~/.local/bin。
fn bin_path() -> String {
    if systemd::is_root() {
        "/usr/local/bin/vagent".to_string()
    } else {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        std::path::PathBuf::from(home)
            .join(".local")
            .join("bin")
            .join("vagent")
            .to_string_lossy()
            .to_string()
    }
}

/// 为指定内核生成并打印单元内容(不落盘)。
#[allow(dead_code)]
pub fn show(core: &str, init: &str) -> anyhow::Result<()> {
    let init = InitSystem::from_str(init).map_err(|e| anyhow::anyhow!(e))?;
    let config = Spec::default_config_path();
    let unit = if core == "api" {
        systemd::api_unit(&bin_path(), &config.to_string_lossy())
    } else {
        systemd::unit_for(init, core, &bin_path(), &config.to_string_lossy())
    };
    println!("{unit}");
    Ok(())
}

/// 生成并写入单元到系统/用户目录(路径从 config 推导)。
pub fn install(core: &str, init: &str) -> anyhow::Result<()> {
    let init = InitSystem::from_str(init).map_err(|e| anyhow::anyhow!(e))?;
    let config = Spec::default_config_path();
    let unit = if core == "api" {
        systemd::api_unit(&bin_path(), &config.to_string_lossy())
    } else {
        systemd::unit_for(init, core, &bin_path(), &config.to_string_lossy())
    };
    let base = Spec::base_dir(&config);
    let svc_core = if core == "api" { "api" } else { core };
    systemd::install_unit(init, svc_core, &unit, &base).map_err(|e| anyhow::anyhow!(e))?;
    println!(
        "已安装 {core} 单元 ({init:?}) → {}",
        systemd::unit_install_path(init, svc_core, &base).to_string_lossy()
    );
    Ok(())
}
