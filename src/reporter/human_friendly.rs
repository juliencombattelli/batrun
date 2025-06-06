use crate::error::Error;
use crate::reporter::Reporter;
use crate::test_suite::TestSuite;

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
    fn report_target_list(&self, targets: &Vec<String>) {}

    fn report_test_list(&self, test_suite: &TestSuite) {
        println!(
            "{}",
            format!(
                "Tests defined in test suite `{}`",
                test_suite.path().display()
            )
            .bright_white()
        );
        test_suite.visit::<Result<(), ()>, _>(|tc| {
            println!("  {}", tc.id().white());
            Ok(())
        });
        println!("");
    }

    fn report_test_suite_time(&self) {}

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
}
