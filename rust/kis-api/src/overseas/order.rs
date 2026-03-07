use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

use crate::client::{ApiClient, parse_output, to_json_value};

use super::exchange::OrderExchange;

const PATH_ORDER: &str = "/uapi/overseas-stock/v1/trading/order";
const PATH_ORDER_RESV: &str = "/uapi/overseas-stock/v1/trading/order-resv";
const PATH_ORDER_RVSECNCL: &str = "/uapi/overseas-stock/v1/trading/order-rvsecncl";
const PATH_DAYTIME_ORDER: &str = "/uapi/overseas-stock/v1/trading/daytime-order";
const PATH_DAYTIME_ORDER_RVSECNCL: &str = "/uapi/overseas-stock/v1/trading/daytime-order-rvsecncl";
const TR_ID_MODIFY_REAL: &str = "TTTT1004U";
const TR_ID_MODIFY_VIRTUAL: &str = "VTTT1004U";
const TR_ID_CANCEL_REAL: &str = "TTTT1004U";
const TR_ID_CANCEL_VIRTUAL: &str = "VTTT1004U";
const TR_ID_RESV_US_BUY_REAL: &str = "TTTT3014U";
const TR_ID_RESV_US_BUY_VIRTUAL: &str = "VTTT3014U";
const TR_ID_RESV_US_SELL_REAL: &str = "TTTT3016U";
const TR_ID_RESV_US_SELL_VIRTUAL: &str = "VTTT3016U";
const TR_ID_RESV_ASIA_REAL: &str = "TTTS3013U";
const TR_ID_RESV_ASIA_VIRTUAL: &str = "VTTS3013U";
const TR_ID_DAYTIME_BUY_REAL: &str = "TTTS6036U";
const TR_ID_DAYTIME_SELL_REAL: &str = "TTTS6037U";
const TR_ID_DAYTIME_MODIFY_REAL: &str = "TTTS6038U";
const TR_ID_DAYTIME_CANCEL_REAL: &str = "TTTS6038U";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct OrderRequest {
    pub account_no: String,
    pub account_prod: String,
    pub exchange_code: String,
    pub stock_code: String,
    pub quantity: String,
    pub price: String,
    pub order_div: String,
    pub contact_phone: String,
    pub management_order_no: String,
    pub order_server_div: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ModifyCancelRequest {
    pub account_no: String,
    pub account_prod: String,
    pub exchange_code: String,
    pub stock_code: String,
    pub orig_order_no: String,
    pub quantity: String,
    pub price: String,
    pub contact_phone: String,
    pub management_order_no: String,
    pub order_server_div: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct OrderResponse {
    #[serde(rename = "KRX_FWDG_ORD_ORGNO")]
    pub order_org_no: String,
    #[serde(rename = "ODNO")]
    pub order_no: String,
    #[serde(rename = "ORD_TMD")]
    pub order_time: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ReservationOrderResponse {
    #[serde(rename = "ODNO")]
    pub order_no: String,
    #[serde(rename = "RSVN_ORD_RCIT_DT")]
    pub receipt_date: String,
    #[serde(rename = "OVRS_RSVN_ODNO")]
    pub reservation_order_no: String,
}

#[derive(Debug, Serialize)]
struct OrderBody<'a> {
    #[serde(rename = "CANO")]
    cano: &'a str,
    #[serde(rename = "ACNT_PRDT_CD")]
    acnt_prdt_cd: &'a str,
    #[serde(rename = "OVRS_EXCG_CD")]
    ovrs_excg_cd: &'a str,
    #[serde(rename = "PDNO")]
    pdno: &'a str,
    #[serde(rename = "ORD_QTY")]
    ord_qty: &'a str,
    #[serde(rename = "OVRS_ORD_UNPR")]
    ovrs_ord_unpr: &'a str,
    #[serde(rename = "CTAC_TLNO")]
    ctac_tlno: &'a str,
    #[serde(rename = "MGCO_APTM_ODNO")]
    mgco_aptm_odno: &'a str,
    #[serde(rename = "SLL_TYPE")]
    sll_type: &'a str,
    #[serde(rename = "ORD_SVR_DVSN_CD")]
    ord_svr_dvsn_cd: &'a str,
    #[serde(rename = "ORD_DVSN")]
    ord_dvsn: &'a str,
}

#[derive(Debug, Serialize)]
struct ReserveOrderBody<'a> {
    #[serde(rename = "CANO")]
    cano: &'a str,
    #[serde(rename = "ACNT_PRDT_CD")]
    acnt_prdt_cd: &'a str,
    #[serde(rename = "PDNO")]
    pdno: &'a str,
    #[serde(rename = "OVRS_EXCG_CD")]
    ovrs_excg_cd: &'a str,
    #[serde(rename = "FT_ORD_QTY")]
    ft_ord_qty: &'a str,
    #[serde(rename = "FT_ORD_UNPR3")]
    ft_ord_unpr3: &'a str,
    #[serde(rename = "SLL_BUY_DVSN_CD", skip_serializing_if = "str::is_empty")]
    sll_buy_dvsn_cd: &'a str,
    #[serde(rename = "RVSE_CNCL_DVSN_CD", skip_serializing_if = "str::is_empty")]
    rvse_cncl_dvsn_cd: &'a str,
    #[serde(rename = "ORD_SVR_DVSN_CD", skip_serializing_if = "str::is_empty")]
    ord_svr_dvsn_cd: &'a str,
    #[serde(rename = "ORD_DVSN", skip_serializing_if = "str::is_empty")]
    ord_dvsn: &'a str,
}

#[derive(Debug, Serialize)]
struct ModifyCancelBody<'a> {
    #[serde(rename = "CANO")]
    cano: &'a str,
    #[serde(rename = "ACNT_PRDT_CD")]
    acnt_prdt_cd: &'a str,
    #[serde(rename = "OVRS_EXCG_CD")]
    ovrs_excg_cd: &'a str,
    #[serde(rename = "PDNO")]
    pdno: &'a str,
    #[serde(rename = "ORGN_ODNO")]
    orgn_odno: &'a str,
    #[serde(rename = "RVSE_CNCL_DVSN_CD")]
    rvse_cncl_dvsn_cd: &'a str,
    #[serde(rename = "ORD_QTY")]
    ord_qty: &'a str,
    #[serde(rename = "OVRS_ORD_UNPR")]
    ovrs_ord_unpr: &'a str,
    #[serde(rename = "MGCO_APTM_ODNO")]
    mgco_aptm_odno: &'a str,
    #[serde(rename = "ORD_SVR_DVSN_CD")]
    ord_svr_dvsn_cd: &'a str,
}

#[derive(Debug, Serialize)]
struct DaytimeOrderBody<'a> {
    #[serde(rename = "CANO")]
    cano: &'a str,
    #[serde(rename = "ACNT_PRDT_CD")]
    acnt_prdt_cd: &'a str,
    #[serde(rename = "OVRS_EXCG_CD")]
    ovrs_excg_cd: &'a str,
    #[serde(rename = "PDNO")]
    pdno: &'a str,
    #[serde(rename = "ORD_QTY")]
    ord_qty: &'a str,
    #[serde(rename = "OVRS_ORD_UNPR")]
    ovrs_ord_unpr: &'a str,
    #[serde(rename = "CTAC_TLNO")]
    ctac_tlno: &'a str,
    #[serde(rename = "MGCO_APTM_ODNO")]
    mgco_aptm_odno: &'a str,
    #[serde(rename = "ORD_SVR_DVSN_CD")]
    ord_svr_dvsn_cd: &'a str,
    #[serde(rename = "ORD_DVSN")]
    ord_dvsn: &'a str,
}

#[derive(Debug, Serialize)]
struct DaytimeModifyCancelBody<'a> {
    #[serde(rename = "CANO")]
    cano: &'a str,
    #[serde(rename = "ACNT_PRDT_CD")]
    acnt_prdt_cd: &'a str,
    #[serde(rename = "OVRS_EXCG_CD")]
    ovrs_excg_cd: &'a str,
    #[serde(rename = "PDNO")]
    pdno: &'a str,
    #[serde(rename = "ORGN_ODNO")]
    orgn_odno: &'a str,
    #[serde(rename = "RVSE_CNCL_DVSN_CD")]
    rvse_cncl_dvsn_cd: &'a str,
    #[serde(rename = "ORD_QTY")]
    ord_qty: &'a str,
    #[serde(rename = "OVRS_ORD_UNPR")]
    ovrs_ord_unpr: &'a str,
    #[serde(rename = "CTAC_TLNO")]
    ctac_tlno: &'a str,
    #[serde(rename = "MGCO_APTM_ODNO")]
    mgco_aptm_odno: &'a str,
    #[serde(rename = "ORD_SVR_DVSN_CD")]
    ord_svr_dvsn_cd: &'a str,
}

pub async fn buy<C>(client: &C, request: &OrderRequest, is_virtual: bool) -> Result<OrderResponse>
where
    C: ApiClient + Sync,
{
    let exchange = OrderExchange::parse(&request.exchange_code)?;
    place_order(
        client,
        request,
        exchange,
        exchange.buy_tr_id(is_virtual),
        "",
    )
    .await
}

pub async fn sell<C>(client: &C, request: &OrderRequest, is_virtual: bool) -> Result<OrderResponse>
where
    C: ApiClient + Sync,
{
    let exchange = OrderExchange::parse(&request.exchange_code)?;
    place_order(
        client,
        request,
        exchange,
        exchange.sell_tr_id(is_virtual),
        "00",
    )
    .await
}

pub async fn buy_reservation<C>(
    client: &C,
    request: &OrderRequest,
    is_virtual: bool,
) -> Result<ReservationOrderResponse>
where
    C: ApiClient + Sync,
{
    let exchange = OrderExchange::parse(&request.exchange_code)?;
    place_reservation_order(
        client,
        request,
        exchange,
        reservation_buy_tr_id(exchange, is_virtual),
        if exchange.is_us() { "" } else { "02" },
    )
    .await
}

pub async fn sell_reservation<C>(
    client: &C,
    request: &OrderRequest,
    is_virtual: bool,
) -> Result<ReservationOrderResponse>
where
    C: ApiClient + Sync,
{
    let exchange = OrderExchange::parse(&request.exchange_code)?;
    place_reservation_order(
        client,
        request,
        exchange,
        reservation_sell_tr_id(exchange, is_virtual),
        if exchange.is_us() { "" } else { "01" },
    )
    .await
}

pub async fn daytime_buy<C>(
    client: &C,
    request: &OrderRequest,
    is_virtual: bool,
) -> Result<OrderResponse>
where
    C: ApiClient + Sync,
{
    let exchange = OrderExchange::parse(&request.exchange_code)?;
    ensure_daytime_supported(exchange, is_virtual)?;
    place_daytime_order(client, request, exchange, TR_ID_DAYTIME_BUY_REAL).await
}

pub async fn daytime_sell<C>(
    client: &C,
    request: &OrderRequest,
    is_virtual: bool,
) -> Result<OrderResponse>
where
    C: ApiClient + Sync,
{
    let exchange = OrderExchange::parse(&request.exchange_code)?;
    ensure_daytime_supported(exchange, is_virtual)?;
    place_daytime_order(client, request, exchange, TR_ID_DAYTIME_SELL_REAL).await
}

pub async fn modify_order<C>(
    client: &C,
    request: &ModifyCancelRequest,
    is_virtual: bool,
) -> Result<OrderResponse>
where
    C: ApiClient + Sync,
{
    modify_or_cancel(
        client,
        request,
        if is_virtual {
            TR_ID_MODIFY_VIRTUAL
        } else {
            TR_ID_MODIFY_REAL
        },
        "01",
    )
    .await
}

pub async fn cancel_order<C>(
    client: &C,
    request: &ModifyCancelRequest,
    is_virtual: bool,
) -> Result<OrderResponse>
where
    C: ApiClient + Sync,
{
    modify_or_cancel(
        client,
        request,
        if is_virtual {
            TR_ID_CANCEL_VIRTUAL
        } else {
            TR_ID_CANCEL_REAL
        },
        "02",
    )
    .await
}

pub async fn daytime_modify_order<C>(
    client: &C,
    request: &ModifyCancelRequest,
    is_virtual: bool,
) -> Result<OrderResponse>
where
    C: ApiClient + Sync,
{
    let exchange = OrderExchange::parse(&request.exchange_code)?;
    ensure_daytime_supported(exchange, is_virtual)?;
    daytime_modify_or_cancel(client, request, exchange, TR_ID_DAYTIME_MODIFY_REAL, "01").await
}

pub async fn daytime_cancel_order<C>(
    client: &C,
    request: &ModifyCancelRequest,
    is_virtual: bool,
) -> Result<OrderResponse>
where
    C: ApiClient + Sync,
{
    let exchange = OrderExchange::parse(&request.exchange_code)?;
    ensure_daytime_supported(exchange, is_virtual)?;
    daytime_modify_or_cancel(client, request, exchange, TR_ID_DAYTIME_CANCEL_REAL, "02").await
}

async fn place_order<C>(
    client: &C,
    request: &OrderRequest,
    exchange: OrderExchange,
    tr_id: &str,
    sll_type: &str,
) -> Result<OrderResponse>
where
    C: ApiClient + Sync,
{
    let body = to_json_value(OrderBody {
        cano: &request.account_no,
        acnt_prdt_cd: &request.account_prod,
        ovrs_excg_cd: exchange.code(),
        pdno: &request.stock_code,
        ord_qty: &request.quantity,
        ovrs_ord_unpr: &request.price,
        ctac_tlno: &request.contact_phone,
        mgco_aptm_odno: &request.management_order_no,
        sll_type,
        ord_svr_dvsn_cd: &request.order_server_div,
        ord_dvsn: &request.order_div,
    })?;
    let response = client.post_json(PATH_ORDER, tr_id, &body).await?;
    parse_output(response, "overseas order")
}

async fn place_reservation_order<C>(
    client: &C,
    request: &OrderRequest,
    exchange: OrderExchange,
    tr_id: &str,
    sll_buy_dvsn_cd: &str,
) -> Result<ReservationOrderResponse>
where
    C: ApiClient + Sync,
{
    let body = to_json_value(ReserveOrderBody {
        cano: &request.account_no,
        acnt_prdt_cd: &request.account_prod,
        pdno: &request.stock_code,
        ovrs_excg_cd: exchange.code(),
        ft_ord_qty: &request.quantity,
        ft_ord_unpr3: &request.price,
        sll_buy_dvsn_cd,
        rvse_cncl_dvsn_cd: if exchange.is_us() { "" } else { "00" },
        ord_svr_dvsn_cd: &request.order_server_div,
        ord_dvsn: if exchange.is_us() {
            &request.order_div
        } else {
            ""
        },
    })?;
    let response = client.post_json(PATH_ORDER_RESV, tr_id, &body).await?;
    parse_output(response, "overseas reservation order")
}

async fn place_daytime_order<C>(
    client: &C,
    request: &OrderRequest,
    exchange: OrderExchange,
    tr_id: &str,
) -> Result<OrderResponse>
where
    C: ApiClient + Sync,
{
    let body = to_json_value(DaytimeOrderBody {
        cano: &request.account_no,
        acnt_prdt_cd: &request.account_prod,
        ovrs_excg_cd: exchange.code(),
        pdno: &request.stock_code,
        ord_qty: &request.quantity,
        ovrs_ord_unpr: &request.price,
        ctac_tlno: &request.contact_phone,
        mgco_aptm_odno: &request.management_order_no,
        ord_svr_dvsn_cd: &request.order_server_div,
        ord_dvsn: &request.order_div,
    })?;
    let response = client.post_json(PATH_DAYTIME_ORDER, tr_id, &body).await?;
    parse_output(response, "overseas daytime order")
}

async fn modify_or_cancel<C>(
    client: &C,
    request: &ModifyCancelRequest,
    tr_id: &str,
    dvsn: &str,
) -> Result<OrderResponse>
where
    C: ApiClient + Sync,
{
    let exchange = OrderExchange::parse(&request.exchange_code)?;
    let body = to_json_value(ModifyCancelBody {
        cano: &request.account_no,
        acnt_prdt_cd: &request.account_prod,
        ovrs_excg_cd: exchange.code(),
        pdno: &request.stock_code,
        orgn_odno: &request.orig_order_no,
        rvse_cncl_dvsn_cd: dvsn,
        ord_qty: &request.quantity,
        ovrs_ord_unpr: &request.price,
        mgco_aptm_odno: &request.management_order_no,
        ord_svr_dvsn_cd: &request.order_server_div,
    })?;
    let response = client.post_json(PATH_ORDER_RVSECNCL, tr_id, &body).await?;
    parse_output(response, "overseas modify/cancel order")
}

async fn daytime_modify_or_cancel<C>(
    client: &C,
    request: &ModifyCancelRequest,
    exchange: OrderExchange,
    tr_id: &str,
    dvsn: &str,
) -> Result<OrderResponse>
where
    C: ApiClient + Sync,
{
    let body = to_json_value(DaytimeModifyCancelBody {
        cano: &request.account_no,
        acnt_prdt_cd: &request.account_prod,
        ovrs_excg_cd: exchange.code(),
        pdno: &request.stock_code,
        orgn_odno: &request.orig_order_no,
        rvse_cncl_dvsn_cd: dvsn,
        ord_qty: &request.quantity,
        ovrs_ord_unpr: &request.price,
        ctac_tlno: &request.contact_phone,
        mgco_aptm_odno: &request.management_order_no,
        ord_svr_dvsn_cd: &request.order_server_div,
    })?;
    let response = client
        .post_json(PATH_DAYTIME_ORDER_RVSECNCL, tr_id, &body)
        .await?;
    parse_output(response, "overseas daytime modify/cancel order")
}

fn reservation_buy_tr_id(exchange: OrderExchange, is_virtual: bool) -> &'static str {
    match (exchange.is_us(), is_virtual) {
        (true, false) => TR_ID_RESV_US_BUY_REAL,
        (true, true) => TR_ID_RESV_US_BUY_VIRTUAL,
        (false, false) => TR_ID_RESV_ASIA_REAL,
        (false, true) => TR_ID_RESV_ASIA_VIRTUAL,
    }
}

fn reservation_sell_tr_id(exchange: OrderExchange, is_virtual: bool) -> &'static str {
    match (exchange.is_us(), is_virtual) {
        (true, false) => TR_ID_RESV_US_SELL_REAL,
        (true, true) => TR_ID_RESV_US_SELL_VIRTUAL,
        (false, false) => TR_ID_RESV_ASIA_REAL,
        (false, true) => TR_ID_RESV_ASIA_VIRTUAL,
    }
}

fn ensure_daytime_supported(exchange: OrderExchange, is_virtual: bool) -> Result<()> {
    if is_virtual {
        bail!("daytime overseas orders are only supported in the real environment");
    }
    if !exchange.is_us() {
        bail!("daytime overseas orders only support U.S. exchanges: NASD, NYSE, AMEX");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use serde_json::json;

    use super::*;

    #[derive(Debug, Default, Clone)]
    struct PostCall {
        path: String,
        tr_id: String,
        body: serde_json::Value,
    }

    #[derive(Clone)]
    struct MockClient {
        response: serde_json::Value,
        call: Arc<Mutex<Option<PostCall>>>,
    }

    #[async_trait]
    impl ApiClient for MockClient {
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
            path: &str,
            tr_id: &str,
            body: &serde_json::Value,
        ) -> Result<serde_json::Value> {
            *self.call.lock().unwrap() = Some(PostCall {
                path: path.to_string(),
                tr_id: tr_id.to_string(),
                body: body.clone(),
            });
            Ok(self.response.clone())
        }
    }

    fn order_response() -> serde_json::Value {
        json!({
            "rt_cd": "0",
            "msg_cd": "MCA00000",
            "msg1": "정상처리",
            "output": {
                "KRX_FWDG_ORD_ORGNO": "06010",
                "ODNO": "0000123456",
                "ORD_TMD": "100000"
            }
        })
    }

    fn reservation_response() -> serde_json::Value {
        json!({
            "rt_cd": "0",
            "msg_cd": "MCA00000",
            "msg1": "정상처리",
            "output": {
                "ODNO": "06010",
                "RSVN_ORD_RCIT_DT": "20260306",
                "OVRS_RSVN_ODNO": "0000123456"
            }
        })
    }

    fn sample_order(exchange_code: &str) -> OrderRequest {
        OrderRequest {
            account_no: "12345678".to_string(),
            account_prod: "01".to_string(),
            exchange_code: exchange_code.to_string(),
            stock_code: "AAPL".to_string(),
            quantity: "1".to_string(),
            price: "145.00".to_string(),
            order_div: "00".to_string(),
            contact_phone: "".to_string(),
            management_order_no: "".to_string(),
            order_server_div: "0".to_string(),
        }
    }

    fn sample_modify(exchange_code: &str) -> ModifyCancelRequest {
        ModifyCancelRequest {
            account_no: "12345678".to_string(),
            account_prod: "01".to_string(),
            exchange_code: exchange_code.to_string(),
            stock_code: "AAPL".to_string(),
            orig_order_no: "0000123456".to_string(),
            quantity: "1".to_string(),
            price: "145.00".to_string(),
            contact_phone: "".to_string(),
            management_order_no: "".to_string(),
            order_server_div: "0".to_string(),
        }
    }

    #[tokio::test]
    async fn uses_real_buy_tr_id_for_us_exchange() {
        let call = Arc::new(Mutex::new(None));
        let client = MockClient {
            response: order_response(),
            call: call.clone(),
        };

        let response = buy(&client, &sample_order("NASD"), false).await.unwrap();
        assert_eq!(response.order_no, "0000123456");

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.path, PATH_ORDER);
        assert_eq!(call.tr_id, "TTTT1002U");
        assert_eq!(call.body["OVRS_EXCG_CD"], "NASD");
        assert_eq!(call.body["SLL_TYPE"], "");
        assert_eq!(call.body["ORD_SVR_DVSN_CD"], "0");
    }

    #[tokio::test]
    async fn uses_virtual_sell_tr_id_for_vietnam_exchange() {
        let call = Arc::new(Mutex::new(None));
        let client = MockClient {
            response: order_response(),
            call: call.clone(),
        };

        sell(&client, &sample_order("VNSE"), true).await.unwrap();

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.path, PATH_ORDER);
        assert_eq!(call.tr_id, "VTTS0310U");
        assert_eq!(call.body["OVRS_EXCG_CD"], "VNSE");
        assert_eq!(call.body["SLL_TYPE"], "00");
    }

    #[tokio::test]
    async fn uses_us_reservation_tr_id_and_body_shape() {
        let call = Arc::new(Mutex::new(None));
        let client = MockClient {
            response: reservation_response(),
            call: call.clone(),
        };

        let response = buy_reservation(&client, &sample_order("NASD"), true)
            .await
            .unwrap();

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(response.order_no, "06010");
        assert_eq!(response.receipt_date, "20260306");
        assert_eq!(response.reservation_order_no, "0000123456");
        assert_eq!(call.path, PATH_ORDER_RESV);
        assert_eq!(call.tr_id, TR_ID_RESV_US_BUY_VIRTUAL);
        assert_eq!(call.body["OVRS_EXCG_CD"], "NASD");
        assert_eq!(call.body["FT_ORD_QTY"], "1");
        assert_eq!(call.body["FT_ORD_UNPR3"], "145.00");
        assert_eq!(call.body["ORD_DVSN"], "00");
        assert!(call.body.get("SLL_BUY_DVSN_CD").is_none());
        assert!(call.body.get("RVSE_CNCL_DVSN_CD").is_none());
    }

    #[tokio::test]
    async fn uses_asia_reservation_tr_id_and_side_fields() {
        let call = Arc::new(Mutex::new(None));
        let client = MockClient {
            response: reservation_response(),
            call: call.clone(),
        };

        let response = sell_reservation(&client, &sample_order("SEHK"), false)
            .await
            .unwrap();

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(response.reservation_order_no, "0000123456");
        assert_eq!(call.path, PATH_ORDER_RESV);
        assert_eq!(call.tr_id, TR_ID_RESV_ASIA_REAL);
        assert_eq!(call.body["OVRS_EXCG_CD"], "SEHK");
        assert_eq!(call.body["SLL_BUY_DVSN_CD"], "01");
        assert_eq!(call.body["RVSE_CNCL_DVSN_CD"], "00");
        assert!(call.body.get("ORD_DVSN").is_none());
    }

    #[tokio::test]
    async fn uses_daytime_buy_tr_id_and_body_shape() {
        let call = Arc::new(Mutex::new(None));
        let client = MockClient {
            response: order_response(),
            call: call.clone(),
        };

        daytime_buy(&client, &sample_order("NYSE"), false)
            .await
            .unwrap();

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.path, PATH_DAYTIME_ORDER);
        assert_eq!(call.tr_id, TR_ID_DAYTIME_BUY_REAL);
        assert_eq!(call.body["OVRS_EXCG_CD"], "NYSE");
        assert_eq!(call.body["ORD_DVSN"], "00");
        assert!(call.body.get("SLL_TYPE").is_none());
    }

    #[tokio::test]
    async fn rejects_virtual_daytime_orders() {
        let client = MockClient {
            response: order_response(),
            call: Arc::new(Mutex::new(None)),
        };

        let err = daytime_sell(&client, &sample_order("AMEX"), true)
            .await
            .unwrap_err();
        assert!(
            err.to_string()
                .contains("daytime overseas orders are only supported in the real environment")
        );
    }

    #[tokio::test]
    async fn rejects_non_us_daytime_orders() {
        let client = MockClient {
            response: order_response(),
            call: Arc::new(Mutex::new(None)),
        };

        let err = daytime_buy(&client, &sample_order("SEHK"), false)
            .await
            .unwrap_err();
        assert!(
            err.to_string()
                .contains("daytime overseas orders only support U.S. exchanges")
        );
    }

    #[tokio::test]
    async fn uses_modify_tr_id_and_body_shape() {
        let call = Arc::new(Mutex::new(None));
        let client = MockClient {
            response: order_response(),
            call: call.clone(),
        };

        modify_order(&client, &sample_modify("NYSE"), false)
            .await
            .unwrap();

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.path, PATH_ORDER_RVSECNCL);
        assert_eq!(call.tr_id, TR_ID_MODIFY_REAL);
        assert_eq!(call.body["OVRS_EXCG_CD"], "NYSE");
        assert_eq!(call.body["ORGN_ODNO"], "0000123456");
        assert_eq!(call.body["RVSE_CNCL_DVSN_CD"], "01");
    }

    #[tokio::test]
    async fn uses_daytime_cancel_tr_id_and_contact_phone() {
        let call = Arc::new(Mutex::new(None));
        let client = MockClient {
            response: order_response(),
            call: call.clone(),
        };
        let mut request = sample_modify("NASD");
        request.contact_phone = "01012345678".to_string();

        daytime_cancel_order(&client, &request, false)
            .await
            .unwrap();

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.path, PATH_DAYTIME_ORDER_RVSECNCL);
        assert_eq!(call.tr_id, TR_ID_DAYTIME_CANCEL_REAL);
        assert_eq!(call.body["RVSE_CNCL_DVSN_CD"], "02");
        assert_eq!(call.body["CTAC_TLNO"], "01012345678");
    }

    #[tokio::test]
    async fn uses_virtual_cancel_tr_id() {
        let call = Arc::new(Mutex::new(None));
        let client = MockClient {
            response: order_response(),
            call: call.clone(),
        };

        cancel_order(&client, &sample_modify("AMEX"), true)
            .await
            .unwrap();

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.path, PATH_ORDER_RVSECNCL);
        assert_eq!(call.tr_id, TR_ID_CANCEL_VIRTUAL);
        assert_eq!(call.body["RVSE_CNCL_DVSN_CD"], "02");
    }

    #[tokio::test]
    async fn rejects_invalid_exchange() {
        let client = MockClient {
            response: order_response(),
            call: Arc::new(Mutex::new(None)),
        };

        let err = buy(&client, &sample_order("NAS"), false).await.unwrap_err();
        assert!(err.to_string().contains("invalid overseas order exchange"));
    }
}
