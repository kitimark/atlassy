use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use atlassy_confluence::{ConfluenceClient, StubConfluenceClient, StubPage};
use atlassy_contracts::{ContractError, PipelineState};
use atlassy_pipeline::{Orchestrator, PipelineError, RunMode, RunRequest, StateTracker};

fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn load_fixture(name: &str) -> serde_json::Value {
    let text = fs::read_to_string(fixture_path(name)).expect("fixture file should be readable");
    serde_json::from_str(&text).expect("fixture should be valid JSON")
}

fn sample_request(run_id: &str) -> RunRequest {
    RunRequest {
        request_id: run_id.to_string(),
        page_id: "18841604".to_string(),
        edit_intent: "Update one section".to_string(),
        scope_selectors: vec!["heading:Overview".to_string()],
        timestamp: "2026-03-06T10:00:00Z".to_string(),
        run_mode: RunMode::NoOp,
        force_verify_fail: false,
    }
}

fn make_orchestrator(artifact_root: &Path) -> Orchestrator<StubConfluenceClient> {
    let mut pages = HashMap::new();
    pages.insert(
        "18841604".to_string(),
        StubPage {
            version: 7,
            adf: load_fixture("page_adf.json"),
        },
    );

    Orchestrator::new(StubConfluenceClient::new(pages), artifact_root)
}

#[test]
fn happy_path_run_succeeds() {
    let temp = tempfile::tempdir().unwrap();
    let mut orchestrator = make_orchestrator(temp.path());

    let summary = orchestrator.run(sample_request("run-happy")).unwrap();
    assert!(summary.success);
    assert_eq!(orchestrator.client().publish_attempts(), 1);
}

#[test]
fn contract_validation_failure_is_reported() {
    let temp = tempfile::tempdir().unwrap();
    let mut orchestrator = make_orchestrator(temp.path());

    let mut request = sample_request("run-contract-fail");
    request.request_id = String::new();

    let error = orchestrator.run(request).unwrap_err();
    match error {
        PipelineError::Contract(ContractError::MissingField("request_id")) => {}
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn verify_failure_blocks_publish() {
    let temp = tempfile::tempdir().unwrap();
    let mut orchestrator = make_orchestrator(temp.path());

    let mut request = sample_request("run-verify-fail");
    request.force_verify_fail = true;

    let error = orchestrator.run(request).unwrap_err();
    match error {
        PipelineError::Hard { state, .. } => assert_eq!(state, PipelineState::Verify),
        other => panic!("unexpected error: {other:?}"),
    }
    assert_eq!(orchestrator.client().publish_attempts(), 0);
}

#[test]
fn state_tracker_rejects_out_of_order_execution() {
    let mut tracker = StateTracker::new();
    tracker.transition_to(PipelineState::Fetch).unwrap();
    let error = tracker.transition_to(PipelineState::Patch).unwrap_err();
    assert_eq!(
        error,
        ContractError::InvalidTransition {
            expected: "classify".to_string(),
            actual: "patch".to_string(),
        }
    );
}

#[test]
fn replay_artifacts_exist_for_successful_run() {
    let temp = tempfile::tempdir().unwrap();
    let mut orchestrator = make_orchestrator(temp.path());
    orchestrator
        .run(sample_request("run-success-artifacts"))
        .unwrap();

    let states = [
        "fetch",
        "classify",
        "extract_prose",
        "md_assist_edit",
        "adf_table_edit",
        "merge_candidates",
        "patch",
        "verify",
        "publish",
    ];
    for state in states {
        let state_dir = temp
            .path()
            .join("artifacts")
            .join("run-success-artifacts")
            .join(state);
        assert!(state_dir.join("state_input.json").exists());
        assert!(state_dir.join("state_output.json").exists());
        assert!(state_dir.join("diagnostics.json").exists());
    }

    assert!(
        temp.path()
            .join("artifacts")
            .join("run-success-artifacts")
            .join("summary.json")
            .exists()
    );
}

#[test]
fn replay_artifacts_exist_for_failed_run_until_failure_state() {
    let temp = tempfile::tempdir().unwrap();
    let mut orchestrator = make_orchestrator(temp.path());

    let mut request = sample_request("run-failed-artifacts");
    request.force_verify_fail = true;
    let _ = orchestrator.run(request);

    let run_dir = temp.path().join("artifacts").join("run-failed-artifacts");
    for state in [
        "fetch",
        "classify",
        "extract_prose",
        "md_assist_edit",
        "adf_table_edit",
        "merge_candidates",
        "patch",
        "verify",
    ] {
        let state_dir = run_dir.join(state);
        assert!(state_dir.join("state_input.json").exists());
        assert!(state_dir.join("state_output.json").exists());
        assert!(state_dir.join("diagnostics.json").exists());
    }

    assert!(!run_dir.join("publish").exists());
    assert!(run_dir.join("summary.json").exists());
}
