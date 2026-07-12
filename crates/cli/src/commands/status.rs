use std::path::Path;
use vagent_core::{load_spec, Error};

pub fn run(config: &Path) -> anyhow::Result<()> {
    match load_spec(config) {
        Ok(spec) => {
            println!("domain : {}", spec.domain);
            println!(
                "cores  : xray={} singbox={}",
                spec.cores.xray, spec.cores.singbox
            );
            println!("users  : {}", spec.users.len());
            for u in &spec.users {
                let proto = format!("{:?}", u.protocol).to_lowercase();
                println!(
                    "  - {} [{}] port={} reality={}",
                    u.name, proto, u.port, u.reality
                );
            }
            Ok(())
        }
        Err(Error::Io(_)) => {
            eprintln!("config not found: {}", config.display());
            // 返回 Err 让 main 统一非零退出码(domain-cli: 不裸 exit)
            Err(anyhow::anyhow!("配置文件不存在: {}", config.display()))
        }
        Err(e) => Err(e.into()),
    }
}
