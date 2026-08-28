use crate::error::Error;
use crate::reporter::Reporter;
use crate::test_executor::{ExecutionContext, TestCaseExecInfo};
use crate::test_suite::status::{SkipReason, Statistics, TestCaseStatus};
use crate::test_suite::{TestCase, TestSuite};

use serde_json::{Value, json};
use std::cell::RefCell;

impl JsonReporter {
    fn test_case_report(test_case: &TestCase, exec_info: &TestCaseExecInfo) -> Value {
        let (status, skip_reason, driver_output, error) = match exec_info.result() {
            Ok(output) => {
                let (status, skip_reason) = match &output.test_case_status {
                    TestCaseStatus::NotRun => ("not_run", None),
                    TestCaseStatus::Running => ("running", None),
                    TestCaseStatus::Failed => ("failed", None),
                    TestCaseStatus::Passed => ("passed", None),
                    TestCaseStatus::Skipped(reason) => ("skipped", Some(skip_reason(reason))),
                    TestCaseStatus::DryRun => ("dry_run", None),
                };
                (
                    status,
                    skip_reason,
                    output
                        .driver_output
                        .as_ref()
                        .map(|driver_output| driver_output.output()),
                    None,
                )
            }
            Err(error) => ("runner_failed", None, None, Some(error.to_string())),
        };

        json!({
            "id": test_case.id(),
            "path": test_case.path().display().to_string(),
            "name": test_case.name(),
            "status": status,
            "skip_reason": skip_reason,
            "driver_output": driver_output,
            "error": error,
        })
    }

    fn target_report(exec_context: &ExecutionContext) -> Value {
        let statistics = exec_context.get_statistics();
        let mut test_cases = exec_context
            .exec_info()
            .iter()
            .map(|(test_case, exec_info)| Self::test_case_report(test_case, exec_info))
            .collect::<Vec<_>>();
        test_cases.sort_by(|left, right| left["id"].as_str().cmp(&right["id"].as_str()));

        json!({
            "target": exec_context.target(),
            "statistics": statistics_report(&statistics),
            "test_cases": test_cases,
        })
    }
}

fn skip_reason(reason: &SkipReason) -> String {
    match reason {
        SkipReason::TestCaseSpecificReason(reason) => reason.clone(),
        SkipReason::TestCaseSetupError => "test_case_setup_error".to_string(),
        SkipReason::TestSuiteSetupError => "test_suite_setup_error".to_string(),
    }
}

fn statistics_report(statistics: &Statistics) -> Value {
    json!({
        "passed": statistics.passed,
        "failed": statistics.failed,
        "runner_failed": statistics.runner_failed,
        "skipped": statistics.skipped,
        "total": statistics.total(),
    })
}

pub(crate) struct JsonReporter {
    suite_reports: RefCell<Vec<Value>>,
}

impl JsonReporter {
    pub(crate) fn new() -> Self {
        Self {
            suite_reports: RefCell::new(Vec::new()),
        }
    }
}

impl Reporter for JsonReporter {
    fn notice_detailed(&self, _message: &str, _details: &str) {}

    fn info_detailed(&self, _message: &str, _details: &str) {}

    fn warning_detailed(&self, _message: &str, _details: &str) {}

    fn error_detailed(&self, _message: &str, _details: &str) {}

    fn error_from(&self, _error: &Error) {}

    fn report_target_list(&self, _test_suite: &TestSuite) {}

    fn report_test_list(&self, _test_suite: &TestSuite) {}

    fn report_test_suite_time(&self) {}

    fn report_test_suite_execution_summary(
        &self,
        test_suite: &TestSuite,
        exec_contexts: &[ExecutionContext],
    ) {
        self.suite_reports.borrow_mut().push(json!({
            "name": test_suite.config().name,
            "path": test_suite.path().display().to_string(),
            "targets": exec_contexts
                .iter()
                .map(Self::target_report)
                .collect::<Vec<_>>(),
        }));
    }

    fn report_total_time(&self) {
        let suite_reports = std::mem::take(&mut *self.suite_reports.borrow_mut());
        let report = json!({ "test_suites": suite_reports });
        println!(
            "{}",
            serde_json::to_string_pretty(&report).expect("JSON report should be serializable")
        );
    }

    fn report_test_case_execution_result(
        &self,
        _test_case: &TestCase,
        _target: &str,
        _exec_info: &TestCaseExecInfo,
    ) {
    }
}
