use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use atlassy_confluence::{
    ConfluenceClient, ConfluenceError, FetchPageResponse, PublishPageResponse,
    StubConfluenceClient, StubPage,
};
use atlassy_contracts::{
    ContractError, ERR_BOOTSTRAP_INVALID_STATE, ERR_BOOTSTRAP_REQUIRED, ERR_RUNTIME_BACKEND,
    ERR_TABLE_SHAPE_CHANGE, FLOW_OPTIMIZED, PATTERN_A, PIPELINE_VERSION, PipelineState,
    ProvenanceStamp, RUNTIME_STUB, TableOperation,
};
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
        edit_intent_hash: format!("hash-{run_id}"),
        flow: FLOW_OPTIMIZED.to_string(),
        pattern: PATTERN_A.to_string(),
        scope_selectors: vec!["heading:Overview".to_string()],
        timestamp: "2026-03-06T10:00:00Z".to_string(),
        provenance: ProvenanceStamp {
            git_commit_sha: "0123456789abcdef0123456789abcdef01234567".to_string(),
            git_dirty: false,
            pipeline_version: PIPELINE_VERSION.to_string(),
            runtime_mode: RUNTIME_STUB.to_string(),
        },
        run_mode: RunMode::NoOp,
        target_index: 0,
        force_verify_fail: false,
        bootstrap_empty_page: false,
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

#[derive(Debug, Clone)]
struct PublishTransportErrorClient {
    page_version: u64,
    adf: serde_json::Value,
    publish_attempts: usize,
    message: String,
}

impl PublishTransportErrorClient {
    fn new(adf: serde_json::Value, page_version: u64, message: impl Into<String>) -> Self {
        Self {
            page_version,
            adf,
            publish_attempts: 0,
            message: message.into(),
        }
    }
}

impl ConfluenceClient for PublishTransportErrorClient {
    fn fetch_page(&mut self, _page_id: &str) -> Result<FetchPageResponse, ConfluenceError> {
        Ok(FetchPageResponse {
            page_version: self.page_version,
            adf: self.adf.clone(),
        })
    }

    fn publish_page(
        &mut self,
        _page_id: &str,
        _page_version: u64,
        _candidate_adf: &serde_json::Value,
    ) -> Result<PublishPageResponse, ConfluenceError> {
        self.publish_attempts += 1;
        Err(ConfluenceError::Transport(self.message.clone()))
    }

    fn create_page(
        &mut self,
        _title: &str,
        _parent_page_id: &str,
        _space_key: &str,
    ) -> Result<atlassy_confluence::CreatePageResponse, ConfluenceError> {
        Err(ConfluenceError::NotImplemented)
    }

    fn publish_attempts(&self) -> usize {
        self.publish_attempts
    }
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

fn assert_hard_error(error: PipelineError, state: PipelineState, code: &str) {
    match error {
        PipelineError::Hard {
            state: got_state,
            code: got_code,
            ..
        } => {
            assert_eq!(got_state, state);
            assert_eq!(got_code, code);
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn happy_path_run_succeeds() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "prose_only_adf.json");

    let mut request = sample_request("run-happy");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::SimpleScopedProseUpdate {
        target_path: Some("/content/1/content/0/text".to_string()),
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
fn explicit_target_path_skips_discovery() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "prose_only_adf.json");

    let mut request = sample_request("run-explicit-path");
    request.scope_selectors = vec![];
    request.target_index = 0;
    request.run_mode = RunMode::SimpleScopedProseUpdate {
        target_path: Some("/content/1/content/0/text".to_string()),
        markdown: "Updated prose body".to_string(),
    };

    let summary = orchestrator.run(request).expect("run should succeed");
    assert!(summary.success);
    assert_eq!(summary.discovered_target_path, None);
    assert_eq!(
        summary.applied_paths,
        vec!["/content/1/content/0/text".to_string()]
    );
}

#[test]
fn pipeline_auto_discovers_and_patches() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "prose_only_adf.json");

    let mut request = sample_request("run-auto-discovery");
    request.scope_selectors = vec![];
    request.target_index = 0;
    request.run_mode = RunMode::SimpleScopedProseUpdate {
        target_path: None,
        markdown: "Updated prose body".to_string(),
    };

    let summary = orchestrator.run(request).expect("run should succeed");
    assert!(summary.success);
    assert_eq!(
        summary.discovered_target_path,
        Some("/content/1/content/0/text".to_string())
    );
    assert_eq!(
        summary.applied_paths,
        vec!["/content/1/content/0/text".to_string()]
    );

    let patch_output = read_state_output(temp.path(), "run-auto-discovery", "patch");
    assert_eq!(
        patch_output["payload"]["patch_ops"][0]["path"],
        serde_json::json!("/content/1/content/0/text")
    );
}

#[test]
fn scoped_prose_update_only_touches_in_scope_paths() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "multi_section_adf.json");

    let mut request = sample_request("run-scoped-prose-update");
    request.run_mode = RunMode::SimpleScopedProseUpdate {
        target_path: Some("/content/1/content/0/text".to_string()),
        markdown: "Scoped update".to_string(),
    };

    let summary = orchestrator
        .run(request)
        .expect("scoped prose update should succeed");
    assert!(summary.success);
    assert_eq!(
        summary.applied_paths,
        vec!["/content/1/content/0/text".to_string()]
    );
    assert_eq!(orchestrator.client().publish_attempts(), 1);
}

#[test]
fn scoped_auto_discovery_finds_target_within_section() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "multi_section_adf.json");

    let mut request = sample_request("run-scoped-auto-discovery");
    request.run_mode = RunMode::SimpleScopedProseUpdate {
        target_path: None,
        markdown: "Scoped auto discovered update".to_string(),
    };

    let summary = orchestrator
        .run(request)
        .expect("scoped auto discovery should succeed");
    let discovered = summary
        .discovered_target_path
        .clone()
        .expect("discovered target path should be populated");
    assert!(
        discovered.starts_with("/content/0/") || discovered.starts_with("/content/1/"),
        "expected discovered path in Overview section, got {discovered}"
    );
    assert!(
        !discovered.starts_with("/content/2/")
            && !discovered.starts_with("/content/3/")
            && !discovered.starts_with("/content/4/"),
        "expected discovered path to exclude Details section, got {discovered}"
    );
    assert_eq!(summary.applied_paths, vec![discovered]);
    assert_eq!(orchestrator.client().publish_attempts(), 1);
}

#[test]
fn table_cell_update_run_succeeds() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator =
        make_orchestrator_with_fixture(temp.path(), "table_allowed_cell_update_adf.json");

    let mut request = sample_request("run-table-update");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::SimpleScopedTableCellUpdate {
        target_path: Some("/content/1/content/0/content/0/content/0/content/0/text".to_string()),
        text: "Updated table cell".to_string(),
    };

    let summary = orchestrator
        .run(request)
        .expect("table update should succeed");
    assert!(summary.success);
    assert_eq!(
        summary.applied_paths,
        vec!["/content/1/content/0/content/0/content/0/content/0/text".to_string()]
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
        target_path: Some("/content/1/content/0/text".to_string()),
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
fn publish_transport_error_maps_to_runtime_backend_publish_failure() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let client = PublishTransportErrorClient::new(
        load_fixture("prose_only_adf.json"),
        7,
        "http_status=400 body=Must supply an incremented version when updating Content. No version",
    );
    let mut orchestrator = Orchestrator::new(client, temp.path());

    let mut request = sample_request("run-live-publish-400");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::SimpleScopedProseUpdate {
        target_path: Some("/content/1/content/0/text".to_string()),
        markdown: "Updated prose body".to_string(),
    };

    let error = orchestrator
        .run(request)
        .expect_err("publish transport failure should be hard error");
    assert_hard_error(error, PipelineState::Publish, ERR_RUNTIME_BACKEND);
    assert_eq!(orchestrator.client().publish_attempts(), 1);

    let run_dir = temp.path().join("artifacts").join("run-live-publish-400");
    let summary: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("summary.json")).expect("summary should exist"),
    )
    .expect("summary should deserialize");
    assert_eq!(summary["failure_state"], serde_json::json!("publish"));
    assert_eq!(
        summary["error_codes"],
        serde_json::json!([ERR_RUNTIME_BACKEND])
    );
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
    let mut orchestrator =
        make_orchestrator_with_fixture(temp.path(), "table_allowed_cell_update_adf.json");
    let mut request = sample_request("run-success-artifacts");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::SimpleScopedTableCellUpdate {
        target_path: Some("/content/1/content/0/content/0/content/0/content/0/text".to_string()),
        text: "Updated table cell".to_string(),
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
    let mut orchestrator =
        make_orchestrator_with_fixture(temp.path(), "table_allowed_cell_update_adf.json");

    let mut request = sample_request("run-failed-artifacts");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::SimpleScopedTableCellUpdate {
        target_path: Some("/content/1/content/0/content/0/content/0/content/0/text".to_string()),
        text: "Updated table cell".to_string(),
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
        target_path: Some("/content/99/content/0/text".to_string()),
        markdown: "Invalid update".to_string(),
    };

    let error = orchestrator
        .run(request)
        .expect_err("unmapped prose path should fail");
    assert_hard_error(error, PipelineState::MdAssistEdit, "ERR_ROUTE_VIOLATION");
    assert_eq!(orchestrator.client().publish_attempts(), 0);
}

#[test]
fn top_level_boundary_violation_fails_before_publish() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "prose_only_adf.json");

    let mut request = sample_request("run-boundary-violation");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::SimpleScopedProseUpdate {
        target_path: Some("/content/1".to_string()),
        markdown: "Replacement".to_string(),
    };

    let error = orchestrator
        .run(request)
        .expect_err("boundary violation should fail");
    assert_hard_error(error, PipelineState::MdAssistEdit, "ERR_SCHEMA_INVALID");
    assert_eq!(orchestrator.client().publish_attempts(), 0);
}

#[test]
fn table_route_target_is_rejected_for_prose_assist() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "mixed_routes_adf.json");

    let mut request = sample_request("run-table-route-rejection");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::SimpleScopedProseUpdate {
        target_path: Some("/content/2/content/0/content/0/content/0/text".to_string()),
        markdown: "Should fail".to_string(),
    };

    let error = orchestrator
        .run(request)
        .expect_err("table path should be rejected");
    assert_hard_error(error, PipelineState::MdAssistEdit, "ERR_ROUTE_VIOLATION");
}

#[test]
fn prose_state_artifacts_include_mapping_and_candidates() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "prose_only_adf.json");

    let mut request = sample_request("run-prose-artifacts");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::SimpleScopedProseUpdate {
        target_path: Some("/content/1/content/0/text".to_string()),
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

#[test]
fn forbidden_row_operation_fails_with_table_shape_error() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator =
        make_orchestrator_with_fixture(temp.path(), "table_forbidden_ops_adf.json");

    let mut request = sample_request("run-row-add-forbidden");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::ForbiddenTableOperation {
        target_path: "/content/1/content/0".to_string(),
        operation: TableOperation::RowAdd,
    };

    let error = orchestrator
        .run(request)
        .expect_err("forbidden row add should fail");
    assert_hard_error(error, PipelineState::AdfTableEdit, ERR_TABLE_SHAPE_CHANGE);
    assert_eq!(orchestrator.client().publish_attempts(), 0);
}

#[test]
fn forbidden_table_attr_operation_fails_with_table_shape_error() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator =
        make_orchestrator_with_fixture(temp.path(), "table_forbidden_ops_adf.json");

    let mut request = sample_request("run-attr-op-forbidden");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::ForbiddenTableOperation {
        target_path: "/content/1/attrs/layout".to_string(),
        operation: TableOperation::TableAttrUpdate,
    };

    let error = orchestrator
        .run(request)
        .expect_err("forbidden attr update should fail");
    assert_hard_error(error, PipelineState::AdfTableEdit, ERR_TABLE_SHAPE_CHANGE);
}

#[test]
fn verify_blocks_synthetic_table_shape_drift() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator =
        make_orchestrator_with_fixture(temp.path(), "table_forbidden_ops_adf.json");

    let mut request = sample_request("run-shape-drift");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::SyntheticTableShapeDrift {
        path: "/content/1/content/0".to_string(),
    };

    let error = orchestrator
        .run(request)
        .expect_err("shape drift should fail at verify");
    assert_hard_error(error, PipelineState::Verify, ERR_TABLE_SHAPE_CHANGE);
    assert_eq!(orchestrator.client().publish_attempts(), 0);
}

#[test]
fn merge_collision_fails_fast() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator =
        make_orchestrator_with_fixture(temp.path(), "table_allowed_cell_update_adf.json");

    let mut request = sample_request("run-merge-collision");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::SyntheticRouteConflict {
        prose_path: "/content/0/content/0/text".to_string(),
        table_path: "/content/1/content/0/content/0/content/0/content/0/text".to_string(),
    };

    let error = orchestrator
        .run(request)
        .expect_err("merge collision should fail");
    assert_hard_error(error, PipelineState::MergeCandidates, "ERR_ROUTE_VIOLATION");
}

#[test]
fn table_candidate_generation_is_deterministic() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator =
        make_orchestrator_with_fixture(temp.path(), "table_allowed_cell_update_adf.json");

    let mut first = sample_request("run-table-map-a");
    first.scope_selectors = vec![];
    first.run_mode = RunMode::SimpleScopedTableCellUpdate {
        target_path: Some("/content/1/content/0/content/0/content/0/content/0/text".to_string()),
        text: "Updated table cell".to_string(),
    };
    orchestrator.run(first).expect("first run should succeed");

    let mut second = sample_request("run-table-map-b");
    second.scope_selectors = vec![];
    second.run_mode = RunMode::SimpleScopedTableCellUpdate {
        target_path: Some("/content/1/content/0/content/0/content/0/content/0/text".to_string()),
        text: "Updated table cell".to_string(),
    };
    orchestrator.run(second).expect("second run should succeed");

    let candidates_a = read_state_output(temp.path(), "run-table-map-a", "adf_table_edit")
        ["payload"]["table_candidates"]
        .clone();
    let candidates_b = read_state_output(temp.path(), "run-table-map-b", "adf_table_edit")
        ["payload"]["table_candidates"]
        .clone();
    assert_eq!(candidates_a, candidates_b);
}

#[test]
fn table_state_artifacts_include_candidates_and_allowed_ops() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator =
        make_orchestrator_with_fixture(temp.path(), "table_allowed_cell_update_adf.json");

    let mut request = sample_request("run-table-artifacts");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::SimpleScopedTableCellUpdate {
        target_path: Some("/content/1/content/0/content/0/content/0/content/0/text".to_string()),
        text: "Updated table cell".to_string(),
    };
    orchestrator.run(request).expect("run should succeed");

    let table_output = read_state_output(temp.path(), "run-table-artifacts", "adf_table_edit");
    assert!(table_output["payload"]["table_candidates"].is_array());
    assert_eq!(
        table_output["payload"]["allowed_ops"],
        serde_json::json!(["cell_text_update"])
    );
}

// --- Bootstrap empty-page detection integration tests ---

fn make_orchestrator_with_empty_page(artifact_root: &Path) -> Orchestrator<StubConfluenceClient> {
    let mut pages = HashMap::new();
    pages.insert(
        "18841604".to_string(),
        StubPage {
            version: 1,
            adf: load_fixture("empty_page_adf.json"),
        },
    );
    Orchestrator::new(StubConfluenceClient::new(pages), artifact_root)
}

#[test]
fn bootstrap_empty_page_without_flag_fails_with_err_bootstrap_required() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_empty_page(temp.path());

    let mut request = sample_request("run-bootstrap-no-flag");
    request.scope_selectors = vec![];
    request.bootstrap_empty_page = false;
    request.run_mode = RunMode::NoOp;

    let error = orchestrator
        .run(request)
        .expect_err("empty page without bootstrap flag should fail");
    assert_hard_error(error, PipelineState::Fetch, ERR_BOOTSTRAP_REQUIRED);

    // Verify summary records empty_page_detected = true
    let run_dir = temp.path().join("artifacts").join("run-bootstrap-no-flag");
    let summary: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("summary.json")).expect("summary should exist"),
    )
    .expect("summary should deserialize");
    assert_eq!(summary["empty_page_detected"], serde_json::json!(true));
    assert_eq!(summary["bootstrap_applied"], serde_json::json!(false));
}

#[test]
fn bootstrap_empty_page_with_flag_succeeds() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_empty_page(temp.path());

    let mut request = sample_request("run-bootstrap-with-flag");
    request.scope_selectors = vec![];
    request.bootstrap_empty_page = true;
    request.run_mode = RunMode::NoOp;

    let summary = orchestrator
        .run(request)
        .expect("empty page with bootstrap flag should succeed");
    assert!(summary.success);
    assert!(summary.empty_page_detected);
    assert!(summary.bootstrap_applied);
    assert!(summary.full_page_fetch);
    assert!(summary.scope_resolution_failed);
}

#[test]
fn bootstrap_non_empty_page_with_flag_fails_with_err_bootstrap_invalid_state() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "prose_only_adf.json");

    let mut request = sample_request("run-bootstrap-non-empty");
    request.scope_selectors = vec![];
    request.bootstrap_empty_page = true;
    request.run_mode = RunMode::NoOp;

    let error = orchestrator
        .run(request)
        .expect_err("non-empty page with bootstrap flag should fail");
    assert_hard_error(error, PipelineState::Fetch, ERR_BOOTSTRAP_INVALID_STATE);

    // Verify summary records empty_page_detected = false
    let run_dir = temp
        .path()
        .join("artifacts")
        .join("run-bootstrap-non-empty");
    let summary: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("summary.json")).expect("summary should exist"),
    )
    .expect("summary should deserialize");
    assert_eq!(summary["empty_page_detected"], serde_json::json!(false));
    assert_eq!(summary["bootstrap_applied"], serde_json::json!(false));
}

#[test]
fn bootstrap_non_empty_page_without_flag_proceeds_normally() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "prose_only_adf.json");

    let mut request = sample_request("run-bootstrap-normal");
    request.scope_selectors = vec![];
    request.bootstrap_empty_page = false;
    request.run_mode = RunMode::SimpleScopedProseUpdate {
        target_path: Some("/content/1/content/0/text".to_string()),
        markdown: "Updated prose body".to_string(),
    };

    let summary = orchestrator
        .run(request)
        .expect("non-empty page without bootstrap flag should succeed normally");
    assert!(summary.success);
    assert!(!summary.empty_page_detected);
    assert!(!summary.bootstrap_applied);
    assert_eq!(
        summary.applied_paths,
        vec!["/content/1/content/0/text".to_string()]
    );
}
