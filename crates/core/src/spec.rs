use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 分流规则:MVP 仅「黑名单域名」+「BT 阻断」。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Rules {
    /// 禁止访问的域名(黑名单)。
    #[serde(default)]
    pub domain_blocklist: Vec<String>,
    /// 是否阻断 P2P / BT 下载。
    #[serde(default)]
    pub block_bt: bool,
}

impl Rules {
    pub fn empty() -> Self {
        Rules::default()
    }
}

/// 声明式部署规格 —— 整个系统的唯一真相来源。
/// 所有渲染、状态、订阅都从 Spec 推导,不反推 JSON。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Spec {
    pub version: u8,
    pub domain: String,
    #[serde(default)]
    pub cores: Cores,
    #[serde(default)]
    pub users: Vec<User>,
    #[serde(default)]
    pub rules: Rules,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Cores {
    #[serde(default = "default_true")]
    pub xray: bool,
    #[serde(default)]
    pub singbox: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: String,
    pub name: String,
    pub protocol: Protocol,
    pub port: u16,
    #[serde(default)]
    pub reality: bool,
    pub uuid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Vless,
    Vmess,
    Trojan,
    Hysteria2,
    Tuic,
    Naive,
}

impl Default for Cores {
    fn default() -> Self {
        Cores {
            xray: true,
            singbox: false,
        }
    }
}

impl Spec {
    /// 生成一个最小默认 Spec(`vagent init` 用)。
    pub fn default_for(domain: &str) -> Self {
        Spec {
            version: 1,
            domain: domain.to_string(),
            cores: Cores::default(),
            users: vec![],
            rules: Rules::empty(),
        }
    }

    /// 新增一个用户,自动生成 id / uuid。
    pub fn add_user(&mut self, name: &str, protocol: Protocol, port: u16, reality: bool) {
        self.users.push(User::new(name, protocol, port, reality));
    }
}

impl User {
    pub fn new(name: &str, protocol: Protocol, port: u16, reality: bool) -> Self {
        User {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            protocol,
            port,
            reality,
            uuid: Uuid::new_v4().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_for_enables_xray_only() {
        let s = Spec::default_for("x.com");
        assert!(s.cores.xray);
        assert!(!s.cores.singbox);
        assert_eq!(s.users.len(), 0);
        assert_eq!(s.version, 1);
    }

    #[test]
    fn user_new_generates_unique_ids() {
        let a = User::new("alice", Protocol::Vless, 443, true);
        let b = User::new("bob", Protocol::Vless, 443, true);
        assert!(!a.uuid.is_empty());
        assert!(!b.uuid.is_empty());
        assert_ne!(a.id, b.id);
        assert_ne!(a.uuid, b.uuid);
    }

    #[test]
    fn add_user_appends() {
        let mut s = Spec::default_for("x.com");
        s.add_user("alice", Protocol::Vless, 443, true);
        assert_eq!(s.users.len(), 1);
        assert_eq!(s.users[0].name, "alice");
    }

    #[test]
    fn protocol_serde_roundtrip() {
        let toml = r#"
version = 1
domain = "x.com"
[[users]]
id = "u1"
name = "a"
protocol = "vless"
port = 443
reality = true
uuid = "abc"
"#;
        let spec: Spec = toml::from_str(toml).unwrap();
        assert_eq!(spec.users[0].protocol, Protocol::Vless);
        assert!(spec.users[0].reality);
    }
}
