#!/bin/bash

declare -ra KNOWN_DEVICES=(
    test
    other
)

declare -rA KNOWN_DEVICES_IP=(
    [test]="192.168.1.1"
    [other]="192.168.1.2"
)

# If the provided device is part of the KNOWN_DEVICES_IP then use the corresponding IP
# Otherwise assume the provided device is an IP address
function resolve_device_ip {
    local -r DEVICE="$1"
    if [[ -v KNOWN_DEVICES_IP[$DEVICE] ]]; then
        echo "${KNOWN_DEVICES_IP[$DEVICE]}"
    else
        echo "$DEVICE"
    fi
}

function setup {
    local -r DECLARE_DEVICE="$1"
    eval "$DECLARE_DEVICE"
    local -r OUT_DIR="$2"
    for DEVICE in "${DEVICES[@]}"; do
        echo "Connecting to $DEVICE @ $(resolve_device_ip $DEVICE)..."
    done
    return 0
}

function teardown {
    return 0
}
