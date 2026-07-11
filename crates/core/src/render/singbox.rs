//! sing-box 配置渲染。
//! 负责 Xray 不便承载的现代协议:Hysteria2、Tuic。
//! 其余协议由 Xray 渲染(见 render/xray.rs)。

use crate::spec::{Protocol, Spec, User};
use crate::Error;
use serde_json::json;

/// 单个用户 → sing-box inbound(Hysteria2 / Tuic)。
fn inbound_for(u: &User, spec: &Spec) -> Option<serde_json::Value> {
    match (&u.protocol, &u.transport) {
        (Protocol::Hysteria2, _) => Some(hysteria2(u, spec)),
        (Protocol::Tuic, _) => Some(tuic(u, spec)),
        _ => None,
    }
}

fn hysteria2(u: &User, spec: &Spec) -> serde_json::Value {
    json!({
        "type": "hysteria2",
        "tag": format!("hy2-{}", u.id),
        "listen": "::",
        "listen_port": u.port,
        "users": [{ "password": u.uuid }],
        "tls": {
            "enabled": true,
            "certificate_path": format!("/etc/vagent/certs/{}.cer", spec.domain),
            "key_path": format!("/etc/vagent/certs/{}.key", spec.domain)
        }
    })
}

fn tuic(u: &User, spec: &Spec) -> serde_json::Value {
    json!({
        "type": "tuic",
        "tag": format!("tuic-{}", u.id),
        "listen": "::",
        "listen_port": u.port,
        "users": [{ "uuid": u.uuid, "password": u.uuid }],
        "congestion_control": "bbr",
        "tls": {
            "enabled": true,
            "alpn": ["h3"],
            "certificate_path": format!("/etc/vagent/certs/{}.cer", spec.domain),
            "key_path": format!("/etc/vagent/certs/{}.key", spec.domain)
        }
    })
}

/// 渲染 sing-box 配置(JSON)。纯函数。
pub fn render(spec: &Spec) -> Result<serde_json::Value, Error> {
    let inbounds: Vec<serde_json::Value> = spec
        .users
        .iter()
        .filter_map(|u| inbound_for(u, spec))
        .collect();

    Ok(json!({
        "log": { "level": "warn" },
        "inbounds": inbounds,
        "outbounds": [
            { "type": "direct", "tag": "direct" },
            { "type": "block", "tag": "block" }
        ]
    }))
}

pub fn render_string(spec: &Spec) -> Result<String, Error> {
    let v = render(spec)?;
    serde_json::to_string_pretty(&v).map_err(|e| Error::Render(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::Transport;

    #[test]
    fn render_hysteria2_inbound() {
        let mut spec = Spec::default_for("x.com");
        spec.users.push(User::new(
            "h",
            Protocol::Hysteria2,
            8443,
            false,
            Transport::Tcp,
        ));
        let v = render(&spec).unwrap();
        let ib = &v["inbounds"][0];
        assert_eq!(ib["type"], "hysteria2");
        assert_eq!(ib["listen_port"], 8443);
        assert!(ib["tls"]["enabled"].as_bool().unwrap());
    }

    #[test]
    fn render_tuic_inbound() {
        let mut spec = Spec::default_for("x.com");
        spec.users
            .push(User::new("t", Protocol::Tuic, 9443, false, Transport::Tcp));
        let v = render(&spec).unwrap();
        let ib = &v["inbounds"][0];
        assert_eq!(ib["type"], "tuic");
        assert_eq!(ib["congestion_control"], "bbr");
    }

    #[test]
    fn render_filters_xray_protocols() {
        let mut spec = Spec::default_for("x.com");
        spec.users
            .push(User::new("v", Protocol::Vless, 443, true, Transport::Tcp));
        spec.users
            .push(User::new("m", Protocol::Vmess, 2053, false, Transport::Tcp));
        let v = render(&spec).unwrap();
        assert_eq!(v["inbounds"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn render_has_outbounds() {
        let spec = Spec::default_for("x.com");
        let v = render(&spec).unwrap();
        let tags: Vec<&str> = v["outbounds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|o| o["tag"].as_str().unwrap())
            .collect();
        assert!(tags.contains(&"direct"));
        assert!(tags.contains(&"block"));
    }
}
