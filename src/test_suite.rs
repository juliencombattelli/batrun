pub mod config;
pub mod registry;
pub mod status;
pub mod visitor;

use self::config::TestSuiteConfig;

use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct TestSuite {
    path: PathBuf,
    config: TestSuiteConfig,
    fixture: TestSuiteFixture,
    test_files: Vec<TestFile>,
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

    // pub async fn visit_async(
    //     &self,
    //     mut f: impl AsyncFnMut(&TestCase, ShouldSkip) -> TestSuiteVisitResult,
    // ) {
    //     let mut should_skip_suite = ShouldSkip::No;
    //     if let Some(tc) = &self.fixture.setup_test_case {
    //         if let TestSuiteVisitResult::Err = f(tc, should_skip_suite).await {
    //             should_skip_suite = ShouldSkip::Yes;
    //         }
    //     }
    //     for test_file in &self.test_files {
    //         let mut should_skip_file = ShouldSkip::No;
    //         if let Some(tc) = &test_file.setup_test_case {
    //             if let TestSuiteVisitResult::Err = f(tc, should_skip_suite).await {
    //                 should_skip_file = ShouldSkip::Yes;
    //             }
    //         }
    //         for tc in &test_file.test_cases {
    //             let _ = f(tc, should_skip_suite.or(should_skip_file)).await;
    //         }
    //         if let Some(tc) = &test_file.teardown_test_case {
    //             let _ = f(tc, should_skip_suite.or(should_skip_file)).await;
    //         }
    //     }
    //     if let Some(tc) = &self.fixture.teardown_test_case {
    //         f(tc, should_skip_suite).await;
    //     }
    // }
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
