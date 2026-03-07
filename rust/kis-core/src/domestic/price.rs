use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::api_client::{ApiClient, parse_output};

const PATH_INQUIRE_PRICE: &str = "/uapi/domestic-stock/v1/quotations/inquire-price";
const TR_ID_INQUIRE_PRICE: &str = "FHKST01010100";
const PATH_INQUIRE_DAILY_PRICE: &str = "/uapi/domestic-stock/v1/quotations/inquire-daily-price";
const TR_ID_INQUIRE_DAILY_PRICE: &str = "FHKST01010400";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct StockPrice {
    pub stck_prpr: String,
    pub prdy_vrss: String,
    pub prdy_vrss_sign: String,
    pub prdy_ctrt: String,
    pub acml_vol: String,
    pub acml_tr_pbmn: String,
    pub stck_oprc: String,
    pub stck_hgpr: String,
    pub stck_lwpr: String,
    #[serde(default)]
    pub hts_kor_isnm: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct DailyPrice {
    pub stck_bsop_date: String,
    pub stck_oprc: String,
    pub stck_hgpr: String,
    pub stck_lwpr: String,
    pub stck_clpr: String,
    pub acml_vol: String,
    pub prdy_vrss: String,
    pub prdy_vrss_sign: String,
    pub prdy_ctrt: String,
}

pub async fn get_price<C>(client: &C, stock_code: &str) -> Result<StockPrice>
where
    C: ApiClient + Sync,
{
    let params = HashMap::from([
        ("FID_COND_MRKT_DIV_CODE".to_string(), "J".to_string()),
        ("FID_INPUT_ISCD".to_string(), stock_code.to_string()),
    ]);
    let response = client
        .get_json(PATH_INQUIRE_PRICE, TR_ID_INQUIRE_PRICE, &params)
        .await?;
    let mut price: StockPrice = parse_output(response, "domestic price")?;
    if price.hts_kor_isnm.is_empty() {
        price.hts_kor_isnm = stock_code.to_string();
    }
    Ok(price)
}

pub async fn get_daily_price<C>(
    client: &C,
    stock_code: &str,
    period: Option<&str>,
) -> Result<Vec<DailyPrice>>
where
    C: ApiClient + Sync,
{
    let params = HashMap::from([
        ("FID_COND_MRKT_DIV_CODE".to_string(), "J".to_string()),
        ("FID_INPUT_ISCD".to_string(), stock_code.to_string()),
        (
            "FID_PERIOD_DIV_CODE".to_string(),
            period.unwrap_or("D").to_string(),
        ),
        ("FID_ORG_ADJ_PRC".to_string(), "0".to_string()),
    ]);
    let response = client
        .get_json(PATH_INQUIRE_DAILY_PRICE, TR_ID_INQUIRE_DAILY_PRICE, &params)
        .await?;
    parse_output(response, "domestic daily price")
}

#[cfg(test)]
mod tests {
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
    async fn gets_domestic_price() {
        let call = Arc::new(Mutex::new(None));
        let client = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "output": {
                    "stck_prpr": "70000",
                    "prdy_vrss": "1000",
                    "prdy_vrss_sign": "2",
                    "prdy_ctrt": "1.45",
                    "acml_vol": "12345678",
                    "acml_tr_pbmn": "1000000000",
                    "stck_oprc": "69000",
                    "stck_hgpr": "70500",
                    "stck_lwpr": "68000",
                    "hts_kor_isnm": "삼성전자"
                }
            }),
            call: call.clone(),
        };

        let price = get_price(&client, "005930").await.unwrap();
        assert_eq!(price.stck_prpr, "70000");
        assert_eq!(price.hts_kor_isnm, "삼성전자");

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.path, PATH_INQUIRE_PRICE);
        assert_eq!(call.tr_id, TR_ID_INQUIRE_PRICE);
        assert_eq!(call.params["FID_INPUT_ISCD"], "005930");
    }

    #[tokio::test]
    async fn rejects_api_error() {
        let client = MockClient {
            response: json!({
                "rt_cd": "1",
                "msg_cd": "EGW00001",
                "msg1": "잘못된 요청",
                "output": {}
            }),
            call: Arc::new(Mutex::new(None)),
        };

        assert!(get_price(&client, "999999").await.is_err());
    }

    #[tokio::test]
    async fn falls_back_to_stock_code_when_name_is_missing() {
        let client = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리 되었습니다.",
                "output": {
                    "stck_prpr": "188200",
                    "prdy_vrss": "-3400",
                    "prdy_vrss_sign": "5",
                    "prdy_ctrt": "-1.77",
                    "acml_vol": "29434152",
                    "acml_tr_pbmn": "5479631549950",
                    "stck_oprc": "186100",
                    "stck_hgpr": "189700",
                    "stck_lwpr": "181000"
                }
            }),
            call: Arc::new(Mutex::new(None)),
        };

        let price = get_price(&client, "005930").await.unwrap();
        assert_eq!(price.stck_prpr, "188200");
        assert_eq!(price.hts_kor_isnm, "005930");
    }

    #[tokio::test]
    async fn defaults_daily_period_to_d() {
        let call = Arc::new(Mutex::new(None));
        let client = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "output": []
            }),
            call: call.clone(),
        };

        let prices = get_daily_price(&client, "005930", None).await.unwrap();
        assert!(prices.is_empty());

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.params["FID_PERIOD_DIV_CODE"], "D");
    }
}
