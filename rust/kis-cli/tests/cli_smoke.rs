use std::fs;
use std::process::Command as ProcessCommand;

use clap::Parser;
use kis_cli::cli::{
    BalanceArgs, BalanceCommand, Cli, Command, OrderCommand, ReservationCancelRegion,
    ReservationRegion, WsCommand,
};
use kis_cli::render::{render_pairs, render_table};
use tempfile::tempdir;

#[test]
fn parses_config_command() {
    let cli = Cli::try_parse_from(["kis", "--env", "real", "config"]).unwrap();
    assert_eq!(cli.env.as_deref(), Some("real"));
    assert!(matches!(cli.command, Command::Config));
}

#[test]
fn parses_json_flag_for_config_command() {
    let cli = Cli::try_parse_from(["kis", "config", "--json"]).unwrap();
    assert!(cli.json);
    assert!(matches!(cli.command, Command::Config));
}

#[test]
fn parses_balance_execution_command() {
    let cli = Cli::try_parse_from([
        "kis",
        "balance",
        "executions",
        "--start",
        "20260301",
        "--end",
        "20260306",
    ])
    .unwrap();

    let Command::Balance(BalanceArgs { command }) = cli.command else {
        panic!("expected balance command");
    };

    assert!(matches!(command, Some(BalanceCommand::Executions(_))));
}

#[test]
fn parses_balance_period_profit_command() {
    let cli = Cli::try_parse_from([
        "kis",
        "balance",
        "period-profit",
        "--exchange",
        "NASD",
        "--currency",
        "USD",
        "--start",
        "20260301",
        "--end",
        "20260307",
    ])
    .unwrap();

    let Command::Balance(BalanceArgs { command }) = cli.command else {
        panic!("expected balance command");
    };

    assert!(matches!(command, Some(BalanceCommand::PeriodProfit(_))));
}

#[test]
fn parses_quote_overtime_ask_command() {
    let cli = Cli::try_parse_from(["kis", "quote", "overtime-ask", "005930"]).unwrap();

    let Command::Quote(args) = cli.command else {
        panic!("expected quote command");
    };

    assert!(matches!(
        args.command,
        kis_cli::cli::QuoteCommand::OvertimeAsk(_)
    ));
}

#[test]
fn parses_overseas_balance_open_orders_command() {
    let cli = Cli::try_parse_from([
        "kis",
        "balance",
        "open-orders",
        "--exchange",
        "NASD",
        "--sort",
        "AS",
    ])
    .unwrap();

    let Command::Balance(BalanceArgs { command }) = cli.command else {
        panic!("expected balance command");
    };

    let Some(BalanceCommand::OpenOrders(args)) = command else {
        panic!("expected balance open-orders");
    };

    assert_eq!(args.exchange, "NASD");
    assert_eq!(args.sort, "AS");
}

#[test]
fn parses_overseas_order_cancel_command() {
    let cli = Cli::try_parse_from([
        "kis",
        "order",
        "cancel",
        "--exchange",
        "NASD",
        "--stock",
        "AAPL",
        "--order-no",
        "0000123456",
    ])
    .unwrap();

    let Command::Order(args) = cli.command else {
        panic!("expected order command");
    };

    let OrderCommand::Cancel(args) = args.command else {
        panic!("expected cancel command");
    };

    assert_eq!(args.exchange.as_deref(), Some("NASD"));
    assert_eq!(args.stock.as_deref(), Some("AAPL"));
    assert_eq!(args.order_no, "0000123456");
}

#[test]
fn parses_overseas_order_reserve_buy_command() {
    let cli = Cli::try_parse_from([
        "kis",
        "order",
        "buy",
        "--exchange",
        "NASD",
        "--stock",
        "AAPL",
        "--qty",
        "1",
        "--price",
        "145.00",
        "--reserve",
    ])
    .unwrap();

    let Command::Order(args) = cli.command else {
        panic!("expected order command");
    };

    let OrderCommand::Buy(args) = args.command else {
        panic!("expected buy command");
    };

    assert!(args.reserve);
    assert!(!args.daytime);
}

#[test]
fn parses_overseas_order_daytime_cancel_command() {
    let cli = Cli::try_parse_from([
        "kis",
        "order",
        "cancel",
        "--exchange",
        "NASD",
        "--stock",
        "AAPL",
        "--order-no",
        "0000123456",
        "--daytime",
    ])
    .unwrap();

    let Command::Order(args) = cli.command else {
        panic!("expected order command");
    };

    let OrderCommand::Cancel(args) = args.command else {
        panic!("expected cancel command");
    };

    assert!(args.daytime);
}

#[test]
fn parses_reserve_cancel_order_command() {
    let cli = Cli::try_parse_from([
        "kis",
        "order",
        "reserve-cancel",
        "--region",
        "us",
        "--receipt-date",
        "20260307",
        "--reservation-order-no",
        "0030008244",
    ])
    .unwrap();

    let Command::Order(args) = cli.command else {
        panic!("expected order command");
    };

    let OrderCommand::ReserveCancel(args) = args.command else {
        panic!("expected reserve-cancel command");
    };

    assert_eq!(args.region, ReservationCancelRegion::Us);
    assert_eq!(args.receipt_date, "20260307");
}

#[test]
fn rejects_asia_region_for_reserve_cancel_order_command() {
    let cli = Cli::try_parse_from([
        "kis",
        "order",
        "reserve-cancel",
        "--region",
        "asia",
        "--receipt-date",
        "20260307",
        "--reservation-order-no",
        "0030008244",
    ]);

    assert!(cli.is_err());
}

#[test]
fn parses_asia_region_for_reserve_orders_command() {
    let cli = Cli::try_parse_from([
        "kis",
        "balance",
        "reserve-orders",
        "--region",
        "asia",
        "--start",
        "20260301",
        "--end",
        "20260307",
        "--exchange",
        "TKSE",
    ])
    .unwrap();

    let Command::Balance(BalanceArgs { command }) = cli.command else {
        panic!("expected balance command");
    };

    let Some(BalanceCommand::ReserveOrders(args)) = command else {
        panic!("expected reserve-orders command");
    };

    assert_eq!(args.region, ReservationRegion::Asia);
    assert_eq!(args.exchange, "TKSE");
}

#[test]
fn parses_ws_approval_command() {
    let cli = Cli::try_parse_from(["kis", "ws", "approval"]).unwrap();

    let Command::Ws(args) = cli.command else {
        panic!("expected ws command");
    };

    assert!(matches!(args.command, WsCommand::Approval));
}

#[test]
fn renders_key_value_output() {
    let output = render_pairs(&[
        ("환경", "virtual".to_string()),
        ("계좌", "12345678".to_string()),
    ]);

    assert!(output.contains("환경"));
    assert!(output.contains("virtual"));
    assert!(output.contains("계좌"));
}

#[test]
fn renders_tabular_output() {
    let output = render_table(
        &["종목코드", "종목명"],
        &[vec!["005930".to_string(), "삼성전자".to_string()]],
    );

    assert!(output.contains("종목코드"));
    assert!(output.contains("005930"));
    assert!(output.contains("삼성전자"));
}

#[test]
fn runs_config_command_through_binary() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.yaml");
    fs::write(
        &config,
        r#"
app_key: "abcd1234"
app_secret: "secret"
account_no: "12345678"
account_prod: "01"
environment: "virtual"
"#,
    )
    .unwrap();

    let output = ProcessCommand::new(env!("CARGO_BIN_EXE_kis"))
        .arg("--config")
        .arg(&config)
        .arg("config")
        .arg("--json")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"ok\": true"));
    assert!(stdout.contains("\"command\": \"config\""));
    assert!(stdout.contains("\"config_file\""));
    assert!(stdout.contains("\"environment\": \"virtual\""));
    assert!(stdout.contains("\"app_key\": \"abcd****\""));
}

#[test]
fn runs_config_command_with_config_flag_after_subcommand() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.yaml");
    fs::write(
        &config,
        r#"
app_key: "abcd1234"
app_secret: "secret"
account_no: "12345678"
account_prod: "01"
environment: "virtual"
"#,
    )
    .unwrap();

    let output = ProcessCommand::new(env!("CARGO_BIN_EXE_kis"))
        .arg("config")
        .arg("--config")
        .arg(&config)
        .arg("--json")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"ok\": true"));
    assert!(stdout.contains("\"command\": \"config\""));
    assert!(stdout.contains("\"config_file\""));
    assert!(stdout.contains("\"environment\": \"virtual\""));
}

#[test]
fn runs_config_command_with_env_flag_after_subcommand() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.yaml");
    fs::write(
        &config,
        r#"
app_key: "abcd1234"
app_secret: "secret"
account_no: "12345678"
account_prod: "01"
environment: "virtual"
"#,
    )
    .unwrap();

    let output = ProcessCommand::new(env!("CARGO_BIN_EXE_kis"))
        .arg("config")
        .arg("--config")
        .arg(&config)
        .arg("--env")
        .arg("real")
        .arg("--json")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"ok\": true"));
    assert!(stdout.contains("\"environment\": \"real\""));
    assert!(stdout.contains("https://openapi.koreainvestment.com:9443"));
}

#[test]
fn runs_config_command_without_home_environment_variable() {
    let output = ProcessCommand::new(env!("CARGO_BIN_EXE_kis"))
        .arg("config")
        .env_remove("HOME")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn runs_config_command_with_default_xdg_style_config_path() {
    let dir = tempdir().unwrap();
    let config_dir = dir.path().join(".config").join("kis");
    fs::create_dir_all(&config_dir).unwrap();

    let config = config_dir.join("config.yaml");
    fs::write(
        &config,
        r#"
app_key: "abcd1234"
app_secret: "secret"
account_no: "12345678"
account_prod: "01"
environment: "virtual"
"#,
    )
    .unwrap();

    let output = ProcessCommand::new(env!("CARGO_BIN_EXE_kis"))
        .arg("config")
        .arg("--json")
        .env("HOME", dir.path())
        .env_remove("KIS_APP_KEY")
        .env_remove("KIS_APP_SECRET")
        .env_remove("KIS_ACCOUNT_NO")
        .env_remove("KIS_ACCOUNT_PROD")
        .env_remove("KIS_ENVIRONMENT")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"ok\": true"));
    assert!(stdout.contains("\"config_file\""));
    assert!(stdout.contains(&config.display().to_string()));
    assert!(stdout.contains("\"app_key\": \"abcd****\""));
}

#[test]
fn runs_order_buy_dry_run_without_network() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.yaml");
    fs::write(
        &config,
        r#"
app_key: "abcd1234"
app_secret: "secret"
account_no: "12345678"
account_prod: "01"
environment: "virtual"
"#,
    )
    .unwrap();

    let output = ProcessCommand::new(env!("CARGO_BIN_EXE_kis"))
        .arg("--config")
        .arg(&config)
        .arg("order")
        .arg("buy")
        .arg("--stock")
        .arg("005930")
        .arg("--qty")
        .arg("1")
        .arg("--market")
        .arg("--dry-run")
        .arg("--output")
        .arg("json")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"ok\": true"));
    assert!(stdout.contains("\"command\": \"order\""));
    assert!(stdout.contains("\"action\": \"buy\""));
    assert!(stdout.contains("\"market\": \"domestic\""));
    assert!(stdout.contains("\"route\": \"regular\""));
    assert!(stdout.contains("\"tr_id\": \"VTTC0012U\""));
    assert!(stdout.contains("\"stock_code\": \"005930\""));
}

#[test]
fn writes_json_error_envelope_to_stdout() {
    let dir = tempdir().unwrap();
    let missing = dir.path().join("missing.yaml");

    let output = ProcessCommand::new(env!("CARGO_BIN_EXE_kis"))
        .arg("config")
        .arg("--config")
        .arg(&missing)
        .arg("--output")
        .arg("json")
        .output()
        .unwrap();

    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stdout.contains("\"ok\": false"));
    assert!(stdout.contains("\"command\": \"config\""));
    assert!(stdout.contains("\"kind\": \"config\""));
    assert!(stdout.contains("No such file"));
    assert!(
        stderr.trim().is_empty(),
        "stderr should be empty, got: {stderr}"
    );
}

#[test]
fn runs_config_command_with_quiet_flag() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.yaml");
    fs::write(
        &config,
        r#"
app_key: "abcd1234"
app_secret: "secret"
account_no: "12345678"
account_prod: "01"
environment: "virtual"
"#,
    )
    .unwrap();

    let output = ProcessCommand::new(env!("CARGO_BIN_EXE_kis"))
        .arg("config")
        .arg("--config")
        .arg(&config)
        .arg("--quiet")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.contains("Current configuration:"));
    assert!(stdout.contains("config file"));
    assert!(stdout.contains("environment"));
}
