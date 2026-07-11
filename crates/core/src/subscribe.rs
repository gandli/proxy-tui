//! 订阅签发。
//!
//! 单用户链接:`vless://<uuid>@<domain>:<port>?type=tcp&security=reality&pbk=<key>&sid=<sid>#<name>`
//! 多用户 bundle:v2rayN 订阅格式 = Base64(JSON{outbounds:[...]}),每个 Reality 用户一条 outbound。
//! 服务端对 payload 做 HMAC-SHA256 附 `#sig=<hex>`,用于按 id 识别与吊销(客户端不校验)。

use crate::spec::{Spec, User};
use crate::Error;
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

const SECRET_PATH: &str = "/etc/vagent/secret";

/// 单用户 vless:// 链接(MVP 协议 VLESS+Reality)。
pub fn gen_user(user: &User, spec: &Spec) -> Result<String, Error> {
    if user.protocol != crate::spec::Protocol::Vless || !user.reality {
        return Err(Error::Unsupported(format!(
            "subscribe gen_user 仅支持 VLESS+Reality(MVP),got {:?}/reality={}",
            user.protocol, user.reality
        )));
    }
    let pbk = "<generated-by-xray>"; // 真实场景从 reality keys 读取
    let sid = "";
    let query = format!(
        "type=tcp&security=reality&pbk={pbk}&sid={sid}&encryption=none&flow=xtls-rprx-vision"
    );
    let base = format!(
        "vless://{}@{}:{}?{}#{}",
        user.uuid, spec.domain, user.port, query, user.name
    );
    Ok(base)
}

/// 多用户 bundle(v2rayN 订阅):Base64(JSON{outbounds})。
pub fn bundle(spec: &Spec) -> Result<String, Error> {
    let outbounds: Vec<serde_json::Value> = spec
        .users
        .iter()
        .filter(|u| matches!(u.protocol, crate::spec::Protocol::Vless) && u.reality)
        .map(|u| {
            serde_json::json!({
                "tag": u.name,
                "type": "vless",
                "server": spec.domain,
                "server_port": u.port,
                "uuid": u.uuid,
                "flow": "xtls-rprx-vision",
                "tls": {
                    "enabled": true,
                    "server_name": spec.domain,
                    "reality": { "enabled": true, "public_key": "<generated-by-xray>", "short_id": "" }
                },
                "transport": "tcp"
            })
        })
        .collect();
    let json = serde_json::to_string(&serde_json::json!({ "outbounds": outbounds }))
        .map_err(|e| Error::Render(e.to_string()))?;
    Ok(B64.encode(json))
}

/// 对链接/bundle 做服务端签名。
pub fn sign(payload: &str, secret: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret).expect("hmac accepts any key len");
    mac.update(payload.as_bytes());
    let sig = hex::encode(mac.finalize().into_bytes());
    format!("{payload}#sig={sig}")
}

/// 校验签名是否由本服务端签发。
pub fn verify(link: &str, secret: &[u8]) -> bool {
    let (base, sig) = match link.rsplit_once("#sig=") {
        Some((b, s)) => (b, s),
        None => return false,
    };
    let mut mac = HmacSha256::new_from_slice(secret).expect("hmac accepts any key len");
    mac.update(base.as_bytes());
    let expected = hex::encode(mac.finalize().into_bytes());
    expected.len() == sig.len() && expected.bytes().zip(sig.bytes()).all(|(a, b)| a == b)
}

/// 读取或生成 secret(600 权限)。
pub fn load_or_create_secret() -> Result<Vec<u8>, Error> {
    if let Ok(s) = std::fs::read(SECRET_PATH) {
        return Ok(s);
    }
    let secret: Vec<u8> = (0..32).map(|_| rand::random::<u8>()).collect();
    std::fs::write(SECRET_PATH, &secret)?;
    let mut perms = std::fs::metadata(SECRET_PATH)?.permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o600);
    std::fs::set_permissions(SECRET_PATH, perms)?;
    Ok(secret)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::User;

    #[test]
    fn gen_user_formats_vless_link() {
        let mut spec = Spec::default_for("v.example.com");
        let u = User::new("alice", crate::spec::Protocol::Vless, 443, true);
        spec.add_user("alice", crate::spec::Protocol::Vless, 443, true);
        let link = gen_user(&u, &spec).unwrap();
        assert!(link.starts_with("vless://"));
        assert!(link.contains("v.example.com:443"));
        assert!(link.contains("security=reality"));
        assert!(link.contains("#alice"));
    }

    #[test]
    fn gen_user_rejects_non_reality() {
        let spec = Spec::default_for("x.com");
        let u = User::new("bob", crate::spec::Protocol::Vmess, 443, false);
        assert!(gen_user(&u, &spec).is_err());
    }

    #[test]
    fn bundle_includes_all_reality_users() {
        let mut spec = Spec::default_for("v.example.com");
        spec.add_user("alice", crate::spec::Protocol::Vless, 443, true);
        spec.add_user("bob", crate::spec::Protocol::Vless, 8443, true);
        spec.add_user("carol", crate::spec::Protocol::Vmess, 443, false); // 排除
        let b = bundle(&spec).unwrap();
        let decoded = String::from_utf8(B64.decode(&b).unwrap()).unwrap();
        assert!(decoded.contains("alice"));
        assert!(decoded.contains("bob"));
        assert!(!decoded.contains("carol"));
        assert!(decoded.contains("\"outbounds\""));
    }

    #[test]
    fn sign_and_verify_roundtrip() {
        let secret = b"test-secret-32-bytes-long-1234567890";
        let link = "vless://abc@x.com:443?type=tcp#alice";
        let signed = sign(link, secret);
        assert!(signed.contains("#sig="));
        assert!(verify(&signed, secret));
        assert!(!verify(&signed, b"wrong-secret-wrong-secret-wrong-secre"));
    }

    #[test]
    fn verify_rejects_tampered() {
        let secret = b"test-secret-32-bytes-long-1234567890";
        let signed = sign("vless://abc@x.com:443#alice", secret);
        let tampered = signed.replace("alice", "mallory");
        assert!(!verify(&tampered, secret));
    }
}
