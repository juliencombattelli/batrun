use crate::ExecutionStrategy;
use crate::error::{self, Result};
use crate::reporter::Reporter;
use crate::reporter::human_friendly::HumanFriendlyReporter;
use crate::settings::Settings;
use crate::test_driver::{TestDriver, TestDriverRegistry};
use crate::test_executor::round_robin::RoundRobinExecutor;
use crate::test_executor::sequential::SequentialExecutor;
use crate::test_executor::{ExecutionContext, Executor};
use crate::test_suite::TestSuite;
use crate::test_suite::config::TestSuiteConfig;
use crate::test_suite::registry::TestSuiteRegistry;

use std::fs;
use std::path::Path;

pub struct TestRunner {
    settings: Settings,
    test_drivers: TestDriverRegistry,
    test_suites: TestSuiteRegistry,
    console_reporter: Box<dyn Reporter>,
}

impl TestRunner {
    pub fn new(settings: Settings) -> Result<Self> {
        let debug_enabled = settings.debug;
        let mut test_runner = Self {
            settings,
            test_drivers: TestDriverRegistry::new(),
            test_suites: TestSuiteRegistry::new(),
            console_reporter: Box::new(HumanFriendlyReporter::new(debug_enabled)),
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
        let test_suite = self.test_suites.get(test_suite_dir)?;
        self.console_reporter.report_target_list(test_suite);
        Ok(())
    }

    pub fn run_tests(&mut self, test_suite_dir: &Path) -> Result<()> {
        self.prepare_out_dir()?;
        let test_suite = self.test_suites.get_mut(test_suite_dir)?;
        let test_driver = self.test_drivers.get(&test_suite.config().driver)?;

        let mut exec_contexts = self
            .settings
            .targets
            .iter()
            .map(|target| {
                ExecutionContext::new(&test_suite, target.clone(), &self.settings.out_dir)
            })
            .collect::<Vec<_>>();

        Self::run_executor(
            test_suite,
            &test_driver,
            &self.console_reporter,
            &mut exec_contexts,
            self.settings.exec_strategy.clone(),
        );

        for exec_context in &exec_contexts {
            self.console_reporter
                .report_test_suite_execution_summary(&test_suite, &exec_context);
        }

        Ok(())
    }

    fn run_executor(
        test_suite: &TestSuite,
        test_driver: &Box<dyn TestDriver>,
        reporter: &Box<dyn Reporter>,
        exec_contexts: &mut [ExecutionContext],
        exec_strategy: ExecutionStrategy,
    ) {
        let executor: Box<dyn Executor> = match exec_strategy {
            ExecutionStrategy::RoundRobin => Box::new(RoundRobinExecutor {}),
            ExecutionStrategy::Sequential => Box::new(SequentialExecutor {}),
            _ => todo!("{:X?}", exec_strategy),
        };
        executor.execute(reporter, test_driver, test_suite, exec_contexts);
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
