use std::collections::HashMap;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api_client::{ApiClient, parse_outputs};

use super::price::VALID_EXCHANGES;

const PATH_DAILY_CHART: &str = "/uapi/overseas-price/v1/quotations/inquire-daily-chartprice";
const TR_ID_DAILY_CHART: &str = "FHKST03030100";
const PATH_TIME_CHART: &str = "/uapi/overseas-price/v1/quotations/inquire-time-itemchartprice";
const TR_ID_TIME_CHART: &str = "HHDFS76950200";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct OverseasDailyChartItem {
    #[serde(default)]
    pub xymd: String,
    #[serde(default)]
    pub open: String,
    #[serde(default)]
    pub high: String,
    #[serde(default)]
    pub low: String,
    #[serde(default)]
    pub clos: String,
    #[serde(default)]
    pub tvol: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct OverseasTimeChartItem {
    #[serde(default)]
    pub xymd: String,
    #[serde(default)]
    pub xhms: String,
    #[serde(default)]
    pub last: String,
    #[serde(default)]
    pub open: String,
    #[serde(default)]
    pub high: String,
    #[serde(default)]
    pub low: String,
    #[serde(default)]
    pub evol: String,
    #[serde(default)]
    pub tvol: String,
}

pub async fn get_daily_chart<C>(
    client: &C,
    exchange: &str,
    symbol: &str,
    start_date: &str,
    end_date: &str,
    period: Option<&str>,
) -> Result<Vec<OverseasDailyChartItem>>
where
    C: ApiClient + Sync,
{
    let exchange = normalize_exchange(exchange)?;
    if start_date.is_empty() || end_date.is_empty() {
        bail!("--start and --end are required for overseas daily chart");
    }

    let params = HashMap::from([
        ("FID_COND_MRKT_DIV_CODE".to_string(), exchange),
        ("FID_INPUT_ISCD".to_string(), symbol.to_uppercase()),
        ("FID_INPUT_DATE_1".to_string(), start_date.to_string()),
        ("FID_INPUT_DATE_2".to_string(), end_date.to_string()),
        (
            "FID_PERIOD_DIV_CODE".to_string(),
            normalize_period(period)?.to_string(),
        ),
    ]);

    let mut items = Vec::new();
    let mut tr_cont = String::new();

    loop {
        let response = client
            .get_json_response_with_tr_cont(PATH_DAILY_CHART, TR_ID_DAILY_CHART, &tr_cont, &params)
            .await?;
        let next_tr_cont = response.tr_cont.clone();
        let (_meta, page): (Value, Vec<OverseasDailyChartItem>) =
            parse_outputs(response.body, "overseas daily chart")?;
        items.extend(page);

        match next_tr_cont.as_deref() {
            Some("M" | "F") => tr_cont = "N".to_string(),
            _ => return Ok(items),
        }
    }
}

pub async fn get_time_chart<C>(
    client: &C,
    exchange: &str,
    symbol: &str,
    unit: Option<&str>,
) -> Result<Vec<OverseasTimeChartItem>>
where
    C: ApiClient + Sync,
{
    let exchange = normalize_exchange(exchange)?;
    let params = HashMap::from([
        ("AUTH".to_string(), "".to_string()),
        ("EXCD".to_string(), exchange),
        ("SYMB".to_string(), symbol.to_uppercase()),
        ("NMIN".to_string(), unit.unwrap_or("1").to_string()),
        ("PINC".to_string(), "1".to_string()),
        ("NEXT".to_string(), "".to_string()),
        ("NREC".to_string(), "120".to_string()),
        ("FILL".to_string(), "".to_string()),
        ("KEYB".to_string(), "".to_string()),
    ]);
    let response = client
        .get_json(PATH_TIME_CHART, TR_ID_TIME_CHART, &params)
        .await?;
    let (_meta, items): (Value, Vec<OverseasTimeChartItem>) =
        parse_outputs(response, "overseas time chart")?;
    Ok(items)
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
        "D" => Ok("D"),
        "W" => Ok("W"),
        "M" => Ok("M"),
        other => bail!("invalid period {other:?}; valid periods: D, W, M"),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use serde_json::json;

    use crate::client::JsonResponse;

    use super::*;

    #[derive(Debug, Default, Clone)]
    struct GetCall {
        path: String,
        tr_id: String,
        params: HashMap<String, String>,
    }

    #[derive(Debug, Default, Clone)]
    struct ResponseCall {
        path: String,
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
            path: &str,
            tr_id: &str,
            params: &HashMap<String, String>,
        ) -> Result<serde_json::Value> {
            *self.call.lock().unwrap() = Some(GetCall {
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
    async fn gets_overseas_daily_chart_and_paginates() {
        let client = MockClient {
            response: json!({}),
            call: Arc::new(Mutex::new(None)),
            responses: Arc::new(Mutex::new(vec![
                JsonResponse {
                    body: json!({
                        "rt_cd": "0",
                        "msg_cd": "MCA00000",
                        "msg1": "정상처리",
                        "output1": { "rsym": "DNASAAPL" },
                        "output2": [{
                            "xymd": "20260306",
                            "open": "174.20",
                            "high": "176.10",
                            "low": "173.80",
                            "clos": "175.90",
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
                        "output1": { "rsym": "DNASAAPL" },
                        "output2": [{
                            "xymd": "20260305",
                            "open": "173.10",
                            "high": "175.30",
                            "low": "172.40",
                            "clos": "174.80",
                            "tvol": "49876543"
                        }]
                    }),
                    tr_cont: None,
                },
            ])),
            response_calls: Arc::new(Mutex::new(Vec::new())),
        };

        let result = get_daily_chart(&client, "nas", "aapl", "20260301", "20260306", Some("W"))
            .await
            .unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].xymd, "20260306");
        assert_eq!(result[1].xymd, "20260305");

        let calls = client.response_calls.lock().unwrap().clone();
        assert_eq!(calls[0].path, PATH_DAILY_CHART);
        assert_eq!(calls[0].tr_id, TR_ID_DAILY_CHART);
        assert_eq!(calls[0].tr_cont, "");
        assert_eq!(calls[0].params["FID_COND_MRKT_DIV_CODE"], "NAS");
        assert_eq!(calls[0].params["FID_INPUT_ISCD"], "AAPL");
        assert_eq!(calls[0].params["FID_INPUT_DATE_1"], "20260301");
        assert_eq!(calls[0].params["FID_INPUT_DATE_2"], "20260306");
        assert_eq!(calls[0].params["FID_PERIOD_DIV_CODE"], "W");
        assert_eq!(calls[1].tr_cont, "N");
    }

    #[tokio::test]
    async fn gets_overseas_time_chart_with_default_window() {
        let call = Arc::new(Mutex::new(None));
        let client = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "output1": { "rsym": "DNASAAPL" },
                "output2": [{
                    "xymd": "20260307",
                    "xhms": "093000",
                    "last": "175.40",
                    "open": "175.10",
                    "high": "175.60",
                    "low": "175.00",
                    "evol": "1200",
                    "tvol": "5400"
                }]
            }),
            call: call.clone(),
            responses: Arc::new(Mutex::new(Vec::new())),
            response_calls: Arc::new(Mutex::new(Vec::new())),
        };

        let result = get_time_chart(&client, "NAS", "AAPL", None).await.unwrap();
        assert_eq!(result[0].xhms, "093000");
        assert_eq!(result[0].last, "175.40");

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.path, PATH_TIME_CHART);
        assert_eq!(call.tr_id, TR_ID_TIME_CHART);
        assert_eq!(call.params["EXCD"], "NAS");
        assert_eq!(call.params["SYMB"], "AAPL");
        assert_eq!(call.params["NMIN"], "1");
        assert_eq!(call.params["PINC"], "1");
        assert_eq!(call.params["NEXT"], "");
        assert_eq!(call.params["NREC"], "120");
        assert_eq!(call.params["KEYB"], "");
    }

    #[tokio::test]
    async fn rejects_missing_daily_chart_dates() {
        let client = MockClient {
            response: json!({}),
            call: Arc::new(Mutex::new(None)),
            responses: Arc::new(Mutex::new(Vec::new())),
            response_calls: Arc::new(Mutex::new(Vec::new())),
        };

        let err = get_daily_chart(&client, "NAS", "AAPL", "", "20260306", Some("D"))
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            "--start and --end are required for overseas daily chart"
        );
    }
}
