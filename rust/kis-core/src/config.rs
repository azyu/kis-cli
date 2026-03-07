use std::env;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::Deserialize;

use crate::error::{KisError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Real,
    Virtual,
}

impl Environment {
    pub fn base_url(self) -> &'static str {
        match self {
            Self::Real => "https://openapi.koreainvestment.com:9443",
            Self::Virtual => "https://openapivts.koreainvestment.com:29443",
        }
    }

    pub fn ws_base_url(self) -> &'static str {
        match self {
            Self::Real => "ws://ops.koreainvestment.com:21000",
            Self::Virtual => "ws://ops.koreainvestment.com:31000",
        }
    }

    pub fn is_virtual(self) -> bool {
        matches!(self, Self::Virtual)
    }
}

impl FromStr for Environment {
    type Err = KisError;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "real" => Ok(Self::Real),
            "virtual" => Ok(Self::Virtual),
            other => Err(KisError::Config(format!(
                "unknown environment: {other:?} (expected \"real\" or \"virtual\")"
            ))),
        }
    }
}

impl Display for Environment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Real => write!(f, "real"),
            Self::Virtual => write!(f, "virtual"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub app_key: String,
    pub app_secret: String,
    pub account_no: String,
    pub account_prod: String,
    pub environment: Environment,
}

#[derive(Debug, Default, Deserialize)]
struct FileConfig {
    app_key: Option<String>,
    app_secret: Option<String>,
    account_no: Option<String>,
    account_prod: Option<String>,
    environment: Option<String>,
}

pub fn load(config_path: Option<&Path>, env_override: Option<&str>) -> Result<AppConfig> {
    let path = default_config_path(config_path)?;
    let file = read_file_config(&path, config_path.is_none())?;

    let app_key = env::var("KIS_APP_KEY")
        .ok()
        .or(file.app_key)
        .unwrap_or_default();
    let app_secret = env::var("KIS_APP_SECRET")
        .ok()
        .or(file.app_secret)
        .unwrap_or_default();
    let account_no = env::var("KIS_ACCOUNT_NO")
        .ok()
        .or(file.account_no)
        .unwrap_or_default();
    let account_prod = env::var("KIS_ACCOUNT_PROD")
        .ok()
        .or(file.account_prod)
        .unwrap_or_else(|| "01".to_string());
    let environment = env_override
        .map(str::to_string)
        .or_else(|| env::var("KIS_ENVIRONMENT").ok())
        .or(file.environment)
        .unwrap_or_else(|| "virtual".to_string());

    Ok(AppConfig {
        app_key,
        app_secret,
        account_no,
        account_prod,
        environment: Environment::from_str(&environment)?,
    })
}

fn read_file_config(path: &Path, allow_missing: bool) -> Result<FileConfig> {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(serde_yaml::from_str(&contents)?),
        Err(error) if allow_missing && error.kind() == std::io::ErrorKind::NotFound => {
            Ok(FileConfig::default())
        }
        Err(error) => Err(error.into()),
    }
}

fn default_config_path(config_path: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = config_path {
        return Ok(path.to_path_buf());
    }

    let home = dirs::home_dir()
        .ok_or_else(|| KisError::Config("determining home directory".to_string()))?;
    Ok(home.join(".config").join("kis").join("config.yaml"))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{Environment, load, read_file_config};

    #[test]
    fn maps_environment_urls() {
        assert_eq!(
            Environment::Real.base_url(),
            "https://openapi.koreainvestment.com:9443"
        );
        assert_eq!(
            Environment::Virtual.ws_base_url(),
            "ws://ops.koreainvestment.com:31000"
        );
    }

    #[test]
    fn rejects_missing_explicit_config_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing.yaml");
        let err = load(Some(&path), None).unwrap_err();
        assert!(err.to_string().contains("No such file"));
    }

    #[test]
    fn allows_missing_default_config_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing.yaml");
        let file = read_file_config(&path, true).unwrap();

        assert!(file.app_key.is_none());
        assert!(file.environment.is_none());
    }

    #[test]
    fn loads_yaml_file_and_env_override() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.yaml");
        fs::write(
            &path,
            r#"
app_key: "app-key"
app_secret: "app-secret"
account_no: "12345678"
account_prod: "02"
environment: "virtual"
"#,
        )
        .unwrap();

        let config = load(Some(&path), Some("real")).unwrap();
        assert_eq!(config.app_key, "app-key");
        assert_eq!(config.account_no, "12345678");
        assert_eq!(config.account_prod, "02");
        assert_eq!(config.environment, Environment::Real);
    }
}
