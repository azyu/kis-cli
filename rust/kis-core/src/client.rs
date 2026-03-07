use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use reqwest::Client;
use serde::Serialize;
use serde_json::Value;

use crate::auth::TokenManager;
use crate::config::AppConfig;
use crate::error::{KisError, Result};

#[derive(Debug, Clone)]
pub struct KisClient {
    base_url: String,
    app_key: String,
    app_secret: String,
    token_manager: TokenManager,
    client: Client,
    interval: Duration,
    last_call: Arc<Mutex<Option<Instant>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonResponse {
    pub body: Value,
    pub tr_cont: Option<String>,
}

impl KisClient {
    pub fn new(config: &AppConfig) -> Result<Self> {
        let client = Client::builder().timeout(Duration::from_secs(30)).build()?;
        let base_url = config.environment.base_url().to_string();
        let token_manager = TokenManager::new(
            base_url.clone(),
            config.app_key.clone(),
            config.app_secret.clone(),
            config.environment,
        )?;
        Ok(Self::new_with_client(config, client, token_manager))
    }

    pub fn new_with_client(
        config: &AppConfig,
        client: Client,
        token_manager: TokenManager,
    ) -> Self {
        Self {
            base_url: config.environment.base_url().to_string(),
            app_key: config.app_key.clone(),
            app_secret: config.app_secret.clone(),
            token_manager,
            client,
            interval: if config.environment.is_virtual() {
                Duration::from_millis(500)
            } else {
                Duration::from_millis(50)
            },
            last_call: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn get_json(
        &self,
        path: &str,
        tr_id: &str,
        params: &[(String, String)],
    ) -> Result<Value> {
        Ok(self.get_json_response(path, tr_id, params).await?.body)
    }

    pub async fn get_json_response(
        &self,
        path: &str,
        tr_id: &str,
        params: &[(String, String)],
    ) -> Result<JsonResponse> {
        self.get_json_response_with_tr_cont(path, tr_id, "", params)
            .await
    }

    pub async fn get_json_response_with_tr_cont(
        &self,
        path: &str,
        tr_id: &str,
        tr_cont: &str,
        params: &[(String, String)],
    ) -> Result<JsonResponse> {
        self.rate_limit().await;
        let token = self.token_manager.get_token().await?;
        let request = self
            .client
            .get(format!("{}{}", self.base_url, path))
            .query(params)
            .header("content-type", "application/json; charset=utf-8")
            .header("authorization", format!("Bearer {token}"))
            .header("appkey", &self.app_key)
            .header("appsecret", &self.app_secret)
            .header("tr_id", tr_id)
            .header("custtype", "P");
        let request = if tr_cont.is_empty() {
            request
        } else {
            request.header("tr_cont", tr_cont)
        };
        self.send(request).await
    }

    pub async fn post_json<T: Serialize + ?Sized>(
        &self,
        path: &str,
        tr_id: &str,
        body: &T,
    ) -> Result<Value> {
        Ok(self.post_json_response(path, tr_id, body).await?.body)
    }

    pub async fn post_json_response<T: Serialize + ?Sized>(
        &self,
        path: &str,
        tr_id: &str,
        body: &T,
    ) -> Result<JsonResponse> {
        self.rate_limit().await;
        let token = self.token_manager.get_token().await?;
        let hashkey = self.token_manager.get_hashkey(body).await?;
        let request = self
            .client
            .post(format!("{}{}", self.base_url, path))
            .header("content-type", "application/json; charset=utf-8")
            .header("authorization", format!("Bearer {token}"))
            .header("appkey", &self.app_key)
            .header("appsecret", &self.app_secret)
            .header("tr_id", tr_id)
            .header("custtype", "P")
            .header("hashkey", hashkey)
            .json(body);
        self.send(request).await
    }

    async fn send(&self, request: reqwest::RequestBuilder) -> Result<JsonResponse> {
        let response = request.send().await?;
        let status = response.status();
        let tr_cont = response
            .headers()
            .get("tr_cont")
            .and_then(|value| value.to_str().ok())
            .map(ToString::to_string);
        let body = response.text().await?;
        if !status.is_success() {
            return Err(KisError::Parse(format!(
                "API returned status {status}: {body}"
            )));
        }
        Ok(JsonResponse {
            body: serde_json::from_str(&body)?,
            tr_cont,
        })
    }

    async fn rate_limit(&self) {
        let wait_until = {
            let mut guard = self.last_call.lock().unwrap();
            let now = Instant::now();
            let next_slot = guard
                .map(|last| last + self.interval)
                .unwrap_or(now)
                .max(now);
            *guard = Some(next_slot);
            next_slot
        };
        let sleep_for = wait_until.saturating_duration_since(Instant::now());

        if !sleep_for.is_zero() {
            tokio::time::sleep(sleep_for).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use serde_json::json;
    use tokio::sync::{Barrier, mpsc};

    use super::{JsonResponse, KisClient};
    use crate::auth::TokenManager;
    use crate::config::{AppConfig, Environment};

    #[test]
    fn creates_client_with_virtual_interval() {
        let config = AppConfig {
            app_key: "app".to_string(),
            app_secret: "secret".to_string(),
            account_no: "12345678".to_string(),
            account_prod: "01".to_string(),
            environment: Environment::Virtual,
        };

        let token_manager = TokenManager::new_with_cache_path(
            config.environment.base_url().to_string(),
            config.app_key.clone(),
            config.app_secret.clone(),
            config.environment,
            tempfile::tempdir().unwrap().path().join("token-rs.json"),
            reqwest::Client::new(),
        );
        let client = KisClient::new_with_client(&config, reqwest::Client::new(), token_manager);
        assert_eq!(
            client.base_url,
            "https://openapivts.koreainvestment.com:29443"
        );
    }

    #[test]
    fn parses_success_json_payload() {
        let payload = json!({
            "rt_cd": "0",
            "msg_cd": "MCA00000",
            "msg1": "정상처리",
            "output": { "stck_prpr": "70000" }
        });
        assert_eq!(payload["rt_cd"], "0");
        assert_eq!(payload["output"]["stck_prpr"], "70000");
    }

    #[test]
    fn json_response_can_store_tr_cont_header() {
        let response = JsonResponse {
            body: json!({"rt_cd": "0"}),
            tr_cont: Some("M".to_string()),
        };

        assert_eq!(response.tr_cont.as_deref(), Some("M"));
        assert_eq!(response.body["rt_cd"], "0");
    }

    #[test]
    fn rate_limit_serializes_concurrent_callers() {
        let config = AppConfig {
            app_key: "app".to_string(),
            app_secret: "secret".to_string(),
            account_no: "12345678".to_string(),
            account_prod: "01".to_string(),
            environment: Environment::Real,
        };
        let token_manager = TokenManager::new_with_cache_path(
            config.environment.base_url().to_string(),
            config.app_key.clone(),
            config.app_secret.clone(),
            config.environment,
            tempfile::tempdir().unwrap().path().join("token-rs.json"),
            reqwest::Client::new(),
        );
        let mut client = KisClient::new_with_client(&config, reqwest::Client::new(), token_manager);
        client.interval = Duration::from_millis(80);

        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                let barrier = Arc::new(Barrier::new(3));
                let (tx, mut rx) = mpsc::unbounded_channel();

                for _ in 0..3 {
                    let client = client.clone();
                    let barrier = barrier.clone();
                    let tx = tx.clone();
                    tokio::spawn(async move {
                        barrier.wait().await;
                        client.rate_limit().await;
                        tx.send(Instant::now()).unwrap();
                    });
                }
                drop(tx);

                let mut instants = Vec::new();
                while let Some(instant) = rx.recv().await {
                    instants.push(instant);
                }
                instants.sort();

                let min_gap = instants
                    .windows(2)
                    .map(|pair| pair[1].duration_since(pair[0]))
                    .min()
                    .unwrap();
                assert!(
                    min_gap >= Duration::from_millis(60),
                    "expected serialized spacing, got {min_gap:?}"
                );
            });
    }
}
