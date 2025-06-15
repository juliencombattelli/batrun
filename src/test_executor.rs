pub mod parallel;
pub mod round_robin;
pub mod sequential;

mod utils;

use crate::test_driver::TestDriver;
use crate::test_suite::status::{TestCaseStatus, TestSuiteStatus};
use crate::test_suite::visitor::Visitor;
use crate::test_suite::{TestCase, TestSuite};
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
struct TestCaseExecInfo {
    status: TestCaseStatus,
    duration: TimeInterval,
}
impl TestCaseExecInfo {
    fn new() -> Self {
        Self {
            status: TestCaseStatus::NotRun,
            duration: TimeInterval::new(),
        }
    }
    pub fn set_status(&mut self, status: TestCaseStatus) {
        match status {
            TestCaseStatus::NotRun => {
                panic!("Test case status cannot be reset")
            }
            TestCaseStatus::Running => {
                self.status = status;
                self.duration = TimeInterval::new();
            }
            _ => {
                self.status = status;
                self.duration.stop();
            }
        }
    }
}

#[derive(Debug)]
pub struct ExecutionContext<'ts> {
    test_suite: &'ts TestSuite,
    target: String,
    status: TestSuiteStatus,
    exec_info: HashMap<TestCase, TestCaseExecInfo>,
}

impl<'ts> ExecutionContext<'ts> {
    pub fn new(test_suite: &'ts TestSuite, target: String) -> Self {
        let mut exec_info = HashMap::<TestCase, TestCaseExecInfo>::new();
        Visitor::new(&test_suite).visit_all_ok(|tc, _| {
            exec_info.insert(tc.clone(), TestCaseExecInfo::new());
        });
        Self {
            test_suite,
            target,
            status: TestSuiteStatus::NotRun,
            exec_info,
        }
    }
}
