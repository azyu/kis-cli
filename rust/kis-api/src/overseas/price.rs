use std::collections::HashMap;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

use crate::client::{ApiClient, parse_output};

const PATH_PRICE: &str = "/uapi/overseas-price/v1/quotations/price";
const TR_ID_PRICE: &str = "HHDFS00000300";

pub const VALID_EXCHANGES: &[&str] = &[
    "NAS", "NYS", "AMS", "TSE", "HKS", "SHS", "SZS", "HSX", "HNX",
];

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct OverseasPrice {
    pub last: String,
    pub diff: String,
    pub rate: String,
    pub tvol: String,
    pub tamt: String,
    pub ordy: String,
    pub base: String,
    pub name: String,
    pub open: String,
    pub high: String,
    pub low: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct RawOverseasPrice {
    pub last: String,
    pub diff: String,
    pub rate: String,
    pub tvol: String,
    pub tamt: String,
    pub ordy: String,
    pub base: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub open: String,
    #[serde(default)]
    pub high: String,
    #[serde(default)]
    pub low: String,
    #[serde(default)]
    pub rsym: String,
}

impl From<RawOverseasPrice> for OverseasPrice {
    fn from(raw: RawOverseasPrice) -> Self {
        Self {
            last: raw.last,
            diff: raw.diff,
            rate: raw.rate,
            tvol: raw.tvol,
            tamt: raw.tamt,
            ordy: raw.ordy,
            base: raw.base,
            name: if raw.name.is_empty() {
                raw.rsym
            } else {
                raw.name
            },
            open: default_field(raw.open),
            high: default_field(raw.high),
            low: default_field(raw.low),
        }
    }
}

pub async fn get_price<C>(client: &C, exchange: &str, symbol: &str) -> Result<OverseasPrice>
where
    C: ApiClient + Sync,
{
    let exchange = exchange.to_uppercase();
    let symbol = symbol.to_uppercase();
    if !VALID_EXCHANGES.contains(&exchange.as_str()) {
        bail!(
            "invalid exchange code {exchange:?}; valid codes: {}",
            VALID_EXCHANGES.join(", ")
        );
    }

    let params = HashMap::from([
        ("AUTH".to_string(), "".to_string()),
        ("EXCD".to_string(), exchange),
        ("SYMB".to_string(), symbol),
    ]);
    let response = client.get_json(PATH_PRICE, TR_ID_PRICE, &params).await?;
    let raw: RawOverseasPrice = parse_output(response, "overseas price")?;
    Ok(raw.into())
}

fn default_field(value: String) -> String {
    if value.is_empty() {
        "-".to_string()
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use serde_json::json;

    use super::*;

    #[derive(Debug, Default, Clone)]
    struct GetCall {
        tr_id: String,
        params: HashMap<String, String>,
    }

    #[derive(Clone)]
    struct MockClient {
        response: serde_json::Value,
        call: Arc<Mutex<Option<GetCall>>>,
    }

    #[async_trait]
    impl ApiClient for MockClient {
        async fn get_json(
            &self,
            _path: &str,
            tr_id: &str,
            params: &HashMap<String, String>,
        ) -> Result<serde_json::Value> {
            *self.call.lock().unwrap() = Some(GetCall {
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
    async fn gets_overseas_price() {
        let call = Arc::new(Mutex::new(None));
        let client = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "output": {
                    "last": "150.25",
                    "diff": "2.50",
                    "rate": "1.69",
                    "tvol": "50000000",
                    "tamt": "100000000",
                    "ordy": "Y",
                    "base": "147.75",
                    "name": "APPLE INC",
                    "open": "148.00",
                    "high": "151.00",
                    "low": "147.50"
                }
            }),
            call: call.clone(),
        };

        let price = get_price(&client, "nas", "aapl").await.unwrap();
        assert_eq!(price.name, "APPLE INC");

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.tr_id, TR_ID_PRICE);
        assert_eq!(call.params["EXCD"], "NAS");
        assert_eq!(call.params["SYMB"], "AAPL");
    }

    #[tokio::test]
    async fn parses_live_shape_without_optional_fields() {
        let client = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리 되었습니다.",
                "output": {
                    "rsym": "DNASAAPL",
                    "zdiv": "4",
                    "base": "262.5200",
                    "pvol": "39803119",
                    "last": "260.2900",
                    "sign": "5",
                    "diff": "2.2300",
                    "rate": "-0.85",
                    "tvol": "49658626",
                    "tamt": "12902645154",
                    "ordy": "매도불가"
                }
            }),
            call: Arc::new(Mutex::new(None)),
        };

        let price = get_price(&client, "NAS", "AAPL").await.unwrap();
        assert_eq!(price.name, "DNASAAPL");
        assert_eq!(price.open, "-");
        assert_eq!(price.high, "-");
        assert_eq!(price.low, "-");
        assert_eq!(price.last, "260.2900");
    }

    #[tokio::test]
    async fn rejects_invalid_exchange() {
        let client = MockClient {
            response: json!({}),
            call: Arc::new(Mutex::new(None)),
        };

        let err = get_price(&client, "invalid", "AAPL").await.unwrap_err();
        assert!(err.to_string().contains("invalid exchange code"));
    }
}
