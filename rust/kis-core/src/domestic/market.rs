use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api_client::{ApiClient, parse_output, parse_outputs};

const PATH_VOLUME_RANK: &str = "/uapi/domestic-stock/v1/quotations/volume-rank";
const TR_ID_VOLUME_RANK: &str = "FHPST01710000";
const PATH_HOLIDAY: &str = "/uapi/domestic-stock/v1/quotations/chk-holiday";
const TR_ID_HOLIDAY: &str = "CTCA0903R";
const PATH_OVERTIME_FLUCTUATION: &str = "/uapi/domestic-stock/v1/ranking/overtime-fluctuation";
const TR_ID_OVERTIME_FLUCTUATION: &str = "FHPST02340000";
const PATH_OVERTIME_VOLUME: &str = "/uapi/domestic-stock/v1/ranking/overtime-volume";
const TR_ID_OVERTIME_VOLUME: &str = "FHPST02350000";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct VolumeRankItem {
    pub hts_kor_isnm: String,
    pub mksc_shrn_iscd: String,
    pub data_rank: String,
    pub stck_prpr: String,
    pub prdy_vrss: String,
    pub prdy_vrss_sign: String,
    pub prdy_ctrt: String,
    pub acml_vol: String,
    pub acml_tr_pbmn: String,
    pub prdy_vol: String,
    pub avrg_vol: String,
    pub vol_inrt: String,
    pub vol_tnrt: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct HolidayInfo {
    pub bass_dt: String,
    pub wday_dvsn_cd: String,
    pub bzdy_yn: String,
    pub tr_day_yn: String,
    pub opnd_yn: String,
    pub sttl_day_yn: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct DomesticOvertimeRankingResult {
    pub summary: Vec<Value>,
    pub items: Vec<Value>,
}

pub async fn get_volume_rank<C>(client: &C) -> Result<Vec<VolumeRankItem>>
where
    C: ApiClient + Sync,
{
    let params = HashMap::from([
        ("FID_COND_MRKT_DIV_CODE".to_string(), "J".to_string()),
        ("FID_COND_SCR_DIV_CODE".to_string(), "20171".to_string()),
        ("FID_INPUT_ISCD".to_string(), "0000".to_string()),
        ("FID_DIV_CLS_CODE".to_string(), "0".to_string()),
        ("FID_BLNG_CLS_CODE".to_string(), "0".to_string()),
        ("FID_TRGT_CLS_CODE".to_string(), "111111111".to_string()),
        ("FID_TRGT_EXLS_CLS_CODE".to_string(), "000000".to_string()),
        ("FID_INPUT_PRICE_1".to_string(), "".to_string()),
        ("FID_INPUT_PRICE_2".to_string(), "".to_string()),
        ("FID_VOL_CNT".to_string(), "".to_string()),
        ("FID_INPUT_DATE_1".to_string(), "".to_string()),
    ]);
    let response = client
        .get_json(PATH_VOLUME_RANK, TR_ID_VOLUME_RANK, &params)
        .await?;
    parse_output(response, "volume rank")
}

pub async fn get_holidays<C>(client: &C, base_date: &str) -> Result<Vec<HolidayInfo>>
where
    C: ApiClient + Sync,
{
    let params = HashMap::from([
        ("BASS_DT".to_string(), base_date.to_string()),
        ("CTX_AREA_NK".to_string(), "".to_string()),
        ("CTX_AREA_FK".to_string(), "".to_string()),
    ]);
    let response = client
        .get_json(PATH_HOLIDAY, TR_ID_HOLIDAY, &params)
        .await?;
    parse_output(response, "holiday")
}

pub async fn get_overtime_fluctuation_rank<C>(client: &C) -> Result<DomesticOvertimeRankingResult>
where
    C: ApiClient + Sync,
{
    let params = HashMap::from([
        ("FID_COND_MRKT_DIV_CODE".to_string(), "J".to_string()),
        ("FID_MRKT_CLS_CODE".to_string(), "".to_string()),
        ("FID_COND_SCR_DIV_CODE".to_string(), "20234".to_string()),
        ("FID_INPUT_ISCD".to_string(), "0000".to_string()),
        ("FID_DIV_CLS_CODE".to_string(), "2".to_string()),
        ("FID_INPUT_PRICE_1".to_string(), "".to_string()),
        ("FID_INPUT_PRICE_2".to_string(), "".to_string()),
        ("FID_VOL_CNT".to_string(), "".to_string()),
        ("FID_TRGT_CLS_CODE".to_string(), "".to_string()),
        ("FID_TRGT_EXLS_CLS_CODE".to_string(), "".to_string()),
    ]);
    fetch_overtime_ranking(
        client,
        PATH_OVERTIME_FLUCTUATION,
        TR_ID_OVERTIME_FLUCTUATION,
        &params,
        "overtime fluctuation rank",
    )
    .await
}

pub async fn get_overtime_volume_rank<C>(client: &C) -> Result<DomesticOvertimeRankingResult>
where
    C: ApiClient + Sync,
{
    let params = HashMap::from([
        ("FID_COND_MRKT_DIV_CODE".to_string(), "J".to_string()),
        ("FID_COND_SCR_DIV_CODE".to_string(), "20235".to_string()),
        ("FID_INPUT_ISCD".to_string(), "0000".to_string()),
        ("FID_RANK_SORT_CLS_CODE".to_string(), "2".to_string()),
        ("FID_INPUT_PRICE_1".to_string(), "".to_string()),
        ("FID_INPUT_PRICE_2".to_string(), "".to_string()),
        ("FID_VOL_CNT".to_string(), "".to_string()),
        ("FID_TRGT_CLS_CODE".to_string(), "".to_string()),
        ("FID_TRGT_EXLS_CLS_CODE".to_string(), "".to_string()),
    ]);
    fetch_overtime_ranking(
        client,
        PATH_OVERTIME_VOLUME,
        TR_ID_OVERTIME_VOLUME,
        &params,
        "overtime volume rank",
    )
    .await
}

async fn fetch_overtime_ranking<C>(
    client: &C,
    path: &str,
    tr_id: &str,
    params: &HashMap<String, String>,
    label: &str,
) -> Result<DomesticOvertimeRankingResult>
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
            _ => return Ok(DomesticOvertimeRankingResult { summary, items }),
        }
    }
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
    async fn gets_volume_rank() {
        let call = Arc::new(Mutex::new(None));
        let client = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "output": [{
                    "hts_kor_isnm": "삼성전자",
                    "mksc_shrn_iscd": "005930",
                    "data_rank": "1",
                    "stck_prpr": "70000",
                    "prdy_vrss": "1000",
                    "prdy_vrss_sign": "2",
                    "prdy_ctrt": "1.45",
                    "acml_vol": "12345678",
                    "acml_tr_pbmn": "1000000000",
                    "prdy_vol": "10000000",
                    "avrg_vol": "9000000",
                    "vol_inrt": "137.1",
                    "vol_tnrt": "0.21"
                }]
            }),
            call: call.clone(),
        };

        let result = get_volume_rank(&client).await.unwrap();
        assert_eq!(result[0].data_rank, "1");

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.path, PATH_VOLUME_RANK);
        assert_eq!(call.tr_id, TR_ID_VOLUME_RANK);
        assert_eq!(call.params["FID_COND_SCR_DIV_CODE"], "20171");
    }

    #[tokio::test]
    async fn gets_holidays() {
        let call = Arc::new(Mutex::new(None));
        let client = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "output": [{
                    "bass_dt": "20260306",
                    "wday_dvsn_cd": "5",
                    "bzdy_yn": "Y",
                    "tr_day_yn": "Y",
                    "opnd_yn": "Y",
                    "sttl_day_yn": "Y"
                }]
            }),
            call: call.clone(),
        };

        let result = get_holidays(&client, "20260306").await.unwrap();
        assert_eq!(result[0].bass_dt, "20260306");

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.path, PATH_HOLIDAY);
        assert_eq!(call.tr_id, TR_ID_HOLIDAY);
        assert_eq!(call.params["BASS_DT"], "20260306");
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

        let err = get_holidays(&client, "20260306").await.unwrap_err();
        assert_eq!(err.to_string(), "holiday API error: [EGW00001] 잘못된 요청");
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
    async fn gets_overtime_fluctuation_rank_and_paginates() {
        let client = ResponseClient {
            responses: Arc::new(Mutex::new(vec![
                crate::client::JsonResponse {
                    body: json!({
                        "rt_cd": "0",
                        "msg_cd": "MCA00000",
                        "msg1": "정상처리",
                        "output1": {
                            "ovtm_untp_ascn_issu_cnt": "120",
                            "ovtm_untp_acml_vol": "1234567"
                        },
                        "output2": [{
                            "mksc_shrn_iscd": "005930",
                            "hts_kor_isnm": "삼성전자",
                            "ovtm_untp_prpr": "70500",
                            "ovtm_untp_prdy_vrss": "500",
                            "ovtm_untp_prdy_ctrt": "0.71",
                            "ovtm_untp_vol": "123456",
                            "ovtm_vrss_acml_vol_rlim": "12.34"
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
                            "ovtm_untp_ascn_issu_cnt": "121",
                            "ovtm_untp_acml_vol": "2234567"
                        },
                        "output2": [{
                            "mksc_shrn_iscd": "000660",
                            "hts_kor_isnm": "SK하이닉스",
                            "ovtm_untp_prpr": "182000",
                            "ovtm_untp_prdy_vrss": "1000",
                            "ovtm_untp_prdy_ctrt": "0.55",
                            "ovtm_untp_vol": "223456",
                            "ovtm_vrss_acml_vol_rlim": "22.34"
                        }]
                    }),
                    tr_cont: None,
                },
            ])),
            response_calls: Arc::new(Mutex::new(Vec::new())),
        };

        let result = get_overtime_fluctuation_rank(&client).await.unwrap();

        assert_eq!(result.summary.len(), 2);
        assert_eq!(result.items.len(), 2);
        assert_eq!(result.items[0]["mksc_shrn_iscd"], "005930");
        assert_eq!(result.items[1]["mksc_shrn_iscd"], "000660");

        let calls = client.response_calls.lock().unwrap().clone();
        assert_eq!(calls[0].path, PATH_OVERTIME_FLUCTUATION);
        assert_eq!(calls[0].tr_id, TR_ID_OVERTIME_FLUCTUATION);
        assert_eq!(calls[0].tr_cont, "");
        assert_eq!(calls[0].params["FID_COND_MRKT_DIV_CODE"], "J");
        assert_eq!(calls[0].params["FID_COND_SCR_DIV_CODE"], "20234");
        assert_eq!(calls[0].params["FID_INPUT_ISCD"], "0000");
        assert_eq!(calls[0].params["FID_DIV_CLS_CODE"], "2");
        assert_eq!(calls[0].params["FID_INPUT_PRICE_1"], "");
        assert_eq!(calls[0].params["FID_INPUT_PRICE_2"], "");
        assert_eq!(calls[0].params["FID_VOL_CNT"], "");
        assert_eq!(calls[1].tr_cont, "N");
    }

    #[tokio::test]
    async fn gets_overtime_volume_rank() {
        let client = ResponseClient {
            responses: Arc::new(Mutex::new(vec![crate::client::JsonResponse {
                body: json!({
                    "rt_cd": "0",
                    "msg_cd": "MCA00000",
                    "msg1": "정상처리",
                    "output1": {
                        "ovtm_untp_exch_vol": "345678",
                        "ovtm_untp_exch_tr_pbmn": "987654321"
                    },
                    "output2": [{
                        "stck_shrn_iscd": "005930",
                        "hts_kor_isnm": "삼성전자",
                        "ovtm_untp_prpr": "70500",
                        "ovtm_untp_prdy_vrss": "500",
                        "ovtm_untp_prdy_ctrt": "0.71",
                        "ovtm_untp_vol": "123456"
                    }]
                }),
                tr_cont: None,
            }])),
            response_calls: Arc::new(Mutex::new(Vec::new())),
        };

        let result = get_overtime_volume_rank(&client).await.unwrap();

        assert_eq!(result.summary.len(), 1);
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0]["stck_shrn_iscd"], "005930");

        let calls = client.response_calls.lock().unwrap().clone();
        assert_eq!(calls[0].path, PATH_OVERTIME_VOLUME);
        assert_eq!(calls[0].tr_id, TR_ID_OVERTIME_VOLUME);
        assert_eq!(calls[0].params["FID_COND_SCR_DIV_CODE"], "20235");
        assert_eq!(calls[0].params["FID_INPUT_ISCD"], "0000");
        assert_eq!(calls[0].params["FID_RANK_SORT_CLS_CODE"], "2");
        assert_eq!(calls[0].params["FID_INPUT_PRICE_1"], "");
        assert_eq!(calls[0].params["FID_INPUT_PRICE_2"], "");
        assert_eq!(calls[0].params["FID_VOL_CNT"], "");
    }
}
