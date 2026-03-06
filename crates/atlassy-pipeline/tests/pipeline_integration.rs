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

fn make_orchestrator_with_fixture(
    artifact_root: &Path,
    fixture_name: &str,
) -> Orchestrator<StubConfluenceClient> {
    let mut pages = HashMap::new();
    pages.insert(
        "18841604".to_string(),
        StubPage {
            version: 7,
            adf: load_fixture(fixture_name),
        },
    );

    Orchestrator::new(StubConfluenceClient::new(pages), artifact_root)
}

fn read_state_output(artifact_root: &Path, run_id: &str, state: &str) -> serde_json::Value {
    let path = artifact_root
        .join("artifacts")
        .join(run_id)
        .join(state)
        .join("state_output.json");
    let text = fs::read_to_string(path).expect("state output should exist");
    serde_json::from_str(&text).expect("state output should be valid JSON")
}

#[test]
fn happy_path_run_succeeds() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "prose_only_adf.json");

    let mut request = sample_request("run-happy");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::SimpleScopedProseUpdate {
        target_path: "/content/1/content/0/text".to_string(),
        markdown: "Updated prose body".to_string(),
    };

    let summary = orchestrator.run(request).expect("run should succeed");
    assert!(summary.success);
    assert_eq!(
        summary.applied_paths,
        vec!["/content/1/content/0/text".to_string()]
    );
    assert_eq!(orchestrator.client().publish_attempts(), 1);
}

#[test]
fn contract_validation_failure_is_reported() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "prose_only_adf.json");

    let mut request = sample_request("run-contract-fail");
    request.request_id = String::new();

    let error = orchestrator.run(request).expect_err("run should fail");
    match error {
        PipelineError::Contract(ContractError::MissingField("request_id")) => {}
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn verify_failure_blocks_publish() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "prose_only_adf.json");

    let mut request = sample_request("run-verify-fail");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::SimpleScopedProseUpdate {
        target_path: "/content/1/content/0/text".to_string(),
        markdown: "Updated prose body".to_string(),
    };
    request.force_verify_fail = true;

    let error = orchestrator
        .run(request)
        .expect_err("run should fail at verify");
    match error {
        PipelineError::Hard { state, .. } => assert_eq!(state, PipelineState::Verify),
        other => panic!("unexpected error: {other:?}"),
    }
    assert_eq!(orchestrator.client().publish_attempts(), 0);
}

#[test]
fn state_tracker_rejects_out_of_order_execution() {
    let mut tracker = StateTracker::new();
    tracker
        .transition_to(PipelineState::Fetch)
        .expect("fetch transition should pass");
    let error = tracker
        .transition_to(PipelineState::Patch)
        .expect_err("patch transition should fail");
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
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "prose_only_adf.json");
    let mut request = sample_request("run-success-artifacts");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::SimpleScopedProseUpdate {
        target_path: "/content/1/content/0/text".to_string(),
        markdown: "Updated prose body".to_string(),
    };

    orchestrator.run(request).expect("run should succeed");

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
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "prose_only_adf.json");

    let mut request = sample_request("run-failed-artifacts");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::SimpleScopedProseUpdate {
        target_path: "/content/1/content/0/text".to_string(),
        markdown: "Updated prose body".to_string(),
    };
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

#[test]
fn extract_prose_filters_non_prose_routes() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "mixed_routes_adf.json");

    let mut request = sample_request("run-mixed-routes");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::NoOp;
    orchestrator.run(request).expect("run should succeed");

    let output = read_state_output(temp.path(), "run-mixed-routes", "extract_prose");
    let map = output["payload"]["md_to_adf_map"]
        .as_array()
        .expect("md_to_adf_map should be array");

    assert!(!map.is_empty());
    for entry in map {
        let path = entry["adf_path"].as_str().expect("path should be string");
        assert!(
            !path.starts_with("/content/2"),
            "table path leaked into prose map"
        );
        assert!(
            !path.starts_with("/content/3"),
            "locked path leaked into prose map"
        );
    }
}

#[test]
fn extract_prose_mapping_is_deterministic() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "prose_only_adf.json");

    let mut first = sample_request("run-map-a");
    first.scope_selectors = vec![];
    first.run_mode = RunMode::NoOp;
    orchestrator.run(first).expect("first run should succeed");

    let mut second = sample_request("run-map-b");
    second.scope_selectors = vec![];
    second.run_mode = RunMode::NoOp;
    orchestrator.run(second).expect("second run should succeed");

    let map_a =
        read_state_output(temp.path(), "run-map-a", "extract_prose")["payload"]["md_to_adf_map"]
            .clone();
    let map_b =
        read_state_output(temp.path(), "run-map-b", "extract_prose")["payload"]["md_to_adf_map"]
            .clone();
    assert_eq!(map_a, map_b);
}

#[test]
fn unmapped_prose_path_fails_before_publish() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "prose_only_adf.json");

    let mut request = sample_request("run-unmapped-path");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::SimpleScopedProseUpdate {
        target_path: "/content/99/content/0/text".to_string(),
        markdown: "Invalid update".to_string(),
    };

    let error = orchestrator
        .run(request)
        .expect_err("unmapped prose path should fail");
    match error {
        PipelineError::Hard { state, .. } => assert_eq!(state, PipelineState::MdAssistEdit),
        other => panic!("unexpected error: {other:?}"),
    }
    assert_eq!(orchestrator.client().publish_attempts(), 0);
}

#[test]
fn top_level_boundary_violation_fails_before_publish() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "prose_only_adf.json");

    let mut request = sample_request("run-boundary-violation");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::SimpleScopedProseUpdate {
        target_path: "/content/1".to_string(),
        markdown: "Replacement".to_string(),
    };

    let error = orchestrator
        .run(request)
        .expect_err("boundary violation should fail");
    match error {
        PipelineError::Hard { state, .. } => assert_eq!(state, PipelineState::MdAssistEdit),
        other => panic!("unexpected error: {other:?}"),
    }
    assert_eq!(orchestrator.client().publish_attempts(), 0);
}

#[test]
fn table_route_target_is_rejected_for_prose_assist() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "mixed_routes_adf.json");

    let mut request = sample_request("run-table-route-rejection");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::SimpleScopedProseUpdate {
        target_path: "/content/2/content/0/content/0/content/0/text".to_string(),
        markdown: "Should fail".to_string(),
    };

    let error = orchestrator
        .run(request)
        .expect_err("table path should be rejected");
    match error {
        PipelineError::Hard { state, .. } => assert_eq!(state, PipelineState::MdAssistEdit),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn prose_state_artifacts_include_mapping_and_candidates() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "prose_only_adf.json");

    let mut request = sample_request("run-prose-artifacts");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::SimpleScopedProseUpdate {
        target_path: "/content/1/content/0/text".to_string(),
        markdown: "Updated prose body".to_string(),
    };
    orchestrator.run(request).expect("run should succeed");

    let extract_output = read_state_output(temp.path(), "run-prose-artifacts", "extract_prose");
    assert!(extract_output["payload"]["md_to_adf_map"].is_array());
    assert!(extract_output["payload"]["editable_prose_paths"].is_array());

    let md_output = read_state_output(temp.path(), "run-prose-artifacts", "md_assist_edit");
    assert!(md_output["payload"]["prose_change_candidates"].is_array());
    assert_eq!(
        md_output["payload"]["prose_changed_paths"],
        serde_json::json!(["/content/1/content/0/text"])
    );
}
