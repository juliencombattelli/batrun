use crate::error::{Error, Result};
use crate::test_suite::config::TestSuiteConfig;
use crate::test_suite::status::TestCaseStatus;
use crate::test_suite::{TestCase, TestSuite};

use std::collections::HashMap;
use std::fmt::Display;
use std::path::{Path, PathBuf};

pub trait DriverOutput: Display {}

pub struct RunTestOutput {
    pub test_case_status: TestCaseStatus,
    pub driver_output: Option<Box<dyn DriverOutput>>,
}

pub trait TestDriver {
    fn test_file_patterns_default(&self) -> Vec<String>;

    /// Walk through all files in the test suite and return a list of test cases found
    /// The list must be sorted by files and by execution order within a file (setup,
    /// then test functions, and finally teardown)
    fn discover_tests(
        &self,
        test_suite_dir: &Path,
        test_suite_config: &TestSuiteConfig,
    ) -> Result<TestSuite>;

    fn run_test(
        &self,
        test_suite_dir: &Path,
        test_suite_config: &TestSuiteConfig,
        target: &str,
        test_case: &TestCase,
        test_case_out_dir: &Path,
    ) -> Result<RunTestOutput>;

    fn test_file_pattern_or_default(&self, test_suite_config: &TestSuiteConfig) -> Vec<String> {
        if test_suite_config.test_file_patterns.is_empty() {
            self.test_file_patterns_default()
        } else {
            test_suite_config.test_file_patterns.clone()
        }
    }

    fn matches_file_pattern(&self, filename: &Path, test_suite_config: &TestSuiteConfig) -> bool {
        filename.is_file()
            && self
                .test_file_pattern_or_default(&test_suite_config)
                .iter()
                .map(AsRef::as_ref)
                .map(glob::Pattern::new)
                .any(|pattern| {
                    pattern
                        .expect("provided string should be a valid glob pattern")
                        .matches_path(filename)
                })
    }

    fn matches_global_fixture_file(
        &self,
        filename: &Path,
        test_suite_dir: &Path,
        test_suite_config: &TestSuiteConfig,
    ) -> bool {
        match &test_suite_config.global_fixture {
            Some(global_fixture_file) => {
                filename == test_suite_dir.join(Path::new(global_fixture_file))
            }
            None => false,
        }
    }

    /// Returns the list of all test files paths in the test suite.
    /// The paths returned are relative to the test suite root directory (where the json config file
    /// is located).
    fn discover_test_files(
        &self,
        test_suite_dir: &Path,
        test_suite_config: &TestSuiteConfig,
    ) -> Vec<PathBuf> {
        let mut test_files = Vec::new();
        for entry in walkdir::WalkDir::new(test_suite_dir) {
            if let Ok(entry) = entry {
                let path = entry.path();
                if self.matches_file_pattern(path, test_suite_config) {
                    if !self.matches_global_fixture_file(path, test_suite_dir, test_suite_config) {
                        match path.strip_prefix(test_suite_dir) {
                            Ok(local_path) => test_files.push(local_path.to_path_buf()),
                            // As we are retrieving only subdirs of the test suite dir, making the
                            // subdirs absolute paths relative to the parent test suite dir should
                            // never fail
                            Err(_) => panic!("This should not happen"),
                        }
                    }
                }
            }
        }
        test_files.sort();
        test_files
    }
}

mod bash;

use bash::BashTestDriver;

type TestDriverMap = HashMap<&'static str, Box<dyn TestDriver>>;

pub(crate) struct TestDriverRegistry {
    test_drivers: TestDriverMap,
}
impl TestDriverRegistry {
    pub(crate) fn new() -> Self {
        let mut test_drivers = TestDriverMap::new();
        test_drivers.insert("bash", Box::new(BashTestDriver::new()));
        Self { test_drivers }
    }

    pub(crate) fn get(&self, driver_name: &str) -> Result<&Box<dyn TestDriver>> {
        let test_driver = self.test_drivers.get(driver_name);
        match test_driver {
            Some(test_driver) => Ok(test_driver),
            None => Err(Error::UnknownTestDriver(driver_name.to_string())),
        }
    }
}
