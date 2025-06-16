use crate::error::Error;
use crate::test_suite::TestSuite;

pub trait Reporter {
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
    fn report_total_time(&self);
}

pub mod human_friendly;
// pub mod json;
// pub mod logging;
// pub mod null;
