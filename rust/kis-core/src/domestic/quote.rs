use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::api_client::{ApiClient, parse_output};

const PATH_INQUIRE_ASKING_PRICE: &str = "/uapi/domestic-stock/v1/quotations/inquire-asking-price";
const TR_ID_INQUIRE_ASKING_PRICE: &str = "FHKST01010200";
const PATH_INQUIRE_CCNL: &str = "/uapi/domestic-stock/v1/quotations/inquire-ccnl";
const TR_ID_INQUIRE_CCNL: &str = "FHKST01010300";
const PATH_INQUIRE_INVESTOR: &str = "/uapi/domestic-stock/v1/quotations/inquire-investor";
const TR_ID_INQUIRE_INVESTOR: &str = "FHKST01010600";
const PATH_INQUIRE_MEMBER: &str = "/uapi/domestic-stock/v1/quotations/inquire-member";
const TR_ID_INQUIRE_MEMBER: &str = "FHKST01010700";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct AskingPrice {
    pub askp1: String,
    pub askp2: String,
    pub askp3: String,
    pub askp4: String,
    pub askp5: String,
    pub bidp1: String,
    pub bidp2: String,
    pub bidp3: String,
    pub bidp4: String,
    pub bidp5: String,
    pub askp_rsqn1: String,
    pub askp_rsqn2: String,
    pub askp_rsqn3: String,
    pub askp_rsqn4: String,
    pub askp_rsqn5: String,
    pub bidp_rsqn1: String,
    pub bidp_rsqn2: String,
    pub bidp_rsqn3: String,
    pub bidp_rsqn4: String,
    pub bidp_rsqn5: String,
    pub total_askp_rsqn: String,
    pub total_bidp_rsqn: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Conclusion {
    pub stck_cntg_hour: String,
    pub stck_prpr: String,
    pub prdy_vrss: String,
    pub prdy_vrss_sign: String,
    pub cntg_vol: String,
    pub tday_rltv: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct InvestorData {
    pub invr_nm: String,
    pub seln_vol: String,
    pub shnu_vol: String,
    pub ntby_qty: String,
    pub seln_tr_pbmn: String,
    pub shnu_tr_pbmn: String,
    pub ntby_tr_pbmn: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct MemberData {
    pub memb_nm: String,
    pub seln_vol: String,
    pub shnu_vol: String,
    pub ntby_qty: String,
}

pub async fn get_asking_price<C>(client: &C, stock_code: &str) -> Result<AskingPrice>
where
    C: ApiClient + Sync,
{
    let params = stock_params(stock_code);
    let response = client
        .get_json(
            PATH_INQUIRE_ASKING_PRICE,
            TR_ID_INQUIRE_ASKING_PRICE,
            &params,
        )
        .await?;
    parse_output(response, "asking price")
}

pub async fn get_conclusions<C>(client: &C, stock_code: &str) -> Result<Vec<Conclusion>>
where
    C: ApiClient + Sync,
{
    let params = stock_params(stock_code);
    let response = client
        .get_json(PATH_INQUIRE_CCNL, TR_ID_INQUIRE_CCNL, &params)
        .await?;
    parse_output(response, "conclusion")
}

pub async fn get_investors<C>(client: &C, stock_code: &str) -> Result<Vec<InvestorData>>
where
    C: ApiClient + Sync,
{
    let params = stock_params(stock_code);
    let response = client
        .get_json(PATH_INQUIRE_INVESTOR, TR_ID_INQUIRE_INVESTOR, &params)
        .await?;
    parse_output(response, "investor")
}

pub async fn get_members<C>(client: &C, stock_code: &str) -> Result<Vec<MemberData>>
where
    C: ApiClient + Sync,
{
    let params = stock_params(stock_code);
    let response = client
        .get_json(PATH_INQUIRE_MEMBER, TR_ID_INQUIRE_MEMBER, &params)
        .await?;
    parse_output(response, "member")
}

fn stock_params(stock_code: &str) -> HashMap<String, String> {
    HashMap::from([
        ("FID_COND_MRKT_DIV_CODE".to_string(), "J".to_string()),
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
    async fn gets_asking_price() {
        let call = Arc::new(Mutex::new(None));
        let client = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "output": {
                    "askp1": "70100",
                    "askp2": "70200",
                    "askp3": "70300",
                    "askp4": "70400",
                    "askp5": "70500",
                    "bidp1": "70000",
                    "bidp2": "69900",
                    "bidp3": "69800",
                    "bidp4": "69700",
                    "bidp5": "69600",
                    "askp_rsqn1": "10",
                    "askp_rsqn2": "20",
                    "askp_rsqn3": "30",
                    "askp_rsqn4": "40",
                    "askp_rsqn5": "50",
                    "bidp_rsqn1": "11",
                    "bidp_rsqn2": "21",
                    "bidp_rsqn3": "31",
                    "bidp_rsqn4": "41",
                    "bidp_rsqn5": "51",
                    "total_askp_rsqn": "150",
                    "total_bidp_rsqn": "155"
                }
            }),
            call: call.clone(),
        };

        let result = get_asking_price(&client, "005930").await.unwrap();
        assert_eq!(result.askp1, "70100");
        assert_eq!(result.total_bidp_rsqn, "155");

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.path, PATH_INQUIRE_ASKING_PRICE);
        assert_eq!(call.tr_id, TR_ID_INQUIRE_ASKING_PRICE);
        assert_eq!(call.params["FID_INPUT_ISCD"], "005930");
    }

    #[tokio::test]
    async fn gets_conclusions() {
        let client = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "output": [{
                    "stck_cntg_hour": "093000",
                    "stck_prpr": "70000",
                    "prdy_vrss": "1000",
                    "prdy_vrss_sign": "2",
                    "cntg_vol": "12345",
                    "tday_rltv": "110.5"
                }]
            }),
            call: Arc::new(Mutex::new(None)),
        };

        let result = get_conclusions(&client, "005930").await.unwrap();
        assert_eq!(result[0].stck_cntg_hour, "093000");
    }

    #[tokio::test]
    async fn gets_investors_and_members() {
        let call = Arc::new(Mutex::new(None));
        let investors = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "output": [{
                    "invr_nm": "외국인",
                    "seln_vol": "100",
                    "shnu_vol": "120",
                    "ntby_qty": "20",
                    "seln_tr_pbmn": "7000000",
                    "shnu_tr_pbmn": "8400000",
                    "ntby_tr_pbmn": "1400000"
                }]
            }),
            call: call.clone(),
        };
        let members = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "output": [{
                    "memb_nm": "한국투자",
                    "seln_vol": "50",
                    "shnu_vol": "55",
                    "ntby_qty": "5"
                }]
            }),
            call: Arc::new(Mutex::new(None)),
        };

        let investors = get_investors(&investors, "005930").await.unwrap();
        assert_eq!(investors[0].invr_nm, "외국인");

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.path, PATH_INQUIRE_INVESTOR);
        assert_eq!(call.tr_id, TR_ID_INQUIRE_INVESTOR);

        let members = get_members(&members, "005930").await.unwrap();
        assert_eq!(members[0].memb_nm, "한국투자");
    }

    #[tokio::test]
    async fn rejects_api_error() {
        let client = MockClient {
            response: json!({
                "rt_cd": "1",
                "msg_cd": "EGW00001",
                "msg1": "잘못된 요청"
            }),
            call: Arc::new(Mutex::new(None)),
        };

        let err = get_members(&client, "005930").await.unwrap_err();
        assert_eq!(err.to_string(), "member API error: [EGW00001] 잘못된 요청");
    }
}
