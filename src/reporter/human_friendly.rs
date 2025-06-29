use crate::error::Error;
use crate::reporter::Reporter;
use crate::test_executor::{ExecutionContext, TestCaseExecInfo};
use crate::test_suite::status::TestCaseStatus;
use crate::test_suite::visitor::Visitor;
use crate::test_suite::{TestCase, TestSuite};

use colored::{ColoredString, Colorize};

pub struct HumanFriendlyReporter {
    debug_enabled: bool,
}

impl HumanFriendlyReporter {
    pub fn new(debug_enabled: bool) -> Self {
        Self { debug_enabled }
    }

    // fn log_test_result(&self, result: &TestCaseStatus) {
    //     match result {
    //         TestCaseStatus::Failed => println!("{}", "FAILED".red()),
    //         TestCaseStatus::Passed => println!("{}", "PASSED".green()),
    //         TestCaseStatus::Skipped(_) => println!("{}", "SKIPPED".dimmed()),
    //         TestCaseStatus::DryRun => println!("{}", "DRYRUN".dimmed()),
    //         _ => {}
    //     }
    // }

    #[track_caller]
    fn print_with_details(&self, prefix: ColoredString, message: &str, details: &str) {
        println!("{}{}", prefix, message.bright_white());
        if !details.is_empty() {
            println!("  {}", details.white());
        }
        self.print_source_location();
    }

    #[track_caller]
    fn print_source_location(&self) {
        if self.debug_enabled {
            let source_location = format!("[from: {}]", std::panic::Location::caller());
            println!("{}", source_location.dimmed());
        };
    }
}

impl Reporter for HumanFriendlyReporter {
    fn report_target_list(&self, test_suite: &TestSuite) {
        println!(
            "{}",
            format!(
                "Targets supported by test suite `{}`",
                test_suite.path().display()
            )
            .bright_white()
        );
        for target in &test_suite.config().targets {
            println!("  {}", target.white());
        }
        println!("");
    }

    fn report_test_list(&self, test_suite: &TestSuite) {
        println!(
            "{}",
            format!(
                "Tests defined in test suite `{}`",
                test_suite.path().display()
            )
            .bright_white()
        );
        Visitor::new(&test_suite).visit_all_ok(|tc, _| {
            println!("  {}", tc.id().white());
        });
        println!("");
    }

    fn report_test_suite_time(&self) {}

    fn report_test_suite_execution_summary(
        &self,
        test_suite: &TestSuite,
        exec_context: &ExecutionContext,
    ) {
        println!(
            "{}",
            format!(
                "Test suite `{}` execution summary",
                test_suite.path().display()
            )
            .bright_white()
        );
        println!("  Target: {}", exec_context.target().white());
        println!(
            "  Status: {}",
            format!("{:?}", exec_context.status()).white()
        );
        let statistics = exec_context.get_statistics();
        println!(
            "  Statistics: {} passed, {} failed, {} runner failed, {} skipped",
            statistics.passed.to_string().green(),
            statistics.failed.to_string().red(),
            statistics.runner_failed.to_string().red(),
            statistics.skipped.to_string().dimmed(),
        );
        println!("");
    }

    fn report_total_time(&self) {}

    fn info_detailed(&self, message: &str, details: &str) {
        self.print_with_details("Info: ".cyan(), message, details)
    }

    fn warning_detailed(&self, message: &str, details: &str) {
        self.print_with_details("Warning: ".yellow(), message, details)
    }

    fn error_detailed(&self, message: &str, details: &str) {
        self.print_with_details("Error: ".red(), message, details)
    }

    fn error_from(&self, error: &Error) {
        let (message, details): (&str, &str) = match &error {
            Error::SuiteConfigIo(error) => (&error.to_string(), &error.source.to_string()),
            Error::InvalidSuiteConfig(error) => (&error.to_string(), &error.source.to_string()),
            Error::TestDriverIo(error) => (&error.to_string(), &error.source.to_string()),
            Error::TestFileExec(error) => (&error.to_string(), &error.details),
            _ => (&error.to_string(), ""),
        };
        self.error_detailed(&message, &details);
    }

    fn report_test_case_execution_started(
        &self,
        test_case: &TestCase,
        target: &str,
        _exec_info: &TestCaseExecInfo,
    ) {
        print!(
            "Running test case `{}` for target `{}`",
            test_case.id(),
            &target
        );
    }
    fn report_test_case_execution_result(
        &self,
        _test_case: &TestCase,
        _target: &str,
        exec_info: &TestCaseExecInfo,
    ) {
        println!(
            " {}",
            match exec_info.result() {
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
    }
}
