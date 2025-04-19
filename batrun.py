import logging
import subprocess
import sys
import os
from datetime import datetime, timedelta
from pathlib import Path
from dataclasses import dataclass
import colorama
from colorama import Fore, Style
import enum
import abc
from typing import override

# # Configure logging
# logging.basicConfig(
#     level=logging.INFO,
#     format="%(levelname)s: %(message)s",
# )


def script_dir() -> Path:
    """Get the directory where this script is located."""
    return Path(__file__).resolve().parent


INTERNAL_VALIDATION_TEST_SUITE_DIR = script_dir() / "tests"


class TestResult(enum.Enum):
    NOTRUN = enum.auto()
    RUNNING = enum.auto()
    FAILED = enum.auto()
    PASSED = enum.auto()
    SKIPPED = enum.auto()
    DRYRUN = enum.auto()


@dataclass
class Duration:
    start_time: datetime = None
    end_time: datetime = None

    def elapsed(self) -> timedelta:
        return self.end_time - self.start_time


class Timer:
    def __init__(self, duration: Duration):
        self.duration = duration

    def __enter__(self) -> "Timer":
        self.duration.start_time = datetime.now()
        return self

    def __exit__(self, exc_type, exc_value, traceback) -> "Timer":
        self.duration.end_time = datetime.now()
        if exc_type is None and exc_value is None and traceback is None:
            return self


class TestRecord:
    def __init__(self, file: Path, name: str, target: str):
        self.file: Path = file
        self.name: str = name
        self.target: str = target
        self.result: TestResult = TestResult.NOTRUN
        self.duration: Duration = None

    def set_result(self, result: TestResult):
        self.duration = datetime.now()
        self.result = result


@dataclass
class TestSuiteConfig:
    name: str
    description: str
    version: str
    driver: str
    test_file_pattern: list[str]
    global_fixture: str

    def load(test_suite_dir: Path) -> "TestSuiteConfig":
        with open(test_suite_dir / "test-suite.json") as config_file:
            import json

            config_file = json.load(config_file)
            test_suite_config = TestSuiteConfig(
                name=config_file.get("name"),
                description=config_file.get("description"),
                version=config_file.get("version"),
                driver=config_file.get("driver"),
                test_file_pattern=config_file.get("test-file-pattern", []),
                global_fixture=config_file.get("global-fixture", None),
            )
        return test_suite_config


class TestSuite:
    def __init__(self, config: TestSuiteConfig):
        self.config: TestSuiteConfig = config
        self.test_records: list[TestRecord] = []
        self.duration: Duration = None


class Settings:
    def __init__(self):
        self.test_suite_dirs: list[Path] = None
        self.out_dir: Path = None
        self.target: str = None
        self.dry_run: bool = False
        self.test_filter: str = None

class TestDriverConfig:
    def __init__(self):
        self.test_suite_dir: Path
        self.file_pattern: list[str] = None
        self.global_fixture: Path = None
        self.targets: list[str] = []
        self.dry_run: bool = False

class TestDriver(abc.ABC):
    @abc.abstractmethod
    def default_test_file_pattern() -> list[str]:
        pass

    @abc.abstractmethod
    def discover_tests(self) -> list[TestRecord]:
        pass

    @abc.abstractmethod
    def run_test(self, test_record: TestRecord):
        pass

class BashTestDriver(TestDriver):
    def __init__(self, config: TestDriverConfig):
        self.config: TestDriverConfig = config
        self.test_files: list[Path] = []

    @override
    def default_test_file_pattern() -> list[str]:
        return ["*.sh", "*.bash"]

    def _get_test_functions_in_file(self, test_file: Path) -> list[str]:
        file_path = self.test_suite_dir / test_file
        try:
            cmd = [
                "bash",
                "-c",
                f"source '{file_path}'; compgen -A function",
            ]
            result = subprocess.run(cmd, capture_output=True, text=True, check=True)
            return result.stdout.strip().split("\n")
        except Exception as e:
            print(
                f"Failed to extract test functions from {file_path}: {e}\nstdout:\n{e.stdout}\nstderr:\n{e.stderr}"
            )
            return []

    @override
    def discover_tests(self) -> list[TestRecord]:
        test_records = []
        # Find all test files in the test suite directory
        test_files = []
        for root, _, files in os.walk(self.test_suite_dir):
            for file in files:
                if file.endswith(".sh") and file != GLOBAL_TEST_FILE:
                    full_path = Path(os.path.join(root, file))
                    test_files.append(full_path)

        # Sort the test files for consistent ordering
        test_files.sort()

        for test_file in test_files:
            # Get relative path from test suite directory to create test ID
            try:
                test_id = test_file.relative_to(self.test_suite_dir).with_suffix("")
            except ValueError:
                # If test_file is not relative to test_suite_dir
                test_id = test_file.stem

            # Extract test functions from the file
            test_functions = self._get_test_functions_in_file(test_file)

            # Create TestRecord for each test function
            for target in self.targets:
                for test_function in test_functions:
                    test_name = f"{test_id}::{test_function}"
                    test_records.append(TestRecord(test_file, test_name, target))

        return test_records

    @override
    def run_test(self, test_record: TestRecord):
        pass


class TestRunner:
    def __init__(self, settings: Settings):
        self.settings: Settings = settings
        self.test_suites: dict[Path, TestSuite] = {}
        self.duration = Duration()
        self.console_reporter = HumanFriendlyReporter()
        self.reporters: list = []

    def _discover_tests(self, test_suite_dir: Path, test_suite: TestSuite) -> TestSuite:
        pass

    def _prepare_out_dir(self):
        """Prepare the output directory."""
        out_dir = self.settings.out_dir
        if out_dir.exists():
            self.console_reporter.warning(
                f"Output directory '{out_dir}' already exists. Contents may be overwritten."
            )
        else:
            out_dir.mkdir(parents=True, exist_ok=True)
        self.console_reporter.info(f"Output directory '{out_dir}' created.")

    def add_test_suite(self, test_suite_dir: Path):
        config = TestSuiteConfig.load(test_suite_dir)
        self.test_suites[test_suite_dir] = TestSuite(config)

    def list_tests(self, test_suite: TestSuite) -> list[str]:
        pass

    def list_targets(self, test_suite: TestSuite) -> list[str]:
        pass

    def run_tests(self, test_suite: TestSuite):
        self._prepare_out_dir()
        pass


class NullReporter:
    pass


class LoggingReporter:
    pass


class HumanFriendlyReporter:
    def __init__(self):
        colorama.init()

    def _log_test_result(result: TestResult):
        if result == TestResult.FAILED:
            print(Fore.RED + "FAILED" + Style.RESET_ALL)
        elif result == TestResult.PASSED:
            print(Fore.GREEN + "PASSED" + Style.RESET_ALL)
        elif result == TestResult.SKIPPED:
            print(Fore.WHITE + Style.DIM + "SKIPPED" + Style.RESET_ALL)
        elif result == TestResult.DRYRUN:
            print(Fore.WHITE + Style.DIM + "DRYRUN" + Style.RESET_ALL)

    def report_target_list(targets):
        pass

    def report_test_list(ts_context: TestSuite, tests):
        print(
            Fore.WHITE + Style.BRIGHT + "Tests defined in test suite" + Style.RESET_ALL
        )

    def report_test_suite_time():
        pass

    def report_total_time():
        pass


class JsonReporter:
    pass


import argparse


class Cli:
    DEFAULT_OUT_DIR = "out"

    def __init__(self):
        self.parser = Cli._create_arg_parser()
        self.args = self.parser.parse_args()

    def _create_arg_parser() -> argparse.ArgumentParser:
        """Parse command-line arguments."""
        parser = argparse.ArgumentParser(description="Bash test runner.")
        parser.add_argument(
            "test_suite",
            type=Path,
            nargs='+',
            help="directory where the test suite is located.",
        )
        parser.add_argument(
            "--out-dir",
            "-o",
            type=Path,
            default=Cli.DEFAULT_OUT_DIR,
            help="output directory for logs and data.",
        )
        parser.add_argument(
            "--target", "-t", action="append", help="target to run the tests on."
        )
        parser.add_argument(
            "--list-targets",
            "-L",
            action="store_true",
            help="list targets supported by the specified test suite.",
        )
        parser.add_argument(
            "--list-tests",
            "-l",
            action="store_true",
            help="list tests available in the specified test suite.",
        )
        parser.add_argument(
            "--dry-run",
            "-n",
            action="store_true",
            help="go through all tests but execute nothing.",
        )
        return parser

    def settings(self) -> Settings:
        settings = Settings()
        settings.test_suite_dirs = self.args.test_suite
        settings.out_dir = self.args.out_dir
        settings.target = self.args.target
        settings.dry_run = self.args.dry_run
        # settings.test_filter = self.args.test_filter
        return settings


def main():
    cli = Cli()
    test_runner = TestRunner(cli.settings())

    runner_actions = []
    if cli.args.list_targets:
        runner_actions.append(test_runner.list_targets)
    if cli.args.list_tests:
        runner_actions.append(test_runner.list_tests)
    if len(runner_actions) == 0:
        runner_actions.append(test_runner.run_tests)

    with Timer(test_runner.duration):
        print("Timing!")
        print(f"Test suite directories: {test_runner.settings.test_suite_dirs}")
        for test_suite_dir in test_runner.settings.test_suite_dirs:
            test_runner.add_test_suite(test_suite_dir)
            print(f"Test suite directory: {test_suite_dir}")
            print(
                f"Test suite config: {test_runner.test_suites[test_suite_dir].config}"
            )
            # validate_test_suite_dir(test_suite_dir)
            # for action in runner_actions:
            #     action(test_suite_context)

    print(f"Total time taken: {test_runner.duration.elapsed()}")


if __name__ == "__main__":
    main()

#
#
#
#
#
#
#
#
#
#
#
#
#
#
#
#
#
#
#
#
#
#
#
#
#
#
#
#
#
#
#
#
#
#
#
# # Constants
# TEST_FN_SETUP = "setup"
# TEST_FN_TEARDOWN = "teardown"
# TEST_FN_PREFIX = "test_"
# GLOBAL_TEST_FILE = "tests.sh"


# def script_dir() -> Path:
#     """Get the directory where this script is located."""
#     return Path(__file__).resolve().parent


# INTERNAL_VALIDATION_TEST_SUITE_DIR = script_dir() / "tests"

# DEFAULT_OUT_DIR = "out"


# def parse_args() -> argparse.Namespace:
#     """Parse command-line arguments."""
#     parser = argparse.ArgumentParser(description="Bash test runner.")
#     parser.add_argument(
#         "--test-suite-dir",
#         "-d",
#         type=Path,
#         action="append",
#         help="directory where the test suite is located.",
#     )
#     parser.add_argument(
#         "--out-dir",
#         "-o",
#         type=Path,
#         default=DEFAULT_OUT_DIR,
#         help="output directory for logs and data.",
#     )
#     parser.add_argument(
#         "--target", "-t", action="append", help="target to run the tests on."
#     )
#     parser.add_argument(
#         "--list-targets",
#         "-L",
#         action="store_true",
#         help="list targets supported by the specified test suite.",
#     )
#     parser.add_argument(
#         "--list-tests",
#         "-l",
#         action="store_true",
#         help="list tests available in the specified test suite.",
#     )
#     parser.add_argument(
#         "--dry-run",
#         "-n",
#         action="store_true",
#         help="go through all tests but execute nothing.",
#     )
#     return parser.parse_args()


# def validate_test_suite_dir(test_suite_dir: Path):
#     """Validate the test directory."""
#     global_test_file = test_suite_dir / GLOBAL_TEST_FILE
#     if not global_test_file.exists():
#         logging.error(
#             f"Global test file '{GLOBAL_TEST_FILE}' not found in {test_suite_dir}."
#         )
#         sys.exit(1)


# def list_targets(test_suite_dir: Path):
#     pass


# def list_tests(test_suite_dir: Path):
#     """List all tests in the test directory."""
#     test_files = sorted(test_suite_dir.glob("*.sh"))
#     logging.info(f"Tests defined in test suite '{test_suite_dir}':")
#     for test_file in test_files:
#         logging.info(f"  {test_file.stem}")


# def run_command(
#     command: List[str], env=None, capture_output=False
# ) -> subprocess.CompletedProcess:
#     """Run a shell command."""
#     return subprocess.run(
#         command, env=env, capture_output=capture_output, text=True, check=False
#     )


# def run_test_case(test_file: Path, device: str, out_dir: Path, dry_run: bool):
#     """Run a single test case."""
#     logging.info(f"Running test case {test_file.stem} on device {device}")
#     if dry_run:
#         logging.info("STATUS: SKIPPED")
#         return

#     log_file = out_dir / f"{test_file.stem}.log"
#     with log_file.open("w") as log:
#         result = run_command(
#             ["bash", "-e", "-u", "-o", "pipefail", str(test_file)], capture_output=True
#         )
#         log.write(result.stdout or "")
#         log.write(result.stderr or "")
#         if result.returncode == 0:
#             logging.info("STATUS: PASSED")
#         else:
#             logging.info("STATUS: FAILED")


# def run_test_suite(context: TestSuiteRunnerContext):
#     """Discover and run all tests."""
#     test_files = sorted(context.test_suite_dir.glob("*.sh"))
#     total_tests = len(test_files) * len(context.targets)
#     passed, failed, skipped = 0, 0, 0

#     for test_file in test_files:
#         for target in context.targets:
#             if context.dry_run:
#                 skipped += 1
#             else:
#                 run_test_case(test_file, target, out_dir, dry_run)
#                 passed += 1  # Simplified for demonstration

#     logging.info(
#         f"SUMMARY: Total: {total_tests}, Passed: {passed}, Failed: {failed}, Skipped: {skipped}."
#     )


# def prepare_out_dir(out_dir: Path):
#     """Prepare the output directory."""
#     if out_dir.exists():
#         logging.warning(
#             f"Output directory '{out_dir}' already exists. Contents may be overwritten."
#         )
#     else:
#         out_dir.mkdir(parents=True, exist_ok=True)
#         logging.info(f"Output directory '{out_dir}' created.")


# def decide_runner_actions(args: argparse.Namespace) -> List[callable]:
#     """
#     Decide which actions to perform based on command-line arguments.
#     If --list-targets or --list-tests is specified, those actions are added to the list.
#     If neither is specified, the default action is to run all tests.
#     """
#     runner_actions = []
#     if args.list_targets:
#         runner_actions.append(list_targets)
#     if args.list_tests:
#         runner_actions.append(list_tests)
#     if runner_actions.empty():
#         runner_actions.append(run_test_suite)
#     return runner_actions


# def main():
#     batrun_context = BatrunContext(args=parse_args())

#     prepare_out_dir(batrun_context.out_dir)

#     runner_actions = decide_runner_actions(batrun_context.args)

#     batrun_context.duration.start_time = datetime.now()

#     for test_suite_dir in batrun_context.test_suite_dir:
#         test_suite_context = TestSuiteRunnerContext(batrun_context, test_suite_dir)
#         validate_test_suite_dir(test_suite_dir)
#         for action in runner_actions:
#             action(test_suite_context)

#     batrun_context.duration.end_time = datetime.now()

#     logging.info(f"Total time taken: {batrun_context.duration.elapsed()}")


# if __name__ == "__main__":
#     main()
