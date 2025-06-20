#!/bin/bash

function setup {
    return 0
}

function test_01_ok {
    echo "Test 01 on device $DEVICE" > "$OUT_DIR/test1"
    return 0
}

function test_02_fail {
    echo "Test 02 on device $DEVICE" > "$OUT_DIR/test2"
    return 1
}

function teardown {
    return 0
}
