//! 进程退出码约定(MVP)。
//! 0=成功 / 1=配置错误 / 2=系统或权限错误 / 3=网络或下载错误。
//! 供 CLI/bot 据码判状态,避免散落的 #[allow(dead_code)]。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Ok = 0,
    Config = 1,
    System = 2,
    Network = 3,
}

impl From<ExitCode> for i32 {
    fn from(e: ExitCode) -> i32 {
        e as i32
    }
}

impl From<crate::Error> for ExitCode {
    fn from(e: crate::Error) -> Self {
        match e {
            crate::Error::Io(_) => ExitCode::System,
            crate::Error::Toml(_) | crate::Error::TomlSer(_) => ExitCode::Config,
            crate::Error::Render(_) => ExitCode::System,
            crate::Error::Unsupported(_) => ExitCode::Config,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::Spec;
    use crate::Error;

    #[test]
    fn ok_is_zero() {
        assert_eq!(i32::from(ExitCode::Ok), 0);
    }

    #[test]
    fn io_maps_to_system() {
        let e = Error::Io(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "denied",
        ));
        assert_eq!(ExitCode::from(e), ExitCode::System);
    }

    #[test]
    fn toml_maps_to_config() {
        let e = Error::Toml(toml::from_str::<Spec>("this is = = not valid toml").unwrap_err());
        assert_eq!(ExitCode::from(e), ExitCode::Config);
    }

    #[test]
    fn unsupported_maps_to_config() {
        let e = Error::Unsupported("nope".into());
        assert_eq!(ExitCode::from(e), ExitCode::Config);
    }
}
