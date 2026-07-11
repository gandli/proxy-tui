//! 内核生命周期管理:start/stop/restart/enable/disable。
//! 经 RealExecutor 调用 systemctl(真实副作用,需在 VPS 上具备 systemd)。

use vagent_core::core::{ProxyCore, SingboxCore, XrayCore};
use vagent_core::executor::RealExecutor;

fn pick(core: &str) -> anyhow::Result<Box<dyn ProxyCore>> {
    match core {
        "xray" => Ok(Box::new(XrayCore)),
        "singbox" => Ok(Box::new(SingboxCore)),
        other => Err(anyhow::anyhow!("未知内核: {other}(应为 xray / singbox)")),
    }
}

/// 对指定内核执行生命周期动作。
pub fn run(core: &str, action: &str) -> anyhow::Result<()> {
    let c = pick(core)?;
    match action {
        "start" | "stop" | "restart" | "enable" | "disable" => {
            c.lifecycle(action, &RealExecutor)
                .map_err(|e| anyhow::anyhow!(e))?;
            println!("{core} {action} 已执行");
        }
        other => return Err(anyhow::anyhow!("未知动作: {other}")),
    }
    Ok(())
}
