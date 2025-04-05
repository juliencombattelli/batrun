use clap::{Parser};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, ExitStatus};
use std::time::SystemTime;

const TEST_FN_SETUP: &str = "setup";
const TEST_FN_TEARDOWN: &str = "teardown";
const TEST_FN_PREFIX: &str = "test_";
const GLOBAL_TEST_FILE: &str = "tests.sh";

/// Rust test runner for bash scripts
#[derive(Parser, Debug)]
#[command(name = "batrun")]
#[command(about = "Run tests written in bash on specified devices", long_about = None)]
struct Config {
    /// Output directory for logs and data
    #[arg(short, long, value_name = "DIR")]
    out_dir: PathBuf,

    /// Device to run the tests on (can be repeated)
    #[arg(short, long, value_name = "DEVICE")]
    devices: Vec<String>,

    /// Directory where the tests are located
    #[arg(long, value_name = "DIR", default_value = "tests")]
    tests_dir: PathBuf,

    /// Comma-separated list of tests to run
    #[arg(short, long, value_name = "TESTS")]
    tests: Option<String>,

    /// Go through all tests but execute nothing
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    dry_run: bool,

    /// List tests available in the test directory
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    list_tests: bool,
}

fn main() {
    let config = Config::parse();

    if config.list_tests {
        list_tests(&config.tests_dir);
        return;
    }

    if config.devices.is_empty() {
        eprintln!("ERROR: No device specified.");
        std::process::exit(1);
    }

    if config.out_dir.as_os_str().is_empty() {
        eprintln!("ERROR: No output directory specified.");
        std::process::exit(1);
    }

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let out_dir = config.out_dir.join(format!("{}", timestamp));
    fs::create_dir_all(&out_dir).expect("Failed to create output directory");

    validate_test_dir(&config.tests_dir);

    run_all_tests(&config, &out_dir);
}

fn validate_test_dir(tests_dir: &Path) {
    let global_test_file = tests_dir.join(GLOBAL_TEST_FILE);
    if !global_test_file.exists() {
        eprintln!(
            "ERROR: Global test file '{}' not found in {:?}.",
            GLOBAL_TEST_FILE, tests_dir
        );
        std::process::exit(1);
    }
}

fn list_tests(tests_dir: &Path) {
    let test_files = find_test_files(tests_dir);
    println!("Tests defined in test suite '{:?}':", tests_dir);
    for test_file in test_files {
        println!("  {}", test_file.file_stem().unwrap().to_string_lossy());
    }
}

fn find_test_files(tests_dir: &Path) -> Vec<PathBuf> {
    let mut test_files = Vec::new();
    if let Ok(entries) = fs::read_dir(tests_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("sh") && path != tests_dir.join(GLOBAL_TEST_FILE) {
                test_files.push(path);
            }
        }
    }
    test_files.sort();
    test_files
}

fn run_all_tests(config: &Config, out_dir: &Path) {
    let test_files = find_test_files(&config.tests_dir);
    let total_tests = test_files.len() * config.devices.len();
    let mut stats = HashMap::from([
        ("total", total_tests),
        ("passed", 0),
        ("failed", 0),
        ("skipped", 0),
    ]);

    for test_file in test_files {
        for device in &config.devices {
            if config.dry_run {
                println!("SKIPPED: {} on {}", test_file.display(), device);
                *stats.get_mut("skipped").unwrap() += 1;
                continue;
            }

            let log_file = out_dir.join(format!(
                "{}_{}.log",
                test_file.file_stem().unwrap().to_string_lossy(),
                device
            ));
            match run_test_case(&test_file, device, &log_file) {
                Ok(status) if status.success() => {
                    println!("PASSED: {} on {}", test_file.display(), device);
                    *stats.get_mut("passed").unwrap() += 1;
                }
                _ => {
                    println!("FAILED: {} on {}", test_file.display(), device);
                    *stats.get_mut("failed").unwrap() += 1;
                }
            }
        }
    }

    println!(
        "SUMMARY: Total: {}, Passed: {}, Failed: {}, Skipped: {}",
        stats["total"], stats["passed"], stats["failed"], stats["skipped"]
    );
}

fn run_test_case(test_file: &Path, device: &str, log_file: &Path) -> Result<ExitStatus, std::io::Error> {
    let output = ProcessCommand::new("bash")
        .arg("-e")
        .arg("-u")
        .arg("-o")
        .arg("pipefail")
        .arg(test_file)
        .env("DEVICE", device)
        .output()?;

    fs::write(log_file, &output.stdout)?;
    fs::write(log_file, &output.stderr)?;

    Ok(output.status)
}