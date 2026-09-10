#!/usr/bin/env bash

exec 2>&1

readonly __batrun_fixture_path="$1"
readonly __batrun_test_file_path="$2"
readonly __batrun_test_function="$3"
readonly __batrun_target="$4"
readonly __batrun_out_dir="$5"
readonly __batrun_envout_file="$6"
readonly __batrun_trace_marker="$7"

set -o functrace

__batrun_trace() {
    printf '\0%s\0%s\0' "$__batrun_trace_marker" "$1"
}

trap '__batrun_trace "$BASH_COMMAND"' DEBUG

if [[ -n "$__batrun_fixture_path" && "$__batrun_fixture_path" != "$__batrun_test_file_path" ]]; then
    source "$__batrun_fixture_path"
fi
source "$__batrun_test_file_path"
"$__batrun_test_function" "$__batrun_target" "$__batrun_out_dir"
test_status=$?

trap - DEBUG
{ env | grep -E '^BATRUN_' || true; } > "$__batrun_envout_file"
exit "$test_status"
