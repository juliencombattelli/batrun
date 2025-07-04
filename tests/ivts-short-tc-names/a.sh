#!/bin/bash

# Define 10 test cases so that the header with test case count is longer that
# the test cases name:
# "Test cases: 10"
# "a.sh::test_i"

function _test {
    if [ "$1" = "foo" ]; then
        return 0
    elif [ "$1" = "bar" ]; then
        return 1
    else
        export BATRUN_SKIPPED="Not supported target \`$1\`"
    fi
}

function test_0 {
    export BATRUN_TOTO="toto"
    _test "$@"
}

function test_1 {
    _test "$@"
}

function test_2 {
    _test "$@"
}

function test_3 {
    _test "$@"
}

function test_4 {
    _test "$@"
}

function test_5 {
    _test "$@"
}

function test_6 {
    _test "$@"
}

function test_7 {
    _test "$@"
}

function test_8 {
    _test "$@"
}

function test_9 {
    _test "$@"
}
