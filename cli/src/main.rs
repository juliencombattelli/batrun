use batrun::error::Result;
use batrun::execution_strategy::ExecutionStrategy;
use batrun::settings::Settings;
use batrun::test_runner::TestRunner;
use batrun::time::format as format_duration;

use clap::Parser;

use std::path::PathBuf;
use std::time::Instant;

const DEFAULT_OUT_DIR: &str = "out";

use clap::builder::styling::{AnsiColor, Color, Style};

pub fn batrun_cli_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .usage(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Ansi(AnsiColor::Yellow))),
        )
        .header(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Ansi(AnsiColor::Yellow))),
        )
        .literal(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))))
        .invalid(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Red))),
        )
        .error(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Red))),
        )
        .valid(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Ansi(AnsiColor::Green))),
        )
        .placeholder(Style::new().fg_color(Some(Color::Ansi(AnsiColor::White))))
}

#[derive(Parser, Debug)]
#[clap(name = "batrun", styles = batrun_cli_styles(), version)]
struct Cli {
    /// Directory where the test suite is located
    #[arg(required = true, value_name = "TEST_SUITE_DIR")]
    test_suite: Vec<PathBuf>,

    /// Output directory for logs and data
    #[arg(short = 'o', long = "out-dir", default_value = DEFAULT_OUT_DIR)]
    out_dir: PathBuf,

    /// Targets to run the tests on; select all available targets if not provided
    #[arg(short = 't', long = "target", num_args(0..))]
    targets: Vec<String>,

    /// List targets supported by the specified test suite
    #[arg(short = 'L', long = "list-targets")]
    list_targets: bool,

    /// List tests available in the specified test suite
    #[arg(short = 'l', long = "list-tests")]
    list_tests: bool,

    /// Select the test cases execution strategy for each target
    #[arg(value_enum, short = 's', long = "exec-strategy", default_value_t = ExecutionStrategy::RoundRobin)]
    exec_strategy: ExecutionStrategy,

    /// Go through all tests but execute nothing
    #[arg(short = 'n', long = "dry-run")]
    dry_run: bool,

    /// Output additional logs helping to debug batrun itself
    #[arg(short = 'd', long = "debug")]
    debug: bool,

    /// Output the summary using a matrix format with test cases in rows and targets in columns
    #[arg(short = 'm', long = "matrix-summary")]
    matrix_summary: bool,
}

impl From<&Cli> for Settings {
    fn from(cli: &Cli) -> Self {
        Settings {
            test_suite_dirs: cli.test_suite.clone(),
            out_dir: cli.out_dir.clone(),
            targets: cli.targets.clone(),
            exec_strategy: cli.exec_strategy,
            dry_run: cli.dry_run,
            test_filter: None,
            debug: cli.debug,
            matrix_summary: cli.matrix_summary,
        }
    }
}

fn main_impl() -> Result<()> {
    let cli = Cli::parse();
    let mut test_runner = TestRunner::new(Settings::from(&cli))?;

    let start = Instant::now();
    {
        for test_suite_dir in test_runner.settings().test_suite_dirs.clone() {
            let mut run_tests = true;
            if cli.list_targets {
                run_tests = false;
                test_runner.list_targets(&test_suite_dir)?
            }
            if cli.list_tests {
                run_tests = false;
                test_runner.list_tests(&test_suite_dir)?
            }
            if run_tests {
                test_runner.run_tests(&test_suite_dir)?
            }
        }
    }
    let duration = start.elapsed();
    println!();
    println!("Time elapsed: {}", format_duration(duration));

    Ok(())
}

fn main() -> std::process::ExitCode {
    match main_impl() {
        Err(_) => std::process::ExitCode::FAILURE,
        Ok(_) => std::process::ExitCode::SUCCESS,
    }
}
