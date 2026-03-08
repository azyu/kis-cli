use std::collections::HashMap;

use anyhow::{Result, bail};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::api_client::{ApiClient, parse_output, parse_outputs};

const PATH_PRICE: &str = "/uapi/overseas-price/v1/quotations/price";
const TR_ID_PRICE: &str = "HHDFS00000300";
const PATH_DAILY_PRICE: &str = "/uapi/overseas-price/v1/quotations/dailyprice";
const TR_ID_DAILY_PRICE: &str = "HHDFS76240000";

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
pub struct OverseasDailyPrice {
    pub xymd: String,
    pub open: String,
    pub high: String,
    pub low: String,
    pub clos: String,
    pub tvol: String,
    pub name: String,
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct OverseasDailyMeta {
    #[serde(default)]
    rsym: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct RawOverseasDailyPrice {
    xymd: String,
    open: String,
    high: String,
    low: String,
    clos: String,
    tvol: String,
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

impl OverseasDailyPrice {
    fn from_parts(meta: &OverseasDailyMeta, symbol: &str, raw: RawOverseasDailyPrice) -> Self {
        Self {
            xymd: raw.xymd,
            open: raw.open,
            high: raw.high,
            low: raw.low,
            clos: raw.clos,
            tvol: raw.tvol,
            name: daily_price_name(meta, symbol),
        }
    }
}

pub async fn get_price<C>(client: &C, exchange: &str, symbol: &str) -> Result<OverseasPrice>
where
    C: ApiClient + Sync,
{
    let exchange = normalize_exchange(exchange)?;
    let symbol = symbol.to_uppercase();

    let params = HashMap::from([
        ("AUTH".to_string(), "".to_string()),
        ("EXCD".to_string(), exchange),
        ("SYMB".to_string(), symbol),
    ]);
    let response = client.get_json(PATH_PRICE, TR_ID_PRICE, &params).await?;
    let raw: RawOverseasPrice = parse_output(response, "overseas price")?;
    Ok(raw.into())
}

pub async fn get_daily_price<C>(
    client: &C,
    exchange: &str,
    symbol: &str,
    period: Option<&str>,
) -> Result<Vec<OverseasDailyPrice>>
where
    C: ApiClient + Sync,
{
    let exchange = normalize_exchange(exchange)?;
    let symbol = symbol.to_uppercase();
    let period = normalize_period(period)?;
    let params = HashMap::from([
        ("AUTH".to_string(), "".to_string()),
        ("EXCD".to_string(), exchange),
        ("SYMB".to_string(), symbol.clone()),
        ("GUBN".to_string(), period.to_string()),
        ("BYMD".to_string(), Utc::now().format("%Y%m%d").to_string()),
        ("MODP".to_string(), "0".to_string()),
    ]);

    let mut items = Vec::new();
    let mut tr_cont = String::new();

    loop {
        let response = client
            .get_json_response_with_tr_cont(PATH_DAILY_PRICE, TR_ID_DAILY_PRICE, &tr_cont, &params)
            .await?;
        let next_tr_cont = response.tr_cont.clone();
        let (meta, page): (OverseasDailyMeta, Vec<RawOverseasDailyPrice>) =
            parse_outputs(response.body, "overseas daily price")?;
        items.extend(
            page.into_iter()
                .map(|entry| OverseasDailyPrice::from_parts(&meta, &symbol, entry)),
        );

        match next_tr_cont.as_deref() {
            Some("M" | "F") => tr_cont = "N".to_string(),
            _ => return Ok(items),
        }
    }
}

fn default_field(value: String) -> String {
    if value.is_empty() {
        "-".to_string()
    } else {
        value
    }
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

fn normalize_period(period: Option<&str>) -> Result<&'static str> {
    match period.unwrap_or("D").to_ascii_uppercase().as_str() {
        "D" => Ok("0"),
        "W" => Ok("1"),
        "M" => Ok("2"),
        other => bail!("invalid period {other:?}; valid periods: D, W, M"),
    }
}

fn strip_realtime_symbol_prefix(value: &str) -> String {
    if value.len() > 4 {
        value[4..].to_string()
    } else {
        value.to_string()
    }
}

fn daily_price_name(meta: &OverseasDailyMeta, symbol: &str) -> String {
    let name = strip_realtime_symbol_prefix(&meta.rsym);
    if name.is_empty() {
        symbol.to_string()
    } else {
        name
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::client::JsonResponse;
    use async_trait::async_trait;
    use serde_json::json;

    use super::*;

    #[derive(Debug, Default, Clone)]
    struct GetCall {
        tr_id: String,
        params: HashMap<String, String>,
    }

    #[derive(Debug, Default, Clone)]
    struct ResponseCall {
        tr_id: String,
        tr_cont: String,
        params: HashMap<String, String>,
    }

    #[derive(Clone)]
    struct MockClient {
        response: serde_json::Value,
        call: Arc<Mutex<Option<GetCall>>>,
        responses: Arc<Mutex<Vec<JsonResponse>>>,
        response_calls: Arc<Mutex<Vec<ResponseCall>>>,
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

        async fn get_json_response_with_tr_cont(
            &self,
            _path: &str,
            tr_id: &str,
            tr_cont: &str,
            params: &HashMap<String, String>,
        ) -> Result<JsonResponse> {
            self.response_calls.lock().unwrap().push(ResponseCall {
                tr_id: tr_id.to_string(),
                tr_cont: tr_cont.to_string(),
                params: params.clone(),
            });
            Ok(self.responses.lock().unwrap().remove(0))
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
            responses: Arc::new(Mutex::new(Vec::new())),
            response_calls: Arc::new(Mutex::new(Vec::new())),
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
            responses: Arc::new(Mutex::new(Vec::new())),
            response_calls: Arc::new(Mutex::new(Vec::new())),
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
            responses: Arc::new(Mutex::new(Vec::new())),
            response_calls: Arc::new(Mutex::new(Vec::new())),
        };

        let err = get_price(&client, "invalid", "AAPL").await.unwrap_err();
        assert!(err.to_string().contains("invalid exchange code"));
    }

    #[tokio::test]
    async fn gets_overseas_daily_price_and_paginates() {
        let response_calls = Arc::new(Mutex::new(Vec::new()));
        let client = MockClient {
            response: json!({}),
            call: Arc::new(Mutex::new(None)),
            responses: Arc::new(Mutex::new(vec![
                JsonResponse {
                    body: json!({
                        "rt_cd": "0",
                        "msg_cd": "MCA00000",
                        "msg1": "정상처리",
                        "output1": {
                            "rsym": "DNASAAPL"
                        },
                        "output2": [{
                            "xymd": "20260306",
                            "open": "174.20",
                            "high": "176.10",
                            "low": "173.80",
                            "clos": "175.90",
                            "tvol": "5000"
                        }]
                    }),
                    tr_cont: Some("M".to_string()),
                },
                JsonResponse {
                    body: json!({
                        "rt_cd": "0",
                        "msg_cd": "MCA00000",
                        "msg1": "정상처리",
                        "output1": {
                            "rsym": "DNASAAPL"
                        },
                        "output2": [{
                            "xymd": "20260305",
                            "open": "173.10",
                            "high": "175.30",
                            "low": "172.40",
                            "clos": "174.80",
                            "tvol": "4800"
                        }]
                    }),
                    tr_cont: None,
                },
            ])),
            response_calls: response_calls.clone(),
        };

        let items = get_daily_price(&client, "nas", "aapl", Some("W"))
            .await
            .unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].xymd, "20260306");
        assert_eq!(items[1].xymd, "20260305");
        assert_eq!(items[0].name, "AAPL");

        let calls = response_calls.lock().unwrap().clone();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].tr_id, "HHDFS76240000");
        assert_eq!(calls[0].tr_cont, "");
        assert_eq!(calls[0].params["EXCD"], "NAS");
        assert_eq!(calls[0].params["SYMB"], "AAPL");
        assert_eq!(calls[0].params["GUBN"], "1");
        assert_eq!(calls[0].params["MODP"], "0");
        assert_eq!(calls[1].tr_cont, "N");
    }

    #[tokio::test]
    async fn rejects_invalid_daily_price_period() {
        let client = MockClient {
            response: json!({}),
            call: Arc::new(Mutex::new(None)),
            responses: Arc::new(Mutex::new(Vec::new())),
            response_calls: Arc::new(Mutex::new(Vec::new())),
        };

        let err = get_daily_price(&client, "NAS", "AAPL", Some("Q"))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("invalid period"));
    }
}
