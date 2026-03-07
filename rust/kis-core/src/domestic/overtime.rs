use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::api_client::{ApiClient, parse_output};

const PATH_INQUIRE_OVERTIME_PRICE: &str =
    "/uapi/domestic-stock/v1/quotations/inquire-overtime-price";
const TR_ID_INQUIRE_OVERTIME_PRICE: &str = "FHPST02300000";
const PATH_INQUIRE_OVERTIME_ASKING_PRICE: &str =
    "/uapi/domestic-stock/v1/quotations/inquire-overtime-asking-price";
const TR_ID_INQUIRE_OVERTIME_ASKING_PRICE: &str = "FHPST02300400";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct OvertimePrice {
    #[serde(default)]
    pub bstp_kor_isnm: String,
    #[serde(default)]
    pub mang_issu_cls_name: String,
    #[serde(default)]
    pub ovtm_untp_prpr: String,
    #[serde(default)]
    pub ovtm_untp_prdy_vrss: String,
    #[serde(default)]
    pub ovtm_untp_prdy_vrss_sign: String,
    #[serde(default)]
    pub ovtm_untp_prdy_ctrt: String,
    #[serde(default)]
    pub ovtm_untp_vol: String,
    #[serde(default)]
    pub ovtm_untp_tr_pbmn: String,
    #[serde(default)]
    pub ovtm_untp_oprc: String,
    #[serde(default)]
    pub ovtm_untp_hgpr: String,
    #[serde(default)]
    pub ovtm_untp_lwpr: String,
    #[serde(default)]
    pub ovtm_untp_antc_cnpr: String,
    #[serde(default)]
    pub ovtm_untp_antc_cnqn: String,
    #[serde(default)]
    pub ovtm_untp_sdpr: String,
    #[serde(default)]
    pub bidp: String,
    #[serde(default)]
    pub askp: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct OvertimeAskingPrice {
    #[serde(default)]
    pub ovtm_untp_last_hour: String,
    #[serde(default)]
    pub ovtm_untp_askp1: String,
    #[serde(default)]
    pub ovtm_untp_askp2: String,
    #[serde(default)]
    pub ovtm_untp_askp3: String,
    #[serde(default)]
    pub ovtm_untp_askp4: String,
    #[serde(default)]
    pub ovtm_untp_askp5: String,
    #[serde(default)]
    pub ovtm_untp_askp6: String,
    #[serde(default)]
    pub ovtm_untp_askp7: String,
    #[serde(default)]
    pub ovtm_untp_askp8: String,
    #[serde(default)]
    pub ovtm_untp_askp9: String,
    #[serde(default)]
    pub ovtm_untp_askp10: String,
    #[serde(default)]
    pub ovtm_untp_bidp1: String,
    #[serde(default)]
    pub ovtm_untp_bidp2: String,
    #[serde(default)]
    pub ovtm_untp_bidp3: String,
    #[serde(default)]
    pub ovtm_untp_bidp4: String,
    #[serde(default)]
    pub ovtm_untp_bidp5: String,
    #[serde(default)]
    pub ovtm_untp_bidp6: String,
    #[serde(default)]
    pub ovtm_untp_bidp7: String,
    #[serde(default)]
    pub ovtm_untp_bidp8: String,
    #[serde(default)]
    pub ovtm_untp_bidp9: String,
    #[serde(default)]
    pub ovtm_untp_bidp10: String,
    #[serde(default)]
    pub ovtm_untp_askp_rsqn1: String,
    #[serde(default)]
    pub ovtm_untp_askp_rsqn2: String,
    #[serde(default)]
    pub ovtm_untp_askp_rsqn3: String,
    #[serde(default)]
    pub ovtm_untp_askp_rsqn4: String,
    #[serde(default)]
    pub ovtm_untp_askp_rsqn5: String,
    #[serde(default)]
    pub ovtm_untp_askp_rsqn6: String,
    #[serde(default)]
    pub ovtm_untp_askp_rsqn7: String,
    #[serde(default)]
    pub ovtm_untp_askp_rsqn8: String,
    #[serde(default)]
    pub ovtm_untp_askp_rsqn9: String,
    #[serde(default)]
    pub ovtm_untp_askp_rsqn10: String,
    #[serde(default)]
    pub ovtm_untp_bidp_rsqn1: String,
    #[serde(default)]
    pub ovtm_untp_bidp_rsqn2: String,
    #[serde(default)]
    pub ovtm_untp_bidp_rsqn3: String,
    #[serde(default)]
    pub ovtm_untp_bidp_rsqn4: String,
    #[serde(default)]
    pub ovtm_untp_bidp_rsqn5: String,
    #[serde(default)]
    pub ovtm_untp_bidp_rsqn6: String,
    #[serde(default)]
    pub ovtm_untp_bidp_rsqn7: String,
    #[serde(default)]
    pub ovtm_untp_bidp_rsqn8: String,
    #[serde(default)]
    pub ovtm_untp_bidp_rsqn9: String,
    #[serde(default)]
    pub ovtm_untp_bidp_rsqn10: String,
    #[serde(default)]
    pub ovtm_untp_total_askp_rsqn: String,
    #[serde(default)]
    pub ovtm_untp_total_bidp_rsqn: String,
    #[serde(default)]
    pub total_askp_rsqn: String,
    #[serde(default)]
    pub total_bidp_rsqn: String,
    #[serde(default)]
    pub ovtm_total_askp_rsqn: String,
    #[serde(default)]
    pub ovtm_total_bidp_rsqn: String,
}

pub async fn inquire_overtime_price<C>(
    client: &C,
    market_div_code: &str,
    stock_code: &str,
) -> Result<OvertimePrice>
where
    C: ApiClient + Sync,
{
    get_overtime_price(client, market_div_code, stock_code).await
}

pub async fn get_overtime_price<C>(
    client: &C,
    market_div_code: &str,
    stock_code: &str,
) -> Result<OvertimePrice>
where
    C: ApiClient + Sync,
{
    let response = client
        .get_json(
            PATH_INQUIRE_OVERTIME_PRICE,
            TR_ID_INQUIRE_OVERTIME_PRICE,
            &stock_params(market_div_code, stock_code),
        )
        .await?;
    parse_output(response, "overtime price")
}

pub async fn inquire_overtime_asking_price<C>(
    client: &C,
    market_div_code: &str,
    stock_code: &str,
) -> Result<OvertimeAskingPrice>
where
    C: ApiClient + Sync,
{
    get_overtime_asking_price(client, market_div_code, stock_code).await
}

pub async fn get_overtime_asking_price<C>(
    client: &C,
    market_div_code: &str,
    stock_code: &str,
) -> Result<OvertimeAskingPrice>
where
    C: ApiClient + Sync,
{
    let response = client
        .get_json(
            PATH_INQUIRE_OVERTIME_ASKING_PRICE,
            TR_ID_INQUIRE_OVERTIME_ASKING_PRICE,
            &stock_params(market_div_code, stock_code),
        )
        .await?;
    parse_output(response, "overtime asking price")
}

fn stock_params(market_div_code: &str, stock_code: &str) -> HashMap<String, String> {
    HashMap::from([
        (
            "FID_COND_MRKT_DIV_CODE".to_string(),
            market_div_code.to_string(),
        ),
        ("FID_INPUT_ISCD".to_string(), stock_code.to_string()),
    ])
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
    async fn gets_overtime_price() {
        let call = Arc::new(Mutex::new(None));
        let client = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "output": {
                    "bstp_kor_isnm": "삼성전자",
                    "ovtm_untp_prpr": "70100",
                    "ovtm_untp_prdy_vrss": "100",
                    "ovtm_untp_prdy_vrss_sign": "2",
                    "ovtm_untp_prdy_ctrt": "0.14",
                    "ovtm_untp_vol": "12345",
                    "ovtm_untp_tr_pbmn": "864150000",
                    "ovtm_untp_oprc": "70000",
                    "ovtm_untp_hgpr": "70200",
                    "ovtm_untp_lwpr": "69900",
                    "ovtm_untp_antc_cnpr": "70100",
                    "ovtm_untp_antc_cnqn": "1000",
                    "ovtm_untp_sdpr": "70000",
                    "bidp": "70000",
                    "askp": "70100"
                }
            }),
            call: call.clone(),
        };

        let result = get_overtime_price(&client, "J", "005930").await.unwrap();
        assert_eq!(result.bstp_kor_isnm, "삼성전자");
        assert_eq!(result.ovtm_untp_prpr, "70100");
        assert_eq!(result.ovtm_untp_antc_cnqn, "1000");

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.path, PATH_INQUIRE_OVERTIME_PRICE);
        assert_eq!(call.tr_id, TR_ID_INQUIRE_OVERTIME_PRICE);
        assert_eq!(call.params["FID_COND_MRKT_DIV_CODE"], "J");
        assert_eq!(call.params["FID_INPUT_ISCD"], "005930");
    }

    #[tokio::test]
    async fn gets_overtime_asking_price() {
        let call = Arc::new(Mutex::new(None));
        let client = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "output": {
                    "ovtm_untp_last_hour": "170001",
                    "ovtm_untp_askp1": "70100",
                    "ovtm_untp_askp2": "70200",
                    "ovtm_untp_askp3": "70300",
                    "ovtm_untp_askp4": "70400",
                    "ovtm_untp_askp5": "70500",
                    "ovtm_untp_bidp1": "70000",
                    "ovtm_untp_bidp2": "69900",
                    "ovtm_untp_bidp3": "69800",
                    "ovtm_untp_bidp4": "69700",
                    "ovtm_untp_bidp5": "69600",
                    "ovtm_untp_askp_rsqn1": "10",
                    "ovtm_untp_askp_rsqn2": "20",
                    "ovtm_untp_askp_rsqn3": "30",
                    "ovtm_untp_askp_rsqn4": "40",
                    "ovtm_untp_askp_rsqn5": "50",
                    "ovtm_untp_bidp_rsqn1": "11",
                    "ovtm_untp_bidp_rsqn2": "21",
                    "ovtm_untp_bidp_rsqn3": "31",
                    "ovtm_untp_bidp_rsqn4": "41",
                    "ovtm_untp_bidp_rsqn5": "51",
                    "ovtm_untp_total_askp_rsqn": "150",
                    "ovtm_untp_total_bidp_rsqn": "155",
                    "total_askp_rsqn": "200",
                    "total_bidp_rsqn": "210",
                    "ovtm_total_askp_rsqn": "300",
                    "ovtm_total_bidp_rsqn": "310"
                }
            }),
            call: call.clone(),
        };

        let result = get_overtime_asking_price(&client, "J", "005930")
            .await
            .unwrap();
        assert_eq!(result.ovtm_untp_last_hour, "170001");
        assert_eq!(result.ovtm_untp_askp1, "70100");
        assert_eq!(result.ovtm_total_bidp_rsqn, "310");

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.path, PATH_INQUIRE_OVERTIME_ASKING_PRICE);
        assert_eq!(call.tr_id, TR_ID_INQUIRE_OVERTIME_ASKING_PRICE);
        assert_eq!(call.params["FID_COND_MRKT_DIV_CODE"], "J");
        assert_eq!(call.params["FID_INPUT_ISCD"], "005930");
    }

    #[tokio::test]
    async fn reports_overtime_price_api_errors() {
        let client = MockClient {
            response: json!({
                "rt_cd": "1",
                "msg_cd": "EGW00001",
                "msg1": "잘못된 요청"
            }),
            call: Arc::new(Mutex::new(None)),
        };

        let err = get_overtime_price(&client, "J", "005930").await.unwrap_err();
        assert_eq!(err.to_string(), "overtime price API error: [EGW00001] 잘못된 요청");
    }
}
