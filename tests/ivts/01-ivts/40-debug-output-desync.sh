#!/bin/bash

function test_debug_output_desync {
    local -r OUT_DIR="$2"

    echo "OUTPUT: before multiline trace"
    local -r multiline_value=$'TRACE VALUE: line 1\nTRACE VALUE: line 2\nTRACE VALUE: line 3'
    echo "OUTPUT: after multiline trace"
    printf '%s\n' "$multiline_value"

    local -r command_substitution_value="$(printf 'SUBSTITUTION VALUE')"
    if [[ "$command_substitution_value" != "SUBSTITUTION VALUE" ]]; then
        echo "Trace output leaked into command substitution"
        return 1
    fi
    echo "OUTPUT: $command_substitution_value"
    echo "OUTPUT: final marker"

    return 0
}
