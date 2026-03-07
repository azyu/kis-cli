mod runtime;

use std::io::{self, Write, stderr};
use std::process::ExitCode;

use clap::Parser;
use kis_cli::cli::Cli;

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    let output_json = cli.output_format().is_json();
    let command_name = cli.command.name();
    let mut stdout = io::stdout();

    match runtime::run(cli, &mut stdout).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            if output_json {
                if runtime::write_json_error(&mut stdout, command_name, &err).is_err() {
                    eprintln!("{err:#}");
                    let _ = stderr().flush();
                }
            } else {
                eprintln!("{err:#}");
                let _ = stderr().flush();
            }
            ExitCode::FAILURE
        }
    }
}
