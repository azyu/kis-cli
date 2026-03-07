use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::api_client::{ApiClient, parse_output};

const PATH_BALANCE_SHEET: &str = "/uapi/domestic-stock/v1/finance/balance-sheet";
const TR_ID_BALANCE_SHEET: &str = "FHKST66430100";
const PATH_INCOME_STATEMENT: &str = "/uapi/domestic-stock/v1/finance/income-statement";
const TR_ID_INCOME_STATEMENT: &str = "FHKST66430200";
const PATH_FINANCIAL_RATIO: &str = "/uapi/domestic-stock/v1/finance/financial-ratio";
const TR_ID_FINANCIAL_RATIO: &str = "FHKST66430300";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct BalanceSheetItem {
    #[serde(alias = "stac_yymm")]
    pub stac_yy: String,
    pub cras: String,
    pub fxas: String,
    pub total_aset: String,
    pub flow_lblt: String,
    pub fix_lblt: String,
    pub total_lblt: String,
    pub cpfn: String,
    pub total_cptl: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct IncomeStatementItem {
    #[serde(alias = "stac_yymm")]
    pub stac_yy: String,
    pub sale_account: String,
    pub sale_cost: String,
    pub sale_totl_prfi: String,
    pub bsop_prti: String,
    pub thtr_ntin: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct FinancialRatioItem {
    #[serde(alias = "stac_yymm")]
    pub stac_yy: String,
    #[serde(default, alias = "grs")]
    pub gros_prfi_rate: String,
    #[serde(default, alias = "bsop_prfi_inrt")]
    pub bsop_prfi_rate: String,
    #[serde(default, alias = "ntin_inrt")]
    pub ntin_rate: String,
    #[serde(default, alias = "roe_val")]
    pub roe: String,
    #[serde(default)]
    pub roa: String,
    pub lblt_rate: String,
    pub rsrv_rate: String,
    pub eps: String,
    pub bps: String,
    #[serde(default)]
    pub per: String,
    #[serde(default)]
    pub pbr: String,
}

pub async fn get_balance_sheet<C>(
    client: &C,
    stock_code: &str,
    div_code: Option<&str>,
) -> Result<Vec<BalanceSheetItem>>
where
    C: ApiClient + Sync,
{
    let params = finance_params(stock_code, div_code);
    let response = client
        .get_json(PATH_BALANCE_SHEET, TR_ID_BALANCE_SHEET, &params)
        .await?;
    parse_output(response, "balance sheet")
}

pub async fn get_income_statement<C>(
    client: &C,
    stock_code: &str,
    div_code: Option<&str>,
) -> Result<Vec<IncomeStatementItem>>
where
    C: ApiClient + Sync,
{
    let params = finance_params(stock_code, div_code);
    let response = client
        .get_json(PATH_INCOME_STATEMENT, TR_ID_INCOME_STATEMENT, &params)
        .await?;
    parse_output(response, "income statement")
}

pub async fn get_financial_ratio<C>(
    client: &C,
    stock_code: &str,
    div_code: Option<&str>,
) -> Result<Vec<FinancialRatioItem>>
where
    C: ApiClient + Sync,
{
    let params = finance_params(stock_code, div_code);
    let response = client
        .get_json(PATH_FINANCIAL_RATIO, TR_ID_FINANCIAL_RATIO, &params)
        .await?;
    parse_output(response, "financial ratio")
}

fn finance_params(stock_code: &str, div_code: Option<&str>) -> HashMap<String, String> {
    HashMap::from([
        (
            "FID_DIV_CLS_CODE".to_string(),
            div_code.unwrap_or("0").to_string(),
        ),
        ("fid_cond_mrkt_div_code".to_string(), "J".to_string()),
        ("fid_input_iscd".to_string(), stock_code.to_string()),
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
    async fn gets_balance_sheet_and_defaults_division() {
        let call = Arc::new(Mutex::new(None));
        let client = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "output": [{
                    "stac_yy": "2025",
                    "cras": "100",
                    "fxas": "200",
                    "total_aset": "300",
                    "flow_lblt": "50",
                    "fix_lblt": "30",
                    "total_lblt": "80",
                    "cpfn": "10",
                    "total_cptl": "220"
                }]
            }),
            call: call.clone(),
        };

        let result = get_balance_sheet(&client, "005930", None).await.unwrap();
        assert_eq!(result[0].total_cptl, "220");

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.path, PATH_BALANCE_SHEET);
        assert_eq!(call.tr_id, TR_ID_BALANCE_SHEET);
        assert_eq!(call.params["FID_DIV_CLS_CODE"], "0");
    }

    #[tokio::test]
    async fn gets_income_statement_and_financial_ratio() {
        let income = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "output": [{
                    "stac_yy": "2025",
                    "sale_account": "300",
                    "sale_cost": "100",
                    "sale_totl_prfi": "200",
                    "bsop_prti": "80",
                    "thtr_ntin": "60"
                }]
            }),
            call: Arc::new(Mutex::new(None)),
        };
        let ratio = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "output": [{
                    "stac_yy": "2025",
                    "gros_prfi_rate": "66.7",
                    "bsop_prfi_rate": "26.7",
                    "ntin_rate": "20.0",
                    "roe": "10.1",
                    "roa": "6.2",
                    "lblt_rate": "36.4",
                    "rsrv_rate": "1200",
                    "eps": "5000",
                    "bps": "60000",
                    "per": "14.0",
                    "pbr": "1.2"
                }]
            }),
            call: Arc::new(Mutex::new(None)),
        };

        let income = get_income_statement(&income, "005930", Some("1"))
            .await
            .unwrap();
        assert_eq!(income[0].sale_account, "300");

        let ratio = get_financial_ratio(&ratio, "005930", Some("1"))
            .await
            .unwrap();
        assert_eq!(ratio[0].roe, "10.1");
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

        let err = get_financial_ratio(&client, "005930", None)
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            "financial ratio API error: [EGW00001] 잘못된 요청"
        );
    }
}
