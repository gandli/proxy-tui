use crate::spec::Spec;
use crate::Error;

/// 渲染 SNI 反代 nginx server block(Reality 透传)。
pub fn render(spec: &Spec) -> Result<String, Error> {
    let domain = &spec.domain;
    let block = format!(
        "server {{\n    listen 443;\n    server_name {domain};\n    location / {{\n        proxy_pass https://{domain}:443;\n        proxy_ssl_server_name on;\n        proxy_ssl_name {domain};\n    }}\n}}"
    );
    Ok(block)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::Spec;

    #[test]
    fn render_contains_domain() {
        let spec = Spec::default_for("v.example.com");
        let cfg = render(&spec).unwrap();
        assert!(cfg.contains("v.example.com"));
        assert!(cfg.contains("proxy_pass https://v.example.com:443"));
    }
}
