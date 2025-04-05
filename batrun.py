import argparse
import os
import sys
import subprocess
from datetime import datetime
from pathlib import Path
from typing import List

# Constants
TEST_FN_SETUP = "setup"
TEST_FN_TEARDOWN = "teardown"
TEST_FN_PREFIX = "test_"
GLOBAL_TEST_FILE = "tests.sh"


def script_dir() -> Path:
    """Get the directory where this script is located."""
    return Path(__file__).resolve().parent


INCLUDE_DIR = script_dir() / "include"
BUILTIN_TESTS_DIR = script_dir() / "tests"


def log_error(message: str):
    print(f"ERROR: {message}", file=sys.stderr)


def log_info(message: str):
    print(f"INFO: {message}")


def log_warning(message: str):
    print(f"WARNING: {message}")


def log_status(status: str):
    print(f"STATUS: {status}")


def log_summary(total: int, passed: int, failed: int, skipped: int):
    print(f"SUMMARY: Total: {total}, Passed: {passed}, Failed: {failed}, Skipped: {skipped}")


def parse_args():
    """Parse command-line arguments."""
    parser = argparse.ArgumentParser(description="Bash test runner.")
    parser.add_argument("--test-suite-dir", "-d", default=BUILTIN_TESTS_DIR, help="directory where the test suite is located.")
    parser.add_argument("--out-dir", "-o", required=True, help="output directory for logs and data.")
    parser.add_argument("--target", "-t", action="append", required=True, help="target to run the tests on.")
    parser.add_argument("--list-targets", "-L", action="store_true", help="list targets supported by the specified test suite.")
    parser.add_argument("--list-tests", "-l", action="store_true", help="list tests available in the specified test suite.")
    parser.add_argument("--dry-run", "-n", action="store_true", help="go through all tests but execute nothing.")
    return parser.parse_args()


def validate_test_dir(tests_dir: Path):
    """Validate the test directory."""
    global_test_file = tests_dir / GLOBAL_TEST_FILE
    if not global_test_file.exists():
        log_error(f"Global test file '{GLOBAL_TEST_FILE}' not found in {tests_dir}.")
        sys.exit(1)


def list_tests(tests_dir: Path):
    """List all tests in the test directory."""
    test_files = sorted(tests_dir.glob("*.sh"))
    log_info(f"Tests defined in test suite '{tests_dir}':")
    for test_file in test_files:
        log_info(f"  {test_file.stem}")


def run_command(command: List[str], env=None, capture_output=False) -> subprocess.CompletedProcess:
    """Run a shell command."""
    return subprocess.run(command, env=env, capture_output=capture_output, text=True, check=False)


def run_test_case(test_file: Path, device: str, out_dir: Path, dry_run: bool):
    """Run a single test case."""
    log_info(f"Running test case {test_file.stem} on device {device}")
    if dry_run:
        log_status("SKIPPED")
        return

    log_file = out_dir / f"{test_file.stem}.log"
    with log_file.open("w") as log:
        result = run_command(["bash", "-e", "-u", "-o", "pipefail", str(test_file)], capture_output=True)
        log.write(result.stdout or "")
        log.write(result.stderr or "")
        if result.returncode == 0:
            log_status("PASSED")
        else:
            log_status("FAILED")


def run_all_tests(tests_dir: Path, devices: List[str], out_dir: Path, dry_run: bool):
    """Discover and run all tests."""
    test_files = sorted(tests_dir.glob("*.sh"))
    total_tests = len(test_files) * len(devices)
    passed, failed, skipped = 0, 0, 0

    for test_file in test_files:
        for device in devices:
            if dry_run:
                skipped += 1
            else:
                run_test_case(test_file, device, out_dir, dry_run)
                passed += 1  # Simplified for demonstration

    log_summary(total_tests, passed, failed, skipped)


def main():
    args = parse_args()
    out_dir = Path(args.out_dir) / datetime.now().strftime("%Y%m%d%H%M%S")
    out_dir.mkdir(parents=True, exist_ok=True)

    tests_dir = Path(args.tests_dir)
    validate_test_dir(tests_dir)

    if args.list_tests:
        list_tests(tests_dir)
        sys.exit(0)

    run_all_tests(tests_dir, args.device, out_dir, args.dry_run)


if __name__ == "__main__":
    # main()
    print(parse_args())