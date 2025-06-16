pub mod parallel;
pub mod round_robin;
pub mod sequential;

use crate::test_driver::TestDriver;
use crate::test_suite::status::{TestCaseStatus, TestSuiteStatus};
use crate::test_suite::visitor::{ShouldSkip, Visitor};
use crate::test_suite::{TestCase, TestSuite};
use crate::time::TimeInterval;

use std::collections::HashMap;

pub trait Executor<'tr> {
    fn execute(
        &self,
        test_driver: &'tr Box<(dyn TestDriver + 'static)>,
        test_suite: &'tr TestSuite,
        execution_contexts: &'tr mut [ExecutionContext],
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

pub struct ExecutionContext {
    target: String,
    status: TestSuiteStatus,
    exec_info: HashMap<TestCase, TestCaseExecInfo>,
}

impl<'tr> ExecutionContext {
    pub fn new(test_suite: &'tr TestSuite, target: String) -> Self {
        let mut exec_info = HashMap::<TestCase, TestCaseExecInfo>::new();
        Visitor::new(&test_suite).visit_all_ok(|tc, _| {
            exec_info.insert(tc.clone(), TestCaseExecInfo::new());
        });
        Self {
            target,
            status: TestSuiteStatus::NotRun,
            exec_info,
        }
    }

    pub fn run(
        &mut self,
        test_driver: &'tr Box<(dyn TestDriver + 'static)>,
        test_suite: &'tr TestSuite,
        test_case: &TestCase,
        should_skip: ShouldSkip,
    ) {
        let test_suite_dir = test_suite.path();
        let result = test_driver.run_test(test_suite_dir, &self.target, test_case);
        // UNWRAP: exec_info is initialized with all test cases so the key is guaranteed to exist
        let tc_exec_info = self.exec_info.get_mut(test_case).unwrap();
        tc_exec_info.duration.stop();
        match result {
            Ok(status) => tc_exec_info.set_status(status),
            _ => tc_exec_info.set_status(TestCaseStatus::Failed), // TODO what to do with the returned error?
        }
    }

    pub fn set_test_suite_status(&mut self, status: TestSuiteStatus) {
        self.status = status;
    }
}
