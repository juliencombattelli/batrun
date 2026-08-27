use std::path::PathBuf;
use std::process::Command;

#[test]
fn test_binary_runs_test_suites() {
    let binary_path = env!("CARGO_BIN_EXE_batrun");
    let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests");
    let output_dir = std::env::temp_dir().join(format!(
        "batrun-cli-tests-{}",
        std::process::id()
    ));

    let output = Command::new(binary_path)
        .arg(tests_dir.join("ivts"))
        .arg(tests_dir.join("ivts-setup-failed"))
        .arg("--out-dir")
        .arg(&output_dir)
        .arg("--target")
        .arg("foo")
        .arg("bar")
        .spawn()
        .expect("failed to execute binary")
        .wait_with_output()
        .expect("failed to read binary output");

    let _ = std::fs::remove_dir_all(output_dir);
    assert!(output.status.success(), "binary exited with an error status");
}
