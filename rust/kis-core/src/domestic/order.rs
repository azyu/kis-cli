use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::api_client::{ApiClient, parse_output, to_json_value};

const PATH_ORDER_CASH: &str = "/uapi/domestic-stock/v1/trading/order-cash";
const PATH_ORDER_RVSECNCL: &str = "/uapi/domestic-stock/v1/trading/order-rvsecncl";
const TR_ID_BUY_REAL: &str = "TTTC0012U";
const TR_ID_BUY_VIRTUAL: &str = "VTTC0012U";
const TR_ID_SELL_REAL: &str = "TTTC0011U";
const TR_ID_SELL_VIRTUAL: &str = "VTTC0011U";
const TR_ID_MODIFY_REAL: &str = "TTTC0803U";
const TR_ID_MODIFY_VIRTUAL: &str = "VTTC0803U";
const TR_ID_CANCEL_REAL: &str = "TTTC0803U";
const TR_ID_CANCEL_VIRTUAL: &str = "VTTC0803U";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct OrderRequest {
    pub account_no: String,
    pub account_prod: String,
    pub stock_code: String,
    pub order_div: String,
    pub quantity: String,
    pub price: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ModifyCancelRequest {
    pub account_no: String,
    pub account_prod: String,
    pub order_org_no: String,
    pub orig_order_no: String,
    pub order_div: String,
    pub quantity: String,
    pub price: String,
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

#[derive(Debug, Serialize)]
struct OrderBody<'a> {
    #[serde(rename = "CANO")]
    cano: &'a str,
    #[serde(rename = "ACNT_PRDT_CD")]
    acnt_prdt_cd: &'a str,
    #[serde(rename = "PDNO")]
    pdno: &'a str,
    #[serde(rename = "ORD_DVSN")]
    ord_dvsn: &'a str,
    #[serde(rename = "ORD_QTY")]
    ord_qty: &'a str,
    #[serde(rename = "ORD_UNPR")]
    ord_unpr: &'a str,
}

#[derive(Debug, Serialize)]
struct ModifyCancelBody<'a> {
    #[serde(rename = "CANO")]
    cano: &'a str,
    #[serde(rename = "ACNT_PRDT_CD")]
    acnt_prdt_cd: &'a str,
    #[serde(rename = "KRX_FWDG_ORD_ORGNO")]
    krx_fwdg_ord_orgno: &'a str,
    #[serde(rename = "ORGN_ODNO")]
    orgn_odno: &'a str,
    #[serde(rename = "ORD_DVSN")]
    ord_dvsn: &'a str,
    #[serde(rename = "RVSE_CNCL_DVSN_CD")]
    rvse_cncl_dvsn_cd: &'a str,
    #[serde(rename = "ORD_QTY")]
    ord_qty: &'a str,
    #[serde(rename = "ORD_UNPR")]
    ord_unpr: &'a str,
    #[serde(rename = "QTY_ALL_ORD_YN")]
    qty_all_ord_yn: &'a str,
}

pub async fn buy<C>(client: &C, request: &OrderRequest, is_virtual: bool) -> Result<OrderResponse>
where
    C: ApiClient + Sync,
{
    place_order(
        client,
        request,
        if is_virtual {
            TR_ID_BUY_VIRTUAL
        } else {
            TR_ID_BUY_REAL
        },
    )
    .await
}

pub async fn sell<C>(client: &C, request: &OrderRequest, is_virtual: bool) -> Result<OrderResponse>
where
    C: ApiClient + Sync,
{
    place_order(
        client,
        request,
        if is_virtual {
            TR_ID_SELL_VIRTUAL
        } else {
            TR_ID_SELL_REAL
        },
    )
    .await
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

async fn place_order<C>(client: &C, request: &OrderRequest, tr_id: &str) -> Result<OrderResponse>
where
    C: ApiClient + Sync,
{
    let body = to_json_value(OrderBody {
        cano: &request.account_no,
        acnt_prdt_cd: &request.account_prod,
        pdno: &request.stock_code,
        ord_dvsn: &request.order_div,
        ord_qty: &request.quantity,
        ord_unpr: &request.price,
    })?;
    let response = client.post_json(PATH_ORDER_CASH, tr_id, &body).await?;
    parse_output(response, "domestic order")
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
    let all_yn = if request.quantity == "0" || request.quantity.is_empty() {
        "Y"
    } else {
        "N"
    };
    let body = to_json_value(ModifyCancelBody {
        cano: &request.account_no,
        acnt_prdt_cd: &request.account_prod,
        krx_fwdg_ord_orgno: &request.order_org_no,
        orgn_odno: &request.orig_order_no,
        ord_dvsn: &request.order_div,
        rvse_cncl_dvsn_cd: dvsn,
        ord_qty: &request.quantity,
        ord_unpr: &request.price,
        qty_all_ord_yn: all_yn,
    })?;
    let response = client.post_json(PATH_ORDER_RVSECNCL, tr_id, &body).await?;
    parse_output(response, "modify/cancel order")
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

    fn sample_order() -> OrderRequest {
        OrderRequest {
            account_no: "12345678".to_string(),
            account_prod: "01".to_string(),
            stock_code: "005930".to_string(),
            order_div: "00".to_string(),
            quantity: "1".to_string(),
            price: "70000".to_string(),
        }
    }

    #[tokio::test]
    async fn uses_real_buy_tr_id() {
        let call = Arc::new(Mutex::new(None));
        let client = MockClient {
            response: order_response(),
            call: call.clone(),
        };

        let response = buy(&client, &sample_order(), false).await.unwrap();
        assert_eq!(response.order_no, "0000123456");

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.path, PATH_ORDER_CASH);
        assert_eq!(call.tr_id, TR_ID_BUY_REAL);
    }

    #[tokio::test]
    async fn uses_virtual_sell_tr_id() {
        let call = Arc::new(Mutex::new(None));
        let client = MockClient {
            response: order_response(),
            call: call.clone(),
        };

        sell(&client, &sample_order(), true).await.unwrap();

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.tr_id, TR_ID_SELL_VIRTUAL);
    }

    #[tokio::test]
    async fn marks_full_cancel_when_quantity_zero() {
        let call = Arc::new(Mutex::new(None));
        let client = MockClient {
            response: order_response(),
            call: call.clone(),
        };
        let request = ModifyCancelRequest {
            account_no: "12345678".to_string(),
            account_prod: "01".to_string(),
            order_org_no: "".to_string(),
            orig_order_no: "0000123456".to_string(),
            order_div: "00".to_string(),
            quantity: "0".to_string(),
            price: "".to_string(),
        };

        cancel_order(&client, &request, false).await.unwrap();

        let call = call.lock().unwrap().clone().unwrap();
        assert_eq!(call.tr_id, TR_ID_CANCEL_REAL);
        assert_eq!(call.body["QTY_ALL_ORD_YN"], "Y");
        assert_eq!(call.body["RVSE_CNCL_DVSN_CD"], "02");
    }
}
