use super::TestSuite;

use crate::error::{Error, Result};

use std::collections::HashMap;
use std::path::{Path, PathBuf};

type TestSuiteMap = HashMap<PathBuf, TestSuite>;

pub struct TestSuiteRegistry {
    test_suites: TestSuiteMap,
}
impl TestSuiteRegistry {
    pub fn new() -> Self {
        Self {
            test_suites: TestSuiteMap::new(),
        }
    }

    pub fn get(&self, test_suite_dir: &Path) -> Result<&TestSuite> {
        let test_suite = self.test_suites.get(test_suite_dir);
        match test_suite {
            Some(test_suite) => Ok(test_suite),
            None => Err(Error::UnknownTestSuite(test_suite_dir.to_path_buf())),
        }
    }

    pub fn get_mut(&mut self, test_suite_dir: &Path) -> Result<&mut TestSuite> {
        let test_suite = self.test_suites.get_mut(test_suite_dir);
        match test_suite {
            Some(test_suite) => Ok(test_suite),
            None => Err(Error::UnknownTestSuite(test_suite_dir.to_path_buf())),
        }
    }

    pub fn insert(&mut self, test_suite_dir: &Path, test_suite: TestSuite) {
        self.test_suites
            .insert(test_suite_dir.to_path_buf(), test_suite);
    }
}
