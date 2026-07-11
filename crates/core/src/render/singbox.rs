//! sing-box 配置渲染。
//! 负责 Xray 不便承载的现代协议:Hysteria2、Tuic。
//! 其余协议由 Xray 渲染(见 render/xray.rs)。

use crate::spec::{Protocol, Spec, User};
use crate::Error;
use serde_json::json;
use std::path::Path;

/// 单个用户 → sing-box inbound(Hysteria2 / Tuic)。
fn inbound_for(u: &User, cert_cer: &str, cert_key: &str) -> Option<serde_json::Value> {
    match (&u.protocol, &u.transport) {
        (Protocol::Hysteria2, _) => Some(hysteria2(u, cert_cer, cert_key)),
        (Protocol::Tuic, _) => Some(tuic(u, cert_cer, cert_key)),
        _ => None,
    }
}

fn hysteria2(u: &User, cert_cer: &str, cert_key: &str) -> serde_json::Value {
    json!({
        "type": "hysteria2",
        "tag": format!("hy2-{}", u.id),
        "listen": "::",
        "listen_port": u.port,
        "users": [{ "password": u.uuid }],
        "tls": {
            "enabled": true,
            "certificate_path": cert_cer,
            "key_path": cert_key
        }
    })
}

fn tuic(u: &User, cert_cer: &str, cert_key: &str) -> serde_json::Value {
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
            "certificate_path": cert_cer,
            "key_path": cert_key
        }
    })
}

/// 渲染 sing-box 配置(JSON)。纯函数。
pub fn render(spec: &Spec, base_dir: &Path) -> Result<serde_json::Value, Error> {
    let cert_cer = base_dir
        .join("certs")
        .join(format!("{}.cer", spec.domain))
        .to_string_lossy()
        .to_string();
    let cert_key = base_dir
        .join("certs")
        .join(format!("{}.key", spec.domain))
        .to_string_lossy()
        .to_string();
    let inbounds: Vec<serde_json::Value> = spec
        .users
        .iter()
        .filter_map(|u| inbound_for(u, &cert_cer, &cert_key))
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

pub fn render_string(spec: &Spec, base_dir: &Path) -> Result<String, Error> {
    let v = render(spec, base_dir)?;
    serde_json::to_string_pretty(&v).map_err(|e| Error::Render(e.to_string()))
}
