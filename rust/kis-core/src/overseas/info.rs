use anyhow::{Result, bail};
use serde_json::Value;

use crate::api_client::{ApiClient, parse_output};

use super::price::VALID_EXCHANGES;

const PATH_SEARCH_INFO: &str = "/uapi/overseas-price/v1/quotations/search-info";
const TR_ID_SEARCH_INFO: &str = "CTPF1702R";

pub async fn get_product_info<C>(client: &C, exchange: &str, symbol: &str) -> Result<Value>
where
    C: ApiClient + Sync,
{
    let exchange = normalize_exchange(exchange)?;
    let params = std::collections::HashMap::from([
        (
            "PRDT_TYPE_CD".to_string(),
            product_type_code(&exchange)?.to_string(),
        ),
        ("PDNO".to_string(), symbol.to_uppercase()),
    ]);
    let response = client
        .get_json(PATH_SEARCH_INFO, TR_ID_SEARCH_INFO, &params)
        .await?;
    parse_output(response, "overseas product info")
}

fn normalize_exchange(exchange: &str) -> Result<String> {
    let exchange = exchange.to_uppercase();
    if !VALID_EXCHANGES.contains(&exchange.as_str()) {
        bail!(
            "invalid exchange code {exchange:?}; valid codes: {}",
            VALID_EXCHANGES.join(", ")
        );
    }
    Ok(exchange)
}

fn product_type_code(exchange: &str) -> Result<&'static str> {
    match exchange {
        "NAS" => Ok("512"),
        "NYS" => Ok("513"),
        "AMS" => Ok("529"),
        "TSE" => Ok("515"),
        "HKS" => Ok("501"),
        "SHS" => Ok("551"),
        "SZS" => Ok("552"),
        "HNX" => Ok("507"),
        "HSX" => Ok("508"),
        other => bail!(
            "invalid exchange code {other:?}; valid codes: {}",
            VALID_EXCHANGES.join(", ")
        ),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use serde_json::json;

    use super::*;

    #[derive(Debug, Default, Clone)]
    struct Call {
        path: String,
        tr_id: String,
        params: HashMap<String, String>,
    }

    #[derive(Clone)]
    struct MockClient {
        response: serde_json::Value,
        call: Arc<Mutex<Option<Call>>>,
    }

    #[async_trait]
    impl ApiClient for MockClient {
        async fn get_json(
            &self,
            path: &str,
            tr_id: &str,
            params: &HashMap<String, String>,
        ) -> Result<serde_json::Value> {
            *self.call.lock().unwrap() = Some(Call {
                path: path.to_string(),
                tr_id: tr_id.to_string(),
                params: params.clone(),
            });
            Ok(self.response.clone())
        }

        async fn post_json(
            &self,
            _path: &str,
            _tr_id: &str,
            _body: &serde_json::Value,
        ) -> Result<serde_json::Value> {
            unreachable!()
        }
    }

    #[tokio::test]
    async fn gets_overseas_product_info() {
        let call = Arc::new(Mutex::new(None));
        let client = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "output": {
                    "pdno": "AAPL",
                    "prdt_name": "애플",
                    "prdt_eng_name": "Apple Inc.",
                    "prdt_type_cd": "512",
                    "ovrs_excg_cd": "NAS",
                    "tr_crcy_cd": "USD"
                }
            }),
            call: call.clone(),
        };

        let result = get_product_info(&client, "nas", "aapl").await.unwrap();
        assert_eq!(result["pdno"], "AAPL");
        assert_eq!(result["prdt_type_cd"], "512");

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.path, PATH_SEARCH_INFO);
        assert_eq!(call.tr_id, TR_ID_SEARCH_INFO);
        assert_eq!(call.params["PDNO"], "AAPL");
        assert_eq!(call.params["PRDT_TYPE_CD"], "512");
    }

    #[tokio::test]
    async fn rejects_invalid_exchange() {
        let client = MockClient {
            response: json!({}),
            call: Arc::new(Mutex::new(None)),
        };

        let err = get_product_info(&client, "NASDAQ", "AAPL")
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            "invalid exchange code \"NASDAQ\"; valid codes: NAS, NYS, AMS, TSE, HKS, SHS, SZS, HSX, HNX"
        );
    }
}
