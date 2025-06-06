use crate::error::{self, Error, Result};
use crate::time::TimeInterval;

use serde::Deserialize;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
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

#[derive(Debug, Deserialize, Clone)]
pub struct TestSuiteConfig {
    pub name: String,
    pub description: String,
    pub version: String,
    pub driver: String,
    #[serde(rename = "test-file-pattern", default)]
    pub test_file_pattern: Vec<String>,
    #[serde(rename = "global-fixture")]
    pub global_fixture: Option<String>,
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

pub enum TestSuiteVisitResult {
    Ok,
    Err,
}

#[derive(Debug, Clone, Copy)]
pub enum ShouldSkip {
    Yes,
    No,
}

impl ShouldSkip {
    fn or(self, other: ShouldSkip) -> ShouldSkip {
        match (self, other) {
            (ShouldSkip::No, ShouldSkip::No) => ShouldSkip::No,
            _ => ShouldSkip::Yes,
        }
    }
}

#[derive(Debug)]
pub struct TestSuite {
    path: PathBuf,
    config: TestSuiteConfig,
    fixture: TestSuiteFixture,
    test_files: Vec<TestFile>,
    duration: TimeInterval,
}

impl TestSuite {
    pub fn new(
        path: &Path,
        config: TestSuiteConfig,
        test_files: Vec<TestFile>,
        fixture: TestSuiteFixture,
    ) -> Self {
        // TODO assert that test files and test cases are sorted
        Self {
            path: path.to_path_buf(),
            config,
            fixture,
            test_files,
            duration: TimeInterval::new(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn config(&self) -> &TestSuiteConfig {
        &self.config
    }

    pub fn fixture(&self) -> &TestSuiteFixture {
        &self.fixture
    }

    pub fn fixture_mut(&mut self) -> &mut TestSuiteFixture {
        &mut self.fixture
    }

    pub fn test_files(&self) -> &[TestFile] {
        &self.test_files
    }

    pub fn test_files_mut(&mut self) -> &mut [TestFile] {
        &mut self.test_files
    }

    pub fn visit<R, F>(&self, mut f: F)
    where
        F: FnMut(&TestCase) -> R,
    {
        if let Some(tc) = &self.fixture.setup_test_case {
            f(tc);
        }
        for test_file in &self.test_files {
            if let Some(tc) = &test_file.setup_test_case {
                f(tc);
            }
            for tc in &test_file.test_cases {
                f(tc);
            }
            if let Some(tc) = &test_file.teardown_test_case {
                f(tc);
            }
        }
        if let Some(tc) = &self.fixture.teardown_test_case {
            f(tc);
        }
    }

    pub async fn visit2(
        &self,
        mut f: impl AsyncFnMut(&TestCase, ShouldSkip) -> TestSuiteVisitResult,
    ) {
        let mut should_skip_suite = ShouldSkip::No;
        if let Some(tc) = &self.fixture.setup_test_case {
            if let TestSuiteVisitResult::Err = f(tc, should_skip_suite).await {
                should_skip_suite = ShouldSkip::Yes;
            }
        }
        for test_file in &self.test_files {
            let mut should_skip_file = ShouldSkip::No;
            if let Some(tc) = &test_file.setup_test_case {
                if let TestSuiteVisitResult::Err = f(tc, should_skip_suite).await {
                    should_skip_file = ShouldSkip::Yes;
                }
            }
            for tc in &test_file.test_cases {
                let _ = f(tc, should_skip_suite.or(should_skip_file)).await;
            }
            if let Some(tc) = &test_file.teardown_test_case {
                let _ = f(tc, should_skip_suite.or(should_skip_file)).await;
            }
        }
        if let Some(tc) = &self.fixture.teardown_test_case {
            f(tc, should_skip_suite).await;
        }
    }
}

#[derive(Debug)]
pub struct TestFile {
    pub setup_test_case: Option<TestCase>,
    pub teardown_test_case: Option<TestCase>,
    pub test_cases: Vec<TestCase>,
}

#[derive(Debug, Default)]
pub struct TestSuiteFixture {
    pub setup_test_case: Option<TestCase>,
    pub teardown_test_case: Option<TestCase>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TestCase {
    path: PathBuf,
    name: String,
}

impl TestCase {
    pub fn new(path: &Path, name: &str) -> Self {
        Self {
            path: path.to_path_buf(),
            name: name.to_string(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn id(&self) -> String {
        format!("{}::{}", self.path.display(), &self.name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TestCaseState {
    NotRun,
    Running,
    Failed,
    Passed,
    Skipped(String),
    DryRun,
}
