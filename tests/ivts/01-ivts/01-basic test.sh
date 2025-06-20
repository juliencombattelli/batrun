#!/bin/bash

function setup {
    return 0
}

function test_01_ok {
    declare -p KNOWN_DEVICES_IP
    local -r DEVICE="$1"
    local -r OUT_DIR="$2"
    local -r DEVICE_IP="$(resolve_device_ip DEVICE)"
    echo "Test 01 on device $DEVICE_IP" > "$OUT_DIR/test1"
    return 0
}

function test_02_fail {
    local -r DEVICE="$1"
    local -r OUT_DIR="$2"
    local -r DEVICE_IP="$(resolve_device_ip DEVICE)"
    echo "Test 02 on device ${KNOWN_DEVICES_IP[$DEVICE]}" > "$OUT_DIR/test2"
    return 1
}

function teardown {
    return 0
}
