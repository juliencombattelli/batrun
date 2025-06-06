pub mod parallel;
pub mod round_robin;
pub mod sequential;

pub mod utils; // TODO make private as executor utilities shall only be used by executor implementations

use crate::test_driver::TestDriver;
use crate::test_suite::{TestCase, TestCaseState, TestSuite};
use crate::time::TimeInterval;

use std::collections::HashMap;

pub trait Executor {
    fn execute(
        &self,
        execution_contexts: &[ExecutionContext],
        test_driver: &Box<(dyn TestDriver + 'static)>,
    );
}

#[derive(Debug)]
struct ExecStats {
    passed: usize,
    failed: usize,
    skipped: usize,
}

#[derive(Debug)]
struct AbortReason(String);

#[derive(Debug)]
enum TestSuiteState {
    NotRun,
    Running,
    Aborted(AbortReason),
    Finished(ExecStats),
}

#[derive(Debug)]
struct TestCaseExecInfo {
    state: TestCaseState,
    duration: TimeInterval,
}
impl TestCaseExecInfo {
    fn new() -> Self {
        Self {
            state: TestCaseState::NotRun,
            duration: TimeInterval::new(),
        }
    }
    pub fn set_status(&mut self, state: TestCaseState) {
        match state {
            TestCaseState::NotRun => {
                panic!("Test case status cannot be reset")
            }
            TestCaseState::Running => {
                self.state = state;
                self.duration = TimeInterval::new();
            }
            _ => {
                self.state = state;
                self.duration.stop();
            }
        }
    }
}

#[derive(Debug)]
pub struct ExecutionContext<'ts> {
    test_suite: &'ts TestSuite,
    target: String,
    state: TestSuiteState,
    exec_info: HashMap<TestCase, TestCaseExecInfo>,
}

impl<'ts> ExecutionContext<'ts> {
    pub fn new(test_suite: &'ts TestSuite, target: String) -> Self {
        let mut exec_info = HashMap::<TestCase, TestCaseExecInfo>::new();
        test_suite.visit(|tc| {
            exec_info.insert(tc.clone(), TestCaseExecInfo::new());
        });
        Self {
            test_suite,
            target,
            state: TestSuiteState::NotRun,
            exec_info,
        }
    }

    pub async fn execute(&self, test_driver: &Box<dyn TestDriver>) {
        self.test_suite
            .visit2(async |tc, should_skip| crate::test_suite::TestSuiteVisitResult::Ok)
            .await;
    }
}
