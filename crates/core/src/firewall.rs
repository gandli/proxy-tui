//! 防火墙端口段开放(对标 v2ray-agent dokodemo-door 端口跳跃的防火墙配置)。
//! vagent 不碰持久化系统调优,但端口跳跃需要防火墙开放 `start..=end` 段才能让客户端连入。
//! 此处只做「开放端口段」这一最小副作用,经 Executor 抽象(测试可注入 FakeExecutor)。

use crate::executor::{Cmd, Executor};
use crate::Error;

/// 防火墙后端。探测顺序:firewalld > ufw > iptables > None。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FirewallBackend {
    Firewalld,
    Ufw,
    Iptables,
    None,
}

/// 探测当前系统的防火墙后端(检查对应二进制是否存在)。
pub fn detect_backend(executor: &dyn Executor) -> FirewallBackend {
    // which 等价:直接尝试执行 --version,非零即不存在
    if executor
        .run(&Cmd::new("firewall-cmd").arg("--version"))
        .map(|o| o.ok())
        .unwrap_or(false)
    {
        return FirewallBackend::Firewalld;
    }
    if executor
        .run(&Cmd::new("ufw").arg("status"))
        .map(|o| o.ok())
        .unwrap_or(false)
    {
        return FirewallBackend::Ufw;
    }
    if executor
        .run(&Cmd::new("iptables").arg("--version"))
        .map(|o| o.ok())
        .unwrap_or(false)
    {
        return FirewallBackend::Iptables;
    }
    FirewallBackend::None
}

/// 开放 [start, end] 端口段(tcp)。按探测到的后端生成对应命令。
/// 无防火墙后端时不报错(仅提示,交给用户手动开放)。
pub fn open_port_range(start: u16, end: u16, executor: &dyn Executor) -> Result<(), Error> {
    let backend = detect_backend(executor);
    match backend {
        FirewallBackend::Firewalld => {
            // firewall-cmd --add-port=30000-31000/tcp --permanent && --reload
            let range = format!("{start}-{end}/tcp");
            executor.run(&Cmd::new("firewall-cmd").args(["--add-port", &range, "--permanent"]))?;
            executor.run(&Cmd::new("firewall-cmd").arg("--reload"))?;
            println!("防火墙(firewalld)已开放端口段 {range}");
        }
        FirewallBackend::Ufw => {
            // ufw allow 30000:31000/tcp
            let range = format!("{start}:{end}/tcp");
            executor.run(&Cmd::new("ufw").args(["allow", &range]))?;
            println!("防火墙(ufw)已开放端口段 {range}");
        }
        FirewallBackend::Iptables => {
            // iptables -I INPUT -p tcp --dport 30000:31000 -j ACCEPT
            let range = format!("{start}:{end}");
            executor.run(&Cmd::new("iptables").args([
                "-I", "INPUT", "-p", "tcp", "--dport", &range, "-j", "ACCEPT",
            ]))?;
            println!("防火墙(iptables)已开放端口段 {range}");
        }
        FirewallBackend::None => {
            eprintln!(
                "警告:未检测到防火墙后端(firewalld/ufw/iptables),请手动开放 {start}-{end}/tcp 端口段"
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::{ExecOutput, FakeExecutor};

    #[test]
    fn detect_firewalld_when_present() {
        let ex = FakeExecutor::new()
            .expect("firewall-cmd", ExecOutput::success("0.9.0"))
            .expect("ufw", ExecOutput::failure(1, "not found"))
            .expect("iptables", ExecOutput::failure(1, "not found"));
        assert_eq!(detect_backend(&ex), FirewallBackend::Firewalld);
    }

    #[test]
    fn detect_ufw_when_firewalld_absent() {
        let ex = FakeExecutor::new()
            .expect("firewall-cmd", ExecOutput::failure(1, ""))
            .expect("ufw", ExecOutput::success("Status: active"))
            .expect("iptables", ExecOutput::failure(1, "not found"));
        assert_eq!(detect_backend(&ex), FirewallBackend::Ufw);
    }

    #[test]
    fn open_port_range_firewalld_runs_two_cmds() {
        let ex = FakeExecutor::new().expect("firewall-cmd", ExecOutput::success("ok"));
        crate::executor::take_history();
        open_port_range(30000, 31000, &ex).unwrap();
        let h = crate::executor::take_history();
        // history 含探测的 firewall-cmd --version + open 的 2 条(add-port + reload)
        let add = h
            .iter()
            .filter(|c| {
                c.program == "firewall-cmd" && c.args.iter().any(|a| a.contains("add-port"))
            })
            .count();
        let reload = h
            .iter()
            .filter(|c| c.program == "firewall-cmd" && c.args == vec!["--reload"])
            .count();
        assert_eq!(add, 1, "应有 1 条 --add-port");
        assert_eq!(reload, 1, "应有 1 条 --reload");
    }

    #[test]
    fn open_port_range_ufw_uses_colon_range() {
        let ex = FakeExecutor::new()
            .expect("firewall-cmd", ExecOutput::failure(1, ""))
            .expect("ufw", ExecOutput::success("active"));
        crate::executor::take_history();
        open_port_range(30000, 31000, &ex).unwrap();
        let h = crate::executor::take_history();
        let allow = h
            .iter()
            .filter(|c| c.program == "ufw" && c.args.iter().any(|a| a.contains("allow")))
            .count();
        assert_eq!(allow, 1, "应有 1 条 ufw allow");
    }

    #[test]
    fn open_port_range_none_backend_only_warns() {
        let ex = FakeExecutor::new()
            .expect("firewall-cmd", ExecOutput::failure(1, "no"))
            .expect("ufw", ExecOutput::failure(1, "no"))
            .expect("iptables", ExecOutput::failure(1, "no"));
        crate::executor::take_history();
        // 无后端不应报错(仅提示)
        open_port_range(30000, 31000, &ex).unwrap();
        // 探测 3 条命令已记录,但无 open 命令
        let h = crate::executor::take_history();
        assert!(h
            .iter()
            .all(|c| c.program != "firewall-cmd" || !c.args.contains(&"--add-port".to_string())));
    }
}
