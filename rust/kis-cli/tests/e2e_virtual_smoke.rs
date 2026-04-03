use std::env;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;

use serde_json::Value;

struct VirtualSmokeContext {
    config: PathBuf,
    stock: String,
}

impl VirtualSmokeContext {
    fn load() -> Option<Self> {
        if env::var("KIS_E2E_VIRTUAL").ok().as_deref() != Some("1") {
            eprintln!(
                "skipping virtual smoke: set KIS_E2E_VIRTUAL=1 and KIS_E2E_VIRTUAL_CONFIG=/path/to/config.yaml"
            );
            return None;
        }

        let config = env::var("KIS_E2E_VIRTUAL_CONFIG")
            .map(PathBuf::from)
            .expect("KIS_E2E_VIRTUAL_CONFIG is required when KIS_E2E_VIRTUAL=1");

        assert!(
            config.is_file(),
            "KIS_E2E_VIRTUAL_CONFIG must point to an existing file: {}",
            config.display()
        );

        let stock = env::var("KIS_E2E_VIRTUAL_STOCK").unwrap_or_else(|_| "005930".to_string());

        Some(Self { config, stock })
    }

    fn run_json(&self, args: &[&str]) -> Value {
        let output = ProcessCommand::new(env!("CARGO_BIN_EXE_kis"))
            .arg("--config")
            .arg(&self.config)
            .arg("--env")
            .arg("virtual")
            .arg("--json")
            .args(args)
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        serde_json::from_slice(&output.stdout).unwrap()
    }

    fn run_json_failure(&self, args: &[&str]) -> Value {
        let output = ProcessCommand::new(env!("CARGO_BIN_EXE_kis"))
            .arg("--config")
            .arg(&self.config)
            .arg("--env")
            .arg("virtual")
            .arg("--json")
            .args(args)
            .output()
            .unwrap();

        assert!(
            !output.status.success(),
            "stdout: {}",
            String::from_utf8_lossy(&output.stdout)
        );

        serde_json::from_slice(&output.stdout).unwrap()
    }
}

fn assert_success_command(value: &Value, command: &str) {
    assert_eq!(value["ok"], true, "response: {value}");
    assert_eq!(value["command"], command, "response: {value}");
}

fn assert_error_command(value: &Value, command: &str) {
    assert_eq!(value["ok"], false, "response: {value}");
    assert_eq!(value["command"], command, "response: {value}");
    assert!(
        value["error"]["message"]
            .as_str()
            .is_some_and(|message| !message.is_empty()),
        "response: {value}"
    );
}

fn assert_error_message_contains(value: &Value, needle: &str) {
    let message = value["error"]["message"]
        .as_str()
        .expect("error message should be a string");
    assert!(
        message.contains(needle),
        "expected error message to contain {needle:?}, got: {message}"
    );
}

#[test]
#[ignore = "opt-in virtual smoke; set KIS_E2E_VIRTUAL=1 and KIS_E2E_VIRTUAL_CONFIG"]
fn runs_virtual_config_command() {
    let Some(ctx) = VirtualSmokeContext::load() else {
        return;
    };

    let value = ctx.run_json(&["config"]);

    assert_success_command(&value, "config");
    assert_eq!(value["data"]["environment"], "virtual");
    assert!(value["data"]["config_file"].is_string());
}

#[test]
#[ignore = "opt-in virtual smoke; set KIS_E2E_VIRTUAL=1 and KIS_E2E_VIRTUAL_CONFIG"]
fn runs_virtual_price_commands() {
    let Some(ctx) = VirtualSmokeContext::load() else {
        return;
    };

    let current = ctx.run_json(&["price", &ctx.stock]);
    assert_success_command(&current, "price");
    assert!(
        current["data"]["stck_prpr"].is_string(),
        "response: {current}"
    );

    let daily = ctx.run_json(&["price", &ctx.stock, "--daily"]);
    assert_success_command(&daily, "price");
    let items = daily["data"]
        .as_array()
        .expect("daily price data should be an array");
    assert!(!items.is_empty(), "response: {daily}");
}

#[test]
#[ignore = "opt-in virtual smoke; set KIS_E2E_VIRTUAL=1 and KIS_E2E_VIRTUAL_CONFIG"]
fn runs_virtual_chart_daily_command() {
    let Some(ctx) = VirtualSmokeContext::load() else {
        return;
    };

    let value = ctx.run_json(&[
        "chart", "daily", &ctx.stock, "--start", "20260301", "--end", "20260324",
    ]);

    assert_success_command(&value, "chart");
    let items = value["data"]
        .as_array()
        .expect("daily chart data should be an array");
    assert!(!items.is_empty(), "response: {value}");
}

#[test]
#[ignore = "opt-in virtual smoke; set KIS_E2E_VIRTUAL=1 and KIS_E2E_VIRTUAL_CONFIG"]
fn virtual_known_blockers_surface_structured_errors() {
    let Some(ctx) = VirtualSmokeContext::load() else {
        return;
    };

    let quote = ctx.run_json_failure(&["quote", "ask", &ctx.stock]);
    assert_error_command(&quote, "quote");
    assert_error_message_contains(&quote, "inquire-asking-price");

    let holiday = ctx.run_json_failure(&["market", "holiday", "20260324"]);
    assert_error_command(&holiday, "market");
    assert_eq!(holiday["error"]["code"], "EGW2004", "response: {holiday}");

    let search = ctx.run_json_failure(&["info", "search", "삼성전자"]);
    assert_error_command(&search, "info");
    assert_eq!(search["error"]["code"], "EGW2004", "response: {search}");

    let news = ctx.run_json_failure(&["info", "news", &ctx.stock]);
    assert_error_command(&news, "info");
    assert_error_message_contains(&news, "OPSQ0002");

    let opinion = ctx.run_json_failure(&["info", "opinion", &ctx.stock]);
    assert_error_command(&opinion, "info");
    assert_error_message_contains(&opinion, "OPSQ0002");

    let detail = ctx.run_json_failure(&["info", "detail", "AAPL", "--exchange", "NAS"]);
    assert_error_command(&detail, "info");
    assert_error_message_contains(&detail, "search-info");
}
