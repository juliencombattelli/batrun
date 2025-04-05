use crate::error::{self, Result};
use crate::reporter::Reporter;
use crate::reporter::human_friendly::HumanFriendlyReporter;
use crate::simple_executor;
use crate::test_driver::{TestDriver, TestDriverRegistry};
use crate::test_suite::TestSuiteConfig;
use crate::test_suite::{self, TestCase, TestCaseState, TestSuite, TestSuiteRegistry};
use crate::time::TimeInterval;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::time::Instant;

// TODO avoid clap dependency here
#[derive(Debug, Copy, Clone, clap::ValueEnum)]
pub enum ExecutionStrategy {
    /// Run all test cases sequentially for a target before passing to the next target
    Sequential,
    /// Run each test case for all targets before passing to the next test case
    RoundRobin,
    /// Run all test cases for each targets in parallel
    Parallel,
}

#[derive(Debug)]
pub struct Settings {
    pub test_suite_dirs: Vec<PathBuf>,
    pub out_dir: PathBuf,
    pub targets: Vec<String>,
    pub exec_strategy: ExecutionStrategy,
    pub dry_run: bool,
    pub test_filter: Option<String>,
    pub debug: bool,
}

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
                test_suite
                    .visit2(async |tc, should_skip| {
                        run_test_async(test_driver, test_suite.path(), target.clone(), &tc).await;
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

#[derive(Debug)]
struct ExecStats {
    passed: usize,
    failed: usize,
    skipped: usize,
}

#[derive(Debug)]
enum TestSuiteState {
    NotRun,
    Running,
    Aborted(String),
    Finished(ExecStats),
}

#[derive(Debug)]
struct TestCaseExecInfo {
    state: TestCaseState,
    duration: TimeInterval,
}
impl TestCaseExecInfo {
    fn new() -> Self {
        Self {
            state: TestCaseState::NotRun,
            duration: TimeInterval::new(),
        }
    }
    pub fn set_status(&mut self, state: TestCaseState) {
        match state {
            TestCaseState::NotRun => {
                panic!("Test case status cannot be reset")
            }
            TestCaseState::Running => {
                self.state = state;
                self.duration = TimeInterval::new();
            }
            _ => {
                self.state = state;
                self.duration.stop();
            }
        }
    }
}

#[derive(Debug)]
struct TestSuiteExecutor<'ts> {
    test_suite: &'ts TestSuite,
    target: String,
    state: TestSuiteState,
    exec_info: HashMap<TestCase, TestCaseExecInfo>,
}

impl<'ts> TestSuiteExecutor<'ts> {
    pub fn new(test_suite: &'ts TestSuite, target: String) -> Self {
        let mut exec_info = HashMap::<TestCase, TestCaseExecInfo>::new();
        test_suite.visit(|tc| {
            exec_info.insert(tc.clone(), TestCaseExecInfo::new());
        });
        Self {
            test_suite,
            target,
            state: TestSuiteState::NotRun,
            exec_info,
        }
    }

    pub async fn execute(&self, test_driver: &Box<dyn TestDriver>) {
        self.test_suite
            .visit2(async |tc, should_skip| {
                run_test_async(
                    test_driver,
                    self.test_suite.path(),
                    self.target.clone(),
                    &tc,
                )
                .await;
                crate::test_suite::TestSuiteVisitResult::Ok
            })
            .await;
        // if let Some(tc) = &self.test_suite.fixture().setup_test_case {
        //     run_test_async(test_driver, self.test_suite.path(), &self.target, &tc).await;
        // }
        // for test_file in self.test_suite.test_files() {
        //     if let Some(tc) = &test_file.setup_test_case {
        //         run_test_async(test_driver, self.test_suite.path(), &self.target, &tc).await;
        //     }
        //     for tc in &test_file.test_cases {
        //         run_test_async(test_driver, self.test_suite.path(), &self.target, &tc).await;
        //     }
        //     if let Some(tc) = &test_file.teardown_test_case {
        //         run_test_async(test_driver, self.test_suite.path(), &self.target, &tc).await;
        //     }
        // }
        // if let Some(tc) = &self.test_suite.fixture().teardown_test_case {
        //     run_test_async(test_driver, self.test_suite.path(), &self.target, &tc).await;
        // }
    }
}

pub fn wait_until_next_poll() -> impl Future<Output = ()> {
    WaitUntilNextPoll {
        already_polled: false,
    }
}

struct WaitUntilNextPoll {
    already_polled: bool,
}

impl Future for WaitUntilNextPoll {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if self.already_polled {
            std::task::Poll::Ready(())
        } else {
            self.already_polled = true;
            std::task::Poll::Pending
        }
    }
}

async fn run_test_async(
    test_driver: &Box<dyn TestDriver>,
    test_suite_dir: &Path,
    target: String,
    test_case: &TestCase,
) {
    test_driver.run_test(test_suite_dir, &target, test_case);
    wait_until_next_poll().await;
}
