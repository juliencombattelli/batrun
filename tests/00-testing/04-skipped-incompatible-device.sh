#!/bin/bash

function setup {
    return 0
}

function test_skip {
    local -r DEVICE="$1"
    if [ "$DEVICE" = "test" ]; then
        return 255
    fi
    return 0
}

function teardown {
    return 1
}
