pub mod parallel;
pub mod round_robin;
pub mod sequential;

use crate::error::{self, Result};
use crate::test_driver::TestDriver;
use crate::test_suite::status::{TestCaseStatus, TestSuiteStatus};
use crate::test_suite::visitor::{ShouldSkip, Visitor};
use crate::test_suite::{TestCase, TestSuite};
use crate::time::TimeInterval;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use colored::Colorize; // temporary

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
    out_dir: PathBuf,
}
impl TestCaseExecInfo {
    fn new(out_dir: PathBuf) -> Self {
        Self {
            status: TestCaseStatus::NotRun,
            duration: TimeInterval::new(),
            out_dir,
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
        test_driver: &'tr Box<(dyn TestDriver + 'static)>,
        test_suite: &'tr TestSuite,
        test_case: &TestCase,
        should_skip: ShouldSkip,
    ) -> std::result::Result<(), ()> {
        let test_suite_dir = test_suite.path();
        // UNWRAP: exec_info is initialized with all test cases so the key is guaranteed to exist
        let tc_exec_info = self.exec_info.get_mut(test_case).unwrap();

        // TODO use the reporter to report status
        print!(
            "Running test case `{}` for target `{}`",
            test_case.id(),
            &self.target
        );
        tc_exec_info.duration = TimeInterval::new();
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
        tc_exec_info.duration.stop();

        println!(
            " {}",
            match result {
                Err(_) => "RUNNER_FAILED".red().to_string(),
                Ok(TestCaseStatus::Failed) => "FAILED".red().to_string(),
                Ok(TestCaseStatus::Passed) => "PASSED".green().to_string(),
                Ok(TestCaseStatus::Skipped(reason)) =>
                    format!("{} (reason: {:?})", "SKIPPED".dimmed(), reason),
                Ok(TestCaseStatus::DryRun) => "DRYRUN".dimmed().to_string(),
                Ok(TestCaseStatus::NotRun) => "NOTRUN".dimmed().to_string(),
                Ok(TestCaseStatus::Running) => "RUNNING".dimmed().to_string(),
            }
        );

        match result {
            Ok(status) => tc_exec_info.set_status(status),
            _ => tc_exec_info.set_status(TestCaseStatus::Failed), // TODO what to do with the returned error?
        }

        match tc_exec_info.status {
            TestCaseStatus::Failed => Err(()),
            _ => Ok(()),
        }
    }

    pub fn set_test_suite_status(&mut self, status: TestSuiteStatus) {
        self.status = status;
    }
}
