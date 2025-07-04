use crate::error::Error;
use crate::reporter::Reporter;
use crate::test_executor::{ExecutionContext, TestCaseExecInfo};
use crate::test_suite::status::TestCaseStatus;
use crate::test_suite::visitor::Visitor;
use crate::test_suite::{TestCase, TestSuite};

use colored::{ColoredString, Colorize};

pub(crate) struct HumanFriendlyReporter {
    debug_enabled: bool,
    matrix_summary: bool,
}

impl HumanFriendlyReporter {
    pub(crate) fn new(debug_enabled: bool, matrix_summary: bool) -> Self {
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

    fn print_summary_header(&self, test_suite: &TestSuite) {
        println!();
        println!(
            "{}",
            format!(
                "Test suite `{}` execution summary",
                test_suite.path().display()
            )
            .bright_white()
        );
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
            self.print_summary_header(test_suite);
            TestSuiteSummaryPrettyPrinter::new(test_suite, exec_contexts).print_matrix_summary();
        } else {
            for exec_context in exec_contexts {
                self.print_summary_header(test_suite);
                println!("  Target: {}", exec_context.target().white());
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
const CHAR_SKIP: &str = "-";

struct TestSuiteSummaryPrettyPrinter<'a> {
    test_suite: &'a TestSuite,
    exec_contexts: &'a [ExecutionContext],
    max_row_width: usize,
    max_column_width: usize,
}

impl<'a> TestSuiteSummaryPrettyPrinter<'a> {
    fn new(test_suite: &'a TestSuite, exec_contexts: &'a [ExecutionContext]) -> Self {
        Self {
            test_suite,
            exec_contexts,
            max_row_width: Self::max_row_width(test_suite, exec_contexts),
            max_column_width: Self::max_column_width(test_suite, exec_contexts),
        }
    }

    fn char_pass() -> ColoredString {
        CHAR_PASS.green()
    }

    fn char_fail() -> ColoredString {
        CHAR_FAIL.red()
    }

    fn char_rfail() -> ColoredString {
        CHAR_RFAIL.bright_red()
    }

    fn char_skip() -> ColoredString {
        CHAR_SKIP.bright_black()
    }

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

    fn pad(width: usize) {
        print!("{:width$}", "");
    }

    fn print_legend(&mut self) {
        Self::pad(self.max_row_width + 1);
        println!(
            "{}: passed    {}: skipped",
            Self::char_pass(),
            Self::char_skip(),
        );
        Self::pad(self.max_row_width + 1);
        println!(
            "{}: failed    {}: runner failed",
            Self::char_fail(),
            Self::char_rfail(),
        );
    }

    fn print_single_statistic(&self, header: &ColoredString, stat: usize, max_stat_len: usize) {
        print!("{header}: {stat:>width$}  ", width = max_stat_len)
    }

    fn print_statistics(&self, exec_context: &ExecutionContext) {
        let stats = exec_context.get_statistics();
        let max_stat_len = stats.max().to_string().len();
        self.print_single_statistic(&Self::char_pass(), stats.passed, max_stat_len);
        self.print_single_statistic(&Self::char_fail(), stats.failed, max_stat_len);
        self.print_single_statistic(&Self::char_rfail(), stats.runner_failed, max_stat_len);
        self.print_single_statistic(&Self::char_skip(), stats.skipped, max_stat_len);
        println!("/ {}", stats.total());
    }

    fn print_target_summary(&self) {
        let mut depth = 0;
        for exec_context in self.exec_contexts {
            Self::pad(self.max_row_width + 1);
            for _ in 0..depth {
                print!("│ ")
            }
            print!("┌─ {}", exec_context.target());
            Self::pad(
                self.max_column_width - exec_context.target().len()
                    + (self.exec_contexts.len() * 2)
                    - (depth * 2),
            );
            self.print_statistics(exec_context);
            depth += 1
        }
        Self::pad(self.max_row_width + 1);
        for _ in 0..depth {
            print!("╵ ");
        }
        println!();
    }

    fn print_test_cases_result(&self) {
        Visitor::new(self.test_suite).visit_all_ok(|tc, _| {
            print!("{} ", tc.id());
            Self::pad(self.max_row_width - tc.id().len());
            for exec_context in self.exec_contexts {
                let exec_info = exec_context.exec_info().get(tc).unwrap();
                let c = match exec_info.result() {
                    Err(_) => Self::char_rfail().to_string(),
                    Ok(TestCaseStatus::Failed) => Self::char_fail().to_string(),
                    Ok(TestCaseStatus::Passed) => Self::char_pass().to_string(),
                    Ok(TestCaseStatus::Skipped(_)) => Self::char_skip().to_string(),
                    Ok(TestCaseStatus::DryRun) => Self::char_skip().to_string(),
                    _ => panic!("aie"), // TODO
                };
                print!("{} ", c);
            }
            println!();
        });
    }

    fn print_matrix_summary(&mut self) {
        println!();
        self.print_legend();
        println!();
        self.print_target_summary();
        self.print_test_cases_result();
    }
}
