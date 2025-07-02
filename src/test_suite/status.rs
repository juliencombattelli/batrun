#[derive(Debug, Default, Clone)]
pub struct Statistics {
    pub passed: usize,
    pub failed: usize,
    pub runner_failed: usize,
    pub skipped: usize,
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
