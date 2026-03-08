use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api_client::{ApiClient, parse_output, parse_outputs};

use super::price::VALID_EXCHANGES;

const PATH_SEARCH_INFO: &str = "/uapi/overseas-price/v1/quotations/search-info";
const TR_ID_SEARCH_INFO: &str = "CTPF1702R";
const PATH_INQUIRE_SEARCH: &str = "/uapi/overseas-price/v1/quotations/inquire-search";
const TR_ID_INQUIRE_SEARCH: &str = "HHDFS76410000";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RangeFilter {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OverseasScreenerRequest {
    pub exchange: String,
    pub price: Option<RangeFilter>,
    pub rate: Option<RangeFilter>,
    pub market_cap: Option<RangeFilter>,
    pub shares: Option<RangeFilter>,
    pub volume: Option<RangeFilter>,
    pub amount: Option<RangeFilter>,
    pub eps: Option<RangeFilter>,
    pub per: Option<RangeFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OverseasScreenerResult {
    pub summary: Vec<Value>,
    pub items: Vec<Value>,
}

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

pub async fn inquire_search<C>(
    client: &C,
    request: &OverseasScreenerRequest,
) -> Result<OverseasScreenerResult>
where
    C: ApiClient + Sync,
{
    let exchange = normalize_exchange(&request.exchange)?;
    let mut params = std::collections::HashMap::from([
        ("AUTH".to_string(), "".to_string()),
        ("EXCD".to_string(), exchange),
        ("KEYB".to_string(), "".to_string()),
    ]);

    insert_range(&mut params, "PRICECUR", request.price.as_ref());
    insert_range(&mut params, "RATE", request.rate.as_ref());
    insert_range(&mut params, "VALX", request.market_cap.as_ref());
    insert_range(&mut params, "SHAR", request.shares.as_ref());
    insert_range(&mut params, "VOLUME", request.volume.as_ref());
    insert_range(&mut params, "AMT", request.amount.as_ref());
    insert_range(&mut params, "EPS", request.eps.as_ref());
    insert_range(&mut params, "PER", request.per.as_ref());

    let mut summary = Vec::new();
    let mut items = Vec::new();
    let mut tr_cont = String::new();

    loop {
        let response = client
            .get_json_response_with_tr_cont(
                PATH_INQUIRE_SEARCH,
                TR_ID_INQUIRE_SEARCH,
                &tr_cont,
                &params,
            )
            .await?;
        let next_tr_cont = response.tr_cont.clone();
        let (page_summary, page_items): (Value, Vec<Value>) =
            parse_outputs(response.body, "overseas screener")?;
        summary.push(page_summary);
        items.extend(page_items);

        match next_tr_cont.as_deref() {
            Some("M" | "F") => tr_cont = "N".to_string(),
            _ => return Ok(OverseasScreenerResult { summary, items }),
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

fn insert_range(
    params: &mut std::collections::HashMap<String, String>,
    suffix: &str,
    range: Option<&RangeFilter>,
) {
    let enabled = range.is_some();
    params.insert(
        format!("CO_YN_{suffix}"),
        if enabled { "1" } else { "" }.to_string(),
    );
    params.insert(
        format!("CO_ST_{suffix}"),
        range.map(|item| item.start.clone()).unwrap_or_default(),
    );
    params.insert(
        format!("CO_EN_{suffix}"),
        range.map(|item| item.end.clone()).unwrap_or_default(),
    );
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

    #[derive(Debug, Default, Clone)]
    struct ResponseCall {
        path: String,
        tr_id: String,
        tr_cont: String,
        params: HashMap<String, String>,
    }

    #[derive(Clone)]
    struct ResponseClient {
        responses: Arc<Mutex<Vec<crate::client::JsonResponse>>>,
        response_calls: Arc<Mutex<Vec<ResponseCall>>>,
    }

    #[async_trait]
    impl ApiClient for ResponseClient {
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
        ) -> Result<crate::client::JsonResponse> {
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
    async fn inquires_overseas_screener_and_paginates() {
        let client = ResponseClient {
            responses: Arc::new(Mutex::new(vec![
                crate::client::JsonResponse {
                    body: json!({
                        "rt_cd": "0",
                        "msg_cd": "MCA00000",
                        "msg1": "정상처리",
                        "output1": {
                            "excd": "NAS",
                            "crec": "20",
                            "trec": "25"
                        },
                        "output2": [{
                            "rank": "1",
                            "symb": "AAPL",
                            "last": "165.10",
                            "rate": "1.24",
                            "tvol": "12345678",
                            "valx": "2500000",
                            "eps": "6.43",
                            "per": "25.66"
                        }]
                    }),
                    tr_cont: Some("M".to_string()),
                },
                crate::client::JsonResponse {
                    body: json!({
                        "rt_cd": "0",
                        "msg_cd": "MCA00000",
                        "msg1": "정상처리",
                        "output1": {
                            "excd": "NAS",
                            "crec": "5",
                            "trec": "25"
                        },
                        "output2": [{
                            "rank": "2",
                            "symb": "AMD",
                            "last": "167.50",
                            "rate": "2.10",
                            "tvol": "87654321",
                            "valx": "320000",
                            "eps": "4.10",
                            "per": "40.85"
                        }]
                    }),
                    tr_cont: None,
                },
            ])),
            response_calls: Arc::new(Mutex::new(Vec::new())),
        };
        let request = OverseasScreenerRequest {
            exchange: "nas".to_string(),
            price: Some(RangeFilter {
                start: "160".to_string(),
                end: "170".to_string(),
            }),
            rate: None,
            market_cap: Some(RangeFilter {
                start: "100000".to_string(),
                end: "3000000".to_string(),
            }),
            shares: None,
            volume: None,
            amount: None,
            eps: None,
            per: None,
        };

        let result = inquire_search(&client, &request).await.unwrap();
        assert_eq!(result.summary.len(), 2);
        assert_eq!(result.items.len(), 2);
        assert_eq!(result.items[0]["symb"], "AAPL");
        assert_eq!(result.items[1]["symb"], "AMD");

        let calls = client.response_calls.lock().unwrap().clone();
        assert_eq!(calls[0].path, PATH_INQUIRE_SEARCH);
        assert_eq!(calls[0].tr_id, TR_ID_INQUIRE_SEARCH);
        assert_eq!(calls[0].tr_cont, "");
        assert_eq!(calls[0].params["EXCD"], "NAS");
        assert_eq!(calls[0].params["CO_YN_PRICECUR"], "1");
        assert_eq!(calls[0].params["CO_ST_PRICECUR"], "160");
        assert_eq!(calls[0].params["CO_EN_PRICECUR"], "170");
        assert_eq!(calls[0].params["CO_YN_VALX"], "1");
        assert_eq!(calls[0].params["CO_ST_VALX"], "100000");
        assert_eq!(calls[0].params["CO_EN_VALX"], "3000000");
        assert_eq!(calls[0].params["CO_YN_RATE"], "");
        assert_eq!(calls[0].params["AUTH"], "");
        assert_eq!(calls[0].params["KEYB"], "");
        assert_eq!(calls[1].tr_cont, "N");
    }
}
