use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::client::{ApiClient, parse_output};

const PATH_DIVIDEND: &str = "/uapi/domestic-stock/v1/ksdinfo/dividend";
const TR_ID_DIVIDEND: &str = "HHKDB669107C0";
const PATH_NEWS: &str = "/uapi/domestic-stock/v1/quotations/news-title";
const TR_ID_NEWS: &str = "FHKST01010800";
const PATH_INVEST_OPINION: &str = "/uapi/domestic-stock/v1/quotations/invest-opinion";
const TR_ID_INVEST_OPINION: &str = "FHKST01010500";
const PATH_SEARCH_STOCK: &str = "/uapi/domestic-stock/v1/quotations/search-stock-info";
const TR_ID_SEARCH_STOCK: &str = "CTPF1002R";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct DividendInfo {
    pub std_dt: String,
    pub per_sto_divi_amt: String,
    pub divi_rate: String,
    pub divi_kind: String,
    pub stck_bsop_date: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct NewsItem {
    pub data_dt: String,
    pub data_tm: String,
    pub cntt_ttle: String,
    pub data_srce: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct InvestOpinionItem {
    pub stck_bsop_date: String,
    pub invt_opnn: String,
    pub invt_opnn_cls: String,
    pub rgbf_nm: String,
    pub mbcr_name: String,
    pub stck_prpr: String,
    pub cnss_prpr: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SearchResult {
    pub pdno: String,
    pub prdt_name: String,
    pub prdt_eng_name: String,
    pub mrkt_cls_code: String,
    pub scty_grp_id: String,
    pub lstg_stqt: String,
}

pub async fn get_dividends<C>(client: &C, stock_code: &str) -> Result<Vec<DividendInfo>>
where
    C: ApiClient + Sync,
{
    let params = HashMap::from([
        ("SHR_ISCD".to_string(), stock_code.to_string()),
        ("SHR_YY".to_string(), "".to_string()),
        ("CTX_AREA_FK".to_string(), "".to_string()),
        ("CTX_AREA_NK".to_string(), "".to_string()),
    ]);
    let response = client
        .get_json(PATH_DIVIDEND, TR_ID_DIVIDEND, &params)
        .await?;
    parse_output(response, "dividend")
}

pub async fn get_news<C>(client: &C, stock_code: &str) -> Result<Vec<NewsItem>>
where
    C: ApiClient + Sync,
{
    let params = HashMap::from([
        ("FID_COND_MRKT_DIV_CODE".to_string(), "J".to_string()),
        ("FID_INPUT_ISCD".to_string(), stock_code.to_string()),
        ("FID_INPUT_DATE_1".to_string(), "".to_string()),
        ("FID_INPUT_DATE_2".to_string(), "".to_string()),
        ("FID_NEWS_CTGR_CLS_CODE".to_string(), "".to_string()),
    ]);
    let response = client.get_json(PATH_NEWS, TR_ID_NEWS, &params).await?;
    parse_output(response, "news")
}

pub async fn get_invest_opinions<C>(client: &C, stock_code: &str) -> Result<Vec<InvestOpinionItem>>
where
    C: ApiClient + Sync,
{
    let params = HashMap::from([
        ("FID_COND_MRKT_DIV_CODE".to_string(), "J".to_string()),
        ("FID_INPUT_ISCD".to_string(), stock_code.to_string()),
    ]);
    let response = client
        .get_json(PATH_INVEST_OPINION, TR_ID_INVEST_OPINION, &params)
        .await?;
    parse_output(response, "invest opinion")
}

pub async fn search_stocks<C>(client: &C, keyword: &str) -> Result<Vec<SearchResult>>
where
    C: ApiClient + Sync,
{
    let params = HashMap::from([
        ("PRDT_TYPE_CD".to_string(), "300".to_string()),
        ("PDNO".to_string(), keyword.to_string()),
    ]);
    let response = client
        .get_json(PATH_SEARCH_STOCK, TR_ID_SEARCH_STOCK, &params)
        .await?;
    parse_output(response, "search stock")
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
    async fn gets_dividends() {
        let call = Arc::new(Mutex::new(None));
        let client = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "output": [{
                    "std_dt": "20251231",
                    "per_sto_divi_amt": "361",
                    "divi_rate": "2.1",
                    "divi_kind": "현금배당",
                    "stck_bsop_date": "20251230"
                }]
            }),
            call: call.clone(),
        };

        let result = get_dividends(&client, "005930").await.unwrap();
        assert_eq!(result[0].per_sto_divi_amt, "361");

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.path, PATH_DIVIDEND);
        assert_eq!(call.tr_id, TR_ID_DIVIDEND);
        assert_eq!(call.params["SHR_ISCD"], "005930");
    }

    #[tokio::test]
    async fn gets_news() {
        let call = Arc::new(Mutex::new(None));
        let client = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "output": [{
                    "data_dt": "20260306",
                    "data_tm": "090000",
                    "cntt_ttle": "삼성전자 관련 뉴스",
                    "data_srce": "연합뉴스"
                }]
            }),
            call: call.clone(),
        };

        let result = get_news(&client, "005930").await.unwrap();
        assert_eq!(result[0].cntt_ttle, "삼성전자 관련 뉴스");

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.path, PATH_NEWS);
        assert_eq!(call.tr_id, TR_ID_NEWS);
        assert_eq!(call.params["FID_INPUT_ISCD"], "005930");
    }

    #[tokio::test]
    async fn gets_invest_opinions() {
        let call = Arc::new(Mutex::new(None));
        let client = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "output": [{
                    "stck_bsop_date": "20260306",
                    "invt_opnn": "매수",
                    "invt_opnn_cls": "1",
                    "rgbf_nm": "한국투자",
                    "mbcr_name": "85000",
                    "stck_prpr": "72000",
                    "cnss_prpr": "83000"
                }]
            }),
            call: call.clone(),
        };

        let result = get_invest_opinions(&client, "005930").await.unwrap();
        assert_eq!(result[0].invt_opnn, "매수");

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.path, PATH_INVEST_OPINION);
        assert_eq!(call.tr_id, TR_ID_INVEST_OPINION);
        assert_eq!(call.params["FID_INPUT_ISCD"], "005930");
    }

    #[tokio::test]
    async fn searches_stocks() {
        let call = Arc::new(Mutex::new(None));
        let client = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "output": [{
                    "pdno": "005930",
                    "prdt_name": "삼성전자",
                    "prdt_eng_name": "Samsung Electronics",
                    "mrkt_cls_code": "KOSPI",
                    "scty_grp_id": "ST",
                    "lstg_stqt": "5969782550"
                }]
            }),
            call: call.clone(),
        };

        let result = search_stocks(&client, "삼성").await.unwrap();
        assert_eq!(result[0].pdno, "005930");

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.path, PATH_SEARCH_STOCK);
        assert_eq!(call.tr_id, TR_ID_SEARCH_STOCK);
        assert_eq!(call.params["PDNO"], "삼성");
        assert_eq!(call.params["PRDT_TYPE_CD"], "300");
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

        let err = get_news(&client, "005930").await.unwrap_err();
        assert_eq!(err.to_string(), "news API error: [EGW00001] 잘못된 요청");
    }
}
