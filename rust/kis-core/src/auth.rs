use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::config::Environment;
use crate::error::{KisError, Result};

const TOKEN_PATH: &str = "/oauth2/tokenP";
const HASHKEY_PATH: &str = "/uapi/hashkey";

#[derive(Debug, Clone)]
pub struct TokenManager {
    base_url: String,
    app_key: String,
    app_secret: String,
    environment: Environment,
    cache_path: PathBuf,
    client: Client,
    state: std::sync::Arc<Mutex<TokenState>>,
}

#[derive(Debug, Default)]
struct TokenState {
    token: Option<String>,
    expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    access_token_token_expired: String,
    expires_in: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedToken {
    access_token: String,
    expired_at: String,
    environment: String,
}

#[derive(Debug, Deserialize)]
struct HashKeyResponse {
    #[serde(rename = "HASH")]
    hash: String,
}

impl TokenManager {
    pub fn new(
        base_url: String,
        app_key: String,
        app_secret: String,
        environment: Environment,
    ) -> Result<Self> {
        let cache_path = default_cache_path()?;
        Ok(Self::new_with_cache_path(
            base_url,
            app_key,
            app_secret,
            environment,
            cache_path,
            Client::new(),
        ))
    }

    pub fn new_with_cache_path(
        base_url: String,
        app_key: String,
        app_secret: String,
        environment: Environment,
        cache_path: PathBuf,
        client: Client,
    ) -> Self {
        Self {
            base_url,
            app_key,
            app_secret,
            environment,
            cache_path,
            client,
            state: std::sync::Arc::new(Mutex::new(TokenState::default())),
        }
    }

    pub async fn get_token(&self) -> Result<String> {
        let mut state = self.state.lock().await;
        if let (Some(token), Some(expires_at)) = (&state.token, state.expires_at)
            && Utc::now() < expires_at
        {
            return Ok(token.clone());
        }

        if let Some(cached) = self.load_cached_token() {
            state.token = Some(cached.0.clone());
            state.expires_at = Some(cached.1);
            return Ok(cached.0);
        }

        let (token, expires_at) = self.fetch_token().await?;
        state.token = Some(token.clone());
        state.expires_at = Some(expires_at);
        self.save_cached_token(&token, expires_at);
        Ok(token)
    }

    pub async fn get_hashkey<T: Serialize + ?Sized>(&self, body: &T) -> Result<String> {
        let response = self
            .client
            .post(format!("{}{}", self.base_url, HASHKEY_PATH))
            .header("content-type", "application/json; charset=utf-8")
            .header("appkey", &self.app_key)
            .header("appsecret", &self.app_secret)
            .json(body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(KisError::Parse(format!(
                "hashkey request failed with status {}",
                response.status()
            )));
        }

        let payload: HashKeyResponse = response.json().await?;
        Ok(payload.hash)
    }

    async fn fetch_token(&self) -> Result<(String, DateTime<Utc>)> {
        #[derive(Serialize)]
        struct TokenRequest<'a> {
            grant_type: &'a str,
            appkey: &'a str,
            appsecret: &'a str,
        }

        let response = self
            .client
            .post(format!("{}{}", self.base_url, TOKEN_PATH))
            .header("content-type", "application/json; charset=utf-8")
            .json(&TokenRequest {
                grant_type: "client_credentials",
                appkey: &self.app_key,
                appsecret: &self.app_secret,
            })
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(KisError::Parse(format!(
                "token request failed with status {}",
                response.status()
            )));
        }

        let payload: TokenResponse = response.json().await?;
        let expires_at = parse_expiry(&payload.access_token_token_expired)
            .unwrap_or_else(|| Utc::now() + chrono::Duration::seconds(payload.expires_in));
        Ok((payload.access_token, expires_at))
    }

    fn load_cached_token(&self) -> Option<(String, DateTime<Utc>)> {
        let data = match fs::read_to_string(&self.cache_path) {
            Ok(data) => data,
            Err(_) => return None,
        };

        let cached: CachedToken = match serde_json::from_str(&data) {
            Ok(cached) => cached,
            Err(_) => return None,
        };
        if cached.environment != self.environment.to_string() {
            return None;
        }

        let expires_at = match DateTime::parse_from_rfc3339(&cached.expired_at) {
            Ok(expires_at) => expires_at.with_timezone(&Utc),
            Err(_) => return None,
        };
        if Utc::now() >= expires_at - chrono::Duration::minutes(5) {
            return None;
        }

        Some((cached.access_token, expires_at))
    }

    fn save_cached_token(&self, token: &str, expires_at: DateTime<Utc>) {
        if let Some(parent) = self.cache_path.parent()
            && fs::create_dir_all(parent).is_err()
        {
            return;
        }

        let cached = CachedToken {
            access_token: token.to_string(),
            expired_at: expires_at.to_rfc3339(),
            environment: self.environment.to_string(),
        };
        let payload = match serde_json::to_vec_pretty(&cached) {
            Ok(payload) => payload,
            Err(_) => return,
        };
        if fs::write(&self.cache_path, payload).is_err() {
            return;
        }
        let _ = set_file_mode(&self.cache_path);
    }
}

fn default_cache_path() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| KisError::Config("determining home directory".to_string()))?;
    Ok(home.join(".kis").join("token-rs.json"))
}

fn parse_expiry(raw: &str) -> Option<DateTime<Utc>> {
    let naive = NaiveDateTime::parse_from_str(raw, "%Y-%m-%d %H:%M:%S").ok()?;
    let offset = FixedOffset::east_opt(9 * 60 * 60)?;
    offset
        .from_local_datetime(&naive)
        .single()
        .map(|dt| dt.with_timezone(&Utc))
}

#[cfg(unix)]
fn set_file_mode(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let permissions = std::fs::Permissions::from_mode(0o600);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(not(unix))]
fn set_file_mode(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use chrono::{Duration, Utc};
    use reqwest::Client;

    use super::{CachedToken, TokenManager};
    use crate::config::Environment;

    #[test]
    fn loads_cached_token_for_matching_environment() {
        let dir = tempfile::tempdir().unwrap();
        let cache = dir.path().join("token-rs.json");
        fs::write(
            &cache,
            serde_json::to_vec(&CachedToken {
                access_token: "cached".to_string(),
                expired_at: (chrono::Utc::now() + Duration::hours(1)).to_rfc3339(),
                environment: "virtual".to_string(),
            })
            .unwrap(),
        )
        .unwrap();

        let manager = TokenManager::new_with_cache_path(
            "https://example.com".to_string(),
            "app".to_string(),
            "secret".to_string(),
            Environment::Virtual,
            cache,
            Client::new(),
        );

        let token = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(manager.get_token())
            .unwrap();
        assert_eq!(token, "cached");
    }

    #[test]
    fn ignores_cached_token_for_different_environment() {
        let dir = tempfile::tempdir().unwrap();
        let cache = dir.path().join("token-rs.json");
        fs::write(
            &cache,
            serde_json::to_vec(&CachedToken {
                access_token: "cached".to_string(),
                expired_at: (chrono::Utc::now() + Duration::hours(1)).to_rfc3339(),
                environment: "real".to_string(),
            })
            .unwrap(),
        )
        .unwrap();

        let manager = TokenManager::new_with_cache_path(
            "https://example.com".to_string(),
            "app".to_string(),
            "secret".to_string(),
            Environment::Virtual,
            cache,
            Client::new(),
        );

        let token = manager.load_cached_token();
        assert!(token.is_none());
    }

    #[test]
    fn ignores_malformed_cached_token() {
        let dir = tempfile::tempdir().unwrap();
        let cache = dir.path().join("token-rs.json");
        fs::write(&cache, "{not-json").unwrap();

        let manager = TokenManager::new_with_cache_path(
            "https://example.com".to_string(),
            "app".to_string(),
            "secret".to_string(),
            Environment::Virtual,
            cache,
            Client::new(),
        );

        let token = manager.load_cached_token();
        assert!(token.is_none());
    }

    #[test]
    fn save_cached_token_does_not_fail_when_cache_path_is_unwritable() {
        let dir = tempfile::tempdir().unwrap();
        let parent = dir.path().join("cache-parent");
        fs::write(&parent, "not-a-directory").unwrap();
        let cache = parent.join("token-rs.json");

        let manager = TokenManager::new_with_cache_path(
            "https://example.com".to_string(),
            "app".to_string(),
            "secret".to_string(),
            Environment::Virtual,
            cache.clone(),
            Client::new(),
        );

        manager.save_cached_token("fresh-token", Utc::now() + Duration::hours(1));
        assert!(!cache.exists());
    }
}
