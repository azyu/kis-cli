use std::collections::HashMap;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api_client::{ApiClient, parse_outputs};

use super::price::VALID_EXCHANGES;

const PATH_TRADE_VOL: &str = "/uapi/overseas-stock/v1/ranking/trade-vol";
const TR_ID_TRADE_VOL: &str = "HHDFS76310010";
const PATH_MARKET_CAP: &str = "/uapi/overseas-stock/v1/ranking/market-cap";
const TR_ID_MARKET_CAP: &str = "HHDFS76350100";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OverseasRankingResult {
    pub summary: Vec<Value>,
    pub items: Vec<Value>,
}

pub async fn get_trade_volume_rank<C>(client: &C, exchange: &str) -> Result<OverseasRankingResult>
where
    C: ApiClient + Sync,
{
    let exchange = normalize_exchange(exchange)?;
    let params = HashMap::from([
        ("EXCD".to_string(), exchange),
        ("NDAY".to_string(), "0".to_string()),
        ("VOL_RANG".to_string(), "0".to_string()),
        ("KEYB".to_string(), "".to_string()),
        ("AUTH".to_string(), "".to_string()),
        ("PRC1".to_string(), "".to_string()),
        ("PRC2".to_string(), "".to_string()),
    ]);
    fetch_ranking(
        client,
        PATH_TRADE_VOL,
        TR_ID_TRADE_VOL,
        &params,
        "overseas trade volume rank",
    )
    .await
}

pub async fn get_market_cap_rank<C>(client: &C, exchange: &str) -> Result<OverseasRankingResult>
where
    C: ApiClient + Sync,
{
    let exchange = normalize_exchange(exchange)?;
    let params = HashMap::from([
        ("EXCD".to_string(), exchange),
        ("VOL_RANG".to_string(), "0".to_string()),
        ("KEYB".to_string(), "".to_string()),
        ("AUTH".to_string(), "".to_string()),
    ]);
    fetch_ranking(
        client,
        PATH_MARKET_CAP,
        TR_ID_MARKET_CAP,
        &params,
        "overseas market cap rank",
    )
    .await
}

async fn fetch_ranking<C>(
    client: &C,
    path: &str,
    tr_id: &str,
    params: &HashMap<String, String>,
    label: &str,
) -> Result<OverseasRankingResult>
where
    C: ApiClient + Sync,
{
    let mut summary = Vec::new();
    let mut items = Vec::new();
    let mut tr_cont = String::new();

    loop {
        let response = client
            .get_json_response_with_tr_cont(path, tr_id, &tr_cont, params)
            .await?;
        let next_tr_cont = response.tr_cont.clone();
        let (page_summary, page_items): (Value, Vec<Value>) = parse_outputs(response.body, label)?;
        summary.push(page_summary);
        items.extend(page_items);

        match next_tr_cont.as_deref() {
            Some("M" | "F") => tr_cont = "N".to_string(),
            _ => return Ok(OverseasRankingResult { summary, items }),
        }
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use serde_json::json;

    use crate::client::JsonResponse;

    use super::*;

    #[derive(Debug, Default, Clone)]
    struct ResponseCall {
        path: String,
        tr_id: String,
        tr_cont: String,
        params: HashMap<String, String>,
    }

    #[derive(Clone)]
    struct MockClient {
        responses: Arc<Mutex<Vec<JsonResponse>>>,
        response_calls: Arc<Mutex<Vec<ResponseCall>>>,
    }

    #[async_trait]
    impl ApiClient for MockClient {
        async fn get_json(
            &self,
            _path: &str,
            _tr_id: &str,
            _params: &HashMap<String, String>,
        ) -> Result<serde_json::Value> {
            unreachable!()
        }

        async fn post_json(
            &self,
            _path: &str,
            _tr_id: &str,
            _body: &serde_json::Value,
        ) -> Result<serde_json::Value> {
            unreachable!()
        }

        async fn get_json_response_with_tr_cont(
            &self,
            path: &str,
            tr_id: &str,
            tr_cont: &str,
            params: &HashMap<String, String>,
        ) -> Result<JsonResponse> {
            self.response_calls.lock().unwrap().push(ResponseCall {
                path: path.to_string(),
                tr_id: tr_id.to_string(),
                tr_cont: tr_cont.to_string(),
                params: params.clone(),
            });
            Ok(self.responses.lock().unwrap().remove(0))
        }
    }

    #[tokio::test]
    async fn gets_trade_volume_rank_and_paginates() {
        let client = MockClient {
            responses: Arc::new(Mutex::new(vec![
                JsonResponse {
                    body: json!({
                        "rt_cd": "0",
                        "msg_cd": "MCA00000",
                        "msg1": "정상처리",
                        "output1": {
                            "excd": "NAS",
                            "crec": "20",
                            "trec": "40"
                        },
                        "output2": [{
                            "rank": "1",
                            "symb": "AAPL",
                            "name": "Apple Inc.",
                            "last": "175.40",
                            "diff": "1.20",
                            "rate": "0.69",
                            "tvol": "51234567"
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
                            "excd": "NAS",
                            "crec": "20",
                            "trec": "40"
                        },
                        "output2": [{
                            "rank": "2",
                            "symb": "MSFT",
                            "name": "Microsoft",
                            "last": "410.00",
                            "diff": "2.10",
                            "rate": "0.51",
                            "tvol": "30123456"
                        }]
                    }),
                    tr_cont: None,
                },
            ])),
            response_calls: Arc::new(Mutex::new(Vec::new())),
        };

        let result = get_trade_volume_rank(&client, "nas").await.unwrap();
        assert_eq!(result.summary.len(), 2);
        assert_eq!(result.items.len(), 2);
        assert_eq!(result.items[0]["symb"], "AAPL");
        assert_eq!(result.items[1]["symb"], "MSFT");

        let calls = client.response_calls.lock().unwrap().clone();
        assert_eq!(calls[0].path, PATH_TRADE_VOL);
        assert_eq!(calls[0].tr_id, TR_ID_TRADE_VOL);
        assert_eq!(calls[0].tr_cont, "");
        assert_eq!(calls[0].params["EXCD"], "NAS");
        assert_eq!(calls[0].params["NDAY"], "0");
        assert_eq!(calls[0].params["VOL_RANG"], "0");
        assert_eq!(calls[0].params["PRC1"], "");
        assert_eq!(calls[0].params["PRC2"], "");
        assert_eq!(calls[1].tr_cont, "N");
    }

    #[tokio::test]
    async fn gets_market_cap_rank() {
        let client = MockClient {
            responses: Arc::new(Mutex::new(vec![JsonResponse {
                body: json!({
                    "rt_cd": "0",
                    "msg_cd": "MCA00000",
                    "msg1": "정상처리",
                    "output1": {
                        "excd": "NYS",
                        "crec": "20",
                        "trec": "20"
                    },
                    "output2": [{
                        "rank": "1",
                        "symb": "BRK.B",
                        "name": "Berkshire Hathaway",
                        "last": "470.00",
                        "rate": "0.42",
                        "tvol": "1234567",
                        "tomv": "1030000000000"
                    }]
                }),
                tr_cont: None,
            }])),
            response_calls: Arc::new(Mutex::new(Vec::new())),
        };

        let result = get_market_cap_rank(&client, "nys").await.unwrap();
        assert_eq!(result.summary.len(), 1);
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0]["symb"], "BRK.B");

        let calls = client.response_calls.lock().unwrap().clone();
        assert_eq!(calls[0].path, PATH_MARKET_CAP);
        assert_eq!(calls[0].tr_id, TR_ID_MARKET_CAP);
        assert_eq!(calls[0].params["EXCD"], "NYS");
        assert_eq!(calls[0].params["VOL_RANG"], "0");
        assert_eq!(calls[0].params["AUTH"], "");
        assert_eq!(calls[0].params["KEYB"], "");
    }
}
