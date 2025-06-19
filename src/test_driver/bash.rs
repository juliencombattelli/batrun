use crate::error::{self, Error, Result};
use crate::test_driver::TestDriver;
use crate::test_suite::config::TestSuiteConfig;
use crate::test_suite::status::TestCaseStatus;
use crate::test_suite::{TestCase, TestFile, TestSuite, TestSuiteFixture};

use std::path::{Path, PathBuf};
use std::process::Command;

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
        let mut run_function_command = Command::new("bash");
        let mut bash_command = String::new();

        if let Some(global_fixture) = &test_suite_config.global_fixture {
            bash_command += &format!(
                "source '{}'; ",
                test_suite_dir.join(global_fixture).display()
            );
        }

        bash_command += &format!("source '{}'; ", test_suite_dir.join(file_path).display());

        // TODO redirect into log_file using Rust facilities?
        bash_command += &format!(
            "\"{fn_name}\" \"{target}\" \"{out_dir}\" &> \"{log_file}\"",
            out_dir = out_dir.display(),
            log_file = log_file.display()
        );

        // TODO redirect also stderr in case of runner failure
        let output = run_function_command
            .args(["-x", "-c", &format!("{}", &bash_command)])
            .output()
            .map_err(|io_err| error::kind::TestDriverIo {
                filename: PathBuf::from(run_function_command.get_program()),
                source: io_err,
            })?;

        if output.status.success() {
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
            test_case.path(),
            test_case.name(),
            target,
            test_case_out_dir,
            &test_case_out_dir.join(format!("{}.log", test_case.name())),
        )
    }
}
