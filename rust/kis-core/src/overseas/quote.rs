use std::collections::HashMap;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api_client::{ApiClient, ensure_success};

use super::price::VALID_EXCHANGES;

const PATH_INQUIRE_ASKING_PRICE: &str = "/uapi/overseas-price/v1/quotations/inquire-asking-price";
const TR_ID_INQUIRE_ASKING_PRICE: &str = "HHDFS76200100";
const PATH_INQUIRE_CCNL: &str = "/uapi/overseas-price/v1/quotations/inquire-ccnl";
const TR_ID_INQUIRE_CCNL: &str = "HHDFS76200300";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct OverseasQuoteSnapshot {
    #[serde(default)]
    pub rsym: String,
    #[serde(default)]
    pub zdiv: String,
    #[serde(default)]
    pub open: String,
    #[serde(default)]
    pub high: String,
    #[serde(default)]
    pub low: String,
    #[serde(default)]
    pub last: String,
    #[serde(default)]
    pub diff: String,
    #[serde(default)]
    pub rate: String,
    #[serde(default)]
    pub pbid: String,
    #[serde(default)]
    pub pask: String,
    #[serde(default)]
    pub tvol: String,
    #[serde(default)]
    pub tamt: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct OverseasAskingLevel {
    #[serde(default)]
    pub askp1: String,
    #[serde(default)]
    pub bidp1: String,
    #[serde(default)]
    pub askv1: String,
    #[serde(default)]
    pub bidv1: String,
    #[serde(default)]
    pub askp2: String,
    #[serde(default)]
    pub bidp2: String,
    #[serde(default)]
    pub askv2: String,
    #[serde(default)]
    pub bidv2: String,
    #[serde(default)]
    pub askp3: String,
    #[serde(default)]
    pub bidp3: String,
    #[serde(default)]
    pub askv3: String,
    #[serde(default)]
    pub bidv3: String,
    #[serde(default)]
    pub askp4: String,
    #[serde(default)]
    pub bidp4: String,
    #[serde(default)]
    pub askv4: String,
    #[serde(default)]
    pub bidv4: String,
    #[serde(default)]
    pub askp5: String,
    #[serde(default)]
    pub bidp5: String,
    #[serde(default)]
    pub askv5: String,
    #[serde(default)]
    pub bidv5: String,
    #[serde(default)]
    pub askp6: String,
    #[serde(default)]
    pub bidp6: String,
    #[serde(default)]
    pub askv6: String,
    #[serde(default)]
    pub bidv6: String,
    #[serde(default)]
    pub askp7: String,
    #[serde(default)]
    pub bidp7: String,
    #[serde(default)]
    pub askv7: String,
    #[serde(default)]
    pub bidv7: String,
    #[serde(default)]
    pub askp8: String,
    #[serde(default)]
    pub bidp8: String,
    #[serde(default)]
    pub askv8: String,
    #[serde(default)]
    pub bidv8: String,
    #[serde(default)]
    pub askp9: String,
    #[serde(default)]
    pub bidp9: String,
    #[serde(default)]
    pub askv9: String,
    #[serde(default)]
    pub bidv9: String,
    #[serde(default)]
    pub askp10: String,
    #[serde(default)]
    pub bidp10: String,
    #[serde(default)]
    pub askv10: String,
    #[serde(default)]
    pub bidv10: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct OverseasAskingSummary {
    #[serde(default)]
    pub total_askp_rsqn: String,
    #[serde(default)]
    pub total_bidp_rsqn: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct OverseasAskingPrice {
    pub quote: OverseasQuoteSnapshot,
    pub levels: Vec<OverseasAskingLevel>,
    pub summary: OverseasAskingSummary,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct OverseasConclusion {
    #[serde(default)]
    pub xymd: String,
    #[serde(default)]
    pub xhms: String,
    #[serde(default)]
    pub last: String,
    #[serde(default)]
    pub sign: String,
    #[serde(default)]
    pub diff: String,
    #[serde(default)]
    pub rate: String,
    #[serde(default)]
    pub pbid: String,
    #[serde(default)]
    pub pask: String,
    #[serde(default)]
    pub evol: String,
    #[serde(default)]
    pub tvol: String,
    #[serde(default)]
    pub tamt: String,
}

#[derive(Debug, Deserialize)]
struct AskingEnvelope {
    output1: OverseasQuoteSnapshot,
    output2: Vec<OverseasAskingLevel>,
    #[serde(default)]
    output3: OverseasAskingSummary,
}

#[derive(Debug, Deserialize)]
struct ConclusionEnvelope {
    output1: Vec<OverseasConclusion>,
}

pub async fn get_asking_price<C>(
    client: &C,
    exchange: &str,
    symbol: &str,
) -> Result<OverseasAskingPrice>
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
    let response = client
        .get_json(
            PATH_INQUIRE_ASKING_PRICE,
            TR_ID_INQUIRE_ASKING_PRICE,
            &params,
        )
        .await?;
    parse_asking_price(response)
}

pub async fn get_conclusions<C>(
    client: &C,
    exchange: &str,
    symbol: &str,
) -> Result<Vec<OverseasConclusion>>
where
    C: ApiClient + Sync,
{
    let exchange = normalize_exchange(exchange)?;
    let symbol = symbol.to_uppercase();
    let params = HashMap::from([
        ("AUTH".to_string(), "".to_string()),
        ("EXCD".to_string(), exchange),
        ("TDAY".to_string(), "1".to_string()),
        ("SYMB".to_string(), symbol),
        ("KEYB".to_string(), "".to_string()),
    ]);
    let response = client
        .get_json(PATH_INQUIRE_CCNL, TR_ID_INQUIRE_CCNL, &params)
        .await?;
    parse_conclusions(response)
}

fn parse_asking_price(value: Value) -> Result<OverseasAskingPrice> {
    ensure_success(&value, "overseas asking price")?;
    let envelope: AskingEnvelope = serde_json::from_value(value)?;
    Ok(OverseasAskingPrice {
        quote: envelope.output1,
        levels: envelope.output2,
        summary: envelope.output3,
    })
}

fn parse_conclusions(value: Value) -> Result<Vec<OverseasConclusion>> {
    ensure_success(&value, "overseas conclusions")?;
    let envelope: ConclusionEnvelope = serde_json::from_value(value)?;
    Ok(envelope.output1)
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use anyhow::Result;
    use async_trait::async_trait;
    use serde_json::json;

    use crate::api_client::ApiClient;

    use super::{
        PATH_INQUIRE_ASKING_PRICE, PATH_INQUIRE_CCNL, TR_ID_INQUIRE_ASKING_PRICE,
        TR_ID_INQUIRE_CCNL, get_asking_price, get_conclusions,
    };

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
    async fn gets_overseas_asking_price() {
        let call = Arc::new(Mutex::new(None));
        let client = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "output1": {
                    "rsym": "AAPL",
                    "zdiv": "0.0000",
                    "open": "145.6300",
                    "high": "147.8400",
                    "low": "145.5600",
                    "last": "147.2700",
                    "diff": "1.2500",
                    "rate": "0.8560",
                    "pbid": "147.2600",
                    "pask": "147.2800",
                    "tvol": "46126782",
                    "tamt": "6797668389"
                },
                "output2": [{
                    "askp1": "147.2800",
                    "bidp1": "147.2600",
                    "askv1": "100",
                    "bidv1": "120",
                    "askp2": "147.2900",
                    "bidp2": "147.2500",
                    "askv2": "80",
                    "bidv2": "95"
                }],
                "output3": {
                    "total_askp_rsqn": "180",
                    "total_bidp_rsqn": "215"
                }
            }),
            call: call.clone(),
        };

        let result = get_asking_price(&client, "nas", "aapl").await.unwrap();
        assert_eq!(result.quote.last, "147.2700");
        assert_eq!(result.levels[0].askp1, "147.2800");
        assert_eq!(result.summary.total_bidp_rsqn, "215");

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.path, PATH_INQUIRE_ASKING_PRICE);
        assert_eq!(call.tr_id, TR_ID_INQUIRE_ASKING_PRICE);
        assert_eq!(call.params["EXCD"], "NAS");
        assert_eq!(call.params["SYMB"], "AAPL");
    }

    #[tokio::test]
    async fn gets_overseas_conclusions() {
        let call = Arc::new(Mutex::new(None));
        let client = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "keyb": "",
                "output1": [{
                    "xymd": "20260307",
                    "xhms": "093000",
                    "last": "147.2700",
                    "sign": "2",
                    "diff": "1.2500",
                    "rate": "0.8560",
                    "pbid": "147.2600",
                    "pask": "147.2800",
                    "evol": "100",
                    "tvol": "46126782",
                    "tamt": "6797668389"
                }]
            }),
            call: call.clone(),
        };

        let result = get_conclusions(&client, "nys", "ibm").await.unwrap();
        assert_eq!(result[0].xhms, "093000");
        assert_eq!(result[0].last, "147.2700");

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.path, PATH_INQUIRE_CCNL);
        assert_eq!(call.tr_id, TR_ID_INQUIRE_CCNL);
        assert_eq!(call.params["EXCD"], "NYS");
        assert_eq!(call.params["TDAY"], "1");
        assert_eq!(call.params["SYMB"], "IBM");
        assert_eq!(call.params["AUTH"], "");
        assert_eq!(call.params["KEYB"], "");
    }

    #[tokio::test]
    async fn rejects_invalid_exchange() {
        let client = MockClient {
            response: json!({}),
            call: Arc::new(Mutex::new(None)),
        };

        let err = get_asking_price(&client, "invalid", "AAPL")
            .await
            .unwrap_err();
        assert!(err.to_string().contains("invalid exchange code"));
    }
}
