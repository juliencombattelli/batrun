use crate::execution_strategy::ExecutionStrategy;

use std::path::PathBuf;

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum OutputFormat {
    /// Human readable, pretty-printed output format
    Human,
    /// Machine-readable output format
    Json,
}

#[derive(Debug)]
pub struct Settings {
    pub test_suite_dirs: Vec<PathBuf>,
    pub out_dir: PathBuf,
    pub targets: Vec<String>,
    pub exec_strategy: ExecutionStrategy,
    pub dry_run: bool,
    pub test_filter: Option<String>,
    pub debug: bool,
    pub hide_error_details: bool,
    pub matrix_summary: bool,
    pub output_format: OutputFormat,
}
