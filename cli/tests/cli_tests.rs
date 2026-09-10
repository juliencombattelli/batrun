use std::path::PathBuf;
use std::process::Command;

use serde_json::Value;

#[test]
fn test_binary_runs_ivts_with_json_reporter() {
    let binary_path = env!("CARGO_BIN_EXE_batrun");
    let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests");
    let output_dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("batrun-cli-tests-{}", std::process::id()));

    let output = Command::new(binary_path)
        .arg(tests_dir.join("ivts"))
        .arg(tests_dir.join("ivts-setup-failed"))
        .arg("--out-dir")
        .arg(&output_dir)
        .arg("--target")
        .arg("foo")
        .arg("bar")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to execute binary");

    assert!(
        output.status.success(),
        "binary exited with an error status: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report: Value = serde_json::from_slice(&output.stdout)
        .expect("JSON reporter output should be one valid JSON document");
    let suites = report["test_suites"]
        .as_array()
        .expect("JSON report should contain test suites");
    assert_eq!(suites.len(), 2);

    for target in ["foo", "bar"] {
        let ivts = suites
            .iter()
            .find(|suite| suite["name"] == "batrun-ivts")
            .expect("normal IVT suite should be present");
        let ivts_target = ivts["targets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|target_report| target_report["target"] == target)
            .expect("normal IVT target should be present");
        assert_eq!(
            ivts_target["statistics"],
            serde_json::json!({
                "passed": 15,
                "failed": 7,
                "runner_failed": 0,
                "skipped": 3,
                "total": 25,
            })
        );

        let setup_failed = suites
            .iter()
            .find(|suite| suite["name"] == "batrun-ivts-setup-failed")
            .expect("setup-failed IVT suite should be present");
        let setup_failed_target = setup_failed["targets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|target_report| target_report["target"] == target)
            .expect("setup-failed IVT target should be present");
        assert_eq!(
            setup_failed_target["statistics"],
            serde_json::json!({
                "passed": 0,
                "failed": 1,
                "runner_failed": 0,
                "skipped": 9,
                "total": 10,
            })
        );
        assert_eq!(
            setup_failed_target["test_cases"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|test_case| {
                    test_case["status"] == "skipped"
                        && test_case["skip_reason"] == "test_suite_setup_error"
                })
                .count(),
            9
        );

        let test_output = output_dir
            .join("batrun-ivts")
            .join(target)
            .join("01-ivts/00-basic test.sh")
            .join("test1");
        let test_output = std::fs::read_to_string(&test_output)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", test_output.display()));
        assert!(test_output.starts_with("Test 01 on device "));

        let trace_output_dir = output_dir
            .join("batrun-ivts")
            .join(target)
            .join("01-ivts/40-debug-output-desync.sh");
        let stdout_path = trace_output_dir.join("test_debug_output_desync.stdout.log");
        let stdout = std::fs::read_to_string(&stdout_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", stdout_path.display()));
        assert_eq!(
            stdout,
            "OUTPUT: before multiline trace\n\
             OUTPUT: after multiline trace\n\
             TRACE VALUE: line 1\n\
             TRACE VALUE: line 2\n\
             TRACE VALUE: line 3\n\
               OUTPUT: SUBSTITUTION VALUE\n\
             OUTPUT: final marker\n"
        );

        let debug_path = trace_output_dir.join("test_debug_output_desync.debug.log");
        let debug = std::fs::read_to_string(&debug_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", debug_path.display()));
        assert!(debug.contains("+ echo \"OUTPUT: before multiline trace\"\n"));
        assert!(debug.contains("+ printf '%s\\n' \"$multiline_value\"\n"));
        assert!(debug.contains("+ printf 'SUBSTITUTION VALUE'\n"));

        let output_before = debug.find("OUTPUT: before multiline trace\n").unwrap();
        let multiline_trace = debug
            .find(
                "+ local -r multiline_value='TRACE VALUE: line 1\n\
                 TRACE VALUE: line 2\n\
                 TRACE VALUE: line 3'\n",
            )
            .unwrap();
        let output_after = debug.find("OUTPUT: after multiline trace\n").unwrap();
        assert!(output_before < multiline_trace && multiline_trace < output_after);
    }
}
