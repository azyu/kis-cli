use std::collections::BTreeMap;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::time::{Instant, timeout};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::config::AppConfig;
use crate::error::{KisError, Result};

const APPROVAL_PATH: &str = "/oauth2/Approval";
const SUBSCRIBE_TYPE: &str = "1";
const UNSUBSCRIBE_TYPE: &str = "2";
const CUSTTYPE_PERSONAL: &str = "P";

pub type RealtimeRow = BTreeMap<String, String>;

pub const DOMESTIC_ASKING_PRICE_COLUMNS: &[&str] = &[
    "mksc_shrn_iscd",
    "bsop_hour",
    "hour_cls_code",
    "askp1",
    "askp2",
    "askp3",
    "askp4",
    "askp5",
    "askp6",
    "askp7",
    "askp8",
    "askp9",
    "askp10",
    "bidp1",
    "bidp2",
    "bidp3",
    "bidp4",
    "bidp5",
    "bidp6",
    "bidp7",
    "bidp8",
    "bidp9",
    "bidp10",
    "askp_rsqn1",
    "askp_rsqn2",
    "askp_rsqn3",
    "askp_rsqn4",
    "askp_rsqn5",
    "askp_rsqn6",
    "askp_rsqn7",
    "askp_rsqn8",
    "askp_rsqn9",
    "askp_rsqn10",
    "bidp_rsqn1",
    "bidp_rsqn2",
    "bidp_rsqn3",
    "bidp_rsqn4",
    "bidp_rsqn5",
    "bidp_rsqn6",
    "bidp_rsqn7",
    "bidp_rsqn8",
    "bidp_rsqn9",
    "bidp_rsqn10",
    "total_askp_rsqn",
    "total_bidp_rsqn",
    "ovtm_total_askp_rsqn",
    "ovtm_total_bidp_rsqn",
    "antc_cnpr",
    "antc_cnqn",
    "antc_vol",
    "antc_cntg_vrss",
    "antc_cntg_vrss_sign",
    "antc_cntg_prdy_ctrt",
    "acml_vol",
    "total_askp_rsqn_icdc",
    "total_bidp_rsqn_icdc",
    "ovtm_total_askp_icdc",
    "ovtm_total_bidp_icdc",
    "stck_deal_cls_code",
];

pub const DOMESTIC_OVERTIME_ASKING_PRICE_COLUMNS: &[&str] = &[
    "mksc_shrn_iscd",
    "bsop_hour",
    "hour_cls_code",
    "askp1",
    "askp2",
    "askp3",
    "askp4",
    "askp5",
    "askp6",
    "askp7",
    "askp8",
    "askp9",
    "bidp1",
    "bidp2",
    "bidp3",
    "bidp4",
    "bidp5",
    "bidp6",
    "bidp7",
    "bidp8",
    "bidp9",
    "askp_rsqn1",
    "askp_rsqn2",
    "askp_rsqn3",
    "askp_rsqn4",
    "askp_rsqn5",
    "askp_rsqn6",
    "askp_rsqn7",
    "askp_rsqn8",
    "askp_rsqn9",
    "bidp_rsqn1",
    "bidp_rsqn2",
    "bidp_rsqn3",
    "bidp_rsqn4",
    "bidp_rsqn5",
    "bidp_rsqn6",
    "bidp_rsqn7",
    "bidp_rsqn8",
    "bidp_rsqn9",
    "total_askp_rsqn",
    "total_bidp_rsqn",
    "ovtm_total_askp_rsqn",
    "ovtm_total_bidp_rsqn",
    "antc_cnpr",
    "antc_cnqn",
    "antc_vol",
    "antc_cntg_vrss",
    "antc_cntg_vrss_sign",
    "antc_cntg_prdy_ctrt",
    "acml_vol",
    "total_askp_rsqn_icdc",
    "total_bidp_rsqn_icdc",
    "ovtm_total_askp_icdc",
    "ovtm_total_bidp_icdc",
];

pub const DOMESTIC_CCNL_COLUMNS: &[&str] = &[
    "mksc_shrn_iscd",
    "stck_cntg_hour",
    "stck_prpr",
    "prdy_vrss_sign",
    "prdy_vrss",
    "prdy_ctrt",
    "wghn_avrg_stck_prc",
    "stck_oprc",
    "stck_hgpr",
    "stck_lwpr",
    "askp1",
    "bidp1",
    "cntg_vol",
    "acml_vol",
    "acml_tr_pbmn",
    "seln_cntg_csnu",
    "shnu_cntg_csnu",
    "ntby_cntg_csnu",
    "cttr",
    "seln_cntg_smtn",
    "shnu_cntg_smtn",
    "ccld_dvsn",
    "shnu_rate",
    "prdy_vol_vrss_acml_vol_rate",
    "oprc_hour",
    "oprc_vrss_prpr_sign",
    "oprc_vrss_prpr",
    "hgpr_hour",
    "hgpr_vrss_prpr_sign",
    "hgpr_vrss_prpr",
    "lwpr_hour",
    "lwpr_vrss_prpr_sign",
    "lwpr_vrss_prpr",
    "bsop_date",
    "new_mkop_cls_code",
    "trht_yn",
    "askp_rsqn1",
    "bidp_rsqn1",
    "total_askp_rsqn",
    "total_bidp_rsqn",
    "vol_tnrt",
    "prdy_smns_hour_acml_vol",
    "prdy_smns_hour_acml_vol_rate",
    "hour_cls_code",
    "mrkt_trtm_cls_code",
    "vi_stnd_prc",
];

pub const DOMESTIC_OVERTIME_CCNL_COLUMNS: &[&str] = &[
    "mksc_shrn_iscd",
    "stck_cntg_hour",
    "stck_prpr",
    "prdy_vrss_sign",
    "prdy_vrss",
    "prdy_ctrt",
    "wghn_avrg_stck_prc",
    "stck_oprc",
    "stck_hgpr",
    "stck_lwpr",
    "askp1",
    "bidp1",
    "cntg_vol",
    "acml_vol",
    "acml_tr_pbmn",
    "seln_cntg_csnu",
    "shnu_cntg_csnu",
    "ntby_cntg_csnu",
    "cttr",
    "seln_cntg_smtn",
    "shnu_cntg_smtn",
    "cntg_cls_code",
    "shnu_rate",
    "prdy_vol_vrss_acml_vol_rate",
    "oprc_hour",
    "oprc_vrss_prpr_sign",
    "oprc_vrss_prpr",
    "hgpr_hour",
    "hgpr_vrss_prpr_sign",
    "hgpr_vrss_prpr",
    "lwpr_hour",
    "lwpr_vrss_prpr_sign",
    "lwpr_vrss_prpr",
    "bsop_date",
    "new_mkop_cls_code",
    "trht_yn",
    "askp_rsqn1",
    "bidp_rsqn1",
    "total_askp_rsqn",
    "total_bidp_rsqn",
    "vol_tnrt",
    "prdy_smns_hour_acml_vol",
    "prdy_smns_hour_acml_vol_rate",
];

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ApprovalKeyResponse {
    pub approval_key: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RealtimeSpec {
    pub tr_id: &'static str,
    pub columns: &'static [&'static str],
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RealtimePayload {
    pub tr_id: String,
    pub rows: Vec<RealtimeRow>,
}

#[derive(Debug, Serialize)]
struct ApprovalKeyRequest<'a> {
    grant_type: &'a str,
    appkey: &'a str,
    secretkey: &'a str,
}

#[derive(Debug, Serialize)]
struct ControlMessage<'a> {
    header: ControlHeader<'a>,
    body: ControlBody<'a>,
}

#[derive(Debug, Serialize)]
struct ControlHeader<'a> {
    approval_key: &'a str,
    custtype: &'a str,
    tr_type: &'a str,
    #[serde(rename = "content-type")]
    content_type: &'a str,
}

#[derive(Debug, Serialize)]
struct ControlBody<'a> {
    input: ControlInput<'a>,
}

#[derive(Debug, Serialize)]
struct ControlInput<'a> {
    tr_id: &'a str,
    tr_key: &'a str,
}

#[derive(Debug, Deserialize)]
struct SystemEnvelope {
    header: SystemHeader,
    #[serde(default)]
    body: Option<SystemBody>,
}

#[derive(Debug, Deserialize)]
struct SystemHeader {
    tr_id: String,
}

#[derive(Debug, Deserialize)]
struct SystemBody {
    rt_cd: String,
    #[serde(default)]
    msg_cd: String,
    #[serde(default)]
    msg1: String,
}

#[derive(Debug, PartialEq, Eq)]
enum ParsedMessage {
    Data(RealtimePayload),
    System,
    PingPong,
}

pub fn domestic_asking_price_spec() -> RealtimeSpec {
    RealtimeSpec {
        tr_id: "H0STASP0",
        columns: DOMESTIC_ASKING_PRICE_COLUMNS,
    }
}

pub fn domestic_overtime_asking_price_spec() -> RealtimeSpec {
    RealtimeSpec {
        tr_id: "H0STOAA0",
        columns: DOMESTIC_OVERTIME_ASKING_PRICE_COLUMNS,
    }
}

pub fn domestic_ccnl_spec() -> RealtimeSpec {
    RealtimeSpec {
        tr_id: "H0STCNT0",
        columns: DOMESTIC_CCNL_COLUMNS,
    }
}

pub fn domestic_overtime_ccnl_spec() -> RealtimeSpec {
    RealtimeSpec {
        tr_id: "H0STOUP0",
        columns: DOMESTIC_OVERTIME_CCNL_COLUMNS,
    }
}

pub fn build_control_message(
    approval_key: &str,
    tr_type: &str,
    tr_id: &str,
    tr_key: &str,
) -> Value {
    serde_json::to_value(ControlMessage {
        header: ControlHeader {
            approval_key,
            custtype: CUSTTYPE_PERSONAL,
            tr_type,
            content_type: "utf-8",
        },
        body: ControlBody {
            input: ControlInput { tr_id, tr_key },
        },
    })
    .expect("control message serialization should not fail")
}

pub async fn fetch_approval_key(config: &AppConfig) -> Result<ApprovalKeyResponse> {
    let client = Client::builder().timeout(Duration::from_secs(30)).build()?;
    fetch_approval_key_with_client(config, &client).await
}

pub async fn fetch_approval_key_with_client(
    config: &AppConfig,
    client: &Client,
) -> Result<ApprovalKeyResponse> {
    let response = client
        .post(format!(
            "{}{}",
            config.environment.base_url(),
            APPROVAL_PATH
        ))
        .header("content-type", "application/json")
        .header("accept", "text/plain")
        .json(&ApprovalKeyRequest {
            grant_type: "client_credentials",
            appkey: &config.app_key,
            secretkey: &config.app_secret,
        })
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(KisError::Parse(format!(
            "approval request failed with status {}",
            response.status()
        )));
    }

    Ok(response.json().await?)
}

pub async fn collect_realtime_messages(
    config: &AppConfig,
    spec: RealtimeSpec,
    tr_key: &str,
    max_messages: usize,
    timeout_per_connection: Duration,
    reconnect_limit: usize,
) -> Result<Vec<RealtimePayload>> {
    let approval = fetch_approval_key(config).await?;
    collect_realtime_messages_with_approval_key(
        config,
        &approval.approval_key,
        spec,
        tr_key,
        max_messages,
        timeout_per_connection,
        reconnect_limit,
    )
    .await
}

pub async fn collect_realtime_messages_with_approval_key(
    config: &AppConfig,
    approval_key: &str,
    spec: RealtimeSpec,
    tr_key: &str,
    max_messages: usize,
    timeout_per_connection: Duration,
    reconnect_limit: usize,
) -> Result<Vec<RealtimePayload>> {
    if tr_key.is_empty() {
        return Err(KisError::Config("websocket tr_key is required".to_string()));
    }
    if max_messages == 0 {
        return Ok(Vec::new());
    }

    let mut collected = Vec::new();
    let mut last_error = None;

    for _ in 0..=reconnect_limit {
        match collect_realtime_messages_once(
            config,
            approval_key,
            spec,
            tr_key,
            max_messages - collected.len(),
            timeout_per_connection,
        )
        .await
        {
            Ok(mut chunk) => {
                collected.append(&mut chunk);
                if collected.len() >= max_messages {
                    collected.truncate(max_messages);
                    return Ok(collected);
                }
            }
            Err(err) => {
                if !collected.is_empty() {
                    return Ok(collected);
                }
                last_error = Some(err);
            }
        }
    }

    if !collected.is_empty() {
        Ok(collected)
    } else {
        Err(last_error.unwrap_or_else(|| {
            KisError::Parse("websocket closed before realtime data arrived".to_string())
        }))
    }
}

async fn collect_realtime_messages_once(
    config: &AppConfig,
    approval_key: &str,
    spec: RealtimeSpec,
    tr_key: &str,
    max_messages: usize,
    timeout_per_connection: Duration,
) -> Result<Vec<RealtimePayload>> {
    let connect = timeout(
        timeout_per_connection,
        connect_async(config.environment.ws_base_url()),
    )
    .await
    .map_err(|_| KisError::Parse("websocket connect timed out".to_string()))?;
    let (mut socket, _) =
        connect.map_err(|error| KisError::Parse(format!("websocket connect failed: {error}")))?;

    let subscribe = build_control_message(approval_key, SUBSCRIBE_TYPE, spec.tr_id, tr_key);
    socket
        .send(Message::Text(subscribe.to_string()))
        .await
        .map_err(|error| KisError::Parse(format!("websocket subscribe failed: {error}")))?;

    let mut collected = Vec::new();
    let deadline = Instant::now() + timeout_per_connection;

    while collected.len() < max_messages {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }

        let next = timeout(remaining, socket.next())
            .await
            .map_err(|_| KisError::Parse("websocket receive timed out".to_string()))?;
        let Some(message) = next else {
            break;
        };
        let message = message
            .map_err(|error| KisError::Parse(format!("websocket receive failed: {error}")))?;

        match message {
            Message::Text(text) => match parse_message(&text, spec.columns)? {
                ParsedMessage::Data(payload) => collected.push(payload),
                ParsedMessage::System => {}
                ParsedMessage::PingPong => {
                    socket
                        .send(Message::Pong(Vec::new()))
                        .await
                        .map_err(|error| {
                            KisError::Parse(format!("websocket pong failed: {error}"))
                        })?;
                }
            },
            Message::Binary(bytes) => {
                let text = String::from_utf8(bytes.to_vec()).map_err(|error| {
                    KisError::Parse(format!("binary payload is not utf-8: {error}"))
                })?;
                match parse_message(&text, spec.columns)? {
                    ParsedMessage::Data(payload) => collected.push(payload),
                    ParsedMessage::System => {}
                    ParsedMessage::PingPong => {
                        socket
                            .send(Message::Pong(Vec::new()))
                            .await
                            .map_err(|error| {
                                KisError::Parse(format!("websocket pong failed: {error}"))
                            })?;
                    }
                }
            }
            Message::Ping(payload) => {
                socket
                    .send(Message::Pong(payload))
                    .await
                    .map_err(|error| KisError::Parse(format!("websocket pong failed: {error}")))?;
            }
            Message::Pong(_) => {}
            Message::Close(_) => break,
            _ => {}
        }
    }

    let unsubscribe = build_control_message(approval_key, UNSUBSCRIBE_TYPE, spec.tr_id, tr_key);
    let _ = socket.send(Message::Text(unsubscribe.to_string())).await;
    let _ = socket.close(None).await;

    Ok(collected)
}

fn parse_message(raw: &str, columns: &[&str]) -> Result<ParsedMessage> {
    match raw.as_bytes().first().copied() {
        Some(b'0') | Some(b'1') => parse_realtime_payload(raw, columns).map(ParsedMessage::Data),
        _ => parse_system_message(raw),
    }
}

fn parse_realtime_payload(raw: &str, columns: &[&str]) -> Result<RealtimePayload> {
    if columns.is_empty() {
        return Err(KisError::Parse("websocket columns are empty".to_string()));
    }

    let mut parts = raw.splitn(4, '|');
    let _kind = parts
        .next()
        .ok_or_else(|| KisError::Parse("missing realtime frame kind".to_string()))?;
    let tr_id = parts
        .next()
        .ok_or_else(|| KisError::Parse("missing realtime tr_id".to_string()))?
        .to_string();
    let count_hint = parts
        .next()
        .ok_or_else(|| KisError::Parse("missing realtime record count".to_string()))?;
    let payload = parts
        .next()
        .ok_or_else(|| KisError::Parse("missing realtime payload".to_string()))?;

    let cells = payload.split('^').collect::<Vec<_>>();
    let width = columns.len();
    if cells.len() % width != 0 {
        return Err(KisError::Parse(format!(
            "realtime payload width mismatch: {} cells for {} columns",
            cells.len(),
            width
        )));
    }

    let expected_rows = count_hint.parse::<usize>().ok();
    let row_count = cells.len() / width;
    if let Some(expected_rows) = expected_rows
        && expected_rows != row_count
    {
        return Err(KisError::Parse(format!(
            "realtime row count mismatch: header={expected_rows}, payload={row_count}"
        )));
    }

    let rows = cells
        .chunks(width)
        .map(|chunk| {
            columns
                .iter()
                .zip(chunk.iter())
                .map(|(column, value)| ((*column).to_string(), (*value).to_string()))
                .collect::<RealtimeRow>()
        })
        .collect::<Vec<_>>();

    Ok(RealtimePayload { tr_id, rows })
}

fn parse_system_message(raw: &str) -> Result<ParsedMessage> {
    let envelope: SystemEnvelope = serde_json::from_str(raw)?;
    if envelope.header.tr_id == "PINGPONG" {
        return Ok(ParsedMessage::PingPong);
    }

    if let Some(body) = envelope.body
        && body.rt_cd != "0"
    {
        return Err(KisError::Api {
            code: if body.msg_cd.is_empty() {
                "WS".to_string()
            } else {
                body.msg_cd
            },
            message: body.msg1,
        });
    }

    Ok(ParsedMessage::System)
}

#[cfg(test)]
mod tests {
    use super::{
        DOMESTIC_OVERTIME_ASKING_PRICE_COLUMNS, ParsedMessage, build_control_message,
        domestic_asking_price_spec, domestic_ccnl_spec, domestic_overtime_asking_price_spec,
        domestic_overtime_ccnl_spec, parse_message, parse_realtime_payload,
    };

    #[test]
    fn builds_control_message() {
        let message = build_control_message("approval", "1", "H0STOAA0", "005930");

        assert_eq!(message["header"]["approval_key"], "approval");
        assert_eq!(message["header"]["custtype"], "P");
        assert_eq!(message["header"]["tr_type"], "1");
        assert_eq!(message["header"]["content-type"], "utf-8");
        assert_eq!(message["body"]["input"]["tr_id"], "H0STOAA0");
        assert_eq!(message["body"]["input"]["tr_key"], "005930");
    }

    #[test]
    fn parses_realtime_payload_rows() {
        let payload = parse_realtime_payload(
            "0|H0STOUP0|2|005930^160000^70000^005930^160001^70100",
            &["mksc_shrn_iscd", "stck_cntg_hour", "stck_prpr"],
        )
        .unwrap();

        assert_eq!(payload.tr_id, "H0STOUP0");
        assert_eq!(payload.rows.len(), 2);
        assert_eq!(payload.rows[0]["mksc_shrn_iscd"], "005930");
        assert_eq!(payload.rows[1]["stck_cntg_hour"], "160001");
    }

    #[test]
    fn rejects_mismatched_realtime_row_count() {
        let err = parse_realtime_payload(
            "0|H0STOUP0|2|005930^160000^70000",
            &["mksc_shrn_iscd", "stck_cntg_hour", "stck_prpr"],
        )
        .unwrap_err();

        assert!(err.to_string().contains("row count mismatch"));
    }

    #[test]
    fn parses_system_message_and_pingpong() {
        let ok = parse_message(
            r#"{"header":{"tr_id":"H0STOAA0","tr_key":"005930"},"body":{"rt_cd":"0","msg_cd":"MCA00000","msg1":"SUBSCRIBE SUCCESS"}}"#,
            DOMESTIC_OVERTIME_ASKING_PRICE_COLUMNS,
        )
        .unwrap();
        assert_eq!(ok, ParsedMessage::System);

        let ping = parse_message(
            r#"{"header":{"tr_id":"PINGPONG"}}"#,
            DOMESTIC_OVERTIME_ASKING_PRICE_COLUMNS,
        )
        .unwrap();
        assert_eq!(ping, ParsedMessage::PingPong);
    }

    #[test]
    fn parses_system_error_as_api_error() {
        let err = parse_message(
            r#"{"header":{"tr_id":"H0STOAA0","tr_key":"005930"},"body":{"rt_cd":"1","msg_cd":"OPS","msg1":"failed"}}"#,
            DOMESTIC_OVERTIME_ASKING_PRICE_COLUMNS,
        )
        .unwrap_err();

        assert_eq!(err.to_string(), "API error [OPS]: failed");
    }

    #[test]
    fn exposes_official_overtime_specs() {
        let ask = domestic_overtime_asking_price_spec();
        let ccnl = domestic_overtime_ccnl_spec();

        assert_eq!(ask.tr_id, "H0STOAA0");
        assert_eq!(ccnl.tr_id, "H0STOUP0");
        assert_eq!(ask.columns.first().copied(), Some("mksc_shrn_iscd"));
        assert_eq!(
            ccnl.columns.last().copied(),
            Some("prdy_smns_hour_acml_vol_rate")
        );
    }

    #[test]
    fn exposes_official_regular_session_specs() {
        let ask = domestic_asking_price_spec();
        let ccnl = domestic_ccnl_spec();

        assert_eq!(ask.tr_id, "H0STASP0");
        assert_eq!(ccnl.tr_id, "H0STCNT0");
        assert_eq!(ask.columns.len(), 59);
        assert_eq!(ccnl.columns.len(), 46);
        assert_eq!(ask.columns.first().copied(), Some("mksc_shrn_iscd"));
        assert_eq!(ask.columns.last().copied(), Some("stck_deal_cls_code"));
        assert_eq!(ccnl.columns.first().copied(), Some("mksc_shrn_iscd"));
        assert_eq!(ccnl.columns.last().copied(), Some("vi_stnd_prc"));
    }
}
