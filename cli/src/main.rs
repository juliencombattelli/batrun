use batrun::time::format as format_duration;
use batrun::{ExecutionStrategy, Result, Settings, TestRunner};

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
#[clap(name = "Bash test runner", styles = batrun_cli_styles())]
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
        }
    }
}

fn main_impl() -> Result<()> {
    let cli = Cli::parse();
    let mut test_runner = TestRunner::new(Settings::from(&cli))?;

    // let mut runner_actions = Vec::new();
    // if cli.list_targets {
    //     runner_actions.push(TestRunner::list_targets);
    // }
    // if cli.list_tests {
    //     runner_actions.push(TestRunner::list_tests);
    // }
    // if runner_actions.is_empty() {
    //     runner_actions.push(TestRunner::run_tests);
    // }

    let start = Instant::now();
    {
        for test_suite_dir in test_runner.settings().test_suite_dirs.clone() {
            if cli.list_tests {
                test_runner.list_tests(&test_suite_dir)?
            } else {
                test_runner.run_tests(&test_suite_dir)?
            }
        }

        // for test_suite_dir in test_runner.settings().test_suite_dirs.clone() {
        //     test_runner.add_test_suite(test_suite_dir.clone());
        //     println!("Test suite directory: {:?}", test_suite_dir);

        //     if let Some(test_suite) = test_runner.test_suites().get(&test_suite_dir) {
        //         println!("Test suite config: {:?}", test_suite.config());
        //     }
        // }
    }
    let duration = start.elapsed();
    println!("Time elapsed: {}", format_duration(duration));

    Ok(())
}

fn main() -> std::process::ExitCode {
    match main_impl() {
        Err(_) => std::process::ExitCode::FAILURE,
        Ok(_) => std::process::ExitCode::SUCCESS,
    }
}
