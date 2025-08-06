#!/bin/bash

function setup {
    return 0
}

function test_skip {
    local -r DEVICE="$1"
    export BATRUN_SKIP="Invalid target"
    return 0
}

function teardown {
    return 1
}
