use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::client::{ApiClient, parse_output};

const PATH_DAILY_ITEM_CHART: &str =
    "/uapi/domestic-stock/v1/quotations/inquire-daily-itemchartprice";
const TR_ID_DAILY_ITEM_CHART: &str = "FHKST03010100";
const PATH_TIME_ITEM_CHART: &str = "/uapi/domestic-stock/v1/quotations/inquire-time-itemchartprice";
const TR_ID_TIME_ITEM_CHART: &str = "FHKST03010200";
const PATH_DAILY_INDEX_CHART: &str =
    "/uapi/domestic-stock/v1/quotations/inquire-daily-indexchartprice";
const TR_ID_DAILY_INDEX_CHART: &str = "FHKUP03500100";
const PATH_INDEX_PRICE: &str = "/uapi/domestic-stock/v1/quotations/inquire-index-price";
const TR_ID_INDEX_PRICE: &str = "FHPUP02100000";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ChartItem {
    pub stck_bsop_date: String,
    pub stck_clpr: String,
    pub stck_oprc: String,
    pub stck_hgpr: String,
    pub stck_lwpr: String,
    pub acml_vol: String,
    pub acml_tr_pbmn: String,
    pub prdy_vrss: String,
    pub prdy_vrss_sign: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct TimeChartItem {
    pub stck_cntg_hour: String,
    pub stck_prpr: String,
    pub stck_oprc: String,
    pub stck_hgpr: String,
    pub stck_lwpr: String,
    pub cntg_vol: String,
    pub acml_vol: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct IndexChartItem {
    pub stck_bsop_date: String,
    pub bstp_nmix_prpr: String,
    pub bstp_nmix_oprc: String,
    pub bstp_nmix_hgpr: String,
    pub bstp_nmix_lwpr: String,
    pub acml_vol: String,
    pub acml_tr_pbmn: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct IndexPrice {
    pub bstp_nmix_prpr: String,
    pub bstp_nmix_prdy_vrss: String,
    pub prdy_vrss_sign: String,
    pub bstp_nmix_prdy_ctrt: String,
    pub acml_vol: String,
    pub acml_tr_pbmn: String,
    pub bstp_nmix_oprc: String,
    pub bstp_nmix_hgpr: String,
    pub bstp_nmix_lwpr: String,
}

pub async fn get_daily_chart<C>(
    client: &C,
    stock_code: &str,
    start_date: &str,
    end_date: &str,
    period: Option<&str>,
) -> Result<Vec<ChartItem>>
where
    C: ApiClient + Sync,
{
    let params = HashMap::from([
        ("FID_COND_MRKT_DIV_CODE".to_string(), "J".to_string()),
        ("FID_INPUT_ISCD".to_string(), stock_code.to_string()),
        ("FID_INPUT_DATE_1".to_string(), start_date.to_string()),
        ("FID_INPUT_DATE_2".to_string(), end_date.to_string()),
        (
            "FID_PERIOD_DIV_CODE".to_string(),
            period.unwrap_or("D").to_string(),
        ),
        ("FID_ORG_ADJ_PRC".to_string(), "0".to_string()),
    ]);
    let response = client
        .get_json(PATH_DAILY_ITEM_CHART, TR_ID_DAILY_ITEM_CHART, &params)
        .await?;
    parse_output(response, "daily chart")
}

pub async fn get_time_chart<C>(
    client: &C,
    stock_code: &str,
    time_unit: Option<&str>,
) -> Result<Vec<TimeChartItem>>
where
    C: ApiClient + Sync,
{
    let params = HashMap::from([
        ("FID_COND_MRKT_DIV_CODE".to_string(), "J".to_string()),
        ("FID_INPUT_ISCD".to_string(), stock_code.to_string()),
        ("FID_ETC_CLS_CODE".to_string(), "".to_string()),
        (
            "FID_INPUT_HOUR_1".to_string(),
            time_unit.unwrap_or("1").to_string(),
        ),
        ("FID_PW_DATA_INCU_YN".to_string(), "N".to_string()),
    ]);
    let response = client
        .get_json(PATH_TIME_ITEM_CHART, TR_ID_TIME_ITEM_CHART, &params)
        .await?;
    parse_output(response, "time chart")
}

pub async fn get_daily_index_chart<C>(
    client: &C,
    index_code: &str,
    start_date: &str,
    end_date: &str,
    period: Option<&str>,
) -> Result<Vec<IndexChartItem>>
where
    C: ApiClient + Sync,
{
    let params = HashMap::from([
        ("FID_COND_MRKT_DIV_CODE".to_string(), "U".to_string()),
        ("FID_INPUT_ISCD".to_string(), index_code.to_string()),
        ("FID_INPUT_DATE_1".to_string(), start_date.to_string()),
        ("FID_INPUT_DATE_2".to_string(), end_date.to_string()),
        (
            "FID_PERIOD_DIV_CODE".to_string(),
            period.unwrap_or("D").to_string(),
        ),
    ]);
    let response = client
        .get_json(PATH_DAILY_INDEX_CHART, TR_ID_DAILY_INDEX_CHART, &params)
        .await?;
    parse_output(response, "daily index chart")
}

pub async fn get_index_price<C>(client: &C, index_code: &str) -> Result<IndexPrice>
where
    C: ApiClient + Sync,
{
    let params = HashMap::from([
        ("FID_COND_MRKT_DIV_CODE".to_string(), "U".to_string()),
        ("FID_INPUT_ISCD".to_string(), index_code.to_string()),
    ]);
    let response = client
        .get_json(PATH_INDEX_PRICE, TR_ID_INDEX_PRICE, &params)
        .await?;
    parse_output(response, "index price")
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
    async fn gets_daily_chart_and_defaults_period() {
        let call = Arc::new(Mutex::new(None));
        let client = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "output": [{
                    "stck_bsop_date": "20260306",
                    "stck_clpr": "70000",
                    "stck_oprc": "69000",
                    "stck_hgpr": "70500",
                    "stck_lwpr": "68800",
                    "acml_vol": "12345678",
                    "acml_tr_pbmn": "1000000000",
                    "prdy_vrss": "1000",
                    "prdy_vrss_sign": "2"
                }]
            }),
            call: call.clone(),
        };

        let result = get_daily_chart(&client, "005930", "20260101", "20260306", None)
            .await
            .unwrap();
        assert_eq!(result[0].stck_clpr, "70000");

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.path, PATH_DAILY_ITEM_CHART);
        assert_eq!(call.tr_id, TR_ID_DAILY_ITEM_CHART);
        assert_eq!(call.params["FID_PERIOD_DIV_CODE"], "D");
    }

    #[tokio::test]
    async fn gets_time_chart_and_defaults_unit() {
        let call = Arc::new(Mutex::new(None));
        let client = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "output": [{
                    "stck_cntg_hour": "100000",
                    "stck_prpr": "70100",
                    "stck_oprc": "70000",
                    "stck_hgpr": "70200",
                    "stck_lwpr": "69900",
                    "cntg_vol": "1000",
                    "acml_vol": "5000"
                }]
            }),
            call: call.clone(),
        };

        let result = get_time_chart(&client, "005930", None).await.unwrap();
        assert_eq!(result[0].stck_cntg_hour, "100000");

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.path, PATH_TIME_ITEM_CHART);
        assert_eq!(call.tr_id, TR_ID_TIME_ITEM_CHART);
        assert_eq!(call.params["FID_INPUT_HOUR_1"], "1");
    }

    #[tokio::test]
    async fn gets_daily_index_chart_and_index_price() {
        let index_chart = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "output": [{
                    "stck_bsop_date": "20260306",
                    "bstp_nmix_prpr": "2650.00",
                    "bstp_nmix_oprc": "2630.00",
                    "bstp_nmix_hgpr": "2660.00",
                    "bstp_nmix_lwpr": "2620.00",
                    "acml_vol": "1000000",
                    "acml_tr_pbmn": "500000000"
                }]
            }),
            call: Arc::new(Mutex::new(None)),
        };
        let index_price = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "output": {
                    "bstp_nmix_prpr": "2650.00",
                    "bstp_nmix_prdy_vrss": "10.00",
                    "prdy_vrss_sign": "2",
                    "bstp_nmix_prdy_ctrt": "0.38",
                    "acml_vol": "1000000",
                    "acml_tr_pbmn": "500000000",
                    "bstp_nmix_oprc": "2630.00",
                    "bstp_nmix_hgpr": "2660.00",
                    "bstp_nmix_lwpr": "2620.00"
                }
            }),
            call: Arc::new(Mutex::new(None)),
        };

        let chart = get_daily_index_chart(&index_chart, "0001", "20260101", "20260306", Some("W"))
            .await
            .unwrap();
        assert_eq!(chart[0].bstp_nmix_prpr, "2650.00");

        let price = get_index_price(&index_price, "0001").await.unwrap();
        assert_eq!(price.bstp_nmix_prpr, "2650.00");
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

        let err = get_index_price(&client, "0001").await.unwrap_err();
        assert_eq!(
            err.to_string(),
            "index price API error: [EGW00001] 잘못된 요청"
        );
    }
}
