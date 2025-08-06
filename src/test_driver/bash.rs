use crate::error::{self, Error, Result};
use crate::test_driver::{DriverOutput, RunTestOutput, TestDriver};
use crate::test_suite::config::TestSuiteConfig;
use crate::test_suite::status::{SkipReason, TestCaseStatus};
use crate::test_suite::{TestCase, TestFile, TestSuite, TestSuiteFixture};

use std::collections::HashMap;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(crate) struct BashTestDriver;

impl BashTestDriver {
    const SETUP_FN_NAME: &str = "setup";
    const TEARDOWN_FN_NAME: &str = "teardown";
    const TEST_FN_PREFIX: &str = "test_";

    pub(crate) fn new() -> Self {
        Self
    }

    fn get_functions_in_file(&self, file_path: &Path, fn_regex: &str) -> Result<Vec<String>> {
        let mut list_functions_command = Command::new("bash");
        let output = list_functions_command
            .arg("-c")
            .arg(format!(
                "source '{}'; compgen -A function | grep '{}'",
                file_path.display(),
                fn_regex,
            ))
            .output()
            .map_err(|io_err| error::kind::TestDriverIo {
                filename: PathBuf::from(list_functions_command.get_program()),
                source: io_err,
            })?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout.trim().split('\n').map(String::from).collect())
        } else {
            if output.stdout.is_empty() && output.stderr.is_empty() {
                // Error::NoTestFound is handled in test runner when needed
                // Return only an empty list here
                return Ok(Vec::new());
            }
            Err(Error::from(error::kind::TestFileExec {
                filename: file_path.to_path_buf(),
                details: String::from_utf8_lossy(&output.stderr).to_string(),
            }))
        }
    }

    fn get_named_function_in_file(
        &self,
        file_path: &Path,
        fn_name: &str,
    ) -> Result<Option<String>> {
        let fn_regex = format!("^{}$", fn_name);
        let functions = self.get_functions_in_file(file_path, &fn_regex)?;
        match functions.len() {
            0 => Ok(None),
            1 => Ok(Some(functions[0].clone())),
            // Function duplication cannot happen in bash as subsequent function
            // definitions override previous ones, but we handle the case anyway
            _ => Err(Error::DuplicatedTestFn(fn_name.to_string())),
        }
    }

    fn get_test_suite_fixture(
        &self,
        test_suite_dir: &Path,
        test_suite_config: &TestSuiteConfig,
    ) -> Result<TestSuiteFixture> {
        test_suite_config
            .global_fixture
            .as_ref()
            .map(PathBuf::from)
            .map_or(Ok(TestSuiteFixture::default()), |local_fixture_path| {
                let fixture_path = test_suite_dir.join(&local_fixture_path);
                Ok(TestSuiteFixture {
                    setup_test_case: self
                        .get_named_function_in_file(&fixture_path, BashTestDriver::SETUP_FN_NAME)?
                        .map(|setup_fn| TestCase::new(&local_fixture_path, &setup_fn)),
                    teardown_test_case: self
                        .get_named_function_in_file(
                            &fixture_path,
                            BashTestDriver::TEARDOWN_FN_NAME,
                        )?
                        .map(|teardown_fn| TestCase::new(&local_fixture_path, &teardown_fn)),
                })
            })
    }

    fn run_test_function_from_file(
        &self,
        test_suite_dir: &Path,
        test_suite_config: &TestSuiteConfig,
        file_path: &Path,
        fn_name: &str,
        target: &str,
        out_dir: &Path,
        log_files: LogFiles,
    ) -> Result<(TestCaseStatus, TestCaseOutput)> {
        let run_fn_command = RunFnCommandBuilder::new()
            .source_fixture_if_necessary(
                test_suite_config.global_fixture.clone(),
                file_path,
                test_suite_dir,
            )
            .source_test_file(&file_path)
            .execute_fn(fn_name, target, out_dir)
            .build();

        let mut bash_command = Command::new("bash");
        bash_command
            .args(["-x", "-e", "-u", "-o", "pipefail"])
            .arg("-c")
            .arg(&format!(
                "{{ {run_fn_command} }} &> \"{log_file}\"; {{ env | grep -E '^BATRUN_' || true; }} > \"{envout_file}\";",
                log_file = log_files.test_case.display(),
                // debug_file = log_files.debug.display(),
                envout_file = log_files.envout.display()
            ));

        let output = bash_command
            .output()
            .map_err(|io_err| error::kind::TestDriverIo {
                filename: PathBuf::from(bash_command.get_program()),
                source: io_err,
            })?;

        let tc_output = TestCaseOutput::new(&log_files.envout);

        if output.status.success() {
            if let Some(ref skipped_reason) = tc_output.skipped {
                return Ok((
                    TestCaseStatus::Skipped(SkipReason::TestCaseSpecificReason(
                        skipped_reason.clone(),
                    )),
                    tc_output,
                ));
            }
            Ok((TestCaseStatus::Passed, tc_output))
        } else {
            Ok((TestCaseStatus::Failed, tc_output))
        }
    }
}

impl TestDriver for BashTestDriver {
    fn test_file_patterns_default(&self) -> Vec<String> {
        vec!["*.sh".to_string(), "*.bash".to_string()]
    }

    fn discover_tests(
        &self,
        test_suite_dir: &Path,
        test_suite_config: &TestSuiteConfig,
    ) -> Result<TestSuite> {
        let mut test_files = Vec::new();

        let test_suite_fixture =
            self.get_test_suite_fixture(&test_suite_dir, &test_suite_config)?;

        let test_files_path = self.discover_test_files(test_suite_dir, test_suite_config);

        for test_file_local_path in &test_files_path {
            let test_file_path = test_suite_dir.join(&test_file_local_path);
            test_files.push(TestFile {
                setup_test_case: self
                    .get_named_function_in_file(&test_file_path, BashTestDriver::SETUP_FN_NAME)?
                    .map(|setup_fn| TestCase::new(&test_file_local_path, &setup_fn)),
                teardown_test_case: self
                    .get_named_function_in_file(&test_file_path, BashTestDriver::TEARDOWN_FN_NAME)?
                    .map(|teardown_fn| TestCase::new(&test_file_local_path, &teardown_fn)),
                test_cases: self
                    .get_functions_in_file(
                        &test_file_path,
                        &format!("^{}", BashTestDriver::TEST_FN_PREFIX),
                    )?
                    .into_iter()
                    .map(|test_fn| TestCase::new(&test_file_local_path, &test_fn))
                    .collect(),
            });
        }

        Ok(TestSuite::new(
            test_suite_dir,
            test_suite_config.clone(),
            test_files,
            test_suite_fixture,
        ))
    }

    fn run_test(
        &self,
        test_suite_dir: &Path,
        test_suite_config: &TestSuiteConfig,
        target: &str,
        test_case: &TestCase,
        test_case_out_dir: &Path,
    ) -> Result<RunTestOutput> {
        self.run_test_function_from_file(
            test_suite_dir,
            test_suite_config,
            &test_suite_dir.join(test_case.path()),
            test_case.name(),
            target,
            test_case_out_dir,
            LogFiles::new(test_case_out_dir, test_case.name()),
        )
        .map(|(test_case_status, test_case_output)| RunTestOutput {
            test_case_status,
            driver_output: Some(Box::new(BashDriverOutput { test_case_output })),
        })
    }
}

struct LogFiles {
    test_case: PathBuf,
    debug: PathBuf,
    envout: PathBuf,
}

impl LogFiles {
    pub fn new(test_case_out_dir: &Path, test_case_name: &str) -> Self {
        Self {
            test_case: test_case_out_dir.join(format!("{}.test.log", test_case_name)),
            debug: test_case_out_dir.join(format!("{}.debug.log", test_case_name)),
            envout: test_case_out_dir.join(format!("{}.envout.log", test_case_name)),
        }
    }
}

struct BashDriverOutput {
    test_case_output: TestCaseOutput,
}
impl DriverOutput for BashDriverOutput {}
impl Display for BashDriverOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.test_case_output.unknown_env_vars.len() != 0 {
            write!(
                f,
                "Unknown output env vars: {:?}, ignoring.",
                self.test_case_output.unknown_env_vars
            )
        } else {
            Ok(())
        }
    }
}

struct RunFnCommandBuilder {
    bash_command: String,
}

impl RunFnCommandBuilder {
    fn new() -> Self {
        Self {
            bash_command: String::new(),
        }
    }

    fn source_fixture(mut self, fixture: &Path) -> RunFnCommandBuilder {
        self.bash_command += &format!(
            "echo Sourcing global fixture '{0}'; source '{0}'; ",
            fixture.display()
        );
        self
    }

    fn source_fixture_if_necessary(
        self,
        fixture: Option<String>,
        file_path: &Path,
        test_suite_dir: &Path,
    ) -> RunFnCommandBuilder {
        if let Some(fixture) = &fixture {
            let fixture = test_suite_dir.join(fixture);
            // Do not source the fixture if we are executing a function from the fixture
            if fixture != file_path {
                return self.source_fixture(&fixture);
            }
        }
        self
    }

    fn source_test_file(mut self, file_path: &Path) -> RunFnCommandBuilder {
        self.bash_command += &format!(
            "echo Sourcing test file '{0}'; source '{0}'; ",
            file_path.display()
        );
        self
    }

    fn execute_fn(mut self, fn_name: &str, target: &str, out_dir: &Path) -> RunFnCommandBuilder {
        self.bash_command += &format!(
            "\"{fn_name}\" \"{target}\" \"{out_dir}\";",
            out_dir = out_dir.display()
        );
        self
    }

    fn build(self) -> String {
        self.bash_command
    }
}

struct TestCaseOutput {
    unknown_env_vars: Vec<String>,
    skipped: Option<String>,
}

impl TestCaseOutput {
    const KNOWN_OUTPUT_ENV_VARS: &'static [&'static str] = &["BATRUN_SKIPPED"];

    fn parse_output_env_vars(envout_file: &Path) -> (HashMap<String, String>, Vec<String>) {
        let mut unknown_env_vars = Vec::new();
        let env_vars = std::fs::read_to_string(envout_file)
            .unwrap_or(String::new())
            .lines()
            .filter_map(|line| {
                if let Some((envvar, value)) = line.split_once('=') {
                    if Self::KNOWN_OUTPUT_ENV_VARS
                        .iter()
                        .find(|known_env_var| envvar == **known_env_var)
                        .is_some()
                    {
                        Some((envvar.to_string(), value.to_string()))
                    } else {
                        unknown_env_vars.push(envvar.to_string());
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        (env_vars, unknown_env_vars)
    }

    fn new(envout_file: &Path) -> Self {
        let (env_vars, unknown_env_vars) = Self::parse_output_env_vars(&envout_file);
        Self {
            unknown_env_vars,
            skipped: env_vars.get("BATRUN_SKIPPED").cloned(),
        }
    }
}
