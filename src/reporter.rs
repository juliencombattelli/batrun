use crate::error::Error;
use crate::test_executor::{ExecutionContext, TestCaseExecInfo};
use crate::test_suite::{TestCase, TestSuite};

pub trait Reporter {
    #[track_caller]
    fn notice(&self, message: &str) {
        self.notice_detailed(message, "");
    }
    #[track_caller]
    fn info(&self, message: &str) {
        self.info_detailed(message, "");
    }
    #[track_caller]
    fn warning(&self, message: &str) {
        self.warning_detailed(message, "");
    }
    #[track_caller]
    fn error(&self, message: &str) {
        self.error_detailed(message, "");
    }

    #[track_caller]
    fn notice_detailed(&self, message: &str, details: &str);
    #[track_caller]
    fn info_detailed(&self, message: &str, details: &str);
    #[track_caller]
    fn warning_detailed(&self, message: &str, details: &str);
    #[track_caller]
    fn error_detailed(&self, message: &str, details: &str);
    #[track_caller]
    fn error_from(&self, error: &Error);

    fn report_target_list(&self, test_suite: &TestSuite);
    fn report_test_list(&self, test_suite: &TestSuite);
    fn report_test_suite_time(&self);
    fn report_test_suite_execution_summary(
        &self,
        test_suite: &TestSuite,
        exec_context: &ExecutionContext,
    );
    fn report_total_time(&self);
    fn report_test_case_execution_started(
        &self,
        _test_case: &TestCase,
        _target: &str,
        _exec_info: &TestCaseExecInfo,
    ) {
    }
    fn report_test_case_execution_result(
        &self,
        test_case: &TestCase,
        target: &str,
        exec_info: &TestCaseExecInfo,
    );
}

pub mod human_friendly;
// pub mod json;
// pub mod logging;
// pub mod null;
