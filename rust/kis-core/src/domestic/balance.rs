use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api_client::{ApiClient, parse_output, parse_outputs};

const PATH_INQUIRE_BALANCE: &str = "/uapi/domestic-stock/v1/trading/inquire-balance";
const TR_ID_BALANCE_REAL: &str = "TTTC8434R";
const TR_ID_BALANCE_VIRTUAL: &str = "VTTC8434R";
const PATH_INQUIRE_PSBL_ORDER: &str = "/uapi/domestic-stock/v1/trading/inquire-psbl-order";
const TR_ID_PSBL_ORDER_REAL: &str = "TTTC8908R";
const TR_ID_PSBL_ORDER_VIRTUAL: &str = "VTTC8908R";
const PATH_INQUIRE_DAILY_CCLD: &str = "/uapi/domestic-stock/v1/trading/inquire-daily-ccld";
const TR_ID_DAILY_CCLD_REAL: &str = "TTTC8001R";
const TR_ID_DAILY_CCLD_VIRTUAL: &str = "VTTC8001R";
const PATH_INQUIRE_PSBL_SELL: &str = "/uapi/domestic-stock/v1/trading/inquire-psbl-sell";
const TR_ID_PSBL_SELL_REAL: &str = "TTTC6012R";
const TR_ID_PSBL_SELL_VIRTUAL: &str = "VTTC6012R";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct BalanceItem {
    pub pdno: String,
    pub prdt_name: String,
    pub hldg_qty: String,
    pub pchs_avg_pric: String,
    pub pchs_amt: String,
    pub prpr: String,
    pub evlu_amt: String,
    pub evlu_pfls_amt: String,
    pub evlu_pfls_rt: String,
    pub fltt_rt: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct BalanceSummary {
    pub dnca_tot_amt: String,
    pub scts_evlu_amt: String,
    pub tot_evlu_amt: String,
    pub nass_amt: String,
    pub pchs_amt_smtl_amt: String,
    pub evlu_amt_smtl_amt: String,
    pub evlu_pfls_smtl_amt: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct BalanceResult {
    pub items: Vec<BalanceItem>,
    pub summary: Vec<BalanceSummary>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PossibleOrder {
    pub ord_psbl_cash: String,
    pub ord_psbl_sbst: String,
    pub ruse_psbl_amt: String,
    pub nrcvb_buy_amt: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct DailyExecution {
    pub ord_dt: String,
    pub ord_gno_brno: String,
    pub odno: String,
    pub orgn_odno: String,
    #[serde(rename = "sll_buy_dvsn_cd")]
    pub sll_buy_dvsn: String,
    pub pdno: String,
    pub prdt_name: String,
    pub ord_qty: String,
    pub ord_unpr: String,
    pub tot_ccld_qty: String,
    pub tot_ccld_amt: String,
    pub ccld_cndt_nm: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PossibleSell {
    pub pdno: String,
    pub prdt_name: String,
    pub ord_psbl_qty: String,
    pub pchs_avg_pric: String,
}

pub async fn get_balance<C>(
    client: &C,
    account_no: &str,
    account_prod: &str,
    is_virtual: bool,
) -> Result<BalanceResult>
where
    C: ApiClient + Sync,
{
    let params = HashMap::from([
        ("CANO".to_string(), account_no.to_string()),
        ("ACNT_PRDT_CD".to_string(), account_prod.to_string()),
        ("AFHR_FLPR_YN".to_string(), "N".to_string()),
        ("OFL_YN".to_string(), "".to_string()),
        ("INQR_DVSN".to_string(), "02".to_string()),
        ("UNPR_DVSN".to_string(), "01".to_string()),
        ("FUND_STTL_ICLD_YN".to_string(), "N".to_string()),
        ("FNCG_AMT_AUTO_RDPT_YN".to_string(), "N".to_string()),
        ("PRCS_DVSN".to_string(), "01".to_string()),
        ("CTX_AREA_FK100".to_string(), "".to_string()),
        ("CTX_AREA_NK100".to_string(), "".to_string()),
    ]);
    let response = client
        .get_json(
            PATH_INQUIRE_BALANCE,
            if is_virtual {
                TR_ID_BALANCE_VIRTUAL
            } else {
                TR_ID_BALANCE_REAL
            },
            &params,
        )
        .await?;
    let (items, summary) = parse_outputs(response, "balance")?;
    Ok(BalanceResult { items, summary })
}

pub async fn get_possible_order<C>(
    client: &C,
    account_no: &str,
    account_prod: &str,
    stock_code: &str,
    order_div: &str,
    price: &str,
    is_virtual: bool,
) -> Result<PossibleOrder>
where
    C: ApiClient + Sync,
{
    let params = HashMap::from([
        ("CANO".to_string(), account_no.to_string()),
        ("ACNT_PRDT_CD".to_string(), account_prod.to_string()),
        ("PDNO".to_string(), stock_code.to_string()),
        ("ORD_UNPR".to_string(), price.to_string()),
        ("ORD_DVSN".to_string(), order_div.to_string()),
        ("CMA_EVLU_AMT_ICLD_YN".to_string(), "Y".to_string()),
        ("OVRS_ICLD_YN".to_string(), "Y".to_string()),
    ]);
    let response = client
        .get_json(
            PATH_INQUIRE_PSBL_ORDER,
            if is_virtual {
                TR_ID_PSBL_ORDER_VIRTUAL
            } else {
                TR_ID_PSBL_ORDER_REAL
            },
            &params,
        )
        .await?;
    parse_output(response, "possible order")
}

pub async fn get_daily_executions<C>(
    client: &C,
    account_no: &str,
    account_prod: &str,
    start_date: &str,
    end_date: &str,
    is_virtual: bool,
) -> Result<Vec<DailyExecution>>
where
    C: ApiClient + Sync,
{
    let params = HashMap::from([
        ("CANO".to_string(), account_no.to_string()),
        ("ACNT_PRDT_CD".to_string(), account_prod.to_string()),
        ("INQR_STRT_DT".to_string(), start_date.to_string()),
        ("INQR_END_DT".to_string(), end_date.to_string()),
        ("SLL_BUY_DVSN_CD".to_string(), "00".to_string()),
        ("INQR_DVSN".to_string(), "00".to_string()),
        ("PDNO".to_string(), "".to_string()),
        ("CCLD_DVSN".to_string(), "00".to_string()),
        ("ORD_GNO_BRNO".to_string(), "".to_string()),
        ("ODNO".to_string(), "".to_string()),
        ("INQR_DVSN_3".to_string(), "00".to_string()),
        ("INQR_DVSN_1".to_string(), "".to_string()),
        ("CTX_AREA_FK100".to_string(), "".to_string()),
        ("CTX_AREA_NK100".to_string(), "".to_string()),
    ]);
    let response = client
        .get_json(
            PATH_INQUIRE_DAILY_CCLD,
            if is_virtual {
                TR_ID_DAILY_CCLD_VIRTUAL
            } else {
                TR_ID_DAILY_CCLD_REAL
            },
            &params,
        )
        .await?;
    let (items, _): (Vec<DailyExecution>, Value) = parse_outputs(response, "daily execution")?;
    Ok(items)
}

pub async fn get_possible_sell<C>(
    client: &C,
    account_no: &str,
    account_prod: &str,
    stock_code: &str,
    is_virtual: bool,
) -> Result<PossibleSell>
where
    C: ApiClient + Sync,
{
    let params = HashMap::from([
        ("CANO".to_string(), account_no.to_string()),
        ("ACNT_PRDT_CD".to_string(), account_prod.to_string()),
        ("PDNO".to_string(), stock_code.to_string()),
    ]);
    let response = client
        .get_json(
            PATH_INQUIRE_PSBL_SELL,
            if is_virtual {
                TR_ID_PSBL_SELL_VIRTUAL
            } else {
                TR_ID_PSBL_SELL_REAL
            },
            &params,
        )
        .await?;
    parse_output(response, "possible sell")
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use serde_json::json;

    use super::*;

    #[derive(Debug, Default, Clone)]
    struct GetCall {
        path: String,
        tr_id: String,
        params: HashMap<String, String>,
    }

    #[derive(Clone)]
    struct MockClient {
        response: serde_json::Value,
        call: Arc<Mutex<Option<GetCall>>>,
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
    }

    #[tokio::test]
    async fn parses_balance_outputs() {
        let client = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "output1": [{
                    "pdno": "005930",
                    "prdt_name": "삼성전자",
                    "hldg_qty": "10",
                    "pchs_avg_pric": "70000",
                    "pchs_amt": "700000",
                    "prpr": "71000",
                    "evlu_amt": "710000",
                    "evlu_pfls_amt": "10000",
                    "evlu_pfls_rt": "1.43",
                    "fltt_rt": "1.43"
                }],
                "output2": [{
                    "dnca_tot_amt": "1000000",
                    "scts_evlu_amt": "710000",
                    "tot_evlu_amt": "1710000",
                    "nass_amt": "1710000",
                    "pchs_amt_smtl_amt": "700000",
                    "evlu_amt_smtl_amt": "710000",
                    "evlu_pfls_smtl_amt": "10000"
                }]
            }),
            call: Arc::new(Mutex::new(None)),
        };

        let result = get_balance(&client, "12345678", "01", false).await.unwrap();
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.summary.len(), 1);
        assert_eq!(result.items[0].prdt_name, "삼성전자");
    }

    #[tokio::test]
    async fn uses_virtual_tr_id_for_possible_order() {
        let call = Arc::new(Mutex::new(None));
        let client = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "output": {
                    "ord_psbl_cash": "100000",
                    "ord_psbl_sbst": "0",
                    "ruse_psbl_amt": "100000",
                    "nrcvb_buy_amt": "100000"
                }
            }),
            call: call.clone(),
        };

        let result = get_possible_order(&client, "12345678", "01", "005930", "01", "0", true)
            .await
            .unwrap();
        assert_eq!(result.ord_psbl_cash, "100000");

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.path, PATH_INQUIRE_PSBL_ORDER);
        assert_eq!(call.tr_id, TR_ID_PSBL_ORDER_VIRTUAL);
        assert_eq!(call.params["PDNO"], "005930");
    }

    #[tokio::test]
    async fn gets_possible_sell() {
        let client = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "output": {
                    "pdno": "005930",
                    "prdt_name": "삼성전자",
                    "ord_psbl_qty": "10",
                    "pchs_avg_pric": "70000"
                }
            }),
            call: Arc::new(Mutex::new(None)),
        };

        let result = get_possible_sell(&client, "12345678", "01", "005930", false)
            .await
            .unwrap();
        assert_eq!(result.ord_psbl_qty, "10");
    }

    #[tokio::test]
    async fn parses_daily_executions_from_output1() {
        let client = MockClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "output1": [{
                    "ord_dt": "20260306",
                    "ord_gno_brno": "00000",
                    "odno": "1234567890",
                    "orgn_odno": "",
                    "sll_buy_dvsn_cd": "02",
                    "pdno": "005930",
                    "prdt_name": "삼성전자",
                    "ord_qty": "1",
                    "ord_unpr": "70000",
                    "tot_ccld_qty": "1",
                    "tot_ccld_amt": "70000",
                    "ccld_cndt_nm": "전량체결"
                }],
                "output2": {
                    "tot_ord_qty": "1",
                    "tot_ccld_qty": "1",
                    "tot_ccld_amt": "70000",
                    "prsm_tlex_smtl": "0",
                    "pchs_avg_pric": "70000"
                }
            }),
            call: Arc::new(Mutex::new(None)),
        };

        let result = get_daily_executions(&client, "12345678", "01", "20260301", "20260306", true)
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].prdt_name, "삼성전자");
    }

    #[tokio::test]
    async fn returns_api_error_when_possible_sell_is_unavailable() {
        let client = MockClient {
            response: json!({
                "rt_cd": "1",
                "msg_cd": "OPSQ0002",
                "msg1": "없는 서비스 코드 입니다"
            }),
            call: Arc::new(Mutex::new(None)),
        };

        let err = get_possible_sell(&client, "12345678", "01", "148020", true)
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            "possible sell API error: [OPSQ0002] 없는 서비스 코드 입니다"
        );
    }
}
