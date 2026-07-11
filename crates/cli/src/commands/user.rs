use std::path::Path;
use vagent_core::{load_spec, save_spec, Protocol};

pub fn add(config: &Path, name: &str, port: u16) -> anyhow::Result<()> {
    let mut spec = match load_spec(config) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("加载配置失败 {}: {e}", config.display());
            std::process::exit(1);
        }
    };
    spec.add_user(name, Protocol::Vless, port, true);
    save_spec(&spec, config)?;
    println!("已新增用户 {name} (端口 {port}, VLESS+Reality)");
    Ok(())
}
