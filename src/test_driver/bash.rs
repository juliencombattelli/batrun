use crate::error::{self, Error, Result};
use crate::test_driver::TestDriver;
use crate::test_suite::config::TestSuiteConfig;
use crate::test_suite::status::{SkipReason, TestCaseStatus};
use crate::test_suite::{TestCase, TestFile, TestSuite, TestSuiteFixture};

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub struct BashTestDriver;

impl BashTestDriver {
    const SETUP_FN_NAME: &str = "setup";
    const TEARDOWN_FN_NAME: &str = "teardown";
    const TEST_FN_PREFIX: &str = "test_";

    pub fn new() -> Self {
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
        log_file: &Path,
    ) -> Result<TestCaseStatus> {
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
                "{{ {run_fn_command} }} &> \"{log_file}\"; {{ env | grep -E '^BATRUN_' || true; }}",
                log_file = log_file.display()
            ));

        // TODO redirect also stderr in case of runner failure
        let command_output = BashCommandOutput::new(bash_command.output().map_err(|io_err| {
            error::kind::TestDriverIo {
                filename: PathBuf::from(bash_command.get_program()),
                source: io_err,
            }
        })?);

        if command_output.output.status.success() {
            if let Some(skipped_reason) = command_output.skipped {
                return Ok(TestCaseStatus::Skipped(SkipReason::TestCaseSpecificReason(
                    skipped_reason,
                )));
            }
            Ok(TestCaseStatus::Passed)
        } else {
            Ok(TestCaseStatus::Failed)
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
    ) -> Result<TestCaseStatus> {
        self.run_test_function_from_file(
            test_suite_dir,
            test_suite_config,
            &test_suite_dir.join(test_case.path()),
            test_case.name(),
            target,
            test_case_out_dir,
            &test_case_out_dir.join(format!("{}.log", test_case.name())),
        )
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

struct BashCommandOutput {
    output: Output,
    skipped: Option<String>,
}

impl BashCommandOutput {
    fn parse_output_env_vars(output: &std::process::Output) -> HashMap<String, String> {
        let stdout = std::str::from_utf8(&output.stdout).unwrap_or("");
        let env_vars: HashMap<String, String> = stdout
            .lines()
            .filter_map(|line| {
                if let Some((key, value)) = line.split_once('=') {
                    Some((key.to_string(), value.to_string()))
                } else {
                    None
                }
            })
            .collect();
        env_vars
    }

    fn new(output: std::process::Output) -> Self {
        let env_vars = Self::parse_output_env_vars(&output);
        Self {
            output,
            skipped: env_vars.get("BATRUN_SKIPPED").cloned(),
        }
    }
}
