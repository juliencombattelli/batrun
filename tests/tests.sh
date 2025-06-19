#!/bin/bash

declare -rA KNOWN_DEVICES_IP=(
    [foo]="192.168.1.1"
    [bar]="192.168.1.2"
    [baz]="192.168.1.3"
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
    local -r DEVICE="$1"
    local -r OUT_DIR="$2"
    echo "Connecting to $DEVICE @ $(resolve_device_ip $DEVICE)..."
    return 0
}

function teardown {
    return 0
}
