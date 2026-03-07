use std::process::Command;

fn cli_bin_path() -> String {
    std::env::var("CARGO_BIN_EXE_atlassy-cli")
        .or_else(|_| std::env::var("CARGO_BIN_EXE_atlassy_cli"))
        .expect("cargo should expose CLI binary path for integration tests")
}

#[test]
fn live_runtime_connection_failure_does_not_panic_runtime() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let output = Command::new(cli_bin_path())
        .arg("run")
        .arg("--request-id")
        .arg("itest-live-connection-failure")
        .arg("--page-id")
        .arg("18841604")
        .arg("--edit-intent")
        .arg("integration test live startup")
        .arg("--mode")
        .arg("no-op")
        .arg("--runtime-backend")
        .arg("live")
        .arg("--force-verify-fail")
        .arg("--artifacts-dir")
        .arg(temp.path())
        .env("ATLASSY_CONFLUENCE_BASE_URL", "http://127.0.0.1:9")
        .env("ATLASSY_CONFLUENCE_EMAIL", "qa@example.com")
        .env("ATLASSY_CONFLUENCE_API_TOKEN", "dummy-token")
        .output()
        .expect("cli invocation should complete");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("pipeline failed:"), "stderr: {stderr}");
    assert!(stderr.contains("ERR_RUNTIME_BACKEND"), "stderr: {stderr}");
    assert!(
        !stderr.contains("Cannot drop a runtime in a context where blocking is not allowed"),
        "stderr: {stderr}"
    );
}

#[test]
fn live_runtime_startup_config_failure_is_deterministic() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let output = Command::new(cli_bin_path())
        .arg("run")
        .arg("--request-id")
        .arg("itest-live-startup-config")
        .arg("--page-id")
        .arg("18841604")
        .arg("--edit-intent")
        .arg("integration test missing startup env")
        .arg("--mode")
        .arg("no-op")
        .arg("--runtime-backend")
        .arg("live")
        .arg("--force-verify-fail")
        .arg("--artifacts-dir")
        .arg(temp.path())
        .env("ATLASSY_CONFLUENCE_BASE_URL", "https://example.atlassian.net")
        .env("ATLASSY_CONFLUENCE_EMAIL", "qa@example.com")
        .env_remove("ATLASSY_CONFLUENCE_API_TOKEN")
        .output()
        .expect("cli invocation should complete");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("pipeline failed:"), "stderr: {stderr}");
    assert!(stderr.contains("ERR_RUNTIME_BACKEND"), "stderr: {stderr}");
    assert!(stderr.contains("live runtime startup failure"), "stderr: {stderr}");
    assert!(
        stderr.contains("missing ATLASSY_CONFLUENCE_API_TOKEN"),
        "stderr: {stderr}"
    );
}
