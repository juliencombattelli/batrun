use crate::test_suite::visitor::SkipReason;

#[derive(Debug)]
pub struct Statistics {
    passed: usize,
    failed: usize,
    skipped: usize,
}

#[derive(Debug)]
pub struct AbortReason(pub String);
impl AbortReason {
    pub fn new(reason: &str) -> Self {
        Self(String::from(reason))
    }
}

#[derive(Debug)]
pub enum TestSuiteStatus {
    NotRun,
    Running(Statistics),
    Aborted(Statistics, AbortReason),
    Finished(Statistics),
}

#[derive(Debug)]
pub enum TestCaseStatus {
    NotRun,
    Running,
    Failed,
    Passed,
    Skipped(SkipReason),
    DryRun,
}
