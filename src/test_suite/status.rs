#[derive(Debug, Default, Clone)]
pub struct Statistics {
    pub passed: usize,
    pub failed: usize,
    pub runner_failed: usize,
    pub skipped: usize,
}
impl Statistics {
    pub fn total(&self) -> usize {
        self.passed + self.failed + self.runner_failed + self.skipped
    }

    pub fn max(&self) -> usize {
        // Safeguard in case of change of struct fields
        match *self {
            Statistics {
                passed,
                failed,
                runner_failed,
                skipped,
            } => [passed, failed, runner_failed, skipped]
                .into_iter()
                .max()
                .unwrap_or(0),
        }
    }
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
