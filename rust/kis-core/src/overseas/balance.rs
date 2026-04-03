use std::collections::HashMap;

use crate::client::JsonResponse;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api_client::{ApiClient, ensure_success, parse_output, to_json_value};

const MAX_PAGES: usize = 10;

const PATH_INQUIRE_BALANCE: &str = "/uapi/overseas-stock/v1/trading/inquire-balance";
const TR_ID_BALANCE_REAL: &str = "TTTS3012R";
const TR_ID_BALANCE_VIRTUAL: &str = "VTTS3012R";

const PATH_INQUIRE_PRESENT_BALANCE: &str =
    "/uapi/overseas-stock/v1/trading/inquire-present-balance";
const TR_ID_PRESENT_BALANCE_REAL: &str = "CTRP6504R";
const TR_ID_PRESENT_BALANCE_VIRTUAL: &str = "VTRP6504R";

const PATH_INQUIRE_PAYMT_STDR_BALANCE: &str =
    "/uapi/overseas-stock/v1/trading/inquire-paymt-stdr-balance";
const TR_ID_PAYMT_STDR_BALANCE: &str = "CTRP6010R";

const PATH_INQUIRE_CCLD: &str = "/uapi/overseas-stock/v1/trading/inquire-ccnl";
const TR_ID_CCLD_REAL: &str = "TTTS3035R";
const TR_ID_CCLD_VIRTUAL: &str = "VTTS3035R";

const PATH_INQUIRE_NCCS: &str = "/uapi/overseas-stock/v1/trading/inquire-nccs";
const TR_ID_NCCS_REAL: &str = "TTTS3018R";
const TR_ID_NCCS_VIRTUAL: &str = "VTTS3018R";

const PATH_INQUIRE_PSAMOUNT: &str = "/uapi/overseas-stock/v1/trading/inquire-psamount";
const TR_ID_PSAMOUNT_REAL: &str = "TTTS3007R";
const TR_ID_PSAMOUNT_VIRTUAL: &str = "VTTS3007R";

const PATH_INQUIRE_PERIOD_PROFIT: &str = "/uapi/overseas-stock/v1/trading/inquire-period-profit";
const TR_ID_PERIOD_PROFIT: &str = "TTTS3039R";

const PATH_INQUIRE_PERIOD_TRANS: &str = "/uapi/overseas-stock/v1/trading/inquire-period-trans";
const TR_ID_PERIOD_TRANS: &str = "CTOS4001R";

const PATH_INQUIRE_ALGO_CCLD: &str = "/uapi/overseas-stock/v1/trading/inquire-algo-ccnl";
const TR_ID_ALGO_CCLD: &str = "TTTS6059R";

const PATH_ORDER_RESV_LIST: &str = "/uapi/overseas-stock/v1/trading/order-resv-list";
const TR_ID_ORDER_RESV_LIST_US: &str = "TTTT3039R";
const TR_ID_ORDER_RESV_LIST_ASIA: &str = "TTTS3014R";

const PATH_ORDER_RESV_CCLD: &str = "/uapi/overseas-stock/v1/trading/order-resv-ccnl";
const TR_ID_ORDER_RESV_CCLD_REAL: &str = "TTTT3017U";
const TR_ID_ORDER_RESV_CCLD_VIRTUAL: &str = "VTTT3017U";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct BalanceItem {
    #[serde(default)]
    pub cano: String,
    #[serde(default)]
    pub acnt_prdt_cd: String,
    #[serde(default)]
    pub prdt_type_cd: String,
    #[serde(default)]
    pub ovrs_pdno: String,
    #[serde(default)]
    pub tr_crcy_cd: String,
    #[serde(default)]
    pub ovrs_excg_cd: String,
    #[serde(default)]
    pub ovrs_cblc_qty: String,
    #[serde(default)]
    pub ord_psbl_qty: String,
    #[serde(default)]
    pub pchs_avg_pric: String,
    #[serde(default)]
    pub now_pric2: String,
    #[serde(default)]
    pub ovrs_stck_evlu_amt: String,
    #[serde(default)]
    pub frcr_evlu_pfls_amt: String,
    #[serde(default)]
    pub evlu_pfls_rt: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct BalanceSummary {
    #[serde(default)]
    pub ovrs_rlzt_pfls_amt: String,
    #[serde(default)]
    pub ovrs_tot_pfls: String,
    #[serde(default)]
    pub rlzt_erng_rt: String,
    #[serde(default)]
    pub tot_evlu_pfls_amt: String,
    #[serde(default)]
    pub tot_pftrt: String,
    #[serde(default)]
    pub frcr_buy_amt_smtl1: String,
    #[serde(default)]
    pub frcr_buy_amt_smtl2: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct BalanceResult {
    pub items: Vec<BalanceItem>,
    pub summary: Vec<BalanceSummary>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PresentBalanceItem {
    #[serde(default)]
    pub pdno: String,
    #[serde(default)]
    pub ovrs_excg_cd: String,
    #[serde(default)]
    pub tr_mket_name: String,
    #[serde(default)]
    pub natn_kor_name: String,
    #[serde(default)]
    pub cblc_qty13: String,
    #[serde(default)]
    pub ord_psbl_qty1: String,
    #[serde(default)]
    pub avg_unpr3: String,
    #[serde(default)]
    pub ovrs_now_pric1: String,
    #[serde(default)]
    pub evlu_pfls_amt2: String,
    #[serde(default)]
    pub evlu_pfls_rt1: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PresentBalanceOutput2 {
    #[serde(default)]
    pub frcr_buy_amt_smtl: String,
    #[serde(default)]
    pub frcr_sll_amt_smtl: String,
    #[serde(default)]
    pub frcr_dncl_amt_2: String,
    #[serde(default)]
    pub frcr_drwg_psbl_amt_1: String,
    #[serde(default)]
    pub nxdy_frcr_drwg_psbl_amt: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PresentBalanceOutput3 {
    #[serde(default)]
    pub pchs_amt_smtl: String,
    #[serde(default)]
    pub evlu_amt_smtl: String,
    #[serde(default)]
    pub evlu_pfls_amt_smtl: String,
    #[serde(default)]
    pub tot_asst_amt: String,
    #[serde(default)]
    pub tot_loan_amt: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PresentBalanceResult {
    pub output1: Vec<PresentBalanceItem>,
    pub output2: Vec<PresentBalanceOutput2>,
    pub output3: Vec<PresentBalanceOutput3>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SettlementBalanceItem {
    #[serde(default)]
    pub pdno: String,
    #[serde(default)]
    pub prdt_name: String,
    #[serde(default)]
    pub ovrs_excg_cd: String,
    #[serde(default)]
    pub cblc_qty13: String,
    #[serde(default)]
    pub ord_psbl_qty1: String,
    #[serde(default)]
    pub avg_unpr3: String,
    #[serde(default)]
    pub ovrs_now_pric1: String,
    #[serde(default)]
    pub evlu_pfls_amt2: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SettlementBalanceOutput2 {
    #[serde(default)]
    pub frcr_dncl_amt_2: String,
    #[serde(default)]
    pub frst_bltn_exrt: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SettlementBalanceOutput3 {
    #[serde(default)]
    pub pchs_amt_smtl_amt: String,
    #[serde(default)]
    pub tot_evlu_pfls_amt: String,
    #[serde(default)]
    pub tot_dncl_amt: String,
    #[serde(default)]
    pub tot_asst_amt2: String,
    #[serde(default)]
    pub tot_loan_amt: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SettlementBalanceResult {
    pub output1: Vec<SettlementBalanceItem>,
    pub output2: Vec<SettlementBalanceOutput2>,
    pub output3: Vec<SettlementBalanceOutput3>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ExecutionItem {
    #[serde(default)]
    pub ord_dt: String,
    #[serde(default)]
    pub ord_gno_brno: String,
    #[serde(default)]
    pub odno: String,
    #[serde(default)]
    pub orgn_odno: String,
    #[serde(default)]
    pub sll_buy_dvsn_cd: String,
    #[serde(default)]
    pub sll_buy_dvsn_cd_name: String,
    #[serde(default)]
    pub rvse_cncl_dvsn: String,
    #[serde(default)]
    pub rvse_cncl_dvsn_name: String,
    #[serde(default)]
    pub pdno: String,
    #[serde(default)]
    pub prdt_name: String,
    #[serde(default)]
    pub ft_ord_qty: String,
    #[serde(default)]
    pub ft_ord_unpr3: String,
    #[serde(default)]
    pub ft_ccld_qty: String,
    #[serde(default)]
    pub ft_ccld_unpr3: String,
    #[serde(default)]
    pub ft_ccld_amt3: String,
    #[serde(default)]
    pub nccs_qty: String,
    #[serde(default)]
    pub prcs_stat_name: String,
    #[serde(default)]
    pub ord_tmd: String,
    #[serde(default)]
    pub tr_mket_name: String,
    #[serde(default)]
    pub tr_crcy_cd: String,
    #[serde(default)]
    pub ovrs_excg_cd: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct OpenOrderItem {
    #[serde(default)]
    pub ord_dt: String,
    #[serde(default)]
    pub ord_gno_brno: String,
    #[serde(default)]
    pub odno: String,
    #[serde(default)]
    pub orgn_odno: String,
    #[serde(default)]
    pub pdno: String,
    #[serde(default)]
    pub sll_buy_dvsn_cd: String,
    #[serde(default)]
    pub rvse_cncl_dvsn_cd: String,
    #[serde(default)]
    pub ord_tmd: String,
    #[serde(default)]
    pub tr_crcy_cd: String,
    #[serde(default)]
    pub natn_cd: String,
    #[serde(default)]
    pub ft_ord_qty: String,
    #[serde(default)]
    pub ft_ccld_qty: String,
    #[serde(default)]
    pub nccs_qty: String,
    #[serde(default)]
    pub ft_ord_unpr3: String,
    #[serde(default)]
    pub ft_ccld_unpr3: String,
    #[serde(default)]
    pub ft_ccld_amt3: String,
    #[serde(default)]
    pub ovrs_excg_cd: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PossibleBuyAmount {
    #[serde(default)]
    pub tr_crcy_cd: String,
    #[serde(default)]
    pub ord_psbl_frcr_amt: String,
    #[serde(default)]
    pub sll_ruse_psbl_amt: String,
    #[serde(default)]
    pub ovrs_ord_psbl_amt: String,
    #[serde(default)]
    pub max_ord_psbl_qty: String,
    #[serde(default)]
    pub echm_af_ord_psbl_amt: String,
    #[serde(default)]
    pub echm_af_ord_psbl_qty: String,
    #[serde(default)]
    pub ord_psbl_qty: String,
    #[serde(default)]
    pub exrt: String,
    #[serde(default)]
    pub frcr_ord_psbl_amt1: String,
    #[serde(default)]
    pub ovrs_max_ord_psbl_qty: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PeriodProfitItem {
    #[serde(default)]
    pub trad_day: String,
    #[serde(default)]
    pub ovrs_pdno: String,
    #[serde(default)]
    pub slcl_qty: String,
    #[serde(default)]
    pub pchs_avg_pric: String,
    #[serde(default)]
    pub frcr_pchs_amt1: String,
    #[serde(default)]
    pub avg_sll_unpr: String,
    #[serde(default)]
    pub frcr_sll_amt_smtl1: String,
    #[serde(default)]
    pub stck_sll_tlex: String,
    #[serde(default)]
    pub ovrs_rlzt_pfls_amt: String,
    #[serde(default)]
    pub pftrt: String,
    #[serde(default)]
    pub exrt: String,
    #[serde(default)]
    pub ovrs_excg_cd: String,
    #[serde(default)]
    pub frst_bltn_exrt: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PeriodProfitSummary {
    #[serde(default)]
    pub stck_sll_amt_smtl: String,
    #[serde(default)]
    pub stck_buy_amt_smtl: String,
    #[serde(default)]
    pub smtl_fee1: String,
    #[serde(default)]
    pub excc_dfrm_amt: String,
    #[serde(default)]
    pub ovrs_rlzt_pfls_tot_amt: String,
    #[serde(default)]
    pub tot_pftrt: String,
    #[serde(default)]
    pub bass_dt: String,
    #[serde(default)]
    pub exrt: String,
    #[serde(default)]
    pub frcr_pchs_amt1: String,
    #[serde(default)]
    pub frcr_sll_amt_smtl1: String,
    #[serde(default)]
    pub ovrs_rlzt_pfls_amt: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PeriodProfitResult {
    pub items: Vec<PeriodProfitItem>,
    pub summary: Vec<PeriodProfitSummary>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PeriodTransactionItem {
    #[serde(default)]
    pub trad_dt: String,
    #[serde(default)]
    pub sttl_dt: String,
    #[serde(default)]
    pub sll_buy_dvsn_cd: String,
    #[serde(default)]
    pub sll_buy_dvsn_name: String,
    #[serde(default)]
    pub pdno: String,
    #[serde(default)]
    pub ovrs_item_name: String,
    #[serde(default)]
    pub ccld_qty: String,
    #[serde(default)]
    pub amt_unit_ccld_qty: String,
    #[serde(default)]
    pub ft_ccld_unpr2: String,
    #[serde(default)]
    pub ovrs_stck_ccld_unpr: String,
    #[serde(default)]
    pub tr_frcr_amt2: String,
    #[serde(default)]
    pub tr_amt: String,
    #[serde(default)]
    pub frcr_excc_amt_1: String,
    #[serde(default)]
    pub wcrc_excc_amt: String,
    #[serde(default)]
    pub dmst_frcr_fee1: String,
    #[serde(default)]
    pub frcr_fee1: String,
    #[serde(default)]
    pub dmst_wcrc_fee: String,
    #[serde(default)]
    pub ovrs_wcrc_fee: String,
    #[serde(default)]
    pub crcy_cd: String,
    #[serde(default)]
    pub std_pdno: String,
    #[serde(default)]
    pub erlm_exrt: String,
    #[serde(default)]
    pub loan_dvsn_cd: String,
    #[serde(default)]
    pub loan_dvsn_name: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PeriodTransactionSummary {
    #[serde(default)]
    pub frcr_buy_amt_smtl: String,
    #[serde(default)]
    pub frcr_sll_amt_smtl: String,
    #[serde(default)]
    pub dmst_fee_smtl: String,
    #[serde(default)]
    pub ovrs_fee_smtl: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PeriodTransactionResult {
    pub items: Vec<PeriodTransactionItem>,
    pub summary: Vec<PeriodTransactionSummary>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AlgoExecutionItem {
    #[serde(rename = "CCLD_SEQ", alias = "ccld_seq", default)]
    pub ccld_seq: String,
    #[serde(rename = "CCLD_BTWN", alias = "ccld_btwn", default)]
    pub ccld_btwn: String,
    #[serde(rename = "PDNO", alias = "pdno", default)]
    pub pdno: String,
    #[serde(rename = "ITEM_NAME", alias = "item_name", default)]
    pub item_name: String,
    #[serde(rename = "FT_CCLD_QTY", alias = "ft_ccld_qty", default)]
    pub ft_ccld_qty: String,
    #[serde(rename = "FT_CCLD_UNPR3", alias = "ft_ccld_unpr3", default)]
    pub ft_ccld_unpr3: String,
    #[serde(rename = "FT_CCLD_AMT3", alias = "ft_ccld_amt3", default)]
    pub ft_ccld_amt3: String,
    #[serde(rename = "ODNO", alias = "odno", default)]
    pub odno: String,
    #[serde(rename = "TRAD_DVSN_NAME", alias = "trad_dvsn_name", default)]
    pub trad_dvsn_name: String,
    #[serde(rename = "FT_ORD_QTY", alias = "ft_ord_qty", default)]
    pub ft_ord_qty: String,
    #[serde(rename = "FT_ORD_UNPR3", alias = "ft_ord_unpr3", default)]
    pub ft_ord_unpr3: String,
    #[serde(rename = "ORD_TMD", alias = "ord_tmd", default)]
    pub ord_tmd: String,
    #[serde(rename = "SPLT_BUY_ATTR_NAME", alias = "splt_buy_attr_name", default)]
    pub splt_buy_attr_name: String,
    #[serde(rename = "TR_CRCY", alias = "tr_crcy", default)]
    pub tr_crcy: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AlgoExecutionSummary {
    #[serde(rename = "CCLD_CNT", alias = "ccld_cnt", default)]
    pub ccld_cnt: String,
    #[serde(rename = "TR_CRCY", alias = "tr_crcy", default)]
    pub tr_crcy: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AlgoExecutionResult {
    pub items: Vec<AlgoExecutionItem>,
    pub summary: Vec<AlgoExecutionSummary>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ReservationOrderItem {
    #[serde(default)]
    pub cncl_yn: String,
    #[serde(default)]
    pub rsvn_ord_rcit_dt: String,
    #[serde(default)]
    pub ovrs_rsvn_odno: String,
    #[serde(default)]
    pub ord_dt: String,
    #[serde(default)]
    pub ord_gno_brno: String,
    #[serde(default)]
    pub odno: String,
    #[serde(default)]
    pub sll_buy_dvsn_cd: String,
    #[serde(default)]
    pub sll_buy_dvsn_cd_name: String,
    #[serde(default)]
    pub ovrs_rsvn_ord_stat_cd: String,
    #[serde(default)]
    pub ovrs_rsvn_ord_stat_cd_name: String,
    #[serde(default)]
    pub pdno: String,
    #[serde(default)]
    pub prdt_type_cd: String,
    #[serde(default)]
    pub prdt_name: String,
    #[serde(default)]
    pub ord_rcit_tmd: String,
    #[serde(default)]
    pub ord_fwdg_tmd: String,
    #[serde(default)]
    pub tr_dvsn_name: String,
    #[serde(default)]
    pub ovrs_excg_cd: String,
    #[serde(default)]
    pub tr_mket_name: String,
    #[serde(default)]
    pub ord_stfno: String,
    #[serde(default)]
    pub ft_ord_qty: String,
    #[serde(default)]
    pub ft_ord_unpr3: String,
    #[serde(default)]
    pub ft_ccld_qty: String,
    #[serde(default)]
    pub nprc_rson_text: String,
    #[serde(default)]
    pub splt_buy_attr_name: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ReservationCancelRequest {
    pub account_no: String,
    pub account_prod: String,
    pub receipt_date: String,
    pub reservation_order_no: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ReservationCancelResponse {
    #[serde(rename = "ODNO", alias = "odno", default)]
    pub odno: String,
    #[serde(rename = "RSVN_ORD_RCIT_DT", alias = "rsvn_ord_rcit_dt", default)]
    pub rsvn_ord_rcit_dt: String,
    #[serde(rename = "OVRS_RSVN_ODNO", alias = "ovrs_rsvn_odno", default)]
    pub ovrs_rsvn_odno: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
struct CursorEnvelope {
    #[serde(default)]
    output: Value,
    #[serde(default)]
    output1: Value,
    #[serde(default)]
    output2: Value,
    #[serde(default)]
    ctx_area_fk200: String,
    #[serde(default)]
    ctx_area_nk200: String,
}

#[derive(Debug, Deserialize)]
struct TripleEnvelope {
    #[serde(default)]
    output1: Value,
    #[serde(default)]
    output2: Value,
    #[serde(default)]
    output3: Value,
    #[serde(default)]
    ctx_area_fk200: String,
    #[serde(default)]
    ctx_area_nk200: String,
}

#[derive(Debug, Deserialize)]
struct Cursor100Envelope {
    #[serde(default)]
    output1: Value,
    #[serde(default)]
    output2: Value,
    #[serde(default)]
    ctx_area_fk100: String,
    #[serde(default)]
    ctx_area_nk100: String,
}

#[derive(Debug, Deserialize)]
struct CursorOutput3Envelope {
    #[serde(default)]
    output: Value,
    #[serde(default)]
    output3: Value,
    #[serde(default)]
    ctx_area_fk200: String,
    #[serde(default)]
    ctx_area_nk200: String,
}

pub async fn inquire_balance<C>(
    client: &C,
    cano: &str,
    acnt_prdt_cd: &str,
    ovrs_excg_cd: &str,
    tr_crcy_cd: &str,
    is_virtual: bool,
) -> Result<BalanceResult>
where
    C: ApiClient + Sync,
{
    get_balance(
        client,
        cano,
        acnt_prdt_cd,
        ovrs_excg_cd,
        tr_crcy_cd,
        is_virtual,
    )
    .await
}

pub async fn get_balance<C>(
    client: &C,
    account_no: &str,
    account_prod: &str,
    exchange_code: &str,
    currency_code: &str,
    is_virtual: bool,
) -> Result<BalanceResult>
where
    C: ApiClient + Sync,
{
    let mut fk200 = String::new();
    let mut nk200 = String::new();
    let mut tr_cont = String::new();
    let mut items = Vec::new();
    let mut summary = Vec::new();

    for _ in 0..MAX_PAGES {
        let params = HashMap::from([
            ("CANO".to_string(), account_no.to_string()),
            ("ACNT_PRDT_CD".to_string(), account_prod.to_string()),
            ("OVRS_EXCG_CD".to_string(), exchange_code.to_string()),
            ("TR_CRCY_CD".to_string(), currency_code.to_string()),
            ("CTX_AREA_FK200".to_string(), fk200.clone()),
            ("CTX_AREA_NK200".to_string(), nk200.clone()),
        ]);
        let response = client
            .get_json_response_with_tr_cont(
                PATH_INQUIRE_BALANCE,
                if is_virtual {
                    TR_ID_BALANCE_VIRTUAL
                } else {
                    TR_ID_BALANCE_REAL
                },
                &tr_cont,
                &params,
            )
            .await?;
        let has_next = has_next_page(response.tr_cont.as_deref());
        let envelope: CursorEnvelope = parse_envelope(response, "overseas balance")?;

        items.extend(
            deserialize_rows::<BalanceItem>(envelope.output1).context("parsing balance items")?,
        );
        summary.extend(
            deserialize_rows::<BalanceSummary>(envelope.output2)
                .context("parsing balance summary")?,
        );

        if !has_next {
            return Ok(BalanceResult { items, summary });
        }

        fk200 = envelope.ctx_area_fk200;
        nk200 = envelope.ctx_area_nk200;
        tr_cont = "N".to_string();
    }

    Ok(BalanceResult { items, summary })
}

#[allow(clippy::too_many_arguments)]
pub async fn inquire_present_balance<C>(
    client: &C,
    cano: &str,
    acnt_prdt_cd: &str,
    wcrc_frcr_dvsn_cd: &str,
    natn_cd: &str,
    tr_mket_cd: &str,
    inqr_dvsn_cd: &str,
    is_virtual: bool,
) -> Result<PresentBalanceResult>
where
    C: ApiClient + Sync,
{
    get_present_balance(
        client,
        cano,
        acnt_prdt_cd,
        wcrc_frcr_dvsn_cd,
        natn_cd,
        tr_mket_cd,
        inqr_dvsn_cd,
        is_virtual,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn get_present_balance<C>(
    client: &C,
    account_no: &str,
    account_prod: &str,
    currency_div: &str,
    nation_code: &str,
    market_code: &str,
    inquiry_div: &str,
    is_virtual: bool,
) -> Result<PresentBalanceResult>
where
    C: ApiClient + Sync,
{
    let mut fk200 = String::new();
    let mut nk200 = String::new();
    let mut tr_cont = String::new();
    let mut output1 = Vec::new();
    let mut output2 = Vec::new();
    let mut output3 = Vec::new();

    for _ in 0..MAX_PAGES {
        let params = HashMap::from([
            ("CANO".to_string(), account_no.to_string()),
            ("ACNT_PRDT_CD".to_string(), account_prod.to_string()),
            ("WCRC_FRCR_DVSN_CD".to_string(), currency_div.to_string()),
            ("NATN_CD".to_string(), nation_code.to_string()),
            ("TR_MKET_CD".to_string(), market_code.to_string()),
            ("INQR_DVSN_CD".to_string(), inquiry_div.to_string()),
            ("CTX_AREA_FK200".to_string(), fk200.clone()),
            ("CTX_AREA_NK200".to_string(), nk200.clone()),
        ]);
        let response = client
            .get_json_response_with_tr_cont(
                PATH_INQUIRE_PRESENT_BALANCE,
                if is_virtual {
                    TR_ID_PRESENT_BALANCE_VIRTUAL
                } else {
                    TR_ID_PRESENT_BALANCE_REAL
                },
                &tr_cont,
                &params,
            )
            .await?;
        let has_next = has_next_page(response.tr_cont.as_deref());
        let envelope: TripleEnvelope = parse_envelope(response, "present balance")?;

        output1.extend(
            deserialize_rows::<PresentBalanceItem>(envelope.output1)
                .context("parsing present balance output1")?,
        );
        output2.extend(
            deserialize_rows::<PresentBalanceOutput2>(envelope.output2)
                .context("parsing present balance output2")?,
        );
        output3.extend(
            deserialize_rows::<PresentBalanceOutput3>(envelope.output3)
                .context("parsing present balance output3")?,
        );

        if !has_next {
            break;
        }

        fk200 = envelope.ctx_area_fk200;
        nk200 = envelope.ctx_area_nk200;
        tr_cont = "N".to_string();
    }

    Ok(PresentBalanceResult {
        output1,
        output2,
        output3,
    })
}

pub async fn inquire_paymt_stdr_balance<C>(
    client: &C,
    cano: &str,
    acnt_prdt_cd: &str,
    bass_dt: &str,
    wcrc_frcr_dvsn_cd: &str,
    inqr_dvsn_cd: &str,
    is_virtual: bool,
) -> Result<SettlementBalanceResult>
where
    C: ApiClient + Sync,
{
    get_payment_balance(
        client,
        cano,
        acnt_prdt_cd,
        bass_dt,
        wcrc_frcr_dvsn_cd,
        inqr_dvsn_cd,
        is_virtual,
    )
    .await
}

pub async fn get_payment_balance<C>(
    client: &C,
    account_no: &str,
    account_prod: &str,
    base_date: &str,
    currency_div: &str,
    inquiry_div: &str,
    _is_virtual: bool,
) -> Result<SettlementBalanceResult>
where
    C: ApiClient + Sync,
{
    let mut fk200 = String::new();
    let mut nk200 = String::new();
    let mut tr_cont = String::new();
    let mut output1 = Vec::new();
    let mut output2 = Vec::new();
    let mut output3 = Vec::new();

    for _ in 0..MAX_PAGES {
        let params = HashMap::from([
            ("CANO".to_string(), account_no.to_string()),
            ("ACNT_PRDT_CD".to_string(), account_prod.to_string()),
            ("BASS_DT".to_string(), base_date.to_string()),
            ("WCRC_FRCR_DVSN_CD".to_string(), currency_div.to_string()),
            ("INQR_DVSN_CD".to_string(), inquiry_div.to_string()),
            ("CTX_AREA_FK200".to_string(), fk200.clone()),
            ("CTX_AREA_NK200".to_string(), nk200.clone()),
        ]);
        let response = client
            .get_json_response_with_tr_cont(
                PATH_INQUIRE_PAYMT_STDR_BALANCE,
                TR_ID_PAYMT_STDR_BALANCE,
                &tr_cont,
                &params,
            )
            .await?;
        let has_next = has_next_page(response.tr_cont.as_deref());
        let envelope: TripleEnvelope = parse_envelope(response, "payment balance")?;

        output1.extend(
            deserialize_rows::<SettlementBalanceItem>(envelope.output1)
                .context("parsing payment balance output1")?,
        );
        output2.extend(
            deserialize_rows::<SettlementBalanceOutput2>(envelope.output2)
                .context("parsing payment balance output2")?,
        );
        output3.extend(
            deserialize_rows::<SettlementBalanceOutput3>(envelope.output3)
                .context("parsing payment balance output3")?,
        );

        if !has_next {
            break;
        }

        fk200 = envelope.ctx_area_fk200;
        nk200 = envelope.ctx_area_nk200;
        tr_cont = "N".to_string();
    }

    Ok(SettlementBalanceResult {
        output1,
        output2,
        output3,
    })
}

#[allow(clippy::too_many_arguments)]
pub async fn inquire_ccnl<C>(
    client: &C,
    cano: &str,
    acnt_prdt_cd: &str,
    pdno: &str,
    ord_strt_dt: &str,
    ord_end_dt: &str,
    sll_buy_dvsn: &str,
    ccld_nccs_dvsn: &str,
    ovrs_excg_cd: &str,
    sort_sqn: &str,
    is_virtual: bool,
) -> Result<Vec<ExecutionItem>>
where
    C: ApiClient + Sync,
{
    get_executions(
        client,
        cano,
        acnt_prdt_cd,
        pdno,
        ord_strt_dt,
        ord_end_dt,
        sll_buy_dvsn,
        ccld_nccs_dvsn,
        ovrs_excg_cd,
        sort_sqn,
        is_virtual,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn get_executions<C>(
    client: &C,
    account_no: &str,
    account_prod: &str,
    stock_code: &str,
    start_date: &str,
    end_date: &str,
    side: &str,
    status: &str,
    exchange_code: &str,
    sort: &str,
    is_virtual: bool,
) -> Result<Vec<ExecutionItem>>
where
    C: ApiClient + Sync,
{
    let mut fk200 = String::new();
    let mut nk200 = String::new();
    let mut tr_cont = String::new();
    let mut items = Vec::new();

    for _ in 0..MAX_PAGES {
        let params = HashMap::from([
            ("CANO".to_string(), account_no.to_string()),
            ("ACNT_PRDT_CD".to_string(), account_prod.to_string()),
            ("PDNO".to_string(), stock_code.to_string()),
            ("ORD_STRT_DT".to_string(), start_date.to_string()),
            ("ORD_END_DT".to_string(), end_date.to_string()),
            ("SLL_BUY_DVSN".to_string(), side.to_string()),
            ("CCLD_NCCS_DVSN".to_string(), status.to_string()),
            ("OVRS_EXCG_CD".to_string(), exchange_code.to_string()),
            ("SORT_SQN".to_string(), sort.to_string()),
            ("ORD_DT".to_string(), "".to_string()),
            ("ORD_GNO_BRNO".to_string(), "".to_string()),
            ("ODNO".to_string(), "".to_string()),
            ("CTX_AREA_NK200".to_string(), nk200.clone()),
            ("CTX_AREA_FK200".to_string(), fk200.clone()),
        ]);
        let response = client
            .get_json_response_with_tr_cont(
                PATH_INQUIRE_CCLD,
                if is_virtual {
                    TR_ID_CCLD_VIRTUAL
                } else {
                    TR_ID_CCLD_REAL
                },
                &tr_cont,
                &params,
            )
            .await?;
        let has_next = has_next_page(response.tr_cont.as_deref());
        let envelope: CursorEnvelope = parse_envelope(response, "overseas executions")?;

        items.extend(
            deserialize_rows::<ExecutionItem>(envelope.output)
                .context("parsing overseas executions")?,
        );

        if !has_next {
            return Ok(items);
        }

        fk200 = envelope.ctx_area_fk200;
        nk200 = envelope.ctx_area_nk200;
        tr_cont = "N".to_string();
    }

    Ok(items)
}

pub async fn inquire_nccs<C>(
    client: &C,
    cano: &str,
    acnt_prdt_cd: &str,
    ovrs_excg_cd: &str,
    sort_sqn: &str,
    is_virtual: bool,
) -> Result<Vec<OpenOrderItem>>
where
    C: ApiClient + Sync,
{
    get_open_orders(
        client,
        cano,
        acnt_prdt_cd,
        ovrs_excg_cd,
        sort_sqn,
        is_virtual,
    )
    .await
}

pub async fn get_open_orders<C>(
    client: &C,
    account_no: &str,
    account_prod: &str,
    exchange_code: &str,
    sort: &str,
    is_virtual: bool,
) -> Result<Vec<OpenOrderItem>>
where
    C: ApiClient + Sync,
{
    let mut fk200 = String::new();
    let mut nk200 = String::new();
    let mut tr_cont = String::new();
    let mut items = Vec::new();

    for _ in 0..MAX_PAGES {
        let params = HashMap::from([
            ("CANO".to_string(), account_no.to_string()),
            ("ACNT_PRDT_CD".to_string(), account_prod.to_string()),
            ("OVRS_EXCG_CD".to_string(), exchange_code.to_string()),
            ("SORT_SQN".to_string(), sort.to_string()),
            ("CTX_AREA_FK200".to_string(), fk200.clone()),
            ("CTX_AREA_NK200".to_string(), nk200.clone()),
        ]);
        let response = client
            .get_json_response_with_tr_cont(
                PATH_INQUIRE_NCCS,
                if is_virtual {
                    TR_ID_NCCS_VIRTUAL
                } else {
                    TR_ID_NCCS_REAL
                },
                &tr_cont,
                &params,
            )
            .await?;
        let has_next = has_next_page(response.tr_cont.as_deref());
        let envelope: CursorEnvelope = parse_envelope(response, "open orders")?;

        items.extend(
            deserialize_rows::<OpenOrderItem>(envelope.output).context("parsing open orders")?,
        );

        if !has_next {
            return Ok(items);
        }

        fk200 = envelope.ctx_area_fk200;
        nk200 = envelope.ctx_area_nk200;
        tr_cont = "N".to_string();
    }

    Ok(items)
}

pub async fn inquire_psamount<C>(
    client: &C,
    cano: &str,
    acnt_prdt_cd: &str,
    ovrs_excg_cd: &str,
    ovrs_ord_unpr: &str,
    item_cd: &str,
    is_virtual: bool,
) -> Result<PossibleBuyAmount>
where
    C: ApiClient + Sync,
{
    get_possible_buy_amount(
        client,
        cano,
        acnt_prdt_cd,
        ovrs_excg_cd,
        ovrs_ord_unpr,
        item_cd,
        is_virtual,
    )
    .await
}

pub async fn get_possible_buy_amount<C>(
    client: &C,
    account_no: &str,
    account_prod: &str,
    exchange_code: &str,
    order_price: &str,
    item_code: &str,
    is_virtual: bool,
) -> Result<PossibleBuyAmount>
where
    C: ApiClient + Sync,
{
    let params = HashMap::from([
        ("CANO".to_string(), account_no.to_string()),
        ("ACNT_PRDT_CD".to_string(), account_prod.to_string()),
        ("OVRS_EXCG_CD".to_string(), exchange_code.to_string()),
        ("OVRS_ORD_UNPR".to_string(), order_price.to_string()),
        ("ITEM_CD".to_string(), item_code.to_string()),
    ]);
    let response = client
        .get_json(
            PATH_INQUIRE_PSAMOUNT,
            if is_virtual {
                TR_ID_PSAMOUNT_VIRTUAL
            } else {
                TR_ID_PSAMOUNT_REAL
            },
            &params,
        )
        .await?;
    parse_output(response, "possible buy amount")
}

#[allow(clippy::too_many_arguments)]
pub async fn inquire_period_profit<C>(
    client: &C,
    cano: &str,
    acnt_prdt_cd: &str,
    ovrs_excg_cd: &str,
    natn_cd: &str,
    crcy_cd: &str,
    pdno: &str,
    inqr_strt_dt: &str,
    inqr_end_dt: &str,
    wcrc_frcr_dvsn_cd: &str,
) -> Result<PeriodProfitResult>
where
    C: ApiClient + Sync,
{
    get_period_profit(
        client,
        cano,
        acnt_prdt_cd,
        ovrs_excg_cd,
        natn_cd,
        crcy_cd,
        pdno,
        inqr_strt_dt,
        inqr_end_dt,
        wcrc_frcr_dvsn_cd,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn get_period_profit<C>(
    client: &C,
    account_no: &str,
    account_prod: &str,
    exchange_code: &str,
    nation_code: &str,
    currency_code: &str,
    product_code: &str,
    start_date: &str,
    end_date: &str,
    currency_div: &str,
) -> Result<PeriodProfitResult>
where
    C: ApiClient + Sync,
{
    let mut fk200 = String::new();
    let mut nk200 = String::new();
    let mut tr_cont = String::new();
    let mut items = Vec::new();
    let mut summary = Vec::new();

    for _ in 0..MAX_PAGES {
        let params = HashMap::from([
            ("CANO".to_string(), account_no.to_string()),
            ("ACNT_PRDT_CD".to_string(), account_prod.to_string()),
            ("OVRS_EXCG_CD".to_string(), exchange_code.to_string()),
            ("NATN_CD".to_string(), nation_code.to_string()),
            ("CRCY_CD".to_string(), currency_code.to_string()),
            ("PDNO".to_string(), product_code.to_string()),
            ("INQR_STRT_DT".to_string(), start_date.to_string()),
            ("INQR_END_DT".to_string(), end_date.to_string()),
            ("WCRC_FRCR_DVSN_CD".to_string(), currency_div.to_string()),
            ("CTX_AREA_FK200".to_string(), fk200.clone()),
            ("CTX_AREA_NK200".to_string(), nk200.clone()),
        ]);
        let response = client
            .get_json_response_with_tr_cont(
                PATH_INQUIRE_PERIOD_PROFIT,
                TR_ID_PERIOD_PROFIT,
                &tr_cont,
                &params,
            )
            .await?;
        let has_next = has_next_page(response.tr_cont.as_deref());
        let envelope: CursorEnvelope = parse_envelope(response, "period profit")?;

        items.extend(
            deserialize_rows::<PeriodProfitItem>(envelope.output1)
                .context("parsing period profit items")?,
        );
        summary.extend(
            deserialize_rows::<PeriodProfitSummary>(envelope.output2)
                .context("parsing period profit summary")?,
        );

        if !has_next {
            return Ok(PeriodProfitResult { items, summary });
        }

        fk200 = envelope.ctx_area_fk200;
        nk200 = envelope.ctx_area_nk200;
        tr_cont = "N".to_string();
    }

    Ok(PeriodProfitResult { items, summary })
}

#[allow(clippy::too_many_arguments)]
pub async fn inquire_period_trans<C>(
    client: &C,
    cano: &str,
    acnt_prdt_cd: &str,
    erlm_strt_dt: &str,
    erlm_end_dt: &str,
    ovrs_excg_cd: &str,
    pdno: &str,
    sll_buy_dvsn_cd: &str,
    loan_dvsn_cd: &str,
) -> Result<PeriodTransactionResult>
where
    C: ApiClient + Sync,
{
    get_period_transactions(
        client,
        cano,
        acnt_prdt_cd,
        erlm_strt_dt,
        erlm_end_dt,
        ovrs_excg_cd,
        pdno,
        sll_buy_dvsn_cd,
        loan_dvsn_cd,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn get_period_transactions<C>(
    client: &C,
    account_no: &str,
    account_prod: &str,
    start_date: &str,
    end_date: &str,
    exchange_code: &str,
    product_code: &str,
    side_code: &str,
    loan_code: &str,
) -> Result<PeriodTransactionResult>
where
    C: ApiClient + Sync,
{
    let mut fk100 = String::new();
    let mut nk100 = String::new();
    let mut tr_cont = String::new();
    let mut items = Vec::new();
    let mut summary = Vec::new();

    for _ in 0..MAX_PAGES {
        let params = HashMap::from([
            ("CANO".to_string(), account_no.to_string()),
            ("ACNT_PRDT_CD".to_string(), account_prod.to_string()),
            ("ERLM_STRT_DT".to_string(), start_date.to_string()),
            ("ERLM_END_DT".to_string(), end_date.to_string()),
            ("OVRS_EXCG_CD".to_string(), exchange_code.to_string()),
            ("PDNO".to_string(), product_code.to_string()),
            ("SLL_BUY_DVSN_CD".to_string(), side_code.to_string()),
            ("LOAN_DVSN_CD".to_string(), loan_code.to_string()),
            ("CTX_AREA_FK100".to_string(), fk100.clone()),
            ("CTX_AREA_NK100".to_string(), nk100.clone()),
        ]);
        let response = client
            .get_json_response_with_tr_cont(
                PATH_INQUIRE_PERIOD_TRANS,
                TR_ID_PERIOD_TRANS,
                &tr_cont,
                &params,
            )
            .await?;
        let has_next = has_next_page(response.tr_cont.as_deref());
        let envelope: Cursor100Envelope = parse_envelope(response, "period transactions")?;

        items.extend(
            deserialize_rows::<PeriodTransactionItem>(envelope.output1)
                .context("parsing period transaction items")?,
        );
        summary.extend(
            deserialize_rows::<PeriodTransactionSummary>(envelope.output2)
                .context("parsing period transaction summary")?,
        );

        if !has_next {
            return Ok(PeriodTransactionResult { items, summary });
        }

        fk100 = envelope.ctx_area_fk100;
        nk100 = envelope.ctx_area_nk100;
        tr_cont = "N".to_string();
    }

    Ok(PeriodTransactionResult { items, summary })
}

pub async fn inquire_algo_ccnl<C>(
    client: &C,
    cano: &str,
    acnt_prdt_cd: &str,
    ord_dt: &str,
    ord_gno_brno: &str,
    odno: &str,
    ttlz_icld_yn: &str,
) -> Result<AlgoExecutionResult>
where
    C: ApiClient + Sync,
{
    get_algo_executions(
        client,
        cano,
        acnt_prdt_cd,
        ord_dt,
        ord_gno_brno,
        odno,
        ttlz_icld_yn,
    )
    .await
}

pub async fn get_algo_executions<C>(
    client: &C,
    account_no: &str,
    account_prod: &str,
    order_date: &str,
    order_branch_no: &str,
    order_no: &str,
    include_totals: &str,
) -> Result<AlgoExecutionResult>
where
    C: ApiClient + Sync,
{
    let mut fk200 = String::new();
    let mut nk200 = String::new();
    let mut tr_cont = String::new();
    let mut items = Vec::new();
    let mut summary = Vec::new();

    for _ in 0..MAX_PAGES {
        let params = HashMap::from([
            ("CANO".to_string(), account_no.to_string()),
            ("ACNT_PRDT_CD".to_string(), account_prod.to_string()),
            ("ORD_DT".to_string(), order_date.to_string()),
            ("ORD_GNO_BRNO".to_string(), order_branch_no.to_string()),
            ("ODNO".to_string(), order_no.to_string()),
            ("TTLZ_ICLD_YN".to_string(), include_totals.to_string()),
            ("CTX_AREA_FK200".to_string(), fk200.clone()),
            ("CTX_AREA_NK200".to_string(), nk200.clone()),
        ]);
        let response = client
            .get_json_response_with_tr_cont(
                PATH_INQUIRE_ALGO_CCLD,
                TR_ID_ALGO_CCLD,
                &tr_cont,
                &params,
            )
            .await?;
        let has_next = has_next_page(response.tr_cont.as_deref());
        let envelope: CursorOutput3Envelope = parse_envelope(response, "algo executions")?;

        items.extend(
            deserialize_rows::<AlgoExecutionItem>(envelope.output)
                .context("parsing algo execution items")?,
        );
        summary.extend(
            deserialize_rows::<AlgoExecutionSummary>(envelope.output3)
                .context("parsing algo execution summary")?,
        );

        if !has_next {
            return Ok(AlgoExecutionResult { items, summary });
        }

        fk200 = envelope.ctx_area_fk200;
        nk200 = envelope.ctx_area_nk200;
        tr_cont = "N".to_string();
    }

    Ok(AlgoExecutionResult { items, summary })
}

#[allow(clippy::too_many_arguments)]
pub async fn order_resv_list<C>(
    client: &C,
    nat_dv: &str,
    cano: &str,
    acnt_prdt_cd: &str,
    inqr_strt_dt: &str,
    inqr_end_dt: &str,
    inqr_dvsn_cd: &str,
    ovrs_excg_cd: &str,
    prdt_type_cd: &str,
) -> Result<Vec<ReservationOrderItem>>
where
    C: ApiClient + Sync,
{
    get_reservation_orders(
        client,
        nat_dv,
        cano,
        acnt_prdt_cd,
        inqr_strt_dt,
        inqr_end_dt,
        inqr_dvsn_cd,
        ovrs_excg_cd,
        prdt_type_cd,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn get_reservation_orders<C>(
    client: &C,
    market: &str,
    account_no: &str,
    account_prod: &str,
    start_date: &str,
    end_date: &str,
    inquiry_code: &str,
    exchange_code: &str,
    product_type: &str,
) -> Result<Vec<ReservationOrderItem>>
where
    C: ApiClient + Sync,
{
    let tr_id = reservation_list_tr_id(market)?;
    let mut fk200 = String::new();
    let mut nk200 = String::new();
    let mut tr_cont = String::new();
    let mut items = Vec::new();

    for _ in 0..MAX_PAGES {
        let params = HashMap::from([
            ("CANO".to_string(), account_no.to_string()),
            ("ACNT_PRDT_CD".to_string(), account_prod.to_string()),
            ("INQR_STRT_DT".to_string(), start_date.to_string()),
            ("INQR_END_DT".to_string(), end_date.to_string()),
            ("INQR_DVSN_CD".to_string(), inquiry_code.to_string()),
            ("OVRS_EXCG_CD".to_string(), exchange_code.to_string()),
            ("PRDT_TYPE_CD".to_string(), product_type.to_string()),
            ("CTX_AREA_FK200".to_string(), fk200.clone()),
            ("CTX_AREA_NK200".to_string(), nk200.clone()),
        ]);
        let response = client
            .get_json_response_with_tr_cont(PATH_ORDER_RESV_LIST, tr_id, &tr_cont, &params)
            .await?;
        let has_next = has_next_page(response.tr_cont.as_deref());
        let envelope: CursorEnvelope = parse_envelope(response, "reservation orders")?;

        items.extend(
            deserialize_rows::<ReservationOrderItem>(envelope.output)
                .context("parsing reservation orders")?,
        );

        if !has_next {
            return Ok(items);
        }

        fk200 = envelope.ctx_area_fk200;
        nk200 = envelope.ctx_area_nk200;
        tr_cont = "N".to_string();
    }

    Ok(items)
}

pub async fn order_resv_ccnl<C>(
    client: &C,
    nat_dv: &str,
    request: &ReservationCancelRequest,
    is_virtual: bool,
) -> Result<ReservationCancelResponse>
where
    C: ApiClient + Sync,
{
    cancel_reservation_order(client, nat_dv, request, is_virtual).await
}

pub async fn cancel_reservation_order<C>(
    client: &C,
    market: &str,
    request: &ReservationCancelRequest,
    is_virtual: bool,
) -> Result<ReservationCancelResponse>
where
    C: ApiClient + Sync,
{
    let tr_id = reservation_cancel_tr_id(market, is_virtual)?;
    #[derive(Debug, Serialize)]
    struct ReservationCancelBody<'a> {
        #[serde(rename = "CANO")]
        cano: &'a str,
        #[serde(rename = "ACNT_PRDT_CD")]
        acnt_prdt_cd: &'a str,
        #[serde(rename = "RSVN_ORD_RCIT_DT")]
        rsvn_ord_rcit_dt: &'a str,
        #[serde(rename = "OVRS_RSVN_ODNO")]
        ovrs_rsvn_odno: &'a str,
    }

    let body = to_json_value(ReservationCancelBody {
        cano: &request.account_no,
        acnt_prdt_cd: &request.account_prod,
        rsvn_ord_rcit_dt: &request.receipt_date,
        ovrs_rsvn_odno: &request.reservation_order_no,
    })?;
    let response = client.post_json(PATH_ORDER_RESV_CCLD, tr_id, &body).await?;
    parse_output(response, "reservation cancel")
}

fn reservation_list_tr_id(market: &str) -> Result<&'static str> {
    match market.to_ascii_lowercase().as_str() {
        "us" => Ok(TR_ID_ORDER_RESV_LIST_US),
        "asia" => Ok(TR_ID_ORDER_RESV_LIST_ASIA),
        other => bail!("unsupported reservation market {other:?}; expected \"us\" or \"asia\""),
    }
}

fn reservation_cancel_tr_id(market: &str, is_virtual: bool) -> Result<&'static str> {
    match market.to_ascii_lowercase().as_str() {
        "us" => Ok(if is_virtual {
            TR_ID_ORDER_RESV_CCLD_VIRTUAL
        } else {
            TR_ID_ORDER_RESV_CCLD_REAL
        }),
        other => bail!("unsupported reservation cancel market {other:?}; expected \"us\""),
    }
}

fn has_next_page(tr_cont: Option<&str>) -> bool {
    matches!(tr_cont, Some("M" | "F"))
}

fn parse_envelope<T>(response: JsonResponse, label: &str) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    ensure_success(&response.body, label)?;
    serde_json::from_value(response.body).with_context(|| format!("parsing {label} response"))
}

fn deserialize_rows<T>(value: Value) -> Result<Vec<T>>
where
    T: for<'de> Deserialize<'de>,
{
    if value.is_null() {
        return Ok(Vec::new());
    }
    if value.is_array() {
        serde_json::from_value(value).context("parsing response rows")
    } else {
        Ok(vec![
            serde_json::from_value(value).context("parsing response row")?,
        ])
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use crate::client::JsonResponse;
    use anyhow::Result;
    use async_trait::async_trait;
    use serde_json::json;

    use super::*;

    #[derive(Debug, Default, Clone)]
    struct GetCall {
        path: String,
        tr_id: String,
        tr_cont: String,
        params: HashMap<String, String>,
    }

    #[derive(Clone)]
    struct MockClient {
        responses: Arc<Mutex<Vec<JsonResponse>>>,
        calls: Arc<Mutex<Vec<GetCall>>>,
    }

    #[derive(Debug, Default, Clone)]
    struct SimpleGetCall {
        path: String,
        tr_id: String,
        params: HashMap<String, String>,
    }

    #[derive(Clone)]
    struct JsonClient {
        response: Value,
        call: Arc<Mutex<Option<SimpleGetCall>>>,
    }

    #[derive(Debug, Default, Clone)]
    struct PostCall {
        path: String,
        tr_id: String,
        body: Value,
    }

    #[derive(Clone)]
    struct PostClient {
        response: Value,
        call: Arc<Mutex<Option<PostCall>>>,
    }

    #[async_trait]
    impl ApiClient for MockClient {
        async fn get_json(
            &self,
            _path: &str,
            _tr_id: &str,
            _params: &HashMap<String, String>,
        ) -> Result<Value> {
            unreachable!()
        }

        async fn post_json(&self, _path: &str, _tr_id: &str, _body: &Value) -> Result<Value> {
            unreachable!()
        }

        async fn get_json_response(
            &self,
            path: &str,
            tr_id: &str,
            params: &HashMap<String, String>,
        ) -> Result<JsonResponse> {
            self.get_json_response_with_tr_cont(path, tr_id, "", params)
                .await
        }

        async fn get_json_response_with_tr_cont(
            &self,
            path: &str,
            tr_id: &str,
            tr_cont: &str,
            params: &HashMap<String, String>,
        ) -> Result<JsonResponse> {
            self.calls.lock().unwrap().push(GetCall {
                path: path.to_string(),
                tr_id: tr_id.to_string(),
                tr_cont: tr_cont.to_string(),
                params: params.clone(),
            });
            Ok(self.responses.lock().unwrap().remove(0))
        }
    }

    #[async_trait]
    impl ApiClient for JsonClient {
        async fn get_json(
            &self,
            path: &str,
            tr_id: &str,
            params: &HashMap<String, String>,
        ) -> Result<Value> {
            *self.call.lock().unwrap() = Some(SimpleGetCall {
                path: path.to_string(),
                tr_id: tr_id.to_string(),
                params: params.clone(),
            });
            Ok(self.response.clone())
        }

        async fn post_json(&self, _path: &str, _tr_id: &str, _body: &Value) -> Result<Value> {
            unreachable!()
        }
    }

    #[async_trait]
    impl ApiClient for PostClient {
        async fn get_json(
            &self,
            _path: &str,
            _tr_id: &str,
            _params: &HashMap<String, String>,
        ) -> Result<Value> {
            unreachable!()
        }

        async fn post_json(&self, path: &str, tr_id: &str, body: &Value) -> Result<Value> {
            *self.call.lock().unwrap() = Some(PostCall {
                path: path.to_string(),
                tr_id: tr_id.to_string(),
                body: body.clone(),
            });
            Ok(self.response.clone())
        }
    }

    fn response(body: Value, tr_cont: Option<&str>) -> JsonResponse {
        JsonResponse {
            body,
            tr_cont: tr_cont.map(ToString::to_string),
        }
    }

    #[tokio::test]
    async fn paginates_balance_with_ctx_area_tokens() {
        let client = MockClient {
            responses: Arc::new(Mutex::new(vec![
                response(
                    json!({
                        "rt_cd": "0",
                        "msg_cd": "MCA00000",
                        "msg1": "정상처리",
                        "output1": {
                            "ovrs_pdno": "AAPL",
                            "ovrs_cblc_qty": "3"
                        },
                        "output2": {
                            "tot_pftrt": "1.23"
                        },
                        "ctx_area_fk200": "FK1",
                        "ctx_area_nk200": "NK1"
                    }),
                    Some("M"),
                ),
                response(
                    json!({
                        "rt_cd": "0",
                        "msg_cd": "MCA00000",
                        "msg1": "정상처리",
                        "output1": [{
                            "ovrs_pdno": "MSFT",
                            "ovrs_cblc_qty": "4"
                        }],
                        "output2": [{
                            "tot_pftrt": "2.34"
                        }],
                        "ctx_area_fk200": "",
                        "ctx_area_nk200": ""
                    }),
                    None,
                ),
            ])),
            calls: Arc::new(Mutex::new(Vec::new())),
        };

        let result = inquire_balance(&client, "12345678", "01", "NASD", "USD", true)
            .await
            .unwrap();

        assert_eq!(result.items.len(), 2);
        assert_eq!(result.items[0].ovrs_pdno, "AAPL");
        assert_eq!(result.items[1].ovrs_pdno, "MSFT");
        assert_eq!(result.summary.len(), 2);

        let calls = client.calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].path, PATH_INQUIRE_BALANCE);
        assert_eq!(calls[0].tr_id, TR_ID_BALANCE_VIRTUAL);
        assert_eq!(calls[0].tr_cont, "");
        assert_eq!(calls[1].params["CTX_AREA_FK200"], "FK1");
        assert_eq!(calls[1].params["CTX_AREA_NK200"], "NK1");
        assert_eq!(calls[1].tr_cont, "N");
    }

    #[tokio::test]
    async fn uses_real_tr_id_for_balance() {
        let client = MockClient {
            responses: Arc::new(Mutex::new(vec![response(
                json!({
                    "rt_cd": "0",
                    "msg_cd": "MCA00000",
                    "msg1": "정상처리",
                    "output1": [],
                    "output2": []
                }),
                None,
            )])),
            calls: Arc::new(Mutex::new(Vec::new())),
        };

        let result = inquire_balance(&client, "12345678", "01", "NASD", "USD", false)
            .await
            .unwrap();

        assert!(result.items.is_empty());

        let calls = client.calls.lock().unwrap();
        assert_eq!(calls[0].tr_id, TR_ID_BALANCE_REAL);
    }

    #[tokio::test]
    async fn parses_present_balance_outputs_and_uses_virtual_tr_id() {
        let client = MockClient {
            responses: Arc::new(Mutex::new(vec![response(
                json!({
                    "rt_cd": "0",
                    "msg_cd": "MCA00000",
                    "msg1": "정상처리",
                    "output1": {
                        "pdno": "AAPL",
                        "cblc_qty13": "10"
                    },
                    "output2": {
                        "frcr_dncl_amt_2": "1000"
                    },
                    "output3": {
                        "tot_asst_amt": "2000"
                    }
                }),
                None,
            )])),
            calls: Arc::new(Mutex::new(Vec::new())),
        };

        let result =
            inquire_present_balance(&client, "12345678", "01", "02", "000", "00", "00", true)
                .await
                .unwrap();

        assert_eq!(result.output1[0].pdno, "AAPL");
        assert_eq!(result.output2[0].frcr_dncl_amt_2, "1000");
        assert_eq!(result.output3[0].tot_asst_amt, "2000");

        let calls = client.calls.lock().unwrap();
        assert_eq!(calls[0].path, PATH_INQUIRE_PRESENT_BALANCE);
        assert_eq!(calls[0].tr_id, TR_ID_PRESENT_BALANCE_VIRTUAL);
    }

    #[tokio::test]
    async fn paginates_present_balance_with_ctx_area_tokens() {
        let client = MockClient {
            responses: Arc::new(Mutex::new(vec![
                response(
                    json!({
                        "rt_cd": "0",
                        "msg_cd": "MCA00000",
                        "msg1": "정상처리",
                        "output1": {
                            "pdno": "AAPL"
                        },
                        "output2": {
                            "frcr_dncl_amt_2": "1000"
                        },
                        "output3": {
                            "tot_asst_amt": "2000"
                        },
                        "ctx_area_fk200": "PFK1",
                        "ctx_area_nk200": "PNK1"
                    }),
                    Some("M"),
                ),
                response(
                    json!({
                        "rt_cd": "0",
                        "msg_cd": "MCA00000",
                        "msg1": "정상처리",
                        "output1": [{
                            "pdno": "MSFT"
                        }],
                        "output2": [{
                            "frcr_dncl_amt_2": "1100"
                        }],
                        "output3": [{
                            "tot_asst_amt": "2100"
                        }],
                        "ctx_area_fk200": "",
                        "ctx_area_nk200": ""
                    }),
                    None,
                ),
            ])),
            calls: Arc::new(Mutex::new(Vec::new())),
        };

        let result =
            inquire_present_balance(&client, "12345678", "01", "02", "000", "00", "00", true)
                .await
                .unwrap();

        assert_eq!(result.output1.len(), 2);
        assert_eq!(result.output1[0].pdno, "AAPL");
        assert_eq!(result.output1[1].pdno, "MSFT");

        let calls = client.calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[1].params["CTX_AREA_FK200"], "PFK1");
        assert_eq!(calls[1].params["CTX_AREA_NK200"], "PNK1");
        assert_eq!(calls[1].tr_cont, "N");
    }

    #[tokio::test]
    async fn parses_payment_balance_outputs() {
        let client = MockClient {
            responses: Arc::new(Mutex::new(vec![response(
                json!({
                    "rt_cd": "0",
                    "msg_cd": "MCA00000",
                    "msg1": "정상처리",
                    "output1": {
                        "pdno": "AAPL",
                        "prdt_name": "APPLE"
                    },
                    "output2": {
                        "frcr_dncl_amt_2": "300"
                    },
                    "output3": {
                        "tot_asst_amt2": "5000"
                    }
                }),
                None,
            )])),
            calls: Arc::new(Mutex::new(Vec::new())),
        };

        let result =
            inquire_paymt_stdr_balance(&client, "12345678", "01", "20260306", "01", "00", false)
                .await
                .unwrap();

        assert_eq!(result.output1[0].pdno, "AAPL");
        assert_eq!(result.output2[0].frcr_dncl_amt_2, "300");
        assert_eq!(result.output3[0].tot_asst_amt2, "5000");

        let calls = client.calls.lock().unwrap();
        assert_eq!(calls[0].tr_id, TR_ID_PAYMT_STDR_BALANCE);
    }

    #[tokio::test]
    async fn paginates_payment_balance_with_ctx_area_tokens() {
        let client = MockClient {
            responses: Arc::new(Mutex::new(vec![
                response(
                    json!({
                        "rt_cd": "0",
                        "msg_cd": "MCA00000",
                        "msg1": "정상처리",
                        "output1": {
                            "pdno": "AAPL"
                        },
                        "output2": {
                            "frcr_dncl_amt_2": "300"
                        },
                        "output3": {
                            "tot_asst_amt2": "5000"
                        },
                        "ctx_area_fk200": "SFK1",
                        "ctx_area_nk200": "SNK1"
                    }),
                    Some("F"),
                ),
                response(
                    json!({
                        "rt_cd": "0",
                        "msg_cd": "MCA00000",
                        "msg1": "정상처리",
                        "output1": [{
                            "pdno": "TSLA"
                        }],
                        "output2": [{
                            "frcr_dncl_amt_2": "350"
                        }],
                        "output3": [{
                            "tot_asst_amt2": "5500"
                        }],
                        "ctx_area_fk200": "",
                        "ctx_area_nk200": ""
                    }),
                    None,
                ),
            ])),
            calls: Arc::new(Mutex::new(Vec::new())),
        };

        let result =
            inquire_paymt_stdr_balance(&client, "12345678", "01", "20260306", "01", "00", false)
                .await
                .unwrap();

        assert_eq!(result.output1.len(), 2);
        assert_eq!(result.output1[0].pdno, "AAPL");
        assert_eq!(result.output1[1].pdno, "TSLA");

        let calls = client.calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[1].params["CTX_AREA_FK200"], "SFK1");
        assert_eq!(calls[1].params["CTX_AREA_NK200"], "SNK1");
        assert_eq!(calls[1].tr_cont, "N");
    }

    #[tokio::test]
    async fn paginates_executions_and_uses_virtual_tr_id() {
        let client = MockClient {
            responses: Arc::new(Mutex::new(vec![
                response(
                    json!({
                        "rt_cd": "0",
                        "msg_cd": "MCA00000",
                        "msg1": "정상처리",
                        "output": {
                            "odno": "100",
                            "pdno": "AAPL"
                        },
                        "ctx_area_fk200": "FK2",
                        "ctx_area_nk200": "NK2"
                    }),
                    Some("F"),
                ),
                response(
                    json!({
                        "rt_cd": "0",
                        "msg_cd": "MCA00000",
                        "msg1": "정상처리",
                        "output": [{
                            "odno": "101",
                            "pdno": "MSFT"
                        }],
                        "ctx_area_fk200": "",
                        "ctx_area_nk200": ""
                    }),
                    None,
                ),
            ])),
            calls: Arc::new(Mutex::new(Vec::new())),
        };

        let result = inquire_ccnl(
            &client, "12345678", "01", "%", "20260301", "20260306", "00", "00", "NASD", "DS", true,
        )
        .await
        .unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].odno, "100");
        assert_eq!(result[1].odno, "101");

        let calls = client.calls.lock().unwrap();
        assert_eq!(calls[0].tr_id, TR_ID_CCLD_VIRTUAL);
        assert_eq!(calls[1].params["CTX_AREA_FK200"], "FK2");
        assert_eq!(calls[1].params["CTX_AREA_NK200"], "NK2");
        assert_eq!(calls[1].tr_cont, "N");
    }

    #[tokio::test]
    async fn parses_open_orders_and_uses_virtual_tr_id() {
        let client = MockClient {
            responses: Arc::new(Mutex::new(vec![response(
                json!({
                    "rt_cd": "0",
                    "msg_cd": "MCA00000",
                    "msg1": "정상처리",
                    "output": {
                        "odno": "200",
                        "pdno": "AAPL",
                        "nccs_qty": "2"
                    },
                    "ctx_area_fk200": "",
                    "ctx_area_nk200": ""
                }),
                None,
            )])),
            calls: Arc::new(Mutex::new(Vec::new())),
        };

        let result = inquire_nccs(&client, "12345678", "01", "NASD", "DS", true)
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].odno, "200");
        assert_eq!(result[0].nccs_qty, "2");

        let calls = client.calls.lock().unwrap();
        assert_eq!(calls[0].path, PATH_INQUIRE_NCCS);
        assert_eq!(calls[0].tr_id, TR_ID_NCCS_VIRTUAL);
    }

    #[tokio::test]
    async fn parses_possible_buy_amount_and_uses_virtual_tr_id() {
        let call = Arc::new(Mutex::new(None));
        let client = JsonClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "output": {
                    "tr_crcy_cd": "USD",
                    "ord_psbl_frcr_amt": "1000.00",
                    "ovrs_ord_psbl_amt": "1200.00",
                    "ord_psbl_qty": "7"
                }
            }),
            call: call.clone(),
        };

        let result = inquire_psamount(&client, "12345678", "01", "NASD", "123.45", "AAPL", true)
            .await
            .unwrap();

        assert_eq!(result.tr_crcy_cd, "USD");
        assert_eq!(result.ord_psbl_qty, "7");

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.path, PATH_INQUIRE_PSAMOUNT);
        assert_eq!(call.tr_id, TR_ID_PSAMOUNT_VIRTUAL);
        assert_eq!(call.params["OVRS_EXCG_CD"], "NASD");
        assert_eq!(call.params["ITEM_CD"], "AAPL");
    }

    #[tokio::test]
    async fn paginates_period_profit_with_ctx_area_tokens() {
        let client = MockClient {
            responses: Arc::new(Mutex::new(vec![
                response(
                    json!({
                        "rt_cd": "0",
                        "msg_cd": "MCA00000",
                        "msg1": "정상처리",
                        "output1": [{
                            "trad_day": "20240603",
                            "ovrs_pdno": "AAPL",
                            "ovrs_rlzt_pfls_amt": "10"
                        }],
                        "output2": [{
                            "ovrs_rlzt_pfls_tot_amt": "10",
                            "tot_pftrt": "1.0"
                        }],
                        "ctx_area_fk200": "PFK200",
                        "ctx_area_nk200": "PNK200"
                    }),
                    Some("M"),
                ),
                response(
                    json!({
                        "rt_cd": "0",
                        "msg_cd": "MCA00000",
                        "msg1": "정상처리",
                        "output1": [{
                            "trad_day": "20240604",
                            "ovrs_pdno": "MSFT",
                            "ovrs_rlzt_pfls_amt": "12"
                        }],
                        "output2": [{
                            "ovrs_rlzt_pfls_tot_amt": "22",
                            "tot_pftrt": "2.2"
                        }],
                        "ctx_area_fk200": "",
                        "ctx_area_nk200": ""
                    }),
                    None,
                ),
            ])),
            calls: Arc::new(Mutex::new(Vec::new())),
        };

        let result = inquire_period_profit(
            &client, "12345678", "01", "NASD", "", "USD", "", "20240601", "20240630", "01",
        )
        .await
        .unwrap();

        assert_eq!(result.items.len(), 2);
        assert_eq!(result.items[0].ovrs_pdno, "AAPL");
        assert_eq!(result.summary[1].tot_pftrt, "2.2");

        let calls = client.calls.lock().unwrap();
        assert_eq!(calls[0].path, PATH_INQUIRE_PERIOD_PROFIT);
        assert_eq!(calls[0].tr_id, TR_ID_PERIOD_PROFIT);
        assert_eq!(calls[1].params["CTX_AREA_FK200"], "PFK200");
        assert_eq!(calls[1].params["CTX_AREA_NK200"], "PNK200");
        assert_eq!(calls[1].tr_cont, "N");
    }

    #[tokio::test]
    async fn paginates_period_transactions_with_ctx_area_tokens() {
        let client = MockClient {
            responses: Arc::new(Mutex::new(vec![
                response(
                    json!({
                        "rt_cd": "0",
                        "msg_cd": "MCA00000",
                        "msg1": "정상처리",
                        "output1": [{
                            "trad_dt": "20240603",
                            "pdno": "AAPL",
                            "ccld_qty": "3"
                        }],
                        "output2": [{
                            "frcr_buy_amt_smtl": "300",
                            "frcr_sll_amt_smtl": "0"
                        }],
                        "ctx_area_fk100": "TFK100",
                        "ctx_area_nk100": "TNK100"
                    }),
                    Some("F"),
                ),
                response(
                    json!({
                        "rt_cd": "0",
                        "msg_cd": "MCA00000",
                        "msg1": "정상처리",
                        "output1": [{
                            "trad_dt": "20240604",
                            "pdno": "MSFT",
                            "ccld_qty": "2"
                        }],
                        "output2": [{
                            "frcr_buy_amt_smtl": "500",
                            "frcr_sll_amt_smtl": "100"
                        }],
                        "ctx_area_fk100": "",
                        "ctx_area_nk100": ""
                    }),
                    None,
                ),
            ])),
            calls: Arc::new(Mutex::new(Vec::new())),
        };

        let result = inquire_period_trans(
            &client, "12345678", "01", "20240601", "20240630", "NASD", "", "00", "",
        )
        .await
        .unwrap();

        assert_eq!(result.items.len(), 2);
        assert_eq!(result.items[1].pdno, "MSFT");
        assert_eq!(result.summary[0].frcr_buy_amt_smtl, "300");

        let calls = client.calls.lock().unwrap();
        assert_eq!(calls[0].path, PATH_INQUIRE_PERIOD_TRANS);
        assert_eq!(calls[0].tr_id, TR_ID_PERIOD_TRANS);
        assert_eq!(calls[1].params["CTX_AREA_FK100"], "TFK100");
        assert_eq!(calls[1].params["CTX_AREA_NK100"], "TNK100");
        assert_eq!(calls[1].tr_cont, "N");
    }

    #[tokio::test]
    async fn paginates_algo_executions_with_summary() {
        let client = MockClient {
            responses: Arc::new(Mutex::new(vec![
                response(
                    json!({
                        "rt_cd": "0",
                        "msg_cd": "MCA00000",
                        "msg1": "정상처리",
                        "output": [{
                            "ODNO": "5001",
                            "PDNO": "AAPL",
                            "FT_CCLD_QTY": "1"
                        }],
                        "output3": [{
                            "CCLD_CNT": "1",
                            "TR_CRCY": "USD"
                        }],
                        "ctx_area_fk200": "AFK200",
                        "ctx_area_nk200": "ANK200"
                    }),
                    Some("M"),
                ),
                response(
                    json!({
                        "rt_cd": "0",
                        "msg_cd": "MCA00000",
                        "msg1": "정상처리",
                        "output": [{
                            "ODNO": "5002",
                            "PDNO": "MSFT",
                            "FT_CCLD_QTY": "2"
                        }],
                        "output3": [{
                            "CCLD_CNT": "2",
                            "TR_CRCY": "USD"
                        }],
                        "ctx_area_fk200": "",
                        "ctx_area_nk200": ""
                    }),
                    None,
                ),
            ])),
            calls: Arc::new(Mutex::new(Vec::new())),
        };

        let result = inquire_algo_ccnl(&client, "12345678", "01", "", "", "", "")
            .await
            .unwrap();

        assert_eq!(result.items.len(), 2);
        assert_eq!(result.items[0].odno, "5001");
        assert_eq!(result.summary[1].ccld_cnt, "2");

        let calls = client.calls.lock().unwrap();
        assert_eq!(calls[0].path, PATH_INQUIRE_ALGO_CCLD);
        assert_eq!(calls[0].tr_id, TR_ID_ALGO_CCLD);
        assert_eq!(calls[1].params["CTX_AREA_FK200"], "AFK200");
        assert_eq!(calls[1].params["CTX_AREA_NK200"], "ANK200");
        assert_eq!(calls[1].tr_cont, "N");
    }

    #[tokio::test]
    async fn paginates_reservation_orders_and_uses_asia_tr_id() {
        let client = MockClient {
            responses: Arc::new(Mutex::new(vec![
                response(
                    json!({
                        "rt_cd": "0",
                        "msg_cd": "MCA00000",
                        "msg1": "정상처리",
                        "output": [{
                            "ovrs_rsvn_odno": "9001",
                            "pdno": "7203",
                            "ft_ord_qty": "1"
                        }],
                        "ctx_area_fk200": "RFK200",
                        "ctx_area_nk200": "RNK200"
                    }),
                    Some("F"),
                ),
                response(
                    json!({
                        "rt_cd": "0",
                        "msg_cd": "MCA00000",
                        "msg1": "정상처리",
                        "output": [{
                            "ovrs_rsvn_odno": "9002",
                            "pdno": "6758",
                            "ft_ord_qty": "2"
                        }],
                        "ctx_area_fk200": "",
                        "ctx_area_nk200": ""
                    }),
                    None,
                ),
            ])),
            calls: Arc::new(Mutex::new(Vec::new())),
        };

        let result = order_resv_list(
            &client, "asia", "12345678", "01", "20250101", "20250131", "00", "TKSE", "",
        )
        .await
        .unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].ovrs_rsvn_odno, "9001");
        assert_eq!(result[1].pdno, "6758");

        let calls = client.calls.lock().unwrap();
        assert_eq!(calls[0].path, PATH_ORDER_RESV_LIST);
        assert_eq!(calls[0].tr_id, TR_ID_ORDER_RESV_LIST_ASIA);
        assert_eq!(calls[1].params["CTX_AREA_FK200"], "RFK200");
        assert_eq!(calls[1].params["CTX_AREA_NK200"], "RNK200");
    }

    #[tokio::test]
    async fn cancels_reservation_order_with_expected_body() {
        let call = Arc::new(Mutex::new(None));
        let client = PostClient {
            response: json!({
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "정상처리",
                "output": {
                    "ODNO": "30008244",
                    "RSVN_ORD_RCIT_DT": "20220810",
                    "OVRS_RSVN_ODNO": "30008244"
                }
            }),
            call: call.clone(),
        };
        let request = ReservationCancelRequest {
            account_no: "12345678".to_string(),
            account_prod: "01".to_string(),
            receipt_date: "20220810".to_string(),
            reservation_order_no: "30008244".to_string(),
        };

        let result = cancel_reservation_order(&client, "us", &request, true)
            .await
            .unwrap();

        assert_eq!(result.odno, "30008244");
        assert_eq!(result.rsvn_ord_rcit_dt, "20220810");

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.path, PATH_ORDER_RESV_CCLD);
        assert_eq!(call.tr_id, TR_ID_ORDER_RESV_CCLD_VIRTUAL);
        assert_eq!(
            call.body,
            json!({
                "CANO": "12345678",
                "ACNT_PRDT_CD": "01",
                "RSVN_ORD_RCIT_DT": "20220810",
                "OVRS_RSVN_ODNO": "30008244"
            })
        );
    }
}
