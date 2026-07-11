//! Telegram bot 前端(teloxide 接入)。
//!
//! 鉴权:Telegram 用户 ID 白名单,存 /etc/vagent/allowlist(每行一个 UID)。
//! token 走环境变量 VAGENT_BOT_TOKEN,不进代码。
//!
//! handler 业务逻辑调用 vagent_core 纯函数,便于单测。

use std::path::Path;
use std::sync::Arc;

use teloxide::prelude::*;
use teloxide::types::Message;
use vagent_core::Spec;

pub const ALLOWLIST_PATH: &str = "/etc/vagent/allowlist";

/// 读取白名单(每行一个 UID 字符串,支持 # 注释与空行)。
pub fn load_allowlist(path: &Path) -> Vec<String> {
    std::fs::read_to_string(path)
        .map(|s| {
            s.lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .map(|l| l.to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// 鉴权:UID 是否在白名单内。
pub fn authorized(uid: u64, allowlist: &[String]) -> bool {
    let uid_s = uid.to_string();
    allowlist.iter().any(|u| u == &uid_s)
}

/// /status 回复。
pub fn handle_status(spec: &Spec) -> String {
    let mut lines = vec![
        format!("域名: {}", spec.domain),
        format!(
            "内核: xray={} singbox={}",
            spec.cores.xray, spec.cores.singbox
        ),
        format!("用户数: {}", spec.users.len()),
    ];
    for u in &spec.users {
        let proto = format!("{:?}", u.protocol).to_lowercase();
        lines.push(format!(
            "  - {} [{}] port={} reality={}",
            u.name, proto, u.port, u.reality
        ));
    }
    lines.join("\n")
}

/// /adduser 回复(仅构造描述)。
pub fn handle_adduser(spec: &Spec, name: &str) -> String {
    format!(
        "将为 {} 新增 VLESS+Reality 用户(端口待分配),当前用户数 {}",
        name,
        spec.users.len()
    )
}

/// 从 /etc/vagent/spec.toml 读 spec(供 handler 使用)。
fn read_spec() -> Spec {
    let cfg =
        std::env::var("VAGENT_CONFIG").unwrap_or_else(|_| "/etc/vagent/spec.toml".to_string());
    vagent_core::load_spec(Path::new(&cfg)).unwrap_or_else(|_| Spec::default_for("unknown"))
}

async fn status_cmd(bot: Bot, msg: Message, allowlist: Arc<Vec<String>>) -> ResponseResult<()> {
    let uid = msg.from.as_ref().map(|u| u.id.0).unwrap_or(0);
    if !authorized(uid, &allowlist) {
        bot.send_message(msg.chat.id, "未授权").await?;
        return Ok(());
    }
    let spec = read_spec();
    bot.send_message(msg.chat.id, handle_status(&spec)).await?;
    Ok(())
}

async fn adduser_cmd(bot: Bot, msg: Message, allowlist: Arc<Vec<String>>) -> ResponseResult<()> {
    let uid = msg.from.as_ref().map(|u| u.id.0).unwrap_or(0);
    if !authorized(uid, &allowlist) {
        bot.send_message(msg.chat.id, "未授权").await?;
        return Ok(());
    }
    let name = msg.text().map(|t| t.trim().to_string()).unwrap_or_default();
    if name.is_empty() {
        bot.send_message(msg.chat.id, "用法: /adduser <用户名>")
            .await?;
        return Ok(());
    }
    let spec = read_spec();
    bot.send_message(msg.chat.id, handle_adduser(&spec, &name))
        .await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let token = std::env::var("VAGENT_BOT_TOKEN").expect("VAGENT_BOT_TOKEN 未设置");
    let bot = Bot::new(token);
    let allowlist = Arc::new(load_allowlist(Path::new(ALLOWLIST_PATH)));

    let handler =
        Update::filter_message().branch(dptree::entry().filter_command::<Cmd>().endpoint(
            |bot: Bot, msg: Message, cmd: Cmd, allowlist: Arc<Vec<String>>| async move {
                match cmd {
                    Cmd::Status => status_cmd(bot, msg, allowlist).await,
                    Cmd::AddUser => adduser_cmd(bot, msg, allowlist).await,
                }
            },
        ));

    println!("vagent-bot 启动,白名单 {} 条", allowlist.len());
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![allowlist])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

#[derive(teloxide::utils::command::BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Cmd {
    /// 查看节点状态
    Status,
    /// 新增用户: /adduser <用户名>
    AddUser,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowlist_parses_lines() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("allow");
        std::fs::write(&f, "111\n222\n\n# comment\n333\n").unwrap();
        let list = load_allowlist(&f);
        assert_eq!(list, vec!["111", "222", "333"]);
    }

    #[test]
    fn authorized_checks_membership() {
        let list = vec!["111".to_string(), "222".to_string()];
        assert!(authorized(111, &list));
        assert!(!authorized(999, &list));
    }

    #[test]
    fn status_contains_domain_and_count() {
        let spec = Spec::default_for("example.com");
        let r = handle_status(&spec);
        assert!(r.contains("example.com"));
        assert!(r.contains("用户数: 0"));
    }

    #[test]
    fn status_lists_users() {
        let mut spec = Spec::default_for("example.com");
        spec.add_user("alice", vagent_core::Protocol::Vless, 443, true);
        let r = handle_status(&spec);
        assert!(r.contains("用户数: 1"));
        assert!(r.contains("alice [vless]"));
    }

    #[test]
    fn adduser_reply_mentions_name() {
        let spec = Spec::default_for("x.com");
        let r = handle_adduser(&spec, "bob");
        assert!(r.contains("bob"));
    }
}
