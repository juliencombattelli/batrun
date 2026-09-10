#!/usr/bin/env bash

# Merge stderr into the captured output, then retain that same stream on a
# separate descriptor so DEBUG output bypasses command substitutions.
exec 2>&1
exec {__batrun_trace_fd}>&1

# Runtime values are passed as positional arguments by the Rust driver.
readonly __batrun_fixture_path="$1"
readonly __batrun_test_file_path="$2"
readonly __batrun_test_function="$3"
readonly __batrun_target="$4"
readonly __batrun_out_dir="$5"
readonly __batrun_envout_file="$6"
readonly __batrun_trace_marker="$7"
readonly __batrun_trace_fd

# Propagate the DEBUG trap into sourced functions and command substitutions.
set -o functrace

function __batrun_trace {
    local command_status=$?
    # NUL framing lets the Rust driver identify multiline commands unambiguously.
    printf '\0%s\0%s\0' "$__batrun_trace_marker" "$1" >&"$__batrun_trace_fd"
    return "$command_status"
}

trap '__batrun_trace "$BASH_COMMAND"' DEBUG

# Avoid sourcing the fixture twice when it is the test file sourced below.
if [[ -n "$__batrun_fixture_path" && "$__batrun_fixture_path" != "$__batrun_test_file_path" ]]; then
    source "$__batrun_fixture_path"
fi
source "$__batrun_test_file_path"
"$__batrun_test_function" "$__batrun_target" "$__batrun_out_dir"
test_status=$?

# Keep harness bookkeeping out of the trace and preserve the test's status.
trap - DEBUG
{ env | grep -E '^BATRUN_' || true; } > "$__batrun_envout_file"
exit "$test_status"
