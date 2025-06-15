#[derive(Debug)]
pub struct Statistics {
    passed: usize,
    failed: usize,
    skipped: usize,
}

#[derive(Debug)]
pub struct Reason(String);

#[derive(Debug)]
pub enum TestSuiteStatus {
    NotRun,
    Running(Statistics),
    Aborted(Statistics, Reason),
    Finished(Statistics),
}

#[derive(Debug)]
pub enum TestCaseStatus {
    NotRun,
    Running,
    Failed,
    Passed,
    Skipped(Reason),
    DryRun,
}
