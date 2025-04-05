#!/usr/bin/env fish

# Needs at least Fish 3.0
# Ensure the script runs with Fish shell

function script_dir
    # Get the directory where this script is located
    set -l source (status --current-filename)
    while test -L $source
        set -l dir (dirname (realpath $source))
        set source (readlink $source)
        if not string match -q -- "/" $source
            set source "$dir/$source"
        end
    end
    dirname (realpath $source)
end

# Constants
set -g INCLUDE_DIR (script_dir)/include
set -g BUILTIN_TESTS_DIR (script_dir)/tests
set -g GLOBAL_TEST_FILE "tests.sh"
set -g TEST_FN_SETUP "setup"
set -g TEST_FN_TEARDOWN "teardown"
set -g TEST_FN_PREFIX "test_"

# Globals
set -g DEVICES
set -g OUT_DIR
set -g TESTS_DIR $BUILTIN_TESTS_DIR
set -g TESTS
set -g DRY_RUN 0
set -g STAT_TESTS_RUN_TOTAL 0
set -g STAT_TESTS_RUN_PASSED 0
set -g STAT_TESTS_RUN_FAILED 0
set -g STAT_TESTS_RUN_SKIPPED 0

function help
    echo "Fish test runner."
    echo "Run tests written in Fish on some specified devices."
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
end

function parse_args
    # Parse command-line arguments
    set -l argv_copy $argv
    while test (count $argv_copy) -gt 0
        switch $argv_copy[1]
            case '--help' '-h'
                help
                exit 0
            case '--out-dir' '-o'
                set OUT_DIR $argv_copy[2]
                set argv_copy (drop $argv_copy 2)
            case '--device' '-d'
                set DEVICES $DEVICES $argv_copy[2]
                set argv_copy (drop $argv_copy 2)
            case '--list-known-devices'
                validate_test_dir
                list_known_devices
                exit 0
            case '--tests-dir'
                set TESTS_DIR $argv_copy[2]
                set argv_copy (drop $argv_copy 2)
            case '--tests' '-t'
                set TESTS $argv_copy[2]
                set argv_copy (drop $argv_copy 2)
                echo "ERROR: Not implemented."
                exit 1
            case '--list-tests' '-l'
                list_tests
                exit 0
            case '--dry-run' '-n'
                set DRY_RUN 1
                set argv_copy (drop $argv_copy 1)
            case '*'
                echo "ERROR: Unknown option '$argv_copy[1]'."
                exit 1
        end
    end

    if test (count $DEVICES) -eq 0
        echo "ERROR: No device specified."
        exit 1
    end

    if test -z "$OUT_DIR"
        echo "ERROR: No output directory specified."
        exit 1
    end

    set OUT_DIR "$OUT_DIR/(date +%Y%m%d%H%M%S)"
end

function validate_test_dir
    # Validate the test directory
    if not test -f "$TESTS_DIR/$GLOBAL_TEST_FILE"
        echo "ERROR: Global test file '$GLOBAL_TEST_FILE' not found in $TESTS_DIR."
        exit 1
    end
end

function list_tests
    # List all tests in the test directory
    echo "Tests defined in test suite '$TESTS_DIR':"
    for file in (find $TESTS_DIR -type f -name "*.sh" -not -wholename "$TESTS_DIR/$GLOBAL_TEST_FILE" | sort)
        echo "  (basename $file .sh)"
    end
end

function run_all_tests
    # Discover and run all tests
    set -l test_files (find $TESTS_DIR -type f -name "*.sh" -not -wholename "$TESTS_DIR/$GLOBAL_TEST_FILE" | sort)
    set -l total_tests (math (count $test_files) \* (count $DEVICES))
    set -l current_test 0
    set -l skip $DRY_RUN

    echo "Running tests in $TESTS_DIR..."
    if not run_test_suite_setup "$TESTS_DIR/$GLOBAL_TEST_FILE" "$OUT_DIR" $skip
        set skip 1
    end

    for test_file in $test_files
        for device in $DEVICES
            set current_test (math $current_test + 1)
            set test_id (basename $test_file .sh)
            set test_out_dir "$OUT_DIR/$device/$test_id"
            mkdir -p $test_out_dir
            run_test_case_file $test_file $device $test_out_dir $skip
        end
    end

    run_test_suite_teardown "$TESTS_DIR/$GLOBAL_TEST_FILE" "$OUT_DIR" $skip
end

function run_test_suite_setup
    # Run the setup function for the test suite
    set -l test_file $argv[1]
    set -l out_dir $argv[2]
    set -l skipped $argv[3]
    set -l log_file "$out_dir/setup.log"

    if is_fn_in_file $test_file $TEST_FN_SETUP
        echo -n "Running test suite setup "
        if test $skipped -eq 1
            echo "SKIPPED"
        else
            if not bash -c "source $test_file; $TEST_FN_SETUP" > $log_file 2>&1
                echo "ERROR"
                return 1
            else
                echo "OK"
            end
        end
    end
end

function run_test_suite_teardown
    # Run the teardown function for the test suite
    set -l test_file $argv[1]
    set -l out_dir $argv[2]
    set -l skipped $argv[3]
    set -l log_file "$out_dir/teardown.log"

    if is_fn_in_file $test_file $TEST_FN_TEARDOWN
        echo -n "Running test suite teardown "
        if test $skipped -eq 1
            echo "SKIPPED"
        else
            if not bash -c "source $test_file; $TEST_FN_TEARDOWN" > $log_file 2>&1
                echo "ERROR"
            else
                echo "OK"
            end
        end
    end
end

function is_fn_in_file
    # Check if a function exists in a file
    set -l file $argv[1]
    set -l function $argv[2]
    bash -c "source $file; declare -f $function" > /dev/null 2>&1
end

function main
    parse_args $argv
    validate_test_dir
    mkdir -p $OUT_DIR
    run_all_tests
    echo "SUMMARY: Total: $STAT_TESTS_RUN_TOTAL, Passed: $STAT_TESTS_RUN_PASSED, Failed: $STAT_TESTS_RUN_FAILED, Skipped: $STAT_TESTS_RUN_SKIPPED"
end

main $argv
