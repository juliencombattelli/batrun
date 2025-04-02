#!/bin/bash

# Needs at least Bash 4.4
# TODO ensure it run with bash 4.4 using https://ftp.gnu.org/gnu/bash/bash-4.4.tar.gz
# TODO add check on bash version based on BASH_VERSINFO[]

set -e -u -o pipefail

##
# Get the directory where this script is located
#
# Globals
#   BASH_SOURCE (in)
#
# Outputs
#   stdout: the directory of this script
#
function script_dir {
    local DIR
    local SOURCE=${BASH_SOURCE[0]}
    # Resolve $SOURCE until the file is no longer a symlink
    while [ -L "$SOURCE" ]; do
        DIR=$( cd -P "$( dirname "$SOURCE" )" &>/dev/null && pwd )
        SOURCE=$(readlink "$SOURCE")
        # If $SOURCE was a relative symlink, we need to resolve it relative to
        # the path where the symlink file was located
        [[ $SOURCE != /* ]] && SOURCE=$DIR/$SOURCE
    done
    ( cd -P "$( dirname "$SOURCE" )" &>/dev/null && pwd )
}

# The directory containing included shell scripts
# shellcheck disable=SC2155
declare -r INCLUDE_DIR="$(script_dir)/include"

# The directory containing the builtin tests
# shellcheck disable=SC2155
declare -r BUILTIN_TESTS_DIR="$(script_dir)/tests"

# The global test file performing setup and teardown for the whole test execution
# It must also define KNOWN_DEVICES
declare -r GLOBAL_TEST_FILE="tests.sh"

# The function names that must be provided by test files
declare -r TEST_FN_SETUP="setup"
declare -r TEST_FN_TEARDOWN="teardown"
declare -r TEST_FN_PREFIX="test_"

# The mandatory user-provided list of devices to run the test on
declare -a DEVICES=()

# The mandatory user-provided directory where output data and logs are stored
OUT_DIR=

# The optional user-provided directory where tests are searched for
# Defaults to $BUILTIN_TESTS_DIR
TESTS_DIR="${BUILTIN_TESTS_DIR}"

# The optional user-provided list of tests to execute
# Defaults to all test found in $TESTS_DIR
TESTS=

# Whether to run the suite in dry-run mode
DRY_RUN=0

# Test execution statistics
STAT_TESTS_RUN_TOTAL=0
STAT_TESTS_RUN_PASSED=0
STAT_TESTS_RUN_FAILED=0
STAT_TESTS_RUN_SKIPPED=0

# shellcheck disable=SC1091
source "$INCLUDE_DIR/logging"

##
# Display the help message
#
# Globals
#   BUILTIN_TESTS_DIR (in)
#
# Outputs
#   stdout: the help message
#
function help {
    echo "Bash test runner."
    echo "Run tests written in bash on some specified devices."
    echo
    echo "Usage:"
    echo "  batrun [options]"
    echo
    echo "Options:"
    echo "  --help, -h                  Show this help."
    echo "  --device, -d DEVICE         Device to run the tests on (can be repeated)."
    echo "  --list-known-devices        List known devices."
    echo "  --tests-dir DIRECTORY       Directory where the tests are located."
    echo "  --tests, -t TESTS           Comma-separated list of tests to run."
    echo "  --list-tests, -l            List tests available in the test directory."
    echo "  --dry-run, -n               Go through all tests but execute nothing."
    echo
    echo " If --tests-dir is not specified, the default test directory "
    echo " is \`$BUILTIN_TESTS_DIR\`."
    echo
}

##
# Parse the command-line arguments and fill the global variables
#
# Globals
#   OUT_DIR (out)
#   TESTS_DIR (out)
#   TESTS (out)
#   DEVICES (out)
#
# Arguments
#   $@: script arguments to parse
#
function parse_args {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --help|-h)
                help
                exit;;
            --out-dir|-o)
                OUT_DIR="$2"
                shift 2;;
            --device|-d)
                DEVICES+=("$2")
                shift 2;;
            --list-known-devices)
                validate_test_dir
                list_known_devices
                exit 0;;
            --tests-dir)
                TESTS_DIR="$2"
                shift 2;;
            --tests|--t)
                TESTS="$2"
                shift 2
                log_error "Not implemented."
                exit 1;;
            --list-tests)
                list_tests
                exit 0;;
            --dry-run|-n)
                DRY_RUN=1
                shift;;
            -*)
                log_error "Unknown option \`$1\`."
                exit 1;;
            *)
                break;;
        esac
    done

    if [ ${#DEVICES[@]} -eq 0 ]; then
        log_error "No device specified."
        exit 1
    fi

    if [ -z "$OUT_DIR" ]; then
        log_error "No output directory specified."
        exit 1
    fi
    OUT_DIR="$OUT_DIR/$(date +%Y%m%d%H%M%S)"
}

##
# Get the path of the global test file
#
# Globals
#   TESTS_DIR (in)
#   GLOBAL_TEST_FILE (in)
#
function global_test_file {
    echo "$TESTS_DIR/$GLOBAL_TEST_FILE"
}

##
# Get the declaration of KNOWN_DEVICES provided by the current test suite
#
# The function output must be evaluated using `eval "$(known_devices)"` to
# redeclare the variable in the current scope
#
# TEST_DIR must be set before calling this function, otherwise KNOWN_DEVICES
# from the internal validation test suite is returned
#
# Globals
#   TESTS_DIR (in)
#   GLOBAL_TEST_FILE (in)
#   KNOWN_DEVICES (in)
#
# Outputs
#   stdout: the bash declaration command for the KNOWN_DEVICES array
#
function known_devices {
    bash -e -u -o pipefail -c "source \"$(global_test_file)\" && declare -p KNOWN_DEVICES" 2>/dev/null
}

##
# Validate the KNOWN_DEVICE variable from the user-provided test suite
#
# Globals
#   TESTS_DIR (in)
#   GLOBAL_TEST_FILE (in)
#   KNOWN_DEVICES (in)
#
function validate_known_devices {
    eval "$(known_devices)"
    if [[ "$(declare -p KNOWN_DEVICES 2>/dev/null)" =~ "declare -a" ]]; then
        return 0
    elif [[ "$(declare -p KNOWN_DEVICES 2>/dev/null)" =~ "declare" ]]; then
        log_error "KNOWN_DEVICES is expected to be a bash array"
    else
        log_error "KNOWN_DEVICES is not defined by $(global_test_file)"
    fi
    return 1
}

##
# Validate all required components that a user-provided test suite must define
#
# Globals
#   TESTS_DIR (in)
#   GLOBAL_TEST_FILE (in)
#   KNOWN_DEVICES (in)
#
function validate_test_dir {
    validate_known_devices
}

##
# List the known devices
#
# Globals
#   KNOWN_DEVICES (in)
#
function list_known_devices {
    eval "$(known_devices)"
    log_info_bright "Known devices in test suite \`$TESTS_DIR\`:"
    for DEVICE in "${!KNOWN_DEVICES[@]}"; do
        log_info "  ${KNOWN_DEVICES[$DEVICE]}"
    done
}

##
# Get all the tests defined by the current test suite
#
# TEST_DIR must be set before calling this function, otherwise the tests from
# the internal validation test suite is returned
#
# Arguments
#   $1 (out): name of the array to populate
#
# Globals
#   TESTS_DIR (in)
#   GLOBAL_TEST_FILE (in)
#
function get_tests {
    local -n MUT_REF_TESTS="$1"
    MUT_REF_TESTS=()
    local -a TEST_FILES
    readarray -t TEST_FILES < <(find "$TESTS_DIR" -type f -name "*.sh" -not -wholename "$(global_test_file)" -follow -print | sort)
    for TEST_FILE in "${TEST_FILES[@]}"; do
        TEST_ID="$(realpath --relative-to="$TESTS_DIR" "${TEST_FILE%.sh}")"
        readarray -t TEST_FUNCTIONS < <(get_test_fn_in_file "$TEST_FILE")
        MUT_REF_TESTS+=( "${TEST_FUNCTIONS[@]/#/"${TEST_ID}::"}" )
    done
}

##
# List the tests defined by the current test suite
#
# Globals
#   TESTS_DIR (in)
#   GLOBAL_TEST_FILE (in)
#
function list_tests {
    get_tests TESTS
    log_info_bright "Tests defined in test suite \`$TESTS_DIR\`:"
    for TEST in "${TESTS[@]}"; do
        log_info "  $TEST"
    done
}

##
# Return whether a given file defines a function
#
# Arguments
#   $1: file path
#   $2: function name
#
# Return
#   0 if file defines the function, non-zero otherwise
#
function is_fn_in_file {
    local -r FILE="$1"
    local -r FUNCTION="$2"
    bash -e -u -o pipefail -c "source \"$FILE\"; typeset -f $FUNCTION" &>/dev/null
}

##
# Get all the test functions in a given file
#
# Arguments
#   $1: path to the file
#
# Outputs
#   stdout: the list of test functions
#
# Return
#   always 0
#
function get_test_fn_in_file {
    local -r FILE="$1"
    bash -euo pipefail -c "source \"$FILE\"; compgen -A function | grep '^$TEST_FN_PREFIX'"
    return 0
}

##
# Run a given function from a given file with the global test file sourced
#
# Arguments
#   $1: path to the file
#   $2: the function to execute
#   $3: log file path
#
# Outputs
#   file at $3: bash traces and function output
#
function run_fn_from_file {
    local -r FILE="$1"
    local -r FUNCTION="$2"
    local -r LOG_FILE="$3"
    shift 3
    # User-provided test callbacks are run from a dedicated subprocess
    # This allow using a restrictive environment (errexit, nounset, pipefail)
    # without impacting the test runner script
    # ${*@Q} is used to pass all remaining arguments individually quoted mainly to support spaces in paths
    bash -x -e -u -o pipefail -c "source \"$FILE\"; $FUNCTION ${*@Q}" &> "$LOG_FILE"
}


##
# Run a given function from a given file with the global test file sourced
#
# Arguments
#   $1: path to the file
#   $2: the function to execute
#   $3: log file path
#
# Outputs
#   file at $3: bash traces and function output
#
function run_fn_from_file_with_global_file {
    local -r FILE="$1"
    local -r FUNCTION="$2"
    local -r LOG_FILE="$3"
    shift 3
    # User-provided test callbacks are run from a dedicated subprocess
    # This allow using a restrictive environment (errexit, nounset, pipefail)
    # without impacting the test runner script
    # ${*@Q} is used to pass all remaining arguments individually quoted mainly to support spaces in paths
    bash -x -e -u -o pipefail -c "source \"$(global_test_file)\"; source \"$FILE\"; $FUNCTION ${*@Q}" &> "$LOG_FILE"
}

##
# Run the setup function for the current test suite
#
# Arguments
#   $1: path to the file
#   $2: the output directory
#   $3: whether this call is skipped
#
# Output
#   stdout: the completion status of the setup function
#
# Return
#   0 if setup is successful, non-zero otherwise
#
function run_test_suite_setup {
    local -r TEST_FILE="$1"
    local -r OUT_DIR="$2"
    local -r SKIPPED="$3"
    local -r LOG_FILE="$OUT_DIR/setup.log"
    if is_fn_in_file "$TEST_FILE" "$TEST_FN_SETUP"; then
        log_info_nonewline "Running test suite setup "
        if [ "$SKIPPED" = "1" ]; then
            log_status SKIPPED
        elif ! run_fn_from_file "$TEST_FILE" "$TEST_FN_SETUP" "$LOG_FILE" "$(declare -p DEVICES)" "$OUT_DIR"; then
            log_status ERROR
            return 1
        else
            log_status OK
        fi
    fi
}

##
# Run the teardown function for the current test suite
#
# Arguments
#   $1: path to the file
#   $2: the output directory
#   $3: whether this call is skipped
#
# Output
#   stdout: the completion status of the teardown function
#
# Return
#   always 0, errors are only printed
#
function run_test_suite_teardown {
    local -r TEST_FILE="$1"
    local -r OUT_DIR="$2"
    local -r SKIPPED="$3"
    local -r LOG_FILE="$OUT_DIR/teardown.log"
    if is_fn_in_file "$TEST_FILE" "$TEST_FN_TEARDOWN"; then
        log_info_nonewline "Running test suite teardown "
        if [ "$SKIPPED" = "1" ]; then
            log_status SKIPPED
        elif ! run_fn_from_file "$TEST_FILE" "$TEST_FN_TEARDOWN" "$LOG_FILE" "$(declare -p DEVICES)" "$OUT_DIR"; then
            log_status ERROR
        else
            log_status OK
        fi
    fi
}

##
# Run the setup function for a given test file
#
# Arguments
#   $1: path to the file
#   $2: the device to run the test on
#   $3: the output directory
#
# Output
#   stdout: the completion status of the setup function
#
# Return
#   0 if setup is successful, non-zero otherwise
#
function run_test_case_setup {
    local -r TEST_FILE="$1"
    local -r DEVICE="$2"
    local -r OUT_DIR="$3"
    local -r LOG_FILE="$OUT_DIR/setup.log"
    if is_fn_in_file "$TEST_FILE" "$TEST_FN_SETUP"; then
        log_info_nonewline "Running setup "
        if [ "$SKIPPED" = "1" ]; then
            log_status SKIPPED
        elif ! run_fn_from_file_with_global_file "$TEST_FILE" "$TEST_FN_SETUP" "$LOG_FILE" "$DEVICE" "$OUT_DIR"; then
            log_status ERROR
            return 1
        else
            log_status OK
        fi
    fi
}

##
# Run a single test function from a given test file
#
# Arguments
#   $1: path to the file
#   $2: the test function to run
#   $3: the device to run the test on
#   $4: the output directory
#   $5: whether this call is skipped
#
# Output
#   stdout: the result of the test
#
# Return
#   always 0, errors are only printed
#
function run_test_case {
    local -r TEST_FILE="$1"
    local -r TEST_FN="$2"
    local -r DEVICE="$3"
    local -r OUT_DIR="$4"
    local -r SKIPPED="$5"
    local -r TEST_NAME=${TEST_FUNCTION#"$TEST_FN_PREFIX"}
    local -r LOG_FILE="$OUT_DIR/test_$TEST_NAME.log"
    log_info_nonewline "Running test $TEST_NAME "
    if [ "$SKIPPED" = "1" ]; then
        log_status SKIPPED
        STAT_TESTS_RUN_SKIPPED=$(( STAT_TESTS_RUN_SKIPPED + 1 ))
    elif run_fn_from_file_with_global_file "$TEST_FILE" "$TEST_FN" "$LOG_FILE" "$DEVICE" "$OUT_DIR"; then
        log_status PASSED
        STAT_TESTS_RUN_PASSED=$(( STAT_TESTS_RUN_PASSED + 1 ))
    else
        case "$?" in
            255)
                log_status SKIPPED
                STAT_TESTS_RUN_SKIPPED=$(( STAT_TESTS_RUN_SKIPPED + 1 ));;
            *)
                log_status FAILED
                STAT_TESTS_RUN_FAILED=$(( STAT_TESTS_RUN_FAILED + 1 ));;
        esac
    fi
}

##
# Run the teardown function for a given test file
#
# Arguments
#   $1: path to the file
#   $2: the device to run the test on
#   $3: the output directory
#   $4: whether this call is skipped
#
# Output
#   stdout: the completion status of the teardown function
#
# Return
#   always 0, errors are only printed
#
function run_test_case_teardown {
    local -r TEST_FILE="$1"
    local -r DEVICE="$2"
    local -r OUT_DIR="$3"
    local -r SKIPPED="$4"
    local -r LOG_FILE="$OUT_DIR/teardown.log"
    if is_fn_in_file "$TEST_FILE" "$TEST_FN_TEARDOWN"; then
        log_info_nonewline "Running teardown "
        if [ "$SKIPPED" = "1" ]; then
            log_status SKIPPED
        elif ! run_fn_from_file_with_global_file "$TEST_FILE" "$TEST_FN_TEARDOWN" "$LOG_FILE" "$DEVICE" "$OUT_DIR"; then
            log_status ERROR
        else
            log_status OK
        fi
    fi
}

##
# Run a complete test case from a given test case file
#
# Arguments
#   $1: path to the file
#   $2: the device to run the test on
#   $3: the output directory
#   $4: whether this call is skipped
#
function run_test_case_file {
    local -r TEST_FILE="$1"
    local -r DEVICE="$2"
    local -r OUT_DIR="$3"
    local SKIPPED="$4"
    local -r TEST_FUNCTIONS=$(get_test_fn_in_file "$TEST_FILE")
    local -r TEST_FUNCTIONS_COUNT="$(printf "%s" "$TEST_FUNCTIONS" | grep -c "" )"
    STAT_TESTS_RUN_TOTAL=$(( STAT_TESTS_RUN_TOTAL + TEST_FUNCTIONS_COUNT ))
    if [ -z "$TEST_FUNCTIONS" ]; then
        log_warning "No test function found"
        return 0
    fi
    if ! run_test_case_setup "$TEST_FILE" "$DEVICE" "$OUT_DIR" "$SKIPPED"; then
        SKIPPED=1
    fi
    for TEST_FUNCTION in $TEST_FUNCTIONS; do
        run_test_case "$TEST_FILE" "$TEST_FUNCTION" "$DEVICE" "$OUT_DIR" "$SKIPPED"
    done
    run_test_case_teardown "$TEST_FILE" "$DEVICE" "$OUT_DIR" "$SKIPPED"
}

##
# Discover and run any test matching the configuration
#
# Globals
#   TESTS_DIR (in)
#   DEVICES (in)
#
# Outputs
#   stdout: the execution progression of the tests
#
function run_all_tests {
    local -r TEST_FILES=$(find "$TESTS_DIR" -type f -name "*.sh" -not -wholename "$(global_test_file)" -follow -print | sort)
    local -r TEST_COUNT=$(( $(echo "$TEST_FILES" | wc -l) * ${#DEVICES[@]} ))
    local CURRENT_TEST=0
    local TEST_ID
    local SKIP="${DRY_RUN}"
    log_info "Running tests in $TESTS_DIR..."
    log_progress "$CURRENT_TEST" "$TEST_COUNT" "Setup"
    if ! run_test_suite_setup "$(global_test_file)" "$OUT_DIR" "$SKIP"; then
        SKIP=1
    fi
    while IFS=$'\n' read -r TEST_FILE; do
        for DEVICE in "${DEVICES[@]}"; do
            CURRENT_TEST=$(( CURRENT_TEST + 1 ))
            TEST_ID="$(realpath --relative-to="$TESTS_DIR" "${TEST_FILE%.sh}")"
            log_progress "$CURRENT_TEST" "$TEST_COUNT" "$TEST_ID"
            local TEST_SUITE_OUT_DIR="$OUT_DIR/$DEVICE/$TEST_ID"
            mkdir -p "$TEST_SUITE_OUT_DIR"
            run_test_case_file "$TEST_FILE" "$DEVICE" "$TEST_SUITE_OUT_DIR" "$SKIP"
        done
    done <<<"$TEST_FILES"
    log_progress "$CURRENT_TEST" "$TEST_COUNT" "Teardown"
    run_test_suite_teardown "$(global_test_file)" "$OUT_DIR" "$SKIP"
}

##
# Script main entry-point
#
# All code (except globals initialization) must be performed here
#
# Arguments
#   $@: the script arguments
#
function main {
    parse_args "$@"
    validate_test_dir
    mkdir -p "$OUT_DIR"
    run_all_tests
    log_summary "$STAT_TESTS_RUN_TOTAL" "$STAT_TESTS_RUN_PASSED" "$STAT_TESTS_RUN_FAILED" "$STAT_TESTS_RUN_SKIPPED"
}

# Run the main entry-point
main "$@"
