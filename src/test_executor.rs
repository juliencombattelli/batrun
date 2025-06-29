pub mod parallel;
pub mod round_robin;
pub mod sequential;

use crate::error::{self, Result};
use crate::reporter::Reporter;
use crate::test_driver::TestDriver;
use crate::test_suite::status::{Statistics, TestCaseStatus, TestSuiteStatus};
use crate::test_suite::visitor::{ShouldSkip, Visitor};
use crate::test_suite::{TestCase, TestSuite};
use crate::time::TimeInterval;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub trait Executor<'tr> {
    fn execute(
        &self,
        reporter: &'tr Box<dyn Reporter>,
        test_driver: &'tr Box<dyn TestDriver>,
        test_suite: &'tr TestSuite,
        exec_contexts: &'tr mut [ExecutionContext],
    );
}

#[derive(Debug)]
pub struct TestCaseExecInfo {
    result: Result<TestCaseStatus>,
    duration: TimeInterval,
    out_dir: PathBuf,
}
impl TestCaseExecInfo {
    fn new(out_dir: PathBuf) -> Self {
        Self {
            result: Ok(TestCaseStatus::NotRun),
            duration: TimeInterval::new(),
            out_dir,
        }
    }
    pub fn set_result(&mut self, result: Result<TestCaseStatus>) {
        match result {
            Ok(TestCaseStatus::NotRun) => {
                panic!("Test case status cannot be reset")
            }
            Ok(TestCaseStatus::Running) => {
                self.result = result;
                self.duration = TimeInterval::new();
            }
            _ => {
                self.result = result;
                self.duration.stop();
            }
        }
    }
    pub fn result(&self) -> &Result<TestCaseStatus> {
        &self.result
    }
}

pub struct ExecutionContext {
    target: String,
    status: TestSuiteStatus,
    exec_info: HashMap<TestCase, TestCaseExecInfo>,
}

impl<'tr> ExecutionContext {
    pub fn new(test_suite: &'tr TestSuite, target: String, out_dir: &Path) -> Self {
        let mut exec_info = HashMap::<TestCase, TestCaseExecInfo>::new();
        Visitor::new(&test_suite).visit_all_ok(|tc, _| {
            let out_dir_result = Self::prepare_test_case_out_dir(out_dir, &target, tc);
            match out_dir_result {
                Ok(out_dir) => exec_info.insert(tc.clone(), TestCaseExecInfo::new(out_dir)),
                Err(err) => panic!("{:?}", err),
            };
        });
        Self {
            target,
            status: TestSuiteStatus::NotRun,
            exec_info,
        }
    }

    pub fn target(&self) -> &str {
        &self.target
    }

    pub fn status(&self) -> TestSuiteStatus {
        self.status.clone()
    }

    pub fn prepare_test_case_out_dir(
        global_out_dir: &Path,
        target: &str,
        test_case: &TestCase,
    ) -> Result<PathBuf> {
        let mut test_case_out_dir = global_out_dir.to_path_buf();
        test_case_out_dir.push(target);
        test_case_out_dir.push(test_case.path());

        if !test_case_out_dir.exists() {
            std::fs::create_dir_all(&test_case_out_dir).map_err(|io_err| {
                error::kind::SuiteConfigIo {
                    filename: test_case_out_dir.clone(),
                    source: io_err,
                }
            })?;
        }

        Ok(test_case_out_dir)
    }

    pub fn run(
        &mut self,
        reporter: &'tr Box<dyn Reporter>,
        test_driver: &'tr Box<dyn TestDriver>,
        test_suite: &'tr TestSuite,
        test_case: &TestCase,
        should_skip: ShouldSkip,
    ) -> std::result::Result<(), ()> {
        let test_suite_dir = test_suite.path();
        // UNWRAP: exec_info is initialized with all test cases so the key is guaranteed to exist
        let tc_exec_info = self.exec_info.get_mut(test_case).unwrap();

        tc_exec_info.set_result(Ok(TestCaseStatus::Running));
        reporter.report_test_case_execution_started(&test_case, &self.target, &tc_exec_info);

        let result = {
            if let ShouldSkip::Yes(reason) = should_skip {
                Ok(TestCaseStatus::Skipped(reason))
            } else {
                test_driver.run_test(
                    test_suite_dir,
                    test_suite.config(),
                    &self.target,
                    test_case,
                    &tc_exec_info.out_dir,
                )
            }
        };

        tc_exec_info.set_result(result);
        reporter.report_test_case_execution_result(&test_case, &self.target, &tc_exec_info);

        match tc_exec_info.result {
            Err(_) | Ok(TestCaseStatus::Failed) => Err(()),
            _ => Ok(()),
        }
    }

    pub fn set_test_suite_status(&mut self, status: TestSuiteStatus) {
        self.status = status;
    }

    pub fn get_statistics(&self) -> Statistics {
        let mut stats = Statistics::default();

        for exec_info in self.exec_info.values() {
            match exec_info.result {
                Ok(TestCaseStatus::Passed) => stats.passed += 1,
                Ok(TestCaseStatus::Failed) => stats.failed += 1,
                Ok(TestCaseStatus::Skipped(_) | TestCaseStatus::DryRun) => stats.skipped += 1,
                Err(_) => stats.runner_failed += 1,
                _ => {}
            }
        }

        stats
    }
}
