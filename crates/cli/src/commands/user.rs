use std::path::Path;
use std::str::FromStr;
use vagent_core::{load_spec, save_spec, Protocol, Transport};

fn load_or_exit(config: &Path) -> vagent_core::Spec {
    match load_spec(config) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("加载配置失败 {}: {e}", config.display());
            std::process::exit(1);
        }
    }
}

/// reality=true 时传输强制 tcp(Reality 仅支持 tcp)。
pub fn add(
    config: &Path,
    name: &str,
    port: u16,
    protocol: &str,
    transport: &str,
) -> anyhow::Result<()> {
    let mut spec = load_or_exit(config);
    let proto = Protocol::from_str(protocol).map_err(|e| anyhow::anyhow!(e))?;
    let mut t = Transport::from_str(transport).map_err(|e| anyhow::anyhow!(e))?;
    // VLESS 默认走 Reality;其他协议不启用 reality。
    let reality = matches!(proto, Protocol::Vless);
    if reality {
        t = Transport::Tcp; // Reality 仅 tcp
    }
    spec.users.push(vagent_core::User::new(
        name,
        proto.clone(),
        port,
        reality,
        t.clone(),
    ));
    save_spec(&spec, config)?;
    let suffix = if reality { " (Reality)" } else { "" };
    println!("已新增用户 {name} (端口 {port}, {proto} {t}{suffix})");
    Ok(())
}

pub fn list(config: &Path) -> anyhow::Result<()> {
    let spec = load_or_exit(config);
    if spec.users.is_empty() {
        println!("(无用户)");
        return Ok(());
    }
    println!(
        "{:<16} {:<10} {:<6} {:<6} UUID",
        "NAME", "PROTOCOL", "PORT", "TRANS"
    );
    for u in &spec.users {
        println!(
            "{:<16} {:<10} {:<6} {:<6} {}",
            u.name, u.protocol, u.port, u.transport, u.uuid
        );
    }
    Ok(())
}

pub fn del(config: &Path, name: &str) -> anyhow::Result<()> {
    let mut spec = load_or_exit(config);
    let n = spec.remove_user(name);
    if n == 0 {
        eprintln!("未找到用户: {name}");
        std::process::exit(1);
    }
    save_spec(&spec, config)?;
    println!("已删除用户 {name} ({n} 条)");
    Ok(())
}

pub fn link(config: &Path, name: &str) -> anyhow::Result<()> {
    let spec = load_or_exit(config);
    let user = match spec.users.iter().find(|u| u.name == name) {
        Some(u) => u,
        None => {
            eprintln!("未找到用户: {name}");
            std::process::exit(1);
        }
    };
    let l = vagent_core::subscribe::gen_user(user, &spec).map_err(|e| anyhow::anyhow!(e))?;
    println!("{l}");
    Ok(())
}
