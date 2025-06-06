use crate::error::{self, Result};
use crate::reporter::Reporter;
use crate::reporter::human_friendly::HumanFriendlyReporter;
use crate::settings::Settings;
use crate::test_driver::TestDriverRegistry;
use crate::test_executor::utils::simple_executor;
use crate::test_suite::TestSuiteConfig;
use crate::test_suite::TestSuiteRegistry;

use std::fs;
use std::path::Path;
use std::pin::Pin;
use std::time::Instant;

pub struct TestRunner {
    settings: Settings,
    test_drivers: TestDriverRegistry,
    test_suites: TestSuiteRegistry,
    duration: Instant,
    console_reporter: HumanFriendlyReporter,
}

impl TestRunner {
    pub fn new(settings: Settings) -> Result<Self> {
        let debug_enabled = settings.debug;
        let mut test_runner = Self {
            settings,
            test_drivers: TestDriverRegistry::new(),
            test_suites: TestSuiteRegistry::new(),
            duration: Instant::now(),
            console_reporter: HumanFriendlyReporter::new(debug_enabled),
        };
        test_runner.load_test_suites()?;
        Ok(test_runner)
    }

    pub fn list_tests(&self, test_suite_dir: &Path) -> Result<()> {
        let test_suite = self.test_suites.get(test_suite_dir)?;
        self.console_reporter.report_test_list(test_suite);
        Ok(())
    }

    pub fn list_targets(&self, test_suite_dir: &Path) -> Result<()> {
        // TODO list_targets
        Ok(())
    }

    pub fn run_tests(&mut self, test_suite_dir: &Path) -> Result<()> {
        self.prepare_out_dir()?;
        let test_suite = self.test_suites.get_mut(test_suite_dir)?;
        let test_driver = self.test_drivers.get(&test_suite.config().driver)?;
        // let _ = test_suite.visit_mut(|tc| {
        //     for target in &self.settings.targets {
        //         match test_driver.run_test(test_suite_dir, target, tc) {
        //             Ok(TestCaseStatus::Skipped(_) | TestCaseStatus::Failed) => crate::test_suite::TestSuiteVisitStatus::Skip,
        //             _ = crate::test_suite::TestSuiteVisitStatus::Continue,
        //         }
        //     }
        //     Ok(crate::test_suite::TestSuiteVisitStatus::Continue)
        // });

        let mut per_target_tasks: Vec<Pin<Box<dyn Future<Output = ()>>>> = Vec::new();

        for target in &self.settings.targets {
            let task = async || {
                let target = target.clone();
                test_suite
                    .visit2(async |test_case, should_skip| {
                        test_driver.run_test(test_suite_dir, &target, test_case);
                        simple_executor::wait_until_next_poll().await;
                        crate::test_suite::TestSuiteVisitResult::Ok
                    })
                    .await;
            };
            per_target_tasks.push(Box::pin(task()));
        }
        simple_executor::execute_many(per_target_tasks);
        Ok(())
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    fn load_test_suites(&mut self) -> Result<()> {
        let mut last_error = None;
        for test_suite_dir in self.settings.test_suite_dirs.clone() {
            if let Err(error) = self.load_test_suite(&test_suite_dir) {
                self.console_reporter.error_from(&error);
                last_error = Some(error);
            }
        }
        if let Some(error) = last_error {
            Err(error)
        } else {
            Ok(())
        }
    }

    fn load_test_suite(&mut self, test_suite_dir: &Path) -> Result<()> {
        let config = TestSuiteConfig::load(&test_suite_dir)?;
        let test_driver = self.test_drivers.get(&config.driver)?;
        let test_suite = test_driver.discover_tests(&test_suite_dir, &config)?;
        self.test_suites.insert(test_suite_dir, test_suite);
        Ok(())
    }

    fn prepare_out_dir(&self) -> Result<()> {
        let out_dir = &self.settings.out_dir;
        if out_dir.exists() {
            self.console_reporter.warning(&format!(
                "Output directory `{}` already exists. Contents may be overwritten.",
                out_dir.display()
            ));
        } else {
            fs::create_dir_all(out_dir).map_err(|io_err| error::kind::SuiteConfigIo {
                filename: out_dir.clone(),
                source: io_err,
            })?;
            self.console_reporter.info(&format!(
                "Output directory `{}` created.",
                out_dir.display()
            ));
        }
        Ok(())
    }
}
