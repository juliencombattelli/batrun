use crate::error::{self, Result};

use serde::Deserialize;

use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct TestSuiteConfig {
    pub name: String,
    pub description: String,
    pub version: String,
    pub driver: String,
    #[serde(rename = "test-file-patterns", default)]
    pub test_file_patterns: Vec<String>,
    #[serde(rename = "global-fixture")]
    pub global_fixture: Option<String>,
    pub targets: Vec<String>,
}

impl TestSuiteConfig {
    pub fn load(test_suite_dir: &Path) -> Result<Self> {
        let config_path = test_suite_dir.join("test-suite.json");
        let mut file = File::open(&config_path).map_err(|io_err| error::kind::SuiteConfigIo {
            filename: config_path.to_path_buf(),
            source: io_err,
        })?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|io_err| error::kind::SuiteConfigIo {
                filename: config_path.to_path_buf(),
                source: io_err,
            })?;
        let config: TestSuiteConfig = serde_json::from_str(&contents).map_err(|serde_err| {
            error::kind::InvalidSuiteConfig {
                filename: config_path.to_path_buf(),
                source: serde_err,
            }
        })?;
        Ok(config)
    }
}
