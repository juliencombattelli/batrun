#[derive(Debug, Default, Clone)]
pub struct Statistics {
    pub passed: usize,
    pub failed: usize,
    pub runner_failed: usize,
    pub skipped: usize,
}

#[derive(Debug, Clone)]
pub struct AbortReason(pub String);
impl AbortReason {
    pub fn new(reason: &str) -> Self {
        Self(String::from(reason))
    }
}

#[derive(Debug, Clone)]
pub enum TestSuiteStatus {
    NotRun,
    Running(Statistics),
    Aborted(Statistics, AbortReason),
    Finished(Statistics),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SkipReason {
    TestCaseSpecificReason(String),
    TestCaseSetupError,
    TestSuiteSetupError,
}

#[derive(Debug, Clone)]
pub enum TestCaseStatus {
    NotRun,
    Running,
    Failed,
    Passed,
    Skipped(SkipReason),
    DryRun,
}
