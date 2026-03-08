use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter},
};

use anyhow::{Context, Result};
use kis_cli::{cli, render};
use kis_core::client::KisClient;
use kis_core::config::AppConfig;
use kis_core::domestic::{balance, chart, finance, info, market, order, overtime, price, quote};
use kis_core::error::KisError;
use kis_core::overseas::{
    balance as overseas_balance, chart as overseas_chart, exchange::OrderExchange,
    info as overseas_info, order as overseas_order, price as overseas_price,
    quote as overseas_quote,
};
use kis_core::ws as kis_ws;
use serde::Serialize;
use serde_json::Value;

struct Runtime {
    config_path: PathBuf,
    config: AppConfig,
    client: KisClient,
    command_name: &'static str,
    output_json: bool,
    quiet: bool,
}

impl Runtime {
    fn quiet_text(&self) -> bool {
        self.quiet && !self.output_json
    }
}

#[derive(Debug)]
struct ValidationError {
    message: String,
}

impl ValidationError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl StdError for ValidationError {}

#[derive(Debug, Serialize)]
struct SuccessEnvelope<'a, T> {
    ok: bool,
    command: &'a str,
    data: &'a T,
}

#[derive(Debug, Serialize)]
struct ErrorEnvelope<'a> {
    ok: bool,
    command: &'a str,
    error: ErrorOutput,
}

#[derive(Debug, Serialize)]
struct ErrorOutput {
    kind: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<String>,
}

#[derive(Debug, Serialize)]
struct DryRunOutput {
    action: &'static str,
    market: &'static str,
    route: &'static str,
    environment: String,
    endpoint: &'static str,
    tr_id: &'static str,
    request: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ClassifiedError {
    kind: &'static str,
    message: String,
    code: Option<String>,
}

#[derive(Debug, Serialize)]
struct ConfigOutput {
    config_file: String,
    environment: String,
    account_no: String,
    account_prod: String,
    base_url: String,
    ws_base_url: String,
    app_key: String,
}

#[derive(Debug, Serialize)]
struct OrderOutput {
    side: &'static str,
    order_org_no: String,
    order_no: String,
    order_time: String,
}

#[derive(Debug, Serialize)]
struct ReserveOrderOutput {
    side: &'static str,
    order_no: String,
    receipt_date: String,
    reservation_order_no: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OverseasPlaceMode {
    Regular,
    Reserve,
    Daytime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OverseasModifyMode {
    Regular,
    Daytime,
}

pub async fn run(cli: cli::Cli, writer: &mut dyn Write) -> Result<()> {
    let runtime = initialize(&cli).await?;

    match cli.command {
        cli::Command::Price(args) => run_price(&runtime, args, writer).await,
        cli::Command::Quote(args) => run_quote(&runtime, args, writer).await,
        cli::Command::Chart(args) => run_chart(&runtime, args, writer).await,
        cli::Command::Order(args) => run_order(&runtime, args, writer).await,
        cli::Command::Balance(args) => run_balance(&runtime, args, writer).await,
        cli::Command::Market(args) => run_market(&runtime, args, writer).await,
        cli::Command::Finance(args) => run_finance(&runtime, args, writer).await,
        cli::Command::Info(args) => run_info(&runtime, args, writer).await,
        cli::Command::Ws(args) => run_ws(&runtime, args, writer).await,
        cli::Command::Config => run_config(&runtime, writer),
    }
}

async fn initialize(cli: &cli::Cli) -> Result<Runtime> {
    let config_path = resolve_config_path(cli.config.as_deref())?;
    let config = kis_core::config::load(cli.config.as_deref(), cli.env.as_deref())
        .context("loading config")?;
    let client = KisClient::new(&config).context("initializing KIS client")?;

    Ok(Runtime {
        config_path,
        config,
        client,
        command_name: cli.command.name(),
        output_json: cli.output_format().is_json(),
        quiet: cli.quiet,
    })
}

fn resolve_config_path(cli_config: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = cli_config {
        return Ok(path.to_path_buf());
    }

    let home = dirs::home_dir().context("determining home directory")?;
    Ok(home.join(".config").join("kis").join("config.yaml"))
}

async fn run_price(runtime: &Runtime, args: cli::PriceArgs, writer: &mut dyn Write) -> Result<()> {
    if let Some(exchange) = args.exchange {
        let exchange = exchange.to_uppercase();
        let symbol = args.symbol.to_uppercase();
        if args.daily {
            let prices = overseas_price::get_daily_price(
                &runtime.client,
                &exchange,
                &symbol,
                Some(args.period.as_str()),
            )
            .await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &prices);
            }

            let rows = prices
                .into_iter()
                .map(|item| {
                    vec![
                        item.xymd, item.open, item.high, item.low, item.clos, item.tvol,
                    ]
                })
                .collect::<Vec<_>>();
            let output =
                render::render_table(&["날짜", "시가", "고가", "저가", "종가", "거래량"], &rows);
            writeln!(writer, "{output}")?;
            return Ok(());
        }

        let price = overseas_price::get_price(&runtime.client, &exchange, &symbol).await?;
        if runtime.output_json {
            return write_command_json(writer, runtime.command_name, &price);
        }
        let output = render::render_pairs(&[
            ("종목명", price.name),
            ("현재가", price.last),
            ("전일대비", format!("{} ({}%)", price.diff, price.rate)),
            ("전일종가", price.base),
            ("시가", price.open),
            ("고가", price.high),
            ("저가", price.low),
            ("거래량", price.tvol),
            ("거래대금", price.tamt),
            ("매수가능", price.ordy),
        ]);
        writeln!(writer, "{output}")?;
        return Ok(());
    }

    if args.daily {
        let prices =
            price::get_daily_price(&runtime.client, &args.symbol, Some(args.period.as_str()))
                .await?;
        if runtime.output_json {
            return write_command_json(writer, runtime.command_name, &prices);
        }
        let rows = prices
            .into_iter()
            .map(|item| {
                vec![
                    item.stck_bsop_date,
                    item.stck_oprc,
                    item.stck_hgpr,
                    item.stck_lwpr,
                    item.stck_clpr,
                    item.acml_vol,
                    format!("{} {}", price_sign(&item.prdy_vrss_sign), item.prdy_vrss),
                ]
            })
            .collect::<Vec<_>>();
        let output = render::render_table(
            &["날짜", "시가", "고가", "저가", "종가", "거래량", "전일대비"],
            &rows,
        );
        writeln!(writer, "{output}")?;
        return Ok(());
    }

    let price = price::get_price(&runtime.client, &args.symbol).await?;
    if runtime.output_json {
        return write_command_json(writer, runtime.command_name, &price);
    }
    let output = render::render_pairs(&[
        ("종목명", price.hts_kor_isnm),
        ("현재가", price.stck_prpr),
        (
            "전일대비",
            format!(
                "{} {} ({}%)",
                price_sign(&price.prdy_vrss_sign),
                price.prdy_vrss,
                price.prdy_ctrt
            ),
        ),
        ("시가", price.stck_oprc),
        ("고가", price.stck_hgpr),
        ("저가", price.stck_lwpr),
        ("거래량", price.acml_vol),
        ("거래대금", price.acml_tr_pbmn),
    ]);
    writeln!(writer, "{output}")?;
    Ok(())
}

async fn run_quote(runtime: &Runtime, args: cli::QuoteArgs, writer: &mut dyn Write) -> Result<()> {
    match args.command {
        cli::QuoteCommand::Ask(args) => {
            if let Some(exchange) = args.exchange.as_deref() {
                let ask = overseas_quote::get_asking_price(&runtime.client, exchange, &args.stock)
                    .await?;
                if runtime.output_json {
                    return write_command_json(writer, runtime.command_name, &ask);
                }
                write_overseas_asking_price(writer, &ask)?;
                return Ok(());
            }

            let ask = quote::get_asking_price(&runtime.client, &args.stock).await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &ask);
            }

            let rows = [
                vec![ask.askp_rsqn5, ask.askp5, ask.bidp5, ask.bidp_rsqn5],
                vec![ask.askp_rsqn4, ask.askp4, ask.bidp4, ask.bidp_rsqn4],
                vec![ask.askp_rsqn3, ask.askp3, ask.bidp3, ask.bidp_rsqn3],
                vec![ask.askp_rsqn2, ask.askp2, ask.bidp2, ask.bidp_rsqn2],
                vec![ask.askp_rsqn1, ask.askp1, ask.bidp1, ask.bidp_rsqn1],
            ];
            let table =
                render::render_table(&["매도잔량", "매도호가", "매수호가", "매수잔량"], &rows);
            writeln!(writer, "{table}")?;
            writeln!(
                writer,
                "\n총매도잔량: {}  총매수잔량: {}",
                ask.total_askp_rsqn, ask.total_bidp_rsqn
            )?;
        }
        cli::QuoteCommand::OvertimePrice(args) => {
            let item = overtime::get_overtime_price(&runtime.client, "J", &args.stock).await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &item);
            }

            let output = render::render_pairs(&[
                ("종목명", display_or_dash(&item.bstp_kor_isnm)),
                ("현재가", display_or_dash(&item.ovtm_untp_prpr)),
                (
                    "전일대비",
                    format!(
                        "{} {} ({}%)",
                        price_sign(&item.ovtm_untp_prdy_vrss_sign),
                        display_or_dash(&item.ovtm_untp_prdy_vrss),
                        display_or_dash(&item.ovtm_untp_prdy_ctrt)
                    ),
                ),
                ("시가", display_or_dash(&item.ovtm_untp_oprc)),
                ("고가", display_or_dash(&item.ovtm_untp_hgpr)),
                ("저가", display_or_dash(&item.ovtm_untp_lwpr)),
                ("거래량", display_or_dash(&item.ovtm_untp_vol)),
                ("거래대금", display_or_dash(&item.ovtm_untp_tr_pbmn)),
                ("예상체결가", display_or_dash(&item.ovtm_untp_antc_cnpr)),
                ("예상체결량", display_or_dash(&item.ovtm_untp_antc_cnqn)),
                ("매도호가", display_or_dash(&item.askp)),
                ("매수호가", display_or_dash(&item.bidp)),
            ]);
            writeln!(writer, "{output}")?;
        }
        cli::QuoteCommand::OvertimeAsk(args) => {
            let ask =
                overtime::get_overtime_asking_price(&runtime.client, "J", &args.stock).await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &ask);
            }

            let rows = [
                vec![
                    ask.ovtm_untp_askp_rsqn5,
                    ask.ovtm_untp_askp5,
                    ask.ovtm_untp_bidp5,
                    ask.ovtm_untp_bidp_rsqn5,
                ],
                vec![
                    ask.ovtm_untp_askp_rsqn4,
                    ask.ovtm_untp_askp4,
                    ask.ovtm_untp_bidp4,
                    ask.ovtm_untp_bidp_rsqn4,
                ],
                vec![
                    ask.ovtm_untp_askp_rsqn3,
                    ask.ovtm_untp_askp3,
                    ask.ovtm_untp_bidp3,
                    ask.ovtm_untp_bidp_rsqn3,
                ],
                vec![
                    ask.ovtm_untp_askp_rsqn2,
                    ask.ovtm_untp_askp2,
                    ask.ovtm_untp_bidp2,
                    ask.ovtm_untp_bidp_rsqn2,
                ],
                vec![
                    ask.ovtm_untp_askp_rsqn1,
                    ask.ovtm_untp_askp1,
                    ask.ovtm_untp_bidp1,
                    ask.ovtm_untp_bidp_rsqn1,
                ],
            ];
            let table =
                render::render_table(&["매도잔량", "매도호가", "매수호가", "매수잔량"], &rows);
            writeln!(writer, "{table}")?;
            writeln!(
                writer,
                "\n시간: {}  총매도잔량: {}  총매수잔량: {}",
                display_or_dash(&ask.ovtm_untp_last_hour),
                display_or_dash(&ask.ovtm_untp_total_askp_rsqn),
                display_or_dash(&ask.ovtm_untp_total_bidp_rsqn),
            )?;
        }
        cli::QuoteCommand::Ccnl(args) => {
            if let Some(exchange) = args.exchange.as_deref() {
                let items =
                    overseas_quote::get_conclusions(&runtime.client, exchange, &args.stock).await?;
                if runtime.output_json {
                    return write_command_json(writer, runtime.command_name, &items);
                }
                let rows = items
                    .into_iter()
                    .map(|item| {
                        vec![
                            display_or_dash(&item.xhms),
                            display_or_dash(&item.last),
                            format!(
                                "{} {} ({}%)",
                                price_sign(&item.sign),
                                display_or_dash(&item.diff),
                                display_or_dash(&item.rate)
                            ),
                            display_or_dash(&item.evol),
                            display_or_dash(&item.tvol),
                        ]
                    })
                    .collect::<Vec<_>>();
                let table = render::render_table(
                    &["시각", "현재가", "전일대비", "체결량", "누적거래량"],
                    &rows,
                );
                writeln!(writer, "{table}")?;
                return Ok(());
            }

            let items = quote::get_conclusions(&runtime.client, &args.stock).await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &items);
            }

            let rows = items
                .into_iter()
                .map(|item| {
                    vec![
                        item.stck_cntg_hour,
                        item.stck_prpr,
                        format!("{} {}", price_sign(&item.prdy_vrss_sign), item.prdy_vrss),
                        item.cntg_vol,
                        item.tday_rltv,
                    ]
                })
                .collect::<Vec<_>>();
            let table =
                render::render_table(&["시각", "현재가", "전일대비", "체결량", "체결강도"], &rows);
            writeln!(writer, "{table}")?;
        }
        cli::QuoteCommand::Investor(args) => {
            let items = quote::get_investors(&runtime.client, &args.stock).await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &items);
            }

            let rows = items
                .into_iter()
                .map(|item| vec![item.invr_nm, item.seln_vol, item.shnu_vol, item.ntby_qty])
                .collect::<Vec<_>>();
            let table = render::render_table(&["투자자", "매도수량", "매수수량", "순매수"], &rows);
            writeln!(writer, "{table}")?;
        }
        cli::QuoteCommand::Member(args) => {
            let items = quote::get_members(&runtime.client, &args.stock).await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &items);
            }

            let rows = items
                .into_iter()
                .map(|item| vec![item.memb_nm, item.seln_vol, item.shnu_vol, item.ntby_qty])
                .collect::<Vec<_>>();
            let table = render::render_table(&["회원사", "매도수량", "매수수량", "순매수"], &rows);
            writeln!(writer, "{table}")?;
        }
    }

    Ok(())
}

fn write_overseas_asking_price(
    writer: &mut dyn Write,
    ask: &overseas_quote::OverseasAskingPrice,
) -> Result<()> {
    let Some(levels) = ask.levels.first() else {
        let output = render::render_pairs(&[
            ("현재가", display_or_dash(&ask.quote.last)),
            ("매도호가", display_or_dash(&ask.quote.pask)),
            ("매수호가", display_or_dash(&ask.quote.pbid)),
            ("거래량", display_or_dash(&ask.quote.tvol)),
        ]);
        writeln!(writer, "{output}")?;
        return Ok(());
    };

    let rows = [
        vec![
            display_or_dash(&levels.askv5),
            display_or_dash(&levels.askp5),
            display_or_dash(&levels.bidp5),
            display_or_dash(&levels.bidv5),
        ],
        vec![
            display_or_dash(&levels.askv4),
            display_or_dash(&levels.askp4),
            display_or_dash(&levels.bidp4),
            display_or_dash(&levels.bidv4),
        ],
        vec![
            display_or_dash(&levels.askv3),
            display_or_dash(&levels.askp3),
            display_or_dash(&levels.bidp3),
            display_or_dash(&levels.bidv3),
        ],
        vec![
            display_or_dash(&levels.askv2),
            display_or_dash(&levels.askp2),
            display_or_dash(&levels.bidp2),
            display_or_dash(&levels.bidv2),
        ],
        vec![
            display_or_dash(&levels.askv1),
            display_or_dash(&levels.askp1),
            display_or_dash(&levels.bidp1),
            display_or_dash(&levels.bidv1),
        ],
    ];
    let table = render::render_table(&["매도잔량", "매도호가", "매수호가", "매수잔량"], &rows);
    writeln!(writer, "{table}")?;
    writeln!(
        writer,
        "\n현재가: {}  총매도잔량: {}  총매수잔량: {}",
        display_or_dash(&ask.quote.last),
        display_or_dash(&ask.summary.total_askp_rsqn),
        display_or_dash(&ask.summary.total_bidp_rsqn),
    )?;
    Ok(())
}

async fn run_chart(runtime: &Runtime, args: cli::ChartArgs, writer: &mut dyn Write) -> Result<()> {
    match args.command {
        cli::ChartCommand::Daily(args) => {
            if let Some(exchange) = args.exchange {
                let items = overseas_chart::get_daily_chart(
                    &runtime.client,
                    &exchange,
                    &args.stock,
                    args.start.as_deref().unwrap_or(""),
                    args.end.as_deref().unwrap_or(""),
                    Some(args.period.as_str()),
                )
                .await?;
                if runtime.output_json {
                    return write_command_json(writer, runtime.command_name, &items);
                }

                let rows = items
                    .into_iter()
                    .map(|item| {
                        vec![
                            item.xymd, item.open, item.high, item.low, item.clos, item.tvol,
                        ]
                    })
                    .collect::<Vec<_>>();
                let table = render::render_table(
                    &["날짜", "시가", "고가", "저가", "종가", "거래량"],
                    &rows,
                );
                writeln!(writer, "{table}")?;
                return Ok(());
            }

            let items = chart::get_daily_chart(
                &runtime.client,
                &args.stock,
                args.start.as_deref().unwrap_or(""),
                args.end.as_deref().unwrap_or(""),
                Some(args.period.as_str()),
            )
            .await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &items);
            }

            let rows = items
                .into_iter()
                .map(|item| {
                    vec![
                        item.stck_bsop_date,
                        item.stck_oprc,
                        item.stck_hgpr,
                        item.stck_lwpr,
                        item.stck_clpr,
                        item.acml_vol,
                        format!("{} {}", price_sign(&item.prdy_vrss_sign), item.prdy_vrss),
                    ]
                })
                .collect::<Vec<_>>();
            let table = render::render_table(
                &["날짜", "시가", "고가", "저가", "종가", "거래량", "전일대비"],
                &rows,
            );
            writeln!(writer, "{table}")?;
        }
        cli::ChartCommand::Time(args) => {
            if let Some(exchange) = args.exchange {
                let items = overseas_chart::get_time_chart(
                    &runtime.client,
                    &exchange,
                    &args.stock,
                    Some(args.unit.as_str()),
                )
                .await?;
                if runtime.output_json {
                    return write_command_json(writer, runtime.command_name, &items);
                }

                let rows = items
                    .into_iter()
                    .map(|item| {
                        vec![
                            item.xymd, item.xhms, item.last, item.open, item.high, item.low,
                            item.evol, item.tvol,
                        ]
                    })
                    .collect::<Vec<_>>();
                let table = render::render_table(
                    &[
                        "일자",
                        "시각",
                        "현재가",
                        "시가",
                        "고가",
                        "저가",
                        "체결량",
                        "누적거래량",
                    ],
                    &rows,
                );
                writeln!(writer, "{table}")?;
                return Ok(());
            }

            let items =
                chart::get_time_chart(&runtime.client, &args.stock, Some(args.unit.as_str()))
                    .await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &items);
            }

            let rows = items
                .into_iter()
                .map(|item| {
                    vec![
                        item.stck_cntg_hour,
                        item.stck_prpr,
                        item.stck_oprc,
                        item.stck_hgpr,
                        item.stck_lwpr,
                        item.cntg_vol,
                        item.acml_vol,
                    ]
                })
                .collect::<Vec<_>>();
            let table = render::render_table(
                &[
                    "시각",
                    "현재가",
                    "시가",
                    "고가",
                    "저가",
                    "체결량",
                    "누적거래량",
                ],
                &rows,
            );
            writeln!(writer, "{table}")?;
        }
        cli::ChartCommand::Index(args) => {
            let items = chart::get_daily_index_chart(
                &runtime.client,
                &args.index,
                args.start.as_deref().unwrap_or(""),
                args.end.as_deref().unwrap_or(""),
                Some(args.period.as_str()),
            )
            .await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &items);
            }

            let rows = items
                .into_iter()
                .map(|item| {
                    vec![
                        item.stck_bsop_date,
                        item.bstp_nmix_oprc,
                        item.bstp_nmix_hgpr,
                        item.bstp_nmix_lwpr,
                        item.bstp_nmix_prpr,
                        item.acml_vol,
                    ]
                })
                .collect::<Vec<_>>();
            let table =
                render::render_table(&["날짜", "시가", "고가", "저가", "종가", "거래량"], &rows);
            writeln!(writer, "{table}")?;
        }
        cli::ChartCommand::IndexPrice(args) => {
            let item = chart::get_index_price(&runtime.client, &args.index).await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &item);
            }

            let output = render::render_pairs(&[
                ("지수", item.bstp_nmix_prpr),
                (
                    "전일대비",
                    format!(
                        "{} {} ({}%)",
                        price_sign(&item.prdy_vrss_sign),
                        item.bstp_nmix_prdy_vrss,
                        item.bstp_nmix_prdy_ctrt
                    ),
                ),
                ("시가", item.bstp_nmix_oprc),
                ("고가", item.bstp_nmix_hgpr),
                ("저가", item.bstp_nmix_lwpr),
                ("거래량", item.acml_vol),
            ]);
            writeln!(writer, "{output}")?;
        }
    }

    Ok(())
}

async fn run_order(runtime: &Runtime, args: cli::OrderArgs, writer: &mut dyn Write) -> Result<()> {
    let is_virtual = runtime.config.environment.is_virtual();

    match args.command {
        cli::OrderCommand::Buy(args) => {
            let dry_run = args.dry_run;
            if let Some(mode) = overseas_place_mode(
                args.exchange.as_deref(),
                args.reserve,
                args.daytime,
                is_virtual,
            )? {
                let request = build_overseas_place_order_request(&runtime.config, args)?;
                if dry_run {
                    return write_order_dry_run(
                        runtime,
                        writer,
                        overseas_place_dry_run("buy", mode, is_virtual, &request)?,
                    );
                }
                match mode {
                    OverseasPlaceMode::Regular => {
                        let result =
                            overseas_order::buy(&runtime.client, &request, is_virtual).await?;
                        write_order_completion(
                            runtime,
                            writer,
                            "buy",
                            "매수",
                            result.order_org_no,
                            result.order_no,
                            result.order_time,
                        )?;
                    }
                    OverseasPlaceMode::Reserve => {
                        let result =
                            overseas_order::buy_reservation(&runtime.client, &request, is_virtual)
                                .await?;
                        write_overseas_reserve_result("buy", "매수", &result, runtime, writer)?;
                    }
                    OverseasPlaceMode::Daytime => {
                        let result =
                            overseas_order::daytime_buy(&runtime.client, &request, is_virtual)
                                .await?;
                        write_order_completion(
                            runtime,
                            writer,
                            "buy",
                            "매수",
                            result.order_org_no,
                            result.order_no,
                            result.order_time,
                        )?;
                    }
                }
            } else {
                let request = build_place_order_request(&runtime.config, args)?;
                if dry_run {
                    return write_order_dry_run(
                        runtime,
                        writer,
                        domestic_place_dry_run("buy", is_virtual, &request)?,
                    );
                }
                let result = order::buy(&runtime.client, &request, is_virtual).await?;
                write_order_completion(
                    runtime,
                    writer,
                    "buy",
                    "매수",
                    result.order_org_no,
                    result.order_no,
                    result.order_time,
                )?;
            }
        }
        cli::OrderCommand::Sell(args) => {
            let dry_run = args.dry_run;
            if let Some(mode) = overseas_place_mode(
                args.exchange.as_deref(),
                args.reserve,
                args.daytime,
                is_virtual,
            )? {
                let request = build_overseas_place_order_request(&runtime.config, args)?;
                if dry_run {
                    return write_order_dry_run(
                        runtime,
                        writer,
                        overseas_place_dry_run("sell", mode, is_virtual, &request)?,
                    );
                }
                match mode {
                    OverseasPlaceMode::Regular => {
                        let result =
                            overseas_order::sell(&runtime.client, &request, is_virtual).await?;
                        write_order_completion(
                            runtime,
                            writer,
                            "sell",
                            "매도",
                            result.order_org_no,
                            result.order_no,
                            result.order_time,
                        )?;
                    }
                    OverseasPlaceMode::Reserve => {
                        let result =
                            overseas_order::sell_reservation(&runtime.client, &request, is_virtual)
                                .await?;
                        write_overseas_reserve_result("sell", "매도", &result, runtime, writer)?;
                    }
                    OverseasPlaceMode::Daytime => {
                        let result =
                            overseas_order::daytime_sell(&runtime.client, &request, is_virtual)
                                .await?;
                        write_order_completion(
                            runtime,
                            writer,
                            "sell",
                            "매도",
                            result.order_org_no,
                            result.order_no,
                            result.order_time,
                        )?;
                    }
                }
            } else {
                let request = build_place_order_request(&runtime.config, args)?;
                if dry_run {
                    return write_order_dry_run(
                        runtime,
                        writer,
                        domestic_place_dry_run("sell", is_virtual, &request)?,
                    );
                }
                let result = order::sell(&runtime.client, &request, is_virtual).await?;
                write_order_completion(
                    runtime,
                    writer,
                    "sell",
                    "매도",
                    result.order_org_no,
                    result.order_no,
                    result.order_time,
                )?;
            }
        }
        cli::OrderCommand::Modify(args) => {
            let dry_run = args.dry_run;
            if let Some(mode) =
                overseas_modify_mode(args.exchange.as_deref(), args.daytime, is_virtual)?
            {
                let request = build_overseas_modify_order_request(&runtime.config, args)?;
                if dry_run {
                    return write_order_dry_run(
                        runtime,
                        writer,
                        overseas_modify_dry_run("modify", mode, is_virtual, &request)?,
                    );
                }
                match mode {
                    OverseasModifyMode::Regular => {
                        let result =
                            overseas_order::modify_order(&runtime.client, &request, is_virtual)
                                .await?;
                        write_order_completion(
                            runtime,
                            writer,
                            "modify",
                            "정정",
                            result.order_org_no,
                            result.order_no,
                            result.order_time,
                        )?;
                    }
                    OverseasModifyMode::Daytime => {
                        let result = overseas_order::daytime_modify_order(
                            &runtime.client,
                            &request,
                            is_virtual,
                        )
                        .await?;
                        write_order_completion(
                            runtime,
                            writer,
                            "modify",
                            "정정",
                            result.order_org_no,
                            result.order_no,
                            result.order_time,
                        )?;
                    }
                }
            } else {
                let request = order::ModifyCancelRequest {
                    account_no: runtime.config.account_no.clone(),
                    account_prod: runtime.config.account_prod.clone(),
                    order_org_no: args.org_no.unwrap_or_default(),
                    orig_order_no: args.order_no,
                    order_div: args.order_type,
                    quantity: args.qty,
                    price: args.price,
                };
                if dry_run {
                    return write_order_dry_run(
                        runtime,
                        writer,
                        domestic_modify_dry_run("modify", is_virtual, &request)?,
                    );
                }
                let result = order::modify_order(&runtime.client, &request, is_virtual).await?;
                write_order_completion(
                    runtime,
                    writer,
                    "modify",
                    "정정",
                    result.order_org_no,
                    result.order_no,
                    result.order_time,
                )?;
            }
        }
        cli::OrderCommand::Cancel(args) => {
            let dry_run = args.dry_run;
            if let Some(mode) =
                overseas_modify_mode(args.exchange.as_deref(), args.daytime, is_virtual)?
            {
                let request = build_overseas_cancel_order_request(&runtime.config, args)?;
                if dry_run {
                    return write_order_dry_run(
                        runtime,
                        writer,
                        overseas_modify_dry_run("cancel", mode, is_virtual, &request)?,
                    );
                }
                match mode {
                    OverseasModifyMode::Regular => {
                        let result =
                            overseas_order::cancel_order(&runtime.client, &request, is_virtual)
                                .await?;
                        write_order_completion(
                            runtime,
                            writer,
                            "cancel",
                            "취소",
                            result.order_org_no,
                            result.order_no,
                            result.order_time,
                        )?;
                    }
                    OverseasModifyMode::Daytime => {
                        let result = overseas_order::daytime_cancel_order(
                            &runtime.client,
                            &request,
                            is_virtual,
                        )
                        .await?;
                        write_order_completion(
                            runtime,
                            writer,
                            "cancel",
                            "취소",
                            result.order_org_no,
                            result.order_no,
                            result.order_time,
                        )?;
                    }
                }
            } else {
                let request = order::ModifyCancelRequest {
                    account_no: runtime.config.account_no.clone(),
                    account_prod: runtime.config.account_prod.clone(),
                    order_org_no: args.org_no.unwrap_or_default(),
                    orig_order_no: args.order_no,
                    order_div: String::new(),
                    quantity: args.qty,
                    price: String::new(),
                };
                if dry_run {
                    return write_order_dry_run(
                        runtime,
                        writer,
                        domestic_modify_dry_run("cancel", is_virtual, &request)?,
                    );
                }
                let result = order::cancel_order(&runtime.client, &request, is_virtual).await?;
                write_order_completion(
                    runtime,
                    writer,
                    "cancel",
                    "취소",
                    result.order_org_no,
                    result.order_no,
                    result.order_time,
                )?;
            }
        }
        cli::OrderCommand::ReserveCancel(args) => {
            let request = overseas_balance::ReservationCancelRequest {
                account_no: runtime.config.account_no.clone(),
                account_prod: runtime.config.account_prod.clone(),
                receipt_date: args.receipt_date,
                reservation_order_no: args.reservation_order_no,
            };
            let result = overseas_balance::cancel_reservation_order(
                &runtime.client,
                args.region.code(),
                &request,
                is_virtual,
            )
            .await?;
            let value = serde_json::to_value(&result)?;
            if runtime.output_json {
                return write_command_json(
                    writer,
                    runtime.command_name,
                    &reserve_order_output("cancel", &value),
                );
            }
            write_reserve_cancel_result(&value, runtime.quiet_text(), writer)?;
        }
    }

    Ok(())
}

fn write_order_completion(
    runtime: &Runtime,
    writer: &mut dyn Write,
    side_json: &'static str,
    side_text: &str,
    order_org_no: String,
    order_no: String,
    order_time: String,
) -> Result<()> {
    if runtime.output_json {
        return write_command_json(
            writer,
            runtime.command_name,
            &order_output(side_json, order_org_no, order_no, order_time),
        );
    }

    write_order_result(
        side_text,
        order_org_no,
        order_no,
        order_time,
        runtime.quiet_text(),
        writer,
    )
}

fn write_order_dry_run(
    runtime: &Runtime,
    writer: &mut dyn Write,
    output: DryRunOutput,
) -> Result<()> {
    if runtime.output_json {
        return write_command_json(writer, runtime.command_name, &output);
    }

    let summary = render::render_pairs(&[
        ("동작", output.action.to_string()),
        ("시장", output.market.to_string()),
        ("모드", output.route.to_string()),
        ("환경", output.environment),
        ("엔드포인트", output.endpoint.to_string()),
        ("TR ID", output.tr_id.to_string()),
    ]);

    if !runtime.quiet_text() {
        writeln!(writer, "dry-run")?;
    }
    writeln!(writer, "{summary}")?;
    writeln!(writer)?;
    if !runtime.quiet_text() {
        writeln!(writer, "요청")?;
    }
    writeln!(writer, "{}", serde_json::to_string_pretty(&output.request)?)?;
    Ok(())
}

fn domestic_place_dry_run(
    action: &'static str,
    is_virtual: bool,
    request: &order::OrderRequest,
) -> Result<DryRunOutput> {
    Ok(DryRunOutput {
        action,
        market: "domestic",
        route: "regular",
        environment: environment_name(is_virtual).to_string(),
        endpoint: "/uapi/domestic-stock/v1/trading/order-cash",
        tr_id: match (action, is_virtual) {
            ("buy", false) => "TTTC0012U",
            ("buy", true) => "VTTC0012U",
            ("sell", false) => "TTTC0011U",
            ("sell", true) => "VTTC0011U",
            _ => unreachable!("unsupported domestic place action"),
        },
        request: serde_json::to_value(request)?,
    })
}

fn domestic_modify_dry_run(
    action: &'static str,
    is_virtual: bool,
    request: &order::ModifyCancelRequest,
) -> Result<DryRunOutput> {
    Ok(DryRunOutput {
        action,
        market: "domestic",
        route: "regular",
        environment: environment_name(is_virtual).to_string(),
        endpoint: "/uapi/domestic-stock/v1/trading/order-rvsecncl",
        tr_id: match (action, is_virtual) {
            ("modify", false) | ("cancel", false) => "TTTC0803U",
            ("modify", true) | ("cancel", true) => "VTTC0803U",
            _ => unreachable!("unsupported domestic modify action"),
        },
        request: serde_json::to_value(request)?,
    })
}

fn overseas_place_dry_run(
    action: &'static str,
    mode: OverseasPlaceMode,
    is_virtual: bool,
    request: &overseas_order::OrderRequest,
) -> Result<DryRunOutput> {
    let exchange = parse_order_exchange(&request.exchange_code)?;
    let (route, endpoint, tr_id) = match mode {
        OverseasPlaceMode::Regular => (
            "regular",
            "/uapi/overseas-stock/v1/trading/order",
            match action {
                "buy" => exchange.buy_tr_id(is_virtual),
                "sell" => exchange.sell_tr_id(is_virtual),
                _ => unreachable!("unsupported overseas place action"),
            },
        ),
        OverseasPlaceMode::Reserve => (
            "reserve",
            "/uapi/overseas-stock/v1/trading/order-resv",
            match action {
                "buy" => overseas_reservation_buy_tr_id(exchange, is_virtual),
                "sell" => overseas_reservation_sell_tr_id(exchange, is_virtual),
                _ => unreachable!("unsupported overseas reserve action"),
            },
        ),
        OverseasPlaceMode::Daytime => (
            "daytime",
            "/uapi/overseas-stock/v1/trading/daytime-order",
            match action {
                "buy" => "TTTS6036U",
                "sell" => "TTTS6037U",
                _ => unreachable!("unsupported overseas daytime action"),
            },
        ),
    };

    Ok(DryRunOutput {
        action,
        market: "overseas",
        route,
        environment: environment_name(is_virtual).to_string(),
        endpoint,
        tr_id,
        request: serde_json::to_value(request)?,
    })
}

fn overseas_modify_dry_run(
    action: &'static str,
    mode: OverseasModifyMode,
    is_virtual: bool,
    request: &overseas_order::ModifyCancelRequest,
) -> Result<DryRunOutput> {
    let _ = parse_order_exchange(&request.exchange_code)?;
    let (route, endpoint, tr_id) = match mode {
        OverseasModifyMode::Regular => (
            "regular",
            "/uapi/overseas-stock/v1/trading/order-rvsecncl",
            if is_virtual { "VTTT1004U" } else { "TTTT1004U" },
        ),
        OverseasModifyMode::Daytime => (
            "daytime",
            "/uapi/overseas-stock/v1/trading/daytime-order-rvsecncl",
            "TTTS6038U",
        ),
    };

    Ok(DryRunOutput {
        action,
        market: "overseas",
        route,
        environment: environment_name(is_virtual).to_string(),
        endpoint,
        tr_id,
        request: serde_json::to_value(request)?,
    })
}

fn environment_name(is_virtual: bool) -> &'static str {
    if is_virtual { "virtual" } else { "real" }
}

fn overseas_reservation_buy_tr_id(exchange: OrderExchange, is_virtual: bool) -> &'static str {
    match (exchange.is_us(), is_virtual) {
        (true, false) => "TTTT3014U",
        (true, true) => "VTTT3014U",
        (false, false) => "TTTS3013U",
        (false, true) => "VTTS3013U",
    }
}

fn overseas_reservation_sell_tr_id(exchange: OrderExchange, is_virtual: bool) -> &'static str {
    match (exchange.is_us(), is_virtual) {
        (true, false) => "TTTT3016U",
        (true, true) => "VTTT3016U",
        (false, false) => "TTTS3013U",
        (false, true) => "VTTS3013U",
    }
}

fn build_place_order_request(
    config: &AppConfig,
    args: cli::PlaceOrderArgs,
) -> Result<order::OrderRequest> {
    let (order_div, price) = if args.market {
        ("01".to_string(), "0".to_string())
    } else if let Some(price) = args.price {
        (args.order_type, price)
    } else {
        return Err(validation_error(
            "--price is required for limit orders (or use --market)",
        ));
    };

    Ok(order::OrderRequest {
        account_no: config.account_no.clone(),
        account_prod: config.account_prod.clone(),
        stock_code: args.stock,
        order_div,
        quantity: args.qty,
        price,
    })
}

fn build_overseas_place_order_request(
    config: &AppConfig,
    args: cli::PlaceOrderArgs,
) -> Result<overseas_order::OrderRequest> {
    if args.market {
        return Err(validation_error(
            "--market is not supported for overseas orders; pass --price and --order-type explicitly",
        ));
    }

    let exchange = normalize_order_exchange(
        args.exchange
            .ok_or_else(|| validation_error("--exchange is required for overseas orders"))?,
    )?;
    let price = args
        .price
        .ok_or_else(|| validation_error("--price is required for overseas orders"))?;

    Ok(overseas_order::OrderRequest {
        account_no: config.account_no.clone(),
        account_prod: config.account_prod.clone(),
        exchange_code: exchange,
        stock_code: args.stock,
        quantity: args.qty,
        price,
        order_div: args.order_type,
        contact_phone: String::new(),
        management_order_no: String::new(),
        order_server_div: "0".to_string(),
    })
}

fn build_overseas_modify_order_request(
    config: &AppConfig,
    args: cli::ModifyOrderArgs,
) -> Result<overseas_order::ModifyCancelRequest> {
    let (exchange, stock_code) = overseas_order_identity(args.exchange, args.stock)?;

    Ok(overseas_order::ModifyCancelRequest {
        account_no: config.account_no.clone(),
        account_prod: config.account_prod.clone(),
        exchange_code: exchange,
        stock_code,
        orig_order_no: args.order_no,
        quantity: args.qty,
        price: args.price,
        contact_phone: String::new(),
        management_order_no: String::new(),
        order_server_div: "0".to_string(),
    })
}

fn build_overseas_cancel_order_request(
    config: &AppConfig,
    args: cli::CancelOrderArgs,
) -> Result<overseas_order::ModifyCancelRequest> {
    let (exchange, stock_code) = overseas_order_identity(args.exchange, args.stock)?;

    Ok(overseas_order::ModifyCancelRequest {
        account_no: config.account_no.clone(),
        account_prod: config.account_prod.clone(),
        exchange_code: exchange,
        stock_code,
        orig_order_no: args.order_no,
        quantity: args.qty,
        price: "0".to_string(),
        contact_phone: String::new(),
        management_order_no: String::new(),
        order_server_div: "0".to_string(),
    })
}

fn overseas_order_identity(
    exchange: Option<String>,
    stock: Option<String>,
) -> Result<(String, String)> {
    Ok((
        normalize_order_exchange(exchange.ok_or_else(|| {
            validation_error("--exchange is required for overseas modify/cancel orders")
        })?)?,
        stock.ok_or_else(|| {
            validation_error("--stock is required for overseas modify/cancel orders")
        })?,
    ))
}

fn overseas_place_mode(
    exchange: Option<&str>,
    reserve: bool,
    daytime: bool,
    is_virtual: bool,
) -> Result<Option<OverseasPlaceMode>> {
    if exchange.is_none() {
        if reserve {
            return Err(validation_error(
                "--reserve is only available for overseas orders; pass --exchange",
            ));
        }
        if daytime {
            return Err(validation_error(
                "--daytime is only available for overseas orders; pass --exchange",
            ));
        }
        return Ok(None);
    }

    if reserve && daytime {
        return Err(validation_error(
            "--reserve and --daytime cannot be used together",
        ));
    }

    if daytime {
        validate_daytime_exchange(exchange.expect("checked"), is_virtual)?;
        return Ok(Some(OverseasPlaceMode::Daytime));
    }

    if reserve {
        return Ok(Some(OverseasPlaceMode::Reserve));
    }

    Ok(Some(OverseasPlaceMode::Regular))
}

fn overseas_modify_mode(
    exchange: Option<&str>,
    daytime: bool,
    is_virtual: bool,
) -> Result<Option<OverseasModifyMode>> {
    if exchange.is_none() {
        if daytime {
            return Err(validation_error(
                "--daytime is only available for overseas orders; pass --exchange",
            ));
        }
        return Ok(None);
    }

    if daytime {
        validate_daytime_exchange(exchange.expect("checked"), is_virtual)?;
        return Ok(Some(OverseasModifyMode::Daytime));
    }

    Ok(Some(OverseasModifyMode::Regular))
}

fn validate_daytime_exchange(exchange: &str, is_virtual: bool) -> Result<()> {
    if is_virtual {
        return Err(validation_error(
            "--daytime is only supported in real environment",
        ));
    }

    if !is_us_overseas_exchange(exchange) {
        return Err(validation_error(
            "--daytime is only supported for U.S. exchanges (NASD, NYSE, AMEX)",
        ));
    }

    Ok(())
}

fn validation_error(message: impl Into<String>) -> anyhow::Error {
    ValidationError::new(message).into()
}

fn normalize_order_exchange(exchange: String) -> Result<String> {
    Ok(parse_order_exchange(&exchange)?.code().to_string())
}

fn parse_order_exchange(exchange: &str) -> Result<OrderExchange> {
    OrderExchange::parse(exchange).map_err(|error| validation_error(error.to_string()))
}

fn is_us_overseas_exchange(exchange: &str) -> bool {
    matches!(
        exchange.to_ascii_uppercase().as_str(),
        "NASD" | "NYSE" | "AMEX"
    )
}

async fn run_balance(
    runtime: &Runtime,
    args: cli::BalanceArgs,
    writer: &mut dyn Write,
) -> Result<()> {
    match args.command {
        Some(cli::BalanceCommand::Overseas(args)) => {
            let result = overseas_balance::get_balance(
                &runtime.client,
                &runtime.config.account_no,
                &runtime.config.account_prod,
                &args.exchange,
                &args.currency,
                runtime.config.environment.is_virtual(),
            )
            .await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &result);
            }

            let value = serde_json::to_value(&result)?;
            let items = json_rows(
                &value,
                "items",
                &[
                    "ovrs_pdno",
                    "ovrs_excg_cd",
                    "tr_crcy_cd",
                    "ovrs_cblc_qty",
                    "ord_psbl_qty",
                    "pchs_avg_pric",
                    "now_pric2",
                    "ovrs_stck_evlu_amt",
                    "frcr_evlu_pfls_amt",
                    "evlu_pfls_rt",
                ],
            );
            let summary = json_first_pairs(
                &value,
                "summary",
                &[
                    ("총손익", "ovrs_tot_pfls"),
                    ("총평가손익", "tot_evlu_pfls_amt"),
                    ("총수익률", "tot_pftrt"),
                ],
            );
            write_value_sections(
                writer,
                &[(
                    &[
                        "종목코드",
                        "거래소",
                        "통화",
                        "잔고수량",
                        "주문가능",
                        "평균단가",
                        "현재가",
                        "평가금액",
                        "평가손익",
                        "손익율",
                    ],
                    items,
                )],
                &[("요약", summary)],
            )?;
        }
        Some(cli::BalanceCommand::Present(args)) => {
            let result = overseas_balance::get_present_balance(
                &runtime.client,
                &runtime.config.account_no,
                &runtime.config.account_prod,
                &args.currency_type,
                &args.country,
                &args.market,
                &args.inquiry,
                runtime.config.environment.is_virtual(),
            )
            .await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &result);
            }

            let value = serde_json::to_value(&result)?;
            let output1 = json_rows(
                &value,
                "output1",
                &[
                    "pdno",
                    "tr_mket_name",
                    "natn_kor_name",
                    "cblc_qty13",
                    "ord_psbl_qty1",
                    "avg_unpr3",
                    "ovrs_now_pric1",
                    "evlu_pfls_amt2",
                    "evlu_pfls_rt1",
                ],
            );
            let output2 = json_first_pairs(
                &value,
                "output2",
                &[
                    ("예수금", "frcr_dncl_amt_2"),
                    ("매수합계", "frcr_buy_amt_smtl"),
                    ("매도합계", "frcr_sll_amt_smtl"),
                    ("출금가능", "frcr_drwg_psbl_amt_1"),
                ],
            );
            let output3 = json_first_pairs(
                &value,
                "output3",
                &[
                    ("매입합계", "pchs_amt_smtl"),
                    ("평가합계", "evlu_amt_smtl"),
                    ("평가손익합계", "evlu_pfls_amt_smtl"),
                    ("총자산", "tot_asst_amt"),
                    ("총대출금액", "tot_loan_amt"),
                ],
            );
            write_value_sections(
                writer,
                &[(
                    &[
                        "종목코드",
                        "시장",
                        "국가",
                        "잔고수량",
                        "주문가능",
                        "평균단가",
                        "현재가",
                        "평가손익",
                        "손익율",
                    ],
                    output1,
                )],
                &[("요약", output2), ("총계", output3)],
            )?;
        }
        Some(cli::BalanceCommand::Settlement(args)) => {
            let result = overseas_balance::get_payment_balance(
                &runtime.client,
                &runtime.config.account_no,
                &runtime.config.account_prod,
                &args.date,
                &args.currency_type,
                &args.inquiry,
                runtime.config.environment.is_virtual(),
            )
            .await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &result);
            }

            let value = serde_json::to_value(&result)?;
            let output1 = json_rows(
                &value,
                "output1",
                &[
                    "pdno",
                    "prdt_name",
                    "ovrs_excg_cd",
                    "cblc_qty13",
                    "ord_psbl_qty1",
                    "avg_unpr3",
                    "ovrs_now_pric1",
                    "evlu_pfls_amt2",
                ],
            );
            let output2 = json_first_pairs(
                &value,
                "output2",
                &[
                    ("예수금", "frcr_dncl_amt_2"),
                    ("고시환율", "frst_bltn_exrt"),
                ],
            );
            let output3 = json_first_pairs(
                &value,
                "output3",
                &[
                    ("매입합계", "pchs_amt_smtl_amt"),
                    ("총평가손익", "tot_evlu_pfls_amt"),
                    ("총예수금", "tot_dncl_amt"),
                    ("총자산", "tot_asst_amt2"),
                    ("총대출금액", "tot_loan_amt"),
                ],
            );
            write_value_sections(
                writer,
                &[(
                    &[
                        "종목코드",
                        "종목명",
                        "거래소",
                        "잔고수량",
                        "주문가능",
                        "평균단가",
                        "현재가",
                        "평가손익",
                    ],
                    output1,
                )],
                &[("요약", output2), ("총계", output3)],
            )?;
        }
        Some(cli::BalanceCommand::OvrsExecutions(args)) => {
            let result = overseas_balance::get_executions(
                &runtime.client,
                &runtime.config.account_no,
                &runtime.config.account_prod,
                &args.stock,
                &args.start,
                &args.end,
                &args.side,
                &args.status,
                &args.exchange,
                &args.sort,
                runtime.config.environment.is_virtual(),
            )
            .await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &result);
            }

            let value = serde_json::to_value(&result)?;
            let rows = json_rows(
                &value,
                "$root",
                &[
                    "ord_dt",
                    "odno",
                    "prdt_name",
                    "sll_buy_dvsn_cd_name",
                    "ft_ord_qty",
                    "ft_ord_unpr3",
                    "ft_ccld_qty",
                    "nccs_qty",
                    "prcs_stat_name",
                    "ovrs_excg_cd",
                ],
            );
            let output = render::render_table(
                &[
                    "일자",
                    "주문번호",
                    "종목명",
                    "구분",
                    "주문수량",
                    "주문단가",
                    "체결수량",
                    "미체결수량",
                    "상태",
                    "거래소",
                ],
                &rows,
            );
            writeln!(writer, "{output}")?;
        }
        Some(cli::BalanceCommand::OpenOrders(args)) => {
            let result = overseas_balance::get_open_orders(
                &runtime.client,
                &runtime.config.account_no,
                &runtime.config.account_prod,
                &args.exchange,
                &args.sort,
                runtime.config.environment.is_virtual(),
            )
            .await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &result);
            }

            let value = serde_json::to_value(&result)?;
            let rows = json_rows(
                &value,
                "$root",
                &[
                    "ord_dt",
                    "odno",
                    "pdno",
                    "sll_buy_dvsn_cd",
                    "ft_ord_qty",
                    "ft_ccld_qty",
                    "nccs_qty",
                    "ft_ord_unpr3",
                    "ord_tmd",
                    "ovrs_excg_cd",
                ],
            );
            let output = render::render_table(
                &[
                    "일자",
                    "주문번호",
                    "종목코드",
                    "구분",
                    "주문수량",
                    "체결수량",
                    "미체결수량",
                    "주문단가",
                    "시각",
                    "거래소",
                ],
                &rows,
            );
            writeln!(writer, "{output}")?;
        }
        Some(cli::BalanceCommand::PsblBuy(args)) => {
            if let Some(exchange) = args.exchange {
                if args.price == "0" {
                    return Err(validation_error(
                        "--price is required for overseas possible-buy queries",
                    ));
                }

                let result = overseas_balance::get_possible_buy_amount(
                    &runtime.client,
                    &runtime.config.account_no,
                    &runtime.config.account_prod,
                    &exchange,
                    &args.price,
                    &args.stock,
                    runtime.config.environment.is_virtual(),
                )
                .await?;
                if runtime.output_json {
                    return write_command_json(writer, runtime.command_name, &result);
                }
                let output = render::render_pairs(&[
                    ("거래통화", display_or_dash(&result.tr_crcy_cd)),
                    ("주문가능외화", display_or_dash(&result.ord_psbl_frcr_amt)),
                    ("재사용가능", display_or_dash(&result.sll_ruse_psbl_amt)),
                    ("해외주문가능", display_or_dash(&result.ovrs_ord_psbl_amt)),
                    ("주문가능수량", display_or_dash(&result.ord_psbl_qty)),
                    (
                        "최대주문가능수량",
                        display_or_dash(&result.max_ord_psbl_qty),
                    ),
                    (
                        "환전후가능금액",
                        display_or_dash(&result.echm_af_ord_psbl_amt),
                    ),
                    (
                        "환전후가능수량",
                        display_or_dash(&result.echm_af_ord_psbl_qty),
                    ),
                    ("환율", display_or_dash(&result.exrt)),
                ]);
                writeln!(writer, "{output}")?;
            } else {
                let result = balance::get_possible_order(
                    &runtime.client,
                    &runtime.config.account_no,
                    &runtime.config.account_prod,
                    &args.stock,
                    &args.order_type,
                    &args.price,
                    runtime.config.environment.is_virtual(),
                )
                .await?;
                if runtime.output_json {
                    return write_command_json(writer, runtime.command_name, &result);
                }
                let output = render::render_pairs(&[
                    ("주문가능현금", result.ord_psbl_cash),
                    ("주문가능대용", result.ord_psbl_sbst),
                    ("재사용가능", result.ruse_psbl_amt),
                    ("미수없는매수", result.nrcvb_buy_amt),
                ]);
                writeln!(writer, "{output}")?;
            }
        }
        Some(cli::BalanceCommand::PsblSell(args)) => {
            let result = balance::get_possible_sell(
                &runtime.client,
                &runtime.config.account_no,
                &runtime.config.account_prod,
                &args.stock,
                runtime.config.environment.is_virtual(),
            )
            .await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &result);
            }
            let output = render::render_pairs(&[
                ("종목코드", result.pdno),
                ("종목명", result.prdt_name),
                ("매도가능수량", result.ord_psbl_qty),
                ("매입평균가", result.pchs_avg_pric),
            ]);
            writeln!(writer, "{output}")?;
        }
        Some(cli::BalanceCommand::PeriodProfit(args)) => {
            let result = overseas_balance::get_period_profit(
                &runtime.client,
                &runtime.config.account_no,
                &runtime.config.account_prod,
                &args.exchange,
                args.country.as_deref().unwrap_or(""),
                &args.currency,
                &args.stock,
                &args.start,
                &args.end,
                &args.currency_type,
            )
            .await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &result);
            }

            let value = serde_json::to_value(&result)?;
            let rows = json_rows(
                &value,
                "items",
                &[
                    "trad_day",
                    "ovrs_pdno",
                    "ovrs_excg_cd",
                    "slcl_qty",
                    "pchs_avg_pric",
                    "avg_sll_unpr",
                    "ovrs_rlzt_pfls_amt",
                    "pftrt",
                ],
            );
            let summary = json_first_pairs(
                &value,
                "summary",
                &[
                    ("총매수", "stck_buy_amt_smtl"),
                    ("총매도", "stck_sll_amt_smtl"),
                    ("총손익", "ovrs_rlzt_pfls_tot_amt"),
                    ("총수익률", "tot_pftrt"),
                ],
            );
            write_value_sections(
                writer,
                &[(
                    &[
                        "거래일",
                        "종목코드",
                        "거래소",
                        "수량",
                        "평균매입가",
                        "평균매도가",
                        "실현손익",
                        "수익률",
                    ],
                    rows,
                )],
                &[("요약", summary)],
            )?;
        }
        Some(cli::BalanceCommand::PeriodTrans(args)) => {
            let result = overseas_balance::get_period_transactions(
                &runtime.client,
                &runtime.config.account_no,
                &runtime.config.account_prod,
                &args.start,
                &args.end,
                &args.exchange,
                &args.stock,
                &args.side,
                &args.loan_type,
            )
            .await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &result);
            }

            let value = serde_json::to_value(&result)?;
            let rows = json_rows(
                &value,
                "items",
                &[
                    "trad_dt",
                    "sttl_dt",
                    "sll_buy_dvsn_name",
                    "pdno",
                    "ovrs_item_name",
                    "ccld_qty",
                    "ovrs_stck_ccld_unpr",
                    "tr_frcr_amt2",
                    "crcy_cd",
                ],
            );
            let summary = json_first_pairs(
                &value,
                "summary",
                &[
                    ("총매수", "frcr_buy_amt_smtl"),
                    ("총매도", "frcr_sll_amt_smtl"),
                    ("국내수수료", "dmst_fee_smtl"),
                    ("해외수수료", "ovrs_fee_smtl"),
                ],
            );
            write_value_sections(
                writer,
                &[(
                    &[
                        "거래일",
                        "결제일",
                        "구분",
                        "종목코드",
                        "종목명",
                        "체결수량",
                        "체결단가",
                        "거래금액",
                        "통화",
                    ],
                    rows,
                )],
                &[("요약", summary)],
            )?;
        }
        Some(cli::BalanceCommand::AlgoExecutions(args)) => {
            let result = overseas_balance::get_algo_executions(
                &runtime.client,
                &runtime.config.account_no,
                &runtime.config.account_prod,
                &args.date,
                &args.org_no,
                &args.order_no,
                &args.totalize,
            )
            .await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &result);
            }

            let value = serde_json::to_value(&result)?;
            let rows = json_rows(
                &value,
                "items",
                &[
                    "odno",
                    "pdno",
                    "item_name",
                    "ft_ord_qty",
                    "ft_ccld_qty",
                    "ft_ccld_unpr3",
                    "ft_ccld_amt3",
                    "ord_tmd",
                    "trad_dvsn_name",
                ],
            );
            let summary = json_first_pairs(
                &value,
                "summary",
                &[("체결건수", "ccld_cnt"), ("통화", "tr_crcy")],
            );
            write_value_sections(
                writer,
                &[(
                    &[
                        "주문번호",
                        "종목코드",
                        "종목명",
                        "주문수량",
                        "체결수량",
                        "체결단가",
                        "체결금액",
                        "주문시각",
                        "구분",
                    ],
                    rows,
                )],
                &[("요약", summary)],
            )?;
        }
        Some(cli::BalanceCommand::ReserveOrders(args)) => {
            let result = overseas_balance::get_reservation_orders(
                &runtime.client,
                args.region.code(),
                &runtime.config.account_no,
                &runtime.config.account_prod,
                &args.start,
                &args.end,
                &args.inquiry,
                &args.exchange,
                &args.product_type,
            )
            .await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &result);
            }

            let value = serde_json::to_value(&result)?;
            let rows = json_rows(
                &value,
                "$root",
                &[
                    "rsvn_ord_rcit_dt",
                    "ovrs_rsvn_odno",
                    "pdno",
                    "prdt_name",
                    "sll_buy_dvsn_cd_name",
                    "ovrs_rsvn_ord_stat_cd_name",
                    "ft_ord_qty",
                    "ft_ord_unpr3",
                    "ovrs_excg_cd",
                ],
            );
            let output = render::render_table(
                &[
                    "접수일자",
                    "예약주문번호",
                    "종목코드",
                    "종목명",
                    "구분",
                    "상태",
                    "주문수량",
                    "주문단가",
                    "거래소",
                ],
                &rows,
            );
            writeln!(writer, "{output}")?;
        }
        Some(cli::BalanceCommand::Executions(args)) => {
            let executions = balance::get_daily_executions(
                &runtime.client,
                &runtime.config.account_no,
                &runtime.config.account_prod,
                args.start.as_deref().unwrap_or(""),
                args.end.as_deref().unwrap_or(""),
                runtime.config.environment.is_virtual(),
            )
            .await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &executions);
            }
            let rows = executions
                .into_iter()
                .map(|item| {
                    vec![
                        item.ord_dt,
                        item.odno,
                        item.prdt_name,
                        if item.sll_buy_dvsn == "01" {
                            "매도".to_string()
                        } else {
                            "매수".to_string()
                        },
                        item.ord_qty,
                        item.ord_unpr,
                        item.tot_ccld_qty,
                        item.tot_ccld_amt,
                    ]
                })
                .collect::<Vec<_>>();
            let output = render::render_table(
                &[
                    "일자",
                    "주문번호",
                    "종목명",
                    "매수/매도",
                    "수량",
                    "단가",
                    "체결수량",
                    "체결금액",
                ],
                &rows,
            );
            writeln!(writer, "{output}")?;
        }
        None => {
            let result = balance::get_balance(
                &runtime.client,
                &runtime.config.account_no,
                &runtime.config.account_prod,
                runtime.config.environment.is_virtual(),
            )
            .await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &result);
            }
            if result.items.is_empty() {
                writeln!(writer, "보유 종목이 없습니다.")?;
            } else {
                let rows = result
                    .items
                    .into_iter()
                    .map(|item| {
                        vec![
                            item.pdno,
                            item.prdt_name,
                            item.hldg_qty,
                            item.pchs_avg_pric,
                            item.prpr,
                            item.evlu_amt,
                            item.evlu_pfls_amt,
                            format!("{}%", item.evlu_pfls_rt),
                        ]
                    })
                    .collect::<Vec<_>>();
                let output = render::render_table(
                    &[
                        "종목코드",
                        "종목명",
                        "보유수량",
                        "매입평균가",
                        "현재가",
                        "평가금액",
                        "손익금액",
                        "손익율",
                    ],
                    &rows,
                );
                writeln!(writer, "{output}")?;
            }
        }
    }

    Ok(())
}

async fn run_market(
    runtime: &Runtime,
    args: cli::MarketArgs,
    writer: &mut dyn Write,
) -> Result<()> {
    match args.command {
        cli::MarketCommand::Volume => {
            let items = market::get_volume_rank(&runtime.client).await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &items);
            }

            let rows = items
                .into_iter()
                .map(|item| {
                    vec![
                        item.data_rank,
                        item.mksc_shrn_iscd,
                        item.hts_kor_isnm,
                        item.stck_prpr,
                        format!("{} {}", price_sign(&item.prdy_vrss_sign), item.prdy_vrss),
                        format!("{}%", item.prdy_ctrt),
                        item.acml_vol,
                    ]
                })
                .collect::<Vec<_>>();
            let table = render::render_table(
                &[
                    "순위",
                    "종목코드",
                    "종목명",
                    "현재가",
                    "전일대비",
                    "등락율",
                    "거래량",
                ],
                &rows,
            );
            writeln!(writer, "{table}")?;
        }
        cli::MarketCommand::Holiday(args) => {
            let items = market::get_holidays(&runtime.client, &args.date).await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &items);
            }

            let rows = items
                .into_iter()
                .map(|item| {
                    vec![
                        item.bass_dt,
                        yn_to_mark(&item.bzdy_yn),
                        yn_to_mark(&item.tr_day_yn),
                        yn_to_mark(&item.opnd_yn),
                    ]
                })
                .collect::<Vec<_>>();
            let table = render::render_table(&["일자", "영업일", "거래일", "개장일"], &rows);
            writeln!(writer, "{table}")?;
        }
    }

    Ok(())
}

async fn run_finance(
    runtime: &Runtime,
    args: cli::FinanceArgs,
    writer: &mut dyn Write,
) -> Result<()> {
    match args.command {
        cli::FinanceCommand::Bs(args) => {
            let items =
                finance::get_balance_sheet(&runtime.client, &args.stock, Some(args.div.as_str()))
                    .await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &items);
            }

            let rows = items
                .into_iter()
                .map(|item| {
                    vec![
                        item.stac_yy,
                        item.cras,
                        item.fxas,
                        item.total_aset,
                        item.flow_lblt,
                        item.fix_lblt,
                        item.total_lblt,
                        item.total_cptl,
                    ]
                })
                .collect::<Vec<_>>();
            let table = render::render_table(
                &[
                    "결산년도",
                    "유동자산",
                    "고정자산",
                    "자산총계",
                    "유동부채",
                    "고정부채",
                    "부채총계",
                    "자본총계",
                ],
                &rows,
            );
            writeln!(writer, "{table}")?;
        }
        cli::FinanceCommand::Is(args) => {
            let items = finance::get_income_statement(
                &runtime.client,
                &args.stock,
                Some(args.div.as_str()),
            )
            .await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &items);
            }

            let rows = items
                .into_iter()
                .map(|item| {
                    vec![
                        item.stac_yy,
                        item.sale_account,
                        item.sale_cost,
                        item.sale_totl_prfi,
                        item.bsop_prti,
                        item.thtr_ntin,
                    ]
                })
                .collect::<Vec<_>>();
            let table = render::render_table(
                &[
                    "결산년도",
                    "매출액",
                    "매출원가",
                    "매출총이익",
                    "영업이익",
                    "당기순이익",
                ],
                &rows,
            );
            writeln!(writer, "{table}")?;
        }
        cli::FinanceCommand::Ratio(args) => {
            let items =
                finance::get_financial_ratio(&runtime.client, &args.stock, Some(args.div.as_str()))
                    .await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &items);
            }

            let rows = items
                .into_iter()
                .map(|item| {
                    vec![
                        item.stac_yy,
                        display_or_dash(&item.per),
                        display_or_dash(&item.pbr),
                        display_or_dash(&item.roe),
                        display_or_dash(&item.roa),
                        display_or_dash(&item.eps),
                        display_or_dash(&item.bps),
                        display_or_dash(&item.bsop_prfi_rate),
                        display_or_dash(&item.lblt_rate),
                    ]
                })
                .collect::<Vec<_>>();
            let table = render::render_table(
                &[
                    "결산",
                    "PER",
                    "PBR",
                    "ROE",
                    "ROA",
                    "EPS",
                    "BPS",
                    "영업이익율",
                    "부채비율",
                ],
                &rows,
            );
            writeln!(writer, "{table}")?;
        }
    }

    Ok(())
}

async fn run_info(runtime: &Runtime, args: cli::InfoArgs, writer: &mut dyn Write) -> Result<()> {
    match args.command {
        cli::InfoCommand::Dividend(args) => {
            let items = info::get_dividends(&runtime.client, &args.stock).await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &items);
            }

            let rows = items
                .into_iter()
                .map(|item| {
                    vec![
                        item.std_dt,
                        item.divi_kind,
                        item.per_sto_divi_amt,
                        format!("{}%", item.divi_rate),
                    ]
                })
                .collect::<Vec<_>>();
            let table =
                render::render_table(&["기준일", "배당종류", "주당배당금", "배당율"], &rows);
            writeln!(writer, "{table}")?;
        }
        cli::InfoCommand::News(args) => {
            let items = info::get_news(&runtime.client, &args.stock).await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &items);
            }

            let rows = items
                .into_iter()
                .map(|item| vec![item.data_dt, item.data_tm, item.data_srce, item.cntt_ttle])
                .collect::<Vec<_>>();
            let table = render::render_table(&["일자", "시각", "출처", "제목"], &rows);
            writeln!(writer, "{table}")?;
        }
        cli::InfoCommand::Opinion(args) => {
            let items = info::get_invest_opinions(&runtime.client, &args.stock).await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &items);
            }

            let rows = items
                .into_iter()
                .map(|item| {
                    vec![
                        item.stck_bsop_date,
                        item.rgbf_nm,
                        item.invt_opnn,
                        item.mbcr_name,
                        item.stck_prpr,
                        item.cnss_prpr,
                    ]
                })
                .collect::<Vec<_>>();
            let table = render::render_table(
                &["일자", "기관", "의견", "목표가", "현재가", "컨센서스"],
                &rows,
            );
            writeln!(writer, "{table}")?;
        }
        cli::InfoCommand::Search(args) => {
            let items = info::search_stocks(&runtime.client, &args.keyword).await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &items);
            }

            let rows = items
                .into_iter()
                .map(|item| {
                    vec![
                        item.pdno,
                        item.prdt_name,
                        item.prdt_eng_name,
                        item.mrkt_cls_code,
                    ]
                })
                .collect::<Vec<_>>();
            let table = render::render_table(&["종목코드", "종목명", "영문명", "시장"], &rows);
            writeln!(writer, "{table}")?;
        }
        cli::InfoCommand::Detail(args) => {
            let item =
                overseas_info::get_product_info(&runtime.client, &args.exchange, &args.stock)
                    .await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &item);
            }

            let output = render::render_pairs(&[
                (
                    "종목코드",
                    display_or_dash(&json_string_alias(&item, &["pdno", "PDNO"])),
                ),
                (
                    "종목명",
                    display_or_dash(&json_string_alias(
                        &item,
                        &["prdt_name", "PRDT_NAME", "prdt_abrv_name", "PRDT_ABRV_NAME"],
                    )),
                ),
                (
                    "영문명",
                    display_or_dash(&json_string_alias(
                        &item,
                        &["prdt_eng_name", "PRDT_ENG_NAME"],
                    )),
                ),
                (
                    "상품유형",
                    display_or_dash(&json_string_alias(&item, &["prdt_type_cd", "PRDT_TYPE_CD"])),
                ),
                (
                    "거래소",
                    display_or_dash(&json_string_alias(
                        &item,
                        &[
                            "ovrs_excg_cd",
                            "OVRS_EXCG_CD",
                            "excg_dvsn_cd",
                            "EXCG_DVSN_CD",
                        ],
                    )),
                ),
                (
                    "통화",
                    display_or_dash(&json_string_alias(&item, &["tr_crcy_cd", "TR_CRCY_CD"])),
                ),
            ]);
            writeln!(writer, "{output}")?;
        }
    }

    Ok(())
}

async fn run_ws(runtime: &Runtime, args: cli::WsArgs, writer: &mut dyn Write) -> Result<()> {
    match args.command {
        cli::WsCommand::Approval => {
            let approval = kis_ws::fetch_approval_key(&runtime.config).await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &approval);
            }

            let output =
                render::render_pairs(&[("approval_key", display_or_dash(&approval.approval_key))]);
            writeln!(writer, "{output}")?;
        }
        cli::WsCommand::OvertimeAsk(args) => {
            let payloads = kis_ws::collect_realtime_messages(
                &runtime.config,
                kis_ws::domestic_overtime_asking_price_spec(),
                &args.stock,
                args.count,
                Duration::from_secs(args.timeout_secs),
                args.reconnects,
            )
            .await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &payloads);
            }
            write_ws_overtime_ask(writer, &payloads)?;
        }
        cli::WsCommand::OvertimeCcnl(args) => {
            let payloads = kis_ws::collect_realtime_messages(
                &runtime.config,
                kis_ws::domestic_overtime_ccnl_spec(),
                &args.stock,
                args.count,
                Duration::from_secs(args.timeout_secs),
                args.reconnects,
            )
            .await?;
            if runtime.output_json {
                return write_command_json(writer, runtime.command_name, &payloads);
            }
            write_ws_overtime_ccnl(writer, &payloads)?;
        }
    }

    Ok(())
}

fn write_ws_overtime_ask(
    writer: &mut dyn Write,
    payloads: &[kis_ws::RealtimePayload],
) -> Result<()> {
    let mut wrote_any = false;

    for (index, payload) in payloads.iter().enumerate() {
        for row in &payload.rows {
            if wrote_any {
                writeln!(writer)?;
            }
            if payloads.len() > 1 {
                writeln!(writer, "message {}", index + 1)?;
            }
            let rows = [
                vec![
                    realtime_cell(row, "askp_rsqn5"),
                    realtime_cell(row, "askp5"),
                    realtime_cell(row, "bidp5"),
                    realtime_cell(row, "bidp_rsqn5"),
                ],
                vec![
                    realtime_cell(row, "askp_rsqn4"),
                    realtime_cell(row, "askp4"),
                    realtime_cell(row, "bidp4"),
                    realtime_cell(row, "bidp_rsqn4"),
                ],
                vec![
                    realtime_cell(row, "askp_rsqn3"),
                    realtime_cell(row, "askp3"),
                    realtime_cell(row, "bidp3"),
                    realtime_cell(row, "bidp_rsqn3"),
                ],
                vec![
                    realtime_cell(row, "askp_rsqn2"),
                    realtime_cell(row, "askp2"),
                    realtime_cell(row, "bidp2"),
                    realtime_cell(row, "bidp_rsqn2"),
                ],
                vec![
                    realtime_cell(row, "askp_rsqn1"),
                    realtime_cell(row, "askp1"),
                    realtime_cell(row, "bidp1"),
                    realtime_cell(row, "bidp_rsqn1"),
                ],
            ];
            writeln!(
                writer,
                "{}",
                render::render_table(&["매도잔량", "매도호가", "매수호가", "매수잔량"], &rows)
            )?;
            writeln!(
                writer,
                "\n시각: {}  총매도잔량: {}  총매수잔량: {}  예상체결가: {}",
                realtime_cell(row, "bsop_hour"),
                realtime_cell(row, "ovtm_total_askp_rsqn"),
                realtime_cell(row, "ovtm_total_bidp_rsqn"),
                realtime_cell(row, "antc_cnpr"),
            )?;
            wrote_any = true;
        }
    }

    if !wrote_any {
        writeln!(writer, "데이터가 없습니다.")?;
    }

    Ok(())
}

fn write_ws_overtime_ccnl(
    writer: &mut dyn Write,
    payloads: &[kis_ws::RealtimePayload],
) -> Result<()> {
    let rows = payloads
        .iter()
        .flat_map(|payload| payload.rows.iter())
        .map(|row| {
            vec![
                realtime_cell(row, "stck_cntg_hour"),
                realtime_cell(row, "stck_prpr"),
                format!(
                    "{} {} ({}%)",
                    price_sign(&realtime_cell(row, "prdy_vrss_sign")),
                    realtime_cell(row, "prdy_vrss"),
                    realtime_cell(row, "prdy_ctrt")
                ),
                realtime_cell(row, "cntg_vol"),
                realtime_cell(row, "acml_vol"),
                realtime_cell(row, "cttr"),
                realtime_cell(row, "askp1"),
                realtime_cell(row, "bidp1"),
            ]
        })
        .collect::<Vec<_>>();

    if rows.is_empty() {
        writeln!(writer, "데이터가 없습니다.")?;
        return Ok(());
    }

    writeln!(
        writer,
        "{}",
        render::render_table(
            &[
                "시각",
                "현재가",
                "전일대비",
                "체결량",
                "누적거래량",
                "체결강도",
                "매도1",
                "매수1",
            ],
            &rows,
        )
    )?;

    Ok(())
}

fn run_config(runtime: &Runtime, writer: &mut dyn Write) -> Result<()> {
    if runtime.output_json {
        return write_command_json(
            writer,
            runtime.command_name,
            &config_output(&runtime.config_path, &runtime.config),
        );
    }

    if !runtime.quiet_text() {
        writeln!(writer, "Current configuration:")?;
    }
    let output = render::render_pairs(&[
        ("config file", runtime.config_path.display().to_string()),
        ("environment", runtime.config.environment.to_string()),
        ("account_no", runtime.config.account_no.clone()),
        ("account_prod", runtime.config.account_prod.clone()),
        (
            "base_url",
            runtime.config.environment.base_url().to_string(),
        ),
        (
            "ws_base_url",
            runtime.config.environment.ws_base_url().to_string(),
        ),
        ("app_key", mask_app_key(&runtime.config.app_key)),
    ]);
    writeln!(writer, "{output}")?;
    Ok(())
}

fn config_output(config_path: &Path, config: &AppConfig) -> ConfigOutput {
    ConfigOutput {
        config_file: config_path.display().to_string(),
        environment: config.environment.to_string(),
        account_no: config.account_no.clone(),
        account_prod: config.account_prod.clone(),
        base_url: config.environment.base_url().to_string(),
        ws_base_url: config.environment.ws_base_url().to_string(),
        app_key: mask_app_key(&config.app_key),
    }
}

fn mask_app_key(app_key: &str) -> String {
    if app_key.len() >= 4 {
        format!("{}****", &app_key[..4])
    } else if app_key.is_empty() {
        "(not set)".to_string()
    } else {
        "****".to_string()
    }
}

fn order_output(
    side: &'static str,
    order_org_no: String,
    order_no: String,
    order_time: String,
) -> OrderOutput {
    OrderOutput {
        side,
        order_org_no,
        order_no,
        order_time,
    }
}

fn reserve_order_output(side: &'static str, value: &Value) -> ReserveOrderOutput {
    ReserveOrderOutput {
        side,
        order_no: json_string_alias(value, &["order_no", "odno", "ODNO"]),
        receipt_date: json_string_alias(
            value,
            &["receipt_date", "rsvn_ord_rcit_dt", "RSVN_ORD_RCIT_DT"],
        ),
        reservation_order_no: json_string_alias(
            value,
            &[
                "reservation_order_no",
                "reserve_order_no",
                "ovrs_rsvn_odno",
                "OVRS_RSVN_ODNO",
            ],
        ),
    }
}

fn write_order_result(
    side: &str,
    order_org_no: String,
    order_no: String,
    order_time: String,
    quiet: bool,
    writer: &mut dyn Write,
) -> Result<()> {
    if !quiet {
        writeln!(writer, "{side} 주문 완료")?;
    }
    writeln!(
        writer,
        "{}",
        render::render_pairs(&[
            ("주문번호", display_or_dash(&order_no)),
            ("주문시각", display_or_dash(&order_time)),
            ("조직번호", display_or_dash(&order_org_no)),
        ])
    )?;
    Ok(())
}

fn write_reserve_order_result(
    side: &str,
    value: &Value,
    quiet: bool,
    writer: &mut dyn Write,
) -> Result<()> {
    let output = reserve_order_output("reserve", value);

    if !quiet {
        writeln!(writer, "{side} 예약주문 완료")?;
    }
    writeln!(
        writer,
        "{}",
        render::render_pairs(&[
            ("주문번호", display_or_dash(&output.order_no)),
            ("접수일자", display_or_dash(&output.receipt_date)),
            (
                "예약주문번호",
                display_or_dash(&output.reservation_order_no)
            ),
        ])
    )?;
    Ok(())
}

fn write_reserve_cancel_result(value: &Value, quiet: bool, writer: &mut dyn Write) -> Result<()> {
    let output = reserve_order_output("cancel", value);

    if !quiet {
        writeln!(writer, "예약주문 취소 완료")?;
    }
    writeln!(
        writer,
        "{}",
        render::render_pairs(&[
            ("주문번호", display_or_dash(&output.order_no)),
            ("접수일자", display_or_dash(&output.receipt_date)),
            (
                "예약주문번호",
                display_or_dash(&output.reservation_order_no)
            ),
        ])
    )?;
    Ok(())
}

fn write_overseas_reserve_result<T>(
    side_json: &'static str,
    side_text: &str,
    result: &T,
    runtime: &Runtime,
    writer: &mut dyn Write,
) -> Result<()>
where
    T: Serialize,
{
    let value = serde_json::to_value(result)?;
    if runtime.output_json {
        return write_command_json(
            writer,
            runtime.command_name,
            &reserve_order_output(side_json, &value),
        );
    }

    write_reserve_order_result(side_text, &value, runtime.quiet_text(), writer)
}

fn write_value_sections(
    writer: &mut dyn Write,
    tables: &[(&[&str], Vec<Vec<String>>)],
    sections: &[(&str, Vec<(&str, String)>)],
) -> Result<()> {
    let mut wrote_any = false;

    for (headers, rows) in tables {
        if rows.is_empty() {
            continue;
        }
        if wrote_any {
            writeln!(writer)?;
        }
        writeln!(writer, "{}", render::render_table(headers, rows))?;
        wrote_any = true;
    }

    for (label, pairs) in sections {
        if pairs.is_empty() {
            continue;
        }
        if wrote_any {
            writeln!(writer)?;
        }
        writeln!(writer, "{label}")?;
        writeln!(writer, "{}", render::render_pairs(pairs))?;
        wrote_any = true;
    }

    if !wrote_any {
        writeln!(writer, "데이터가 없습니다.")?;
    }

    Ok(())
}

fn json_rows(value: &Value, key: &str, fields: &[&str]) -> Vec<Vec<String>> {
    json_array(value, key)
        .iter()
        .filter_map(Value::as_object)
        .map(|row| {
            fields
                .iter()
                .map(|field| json_cell(row, field))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn json_first_pairs(
    value: &Value,
    key: &str,
    fields: &[(&'static str, &str)],
) -> Vec<(&'static str, String)> {
    json_array(value, key)
        .first()
        .and_then(Value::as_object)
        .map(|row| {
            fields
                .iter()
                .map(|(label, field)| (*label, json_cell(row, field)))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn json_array<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    if key == "$root" {
        value.as_array().map(Vec::as_slice).unwrap_or(&[])
    } else {
        value
            .get(key)
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

fn json_cell(row: &serde_json::Map<String, Value>, field: &str) -> String {
    row.get(field)
        .map(|value| match value {
            Value::Null => "-".to_string(),
            Value::String(value) if value.is_empty() => "-".to_string(),
            Value::String(value) => value.to_string(),
            Value::Number(value) => value.to_string(),
            Value::Bool(value) => value.to_string(),
            _ => value.to_string(),
        })
        .unwrap_or_else(|| "-".to_string())
}

fn realtime_cell(row: &kis_ws::RealtimeRow, field: &str) -> String {
    row.get(field)
        .map(|value| {
            if value.is_empty() {
                "-".to_string()
            } else {
                value.to_string()
            }
        })
        .unwrap_or_else(|| "-".to_string())
}

fn write_command_json<T>(writer: &mut dyn Write, command: &str, value: &T) -> Result<()>
where
    T: Serialize,
{
    write_json_raw(
        writer,
        &SuccessEnvelope {
            ok: true,
            command,
            data: value,
        },
    )
}

pub fn write_json_error(writer: &mut dyn Write, command: &str, err: &anyhow::Error) -> Result<()> {
    let classified = classify_error(err);
    write_json_raw(
        writer,
        &ErrorEnvelope {
            ok: false,
            command,
            error: ErrorOutput {
                kind: classified.kind,
                message: classified.message,
                code: classified.code,
            },
        },
    )
}

fn classify_error(err: &anyhow::Error) -> ClassifiedError {
    for cause in err.chain() {
        if let Some(validation) = cause.downcast_ref::<ValidationError>() {
            return ClassifiedError {
                kind: "validation",
                message: validation.to_string(),
                code: None,
            };
        }

        if let Some(kis_error) = cause.downcast_ref::<KisError>() {
            match kis_error {
                KisError::Api { code, message } => {
                    return ClassifiedError {
                        kind: "api",
                        message: message.clone(),
                        code: Some(code.clone()),
                    };
                }
                KisError::Config(message) => {
                    return ClassifiedError {
                        kind: "config",
                        message: message.clone(),
                        code: None,
                    };
                }
                KisError::Yaml(error) => {
                    return ClassifiedError {
                        kind: "config",
                        message: error.to_string(),
                        code: None,
                    };
                }
                KisError::Io(error) if err.to_string().contains("loading config") => {
                    return ClassifiedError {
                        kind: "config",
                        message: error.to_string(),
                        code: None,
                    };
                }
                KisError::Io(_) | KisError::Http(_) | KisError::Json(_) | KisError::Parse(_) => {
                    return ClassifiedError {
                        kind: "runtime",
                        message: err.to_string(),
                        code: None,
                    };
                }
            }
        }
    }

    let kind = if err.to_string().contains("loading config") {
        "config"
    } else {
        "runtime"
    };
    ClassifiedError {
        kind,
        message: err.to_string(),
        code: None,
    }
}

fn write_json_raw<T>(writer: &mut dyn Write, value: &T) -> Result<()>
where
    T: Serialize,
{
    serde_json::to_writer_pretty(&mut *writer, value)?;
    writeln!(writer)?;
    Ok(())
}

fn json_string_alias(value: &Value, keys: &[&str]) -> String {
    keys.iter()
        .find_map(|key| value.get(*key))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn price_sign(code: &str) -> &'static str {
    match code {
        "1" | "2" => "+",
        "4" | "5" => "-",
        _ => " ",
    }
}

fn display_or_dash(value: &str) -> String {
    if value.is_empty() {
        "-".to_string()
    } else {
        value.to_string()
    }
}

fn yn_to_mark(value: &str) -> String {
    if value == "Y" {
        "O".to_string()
    } else {
        "X".to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use kis_core::config::{AppConfig, Environment};
    use serde_json::json;

    use super::{
        KisError, OverseasModifyMode, OverseasPlaceMode, classify_error, config_output,
        display_or_dash, mask_app_key, order_output, overseas_modify_mode, overseas_place_mode,
        price_sign, reserve_order_output, validation_error, write_command_json, write_json_error,
        write_json_raw, yn_to_mark,
    };

    #[test]
    fn masks_app_key_like_go_cli() {
        assert_eq!(mask_app_key("abcd1234"), "abcd****");
        assert_eq!(mask_app_key(""), "(not set)");
        assert_eq!(mask_app_key("abc"), "****");
    }

    #[test]
    fn maps_price_sign_codes() {
        assert_eq!(price_sign("2"), "+");
        assert_eq!(price_sign("5"), "-");
        assert_eq!(price_sign("3"), " ");
    }

    #[test]
    fn maps_yes_no_to_marks() {
        assert_eq!(yn_to_mark("Y"), "O");
        assert_eq!(yn_to_mark("N"), "X");
    }

    #[test]
    fn shows_dash_for_missing_values() {
        assert_eq!(display_or_dash(""), "-");
        assert_eq!(display_or_dash("1.23"), "1.23");
    }

    #[test]
    fn serializes_config_output_with_masked_key() {
        let config = AppConfig {
            app_key: "abcd1234".to_string(),
            app_secret: "secret".to_string(),
            account_no: "12345678".to_string(),
            account_prod: "01".to_string(),
            environment: Environment::Virtual,
        };

        let value =
            serde_json::to_value(config_output(Path::new("/tmp/config.yaml"), &config)).unwrap();

        assert_eq!(value["config_file"], "/tmp/config.yaml");
        assert_eq!(value["environment"], "virtual");
        assert_eq!(value["app_key"], "abcd****");
    }

    #[test]
    fn serializes_order_output_in_snake_case() {
        let value = serde_json::to_value(order_output(
            "buy",
            "06010".to_string(),
            "0000123456".to_string(),
            "100000".to_string(),
        ))
        .unwrap();

        assert_eq!(
            value,
            json!({
                "side": "buy",
                "order_org_no": "06010",
                "order_no": "0000123456",
                "order_time": "100000"
            })
        );
    }

    #[test]
    fn serializes_reserve_order_output_from_snake_case_fields() {
        let value = serde_json::to_value(reserve_order_output(
            "buy",
            &json!({
                "order_no": "0000123456",
                "receipt_date": "20260306",
                "reservation_order_no": "900000001"
            }),
        ))
        .unwrap();

        assert_eq!(
            value,
            json!({
                "side": "buy",
                "order_no": "0000123456",
                "receipt_date": "20260306",
                "reservation_order_no": "900000001"
            })
        );
    }

    #[test]
    fn serializes_reserve_order_output_from_raw_api_fields() {
        let value = serde_json::to_value(reserve_order_output(
            "sell",
            &json!({
                "ODNO": "0000123456",
                "RSVN_ORD_RCIT_DT": "20260306",
                "OVRS_RSVN_ODNO": "900000001"
            }),
        ))
        .unwrap();

        assert_eq!(
            value,
            json!({
                "side": "sell",
                "order_no": "0000123456",
                "receipt_date": "20260306",
                "reservation_order_no": "900000001"
            })
        );
    }

    #[test]
    fn rejects_daytime_without_exchange() {
        let err = overseas_place_mode(None, false, true, false).unwrap_err();
        assert!(err.to_string().contains("--daytime"));
        assert!(err.to_string().contains("--exchange"));
    }

    #[test]
    fn rejects_daytime_for_virtual_environment() {
        let err = overseas_place_mode(Some("NASD"), false, true, true).unwrap_err();
        assert!(err.to_string().contains("real environment"));
    }

    #[test]
    fn rejects_daytime_for_non_us_exchange() {
        let err = overseas_modify_mode(Some("SEHK"), true, false).unwrap_err();
        assert!(err.to_string().contains("U.S. exchanges"));
    }

    #[test]
    fn selects_overseas_modes() {
        assert_eq!(
            overseas_place_mode(Some("NASD"), true, false, false).unwrap(),
            Some(OverseasPlaceMode::Reserve)
        );
        assert_eq!(
            overseas_place_mode(Some("NYSE"), false, true, false).unwrap(),
            Some(OverseasPlaceMode::Daytime)
        );
        assert_eq!(
            overseas_modify_mode(Some("AMEX"), true, false).unwrap(),
            Some(OverseasModifyMode::Daytime)
        );
    }

    #[test]
    fn writes_pretty_json_with_trailing_newline() {
        let mut writer = Vec::new();
        write_json_raw(&mut writer, &json!({ "status": "ok" })).unwrap();

        assert_eq!(
            String::from_utf8(writer).unwrap(),
            "{\n  \"status\": \"ok\"\n}\n"
        );
    }

    #[test]
    fn wraps_success_json_with_command_envelope() {
        let mut writer = Vec::new();
        write_command_json(&mut writer, "config", &json!({ "environment": "virtual" })).unwrap();

        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&writer).unwrap(),
            json!({
                "ok": true,
                "command": "config",
                "data": {
                    "environment": "virtual"
                }
            })
        );
    }

    #[test]
    fn classifies_validation_errors() {
        let classified = classify_error(&validation_error("--price is required"));
        assert_eq!(classified.kind, "validation");
        assert_eq!(classified.message, "--price is required");
        assert_eq!(classified.code, None);
    }

    #[test]
    fn writes_json_error_with_code_for_api_errors() {
        let mut writer = Vec::new();
        let err = anyhow::Error::new(KisError::Api {
            code: "OPSQ0002".to_string(),
            message: "없는 서비스 코드 입니다".to_string(),
        });

        write_json_error(&mut writer, "balance", &err).unwrap();

        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&writer).unwrap(),
            json!({
                "ok": false,
                "command": "balance",
                "error": {
                    "kind": "api",
                    "message": "없는 서비스 코드 입니다",
                    "code": "OPSQ0002"
                }
            })
        );
    }
}
