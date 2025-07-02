use crate::execution_strategy::ExecutionStrategy;

use std::path::PathBuf;

#[derive(Debug)]
pub struct Settings {
    pub test_suite_dirs: Vec<PathBuf>,
    pub out_dir: PathBuf,
    pub targets: Vec<String>,
    pub exec_strategy: ExecutionStrategy,
    pub dry_run: bool,
    pub test_filter: Option<String>,
    pub debug: bool,
    pub matrix_summary: bool,
}
