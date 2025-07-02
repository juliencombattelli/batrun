use crate::error::Error;
use crate::reporter::Reporter;
use crate::test_executor::{ExecutionContext, TestCaseExecInfo};
use crate::test_suite::status::TestCaseStatus;
use crate::test_suite::visitor::Visitor;
use crate::test_suite::{TestCase, TestSuite};

use colored::{ColoredString, Colorize};

pub struct HumanFriendlyReporter {
    debug_enabled: bool,
    matrix_summary: bool,
}

impl HumanFriendlyReporter {
    pub fn new(debug_enabled: bool, matrix_summary: bool) -> Self {
        Self {
            debug_enabled,
            matrix_summary,
        }
    }

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
        exec_contexts: &[ExecutionContext],
    ) {
        if self.matrix_summary {
            println!();
            println!(
                "{}",
                format!(
                    "Test suite `{}` execution summary",
                    test_suite.path().display()
                )
                .bright_white()
            );
            TestSuiteSummaryPrettyPrinter::print_matrix_summary(test_suite, exec_contexts);
        } else {
            for exec_context in exec_contexts {
                println!();
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
            }
        }
    }

    fn report_total_time(&self) {}

    fn notice_detailed(&self, message: &str, details: &str) {
        self.print_with_details("".normal(), message, details)
    }

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

const CHAR_PASS: &str = "V";
const CHAR_FAIL: &str = "X";
const CHAR_RFAIL: &str = "O";
const CHAR_SKIP: &str = ">";

struct TestSuiteSummaryPrettyPrinter;
impl TestSuiteSummaryPrettyPrinter {
    fn max_row_width(test_suite: &TestSuite, _exec_contexts: &[ExecutionContext]) -> usize {
        let mut row_width = 0;
        Visitor::new(test_suite)
            .visit_all_ok(|tc, _| row_width = std::cmp::max(row_width, tc.id().len()));
        row_width
    }

    fn max_column_width(_test_suite: &TestSuite, exec_contexts: &[ExecutionContext]) -> usize {
        let mut column_width = 0;
        for exec_context in exec_contexts {
            column_width = std::cmp::max(column_width, exec_context.target().len());
        }
        column_width
    }

    fn print_matrix_summary(test_suite: &TestSuite, exec_contexts: &[ExecutionContext]) {
        let max_row_width = Self::max_row_width(test_suite, exec_contexts);
        let max_column_width = Self::max_column_width(test_suite, exec_contexts);

        let mut depth = 0;
        for exec_context in exec_contexts {
            print!("{:width$}", "", width = max_row_width + 1);
            for _ in 0..depth {
                print!("│ ")
            }
            print!("┌─ {}", exec_context.target());
            print!(
                "{:width$}",
                "",
                width = max_column_width - exec_context.target().len() + (exec_contexts.len() * 2)
                    - (depth * 2)
            );
            let statistics = exec_context.get_statistics();
            println!(
                "{} passed, {} failed, {} runner failed, {} skipped",
                statistics.passed.to_string().green(),
                statistics.failed.to_string().red(),
                statistics.runner_failed.to_string().red(),
                statistics.skipped.to_string().dimmed(),
            );
            depth += 1
        }

        Visitor::new(test_suite).visit_all_ok(|tc, _| {
            print!("{} ", tc.id());
            print!("{:width$}", "", width = max_row_width - tc.id().len());
            for exec_context in exec_contexts {
                let exec_info = exec_context.exec_info().get(tc).unwrap();
                let c = match exec_info.result() {
                    Err(_) => CHAR_RFAIL.red().to_string(),
                    Ok(TestCaseStatus::Failed) => CHAR_FAIL.red().to_string(),
                    Ok(TestCaseStatus::Passed) => CHAR_PASS.green().to_string(),
                    Ok(TestCaseStatus::Skipped(_)) => CHAR_SKIP.dimmed().to_string(),
                    Ok(TestCaseStatus::DryRun) => CHAR_SKIP.dimmed().to_string(),
                    _ => panic!("aie"),
                };
                print!("{} ", c);
            }
            println!();
        });
    }
}
