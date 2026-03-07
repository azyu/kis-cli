use std::collections::HashMap;

use anyhow::{Context, Result};
use kis_core::client::JsonResponse;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::client::{ApiClient, ensure_success};

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

    use anyhow::Result;
    use async_trait::async_trait;
    use kis_core::client::JsonResponse;
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
}
