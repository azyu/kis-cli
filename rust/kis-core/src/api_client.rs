use std::collections::HashMap;

use crate::client::{JsonResponse, KisClient};
use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[async_trait]
pub trait ApiClient {
    async fn get_json(
        &self,
        path: &str,
        tr_id: &str,
        params: &HashMap<String, String>,
    ) -> Result<Value>;
    async fn post_json(&self, path: &str, tr_id: &str, body: &Value) -> Result<Value>;

    async fn get_json_response(
        &self,
        path: &str,
        tr_id: &str,
        params: &HashMap<String, String>,
    ) -> Result<JsonResponse> {
        self.get_json_response_with_tr_cont(path, tr_id, "", params)
            .await
    }

    async fn get_json_response_with_tr_cont(
        &self,
        path: &str,
        tr_id: &str,
        tr_cont: &str,
        params: &HashMap<String, String>,
    ) -> Result<JsonResponse> {
        let _ = tr_cont;
        Ok(JsonResponse {
            body: self.get_json(path, tr_id, params).await?,
            tr_cont: None,
        })
    }

    async fn post_json_response(
        &self,
        path: &str,
        tr_id: &str,
        body: &Value,
    ) -> Result<JsonResponse> {
        Ok(JsonResponse {
            body: self.post_json(path, tr_id, body).await?,
            tr_cont: None,
        })
    }
}

#[async_trait]
impl ApiClient for KisClient {
    async fn get_json(
        &self,
        path: &str,
        tr_id: &str,
        params: &HashMap<String, String>,
    ) -> Result<Value> {
        let params = params
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect::<Vec<_>>();
        KisClient::get_json(self, path, tr_id, &params)
            .await
            .with_context(|| format!("GET {path} ({tr_id})"))
    }

    async fn post_json(&self, path: &str, tr_id: &str, body: &Value) -> Result<Value> {
        KisClient::post_json(self, path, tr_id, body)
            .await
            .with_context(|| format!("POST {path} ({tr_id})"))
    }

    async fn get_json_response(
        &self,
        path: &str,
        tr_id: &str,
        params: &HashMap<String, String>,
    ) -> Result<JsonResponse> {
        let params = params
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect::<Vec<_>>();
        KisClient::get_json_response(self, path, tr_id, &params)
            .await
            .with_context(|| format!("GET {path} ({tr_id})"))
    }

    async fn get_json_response_with_tr_cont(
        &self,
        path: &str,
        tr_id: &str,
        tr_cont: &str,
        params: &HashMap<String, String>,
    ) -> Result<JsonResponse> {
        let params = params
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect::<Vec<_>>();
        KisClient::get_json_response_with_tr_cont(self, path, tr_id, tr_cont, &params)
            .await
            .with_context(|| format!("GET {path} ({tr_id})"))
    }

    async fn post_json_response(
        &self,
        path: &str,
        tr_id: &str,
        body: &Value,
    ) -> Result<JsonResponse> {
        KisClient::post_json_response(self, path, tr_id, body)
            .await
            .with_context(|| format!("POST {path} ({tr_id})"))
    }
}

#[derive(Debug, Deserialize)]
struct ApiEnvelope<T> {
    rt_cd: String,
    msg_cd: String,
    msg1: String,
    output: T,
}

#[derive(Debug, Deserialize)]
struct ApiEnvelope2<T1, T2> {
    rt_cd: String,
    msg_cd: String,
    msg1: String,
    output1: T1,
    output2: T2,
}

#[derive(Debug, Deserialize)]
struct ApiStatus {
    rt_cd: String,
    msg_cd: String,
    msg1: String,
}

pub(crate) fn ensure_success(value: &Value, label: &str) -> Result<()> {
    let status: ApiStatus = serde_json::from_value(value.clone())
        .with_context(|| format!("parsing {label} response"))?;
    if status.rt_cd != "0" {
        bail!(
            "{label} API error: [{}] {}",
            status.msg_cd,
            status.msg1.trim_end()
        );
    }
    Ok(())
}

pub(crate) fn parse_output<T>(value: Value, label: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    ensure_success(&value, label)?;
    let envelope: ApiEnvelope<T> =
        serde_json::from_value(value).with_context(|| format!("parsing {label} response"))?;
    debug_assert_eq!(envelope.rt_cd, "0");
    debug_assert!(!envelope.msg_cd.is_empty() || !envelope.msg1.is_empty());
    Ok(envelope.output)
}

pub(crate) fn parse_outputs<T1, T2>(value: Value, label: &str) -> Result<(T1, T2)>
where
    T1: DeserializeOwned,
    T2: DeserializeOwned,
{
    ensure_success(&value, label)?;
    let envelope: ApiEnvelope2<T1, T2> =
        serde_json::from_value(value).with_context(|| format!("parsing {label} response"))?;
    debug_assert_eq!(envelope.rt_cd, "0");
    debug_assert!(!envelope.msg_cd.is_empty() || !envelope.msg1.is_empty());
    Ok((envelope.output1, envelope.output2))
}

pub(crate) fn to_json_value<T>(value: T) -> Result<Value>
where
    T: Serialize,
{
    serde_json::to_value(value).context("serializing request body")
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use crate::client::JsonResponse;
    use anyhow::Result;
    use async_trait::async_trait;
    use serde_json::{Value, json};

    use super::{ApiClient, ensure_success, parse_output, parse_outputs};

    #[derive(Clone)]
    struct DefaultResponseClient;

    #[derive(Clone)]
    struct PaginationAwareClient {
        calls: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl ApiClient for DefaultResponseClient {
        async fn get_json(
            &self,
            _path: &str,
            _tr_id: &str,
            _params: &HashMap<String, String>,
        ) -> Result<Value> {
            Ok(json!({ "rt_cd": "0" }))
        }

        async fn post_json(&self, _path: &str, _tr_id: &str, _body: &Value) -> Result<Value> {
            Ok(json!({ "rt_cd": "0" }))
        }
    }

    #[async_trait]
    impl ApiClient for PaginationAwareClient {
        async fn get_json(
            &self,
            _path: &str,
            _tr_id: &str,
            _params: &HashMap<String, String>,
        ) -> Result<Value> {
            unreachable!()
        }

        async fn post_json(&self, _path: &str, _tr_id: &str, _body: &Value) -> Result<Value> {
            unreachable!()
        }

        async fn get_json_response_with_tr_cont(
            &self,
            _path: &str,
            _tr_id: &str,
            tr_cont: &str,
            _params: &HashMap<String, String>,
        ) -> Result<JsonResponse> {
            self.calls.lock().unwrap().push(tr_cont.to_string());
            Ok(JsonResponse {
                body: json!({ "rt_cd": "0" }),
                tr_cont: Some("M".to_string()),
            })
        }
    }

    #[test]
    fn parse_output_reports_api_error_without_output_field() {
        let err = parse_output::<serde_json::Value>(
            json!({
                "rt_cd": "1",
                "msg_cd": "OPSQ0002",
                "msg1": "없는 서비스 코드 입니다"
            }),
            "possible sell",
        )
        .unwrap_err();

        assert_eq!(
            err.to_string(),
            "possible sell API error: [OPSQ0002] 없는 서비스 코드 입니다"
        );
    }

    #[test]
    fn parse_outputs_reports_api_error_without_output_fields() {
        let err = parse_outputs::<Vec<serde_json::Value>, serde_json::Value>(
            json!({
                "rt_cd": "1",
                "msg_cd": "EGW00001",
                "msg1": "잘못된 요청"
            }),
            "daily execution",
        )
        .unwrap_err();

        assert_eq!(
            err.to_string(),
            "daily execution API error: [EGW00001] 잘못된 요청"
        );
    }

    #[test]
    fn ensure_success_reports_api_error_without_output_fields() {
        let err = ensure_success(
            &json!({
                "rt_cd": "1",
                "msg_cd": "EGW00001",
                "msg1": "잘못된 요청"
            }),
            "pagination",
        )
        .unwrap_err();

        assert_eq!(
            err.to_string(),
            "pagination API error: [EGW00001] 잘못된 요청"
        );
    }

    #[tokio::test]
    async fn default_response_wrapper_sets_empty_tr_cont() {
        let client = DefaultResponseClient;
        let response = client
            .get_json_response("/path", "TR", &HashMap::new())
            .await
            .unwrap();

        assert_eq!(response.body["rt_cd"], "0");
        assert_eq!(response.tr_cont, None);
    }

    #[tokio::test]
    async fn get_json_response_uses_pagination_aware_override() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let client = PaginationAwareClient {
            calls: calls.clone(),
        };

        let response = client
            .get_json_response("/path", "TR", &HashMap::new())
            .await
            .unwrap();

        assert_eq!(response.tr_cont.as_deref(), Some("M"));
        assert_eq!(calls.lock().unwrap().as_slice(), [""]);
    }
}
