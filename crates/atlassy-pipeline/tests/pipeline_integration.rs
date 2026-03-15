use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use atlassy_confluence::{
    ConfluenceClient, ConfluenceError, FetchPageResponse, PublishPageResponse,
    StubConfluenceClient, StubPage,
};
use atlassy_contracts::{
    BlockOp, ContractError, ErrorCode, FLOW_OPTIMIZED, Operation, PATTERN_A, PIPELINE_VERSION,
    PipelineState, ProvenanceStamp, RUNTIME_STUB, TableOperation,
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
        block_ops: vec![],
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

fn make_orchestrator_with_adf(
    artifact_root: &Path,
    adf: serde_json::Value,
) -> Orchestrator<StubConfluenceClient> {
    let mut pages = HashMap::new();
    pages.insert("18841604".to_string(), StubPage { version: 7, adf });

    Orchestrator::new(StubConfluenceClient::new(pages), artifact_root)
}

fn table_with_header_and_rows_fixture() -> serde_json::Value {
    serde_json::json!({
        "type": "doc",
        "version": 1,
        "content": [
            {
                "type": "heading",
                "attrs": {"level": 2},
                "content": [{"type": "text", "text": "Overview"}]
            },
            {
                "type": "table",
                "content": [
                    {
                        "type": "tableRow",
                        "content": [
                            {
                                "type": "tableHeader",
                                "content": [{"type": "paragraph", "content": [{"type": "text", "text": "H1"}]}]
                            },
                            {
                                "type": "tableHeader",
                                "content": [{"type": "paragraph", "content": [{"type": "text", "text": "H2"}]}]
                            }
                        ]
                    },
                    {
                        "type": "tableRow",
                        "content": [
                            {
                                "type": "tableCell",
                                "content": [{"type": "paragraph", "content": [{"type": "text", "text": "A1"}]}]
                            },
                            {
                                "type": "tableCell",
                                "content": [{"type": "paragraph", "content": [{"type": "text", "text": "A2"}]}]
                            }
                        ]
                    },
                    {
                        "type": "tableRow",
                        "content": [
                            {
                                "type": "tableCell",
                                "content": [{"type": "paragraph", "content": [{"type": "text", "text": "B1"}]}]
                            },
                            {
                                "type": "tableCell",
                                "content": [{"type": "paragraph", "content": [{"type": "text", "text": "B2"}]}]
                            }
                        ]
                    }
                ]
            }
        ]
    })
}

fn panel_fixture() -> serde_json::Value {
    serde_json::json!({
        "type": "doc",
        "version": 1,
        "content": [
            {
                "type": "heading",
                "attrs": {"level": 2},
                "content": [{"type": "text", "text": "Overview"}]
            },
            {
                "type": "panel",
                "attrs": {"panelType": "info"},
                "content": [
                    {
                        "type": "paragraph",
                        "content": [{"type": "text", "text": "Panel body"}]
                    }
                ]
            }
        ]
    })
}

fn extension_fixture() -> serde_json::Value {
    serde_json::json!({
        "type": "doc",
        "version": 1,
        "content": [
            {
                "type": "heading",
                "attrs": {"level": 2},
                "content": [{"type": "text", "text": "Overview"}]
            },
            {
                "type": "extension",
                "attrs": {"extensionKey": "macro-key"}
            }
        ]
    })
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
            assert_eq!(got_code.as_str(), code);
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
    assert!(patch_output["payload"].get("patch_ops").is_some());
    let patch_ops: Vec<Operation> =
        serde_json::from_value(patch_output["payload"]["patch_ops"].clone())
            .expect("patch_ops should deserialize into Operation list");
    assert!(matches!(
        &patch_ops[0],
        Operation::Replace { path, .. } if path == "/content/1/content/0/text"
    ));
}

#[test]
fn pipeline_auto_discovers_table_cell_and_patches() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator =
        make_orchestrator_with_fixture(temp.path(), "table_allowed_cell_update_adf.json");

    let mut request = sample_request("run-table-auto-discovery");
    request.scope_selectors = vec![];
    request.target_index = 0;
    request.run_mode = RunMode::SimpleScopedTableCellUpdate {
        target_path: None,
        text: "Updated table cell".to_string(),
    };

    let summary = orchestrator.run(request).expect("run should succeed");
    assert!(summary.success);
    assert_eq!(
        summary.discovered_target_path,
        Some("/content/1/content/0/content/0/content/0/content/0/text".to_string())
    );
    assert_eq!(
        summary.applied_paths,
        vec!["/content/1/content/0/content/0/content/0/content/0/text".to_string()]
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
    assert_hard_error(
        error,
        PipelineState::Publish,
        ErrorCode::RuntimeBackend.as_str(),
    );
    assert_eq!(orchestrator.client().publish_attempts(), 1);

    let run_dir = temp.path().join("artifacts").join("run-live-publish-400");
    let summary: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("summary.json")).expect("summary should exist"),
    )
    .expect("summary should deserialize");
    assert_eq!(summary["failure_state"], serde_json::json!("publish"));
    assert_eq!(
        summary["error_codes"],
        serde_json::json!([ErrorCode::RuntimeBackend.as_str()])
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
        "adf_block_ops",
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
        "adf_block_ops",
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
    assert_hard_error(
        error,
        PipelineState::AdfTableEdit,
        ErrorCode::TableShapeChange.as_str(),
    );
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
    assert_hard_error(
        error,
        PipelineState::AdfTableEdit,
        ErrorCode::TableShapeChange.as_str(),
    );
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
    assert_hard_error(
        error,
        PipelineState::Verify,
        ErrorCode::TableShapeChange.as_str(),
    );
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

#[test]
fn insert_only_run_produces_correct_adf_and_publishes() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "prose_only_adf.json");

    let mut request = sample_request("run-insert-only");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::NoOp;
    request.block_ops = vec![BlockOp::Insert {
        parent_path: "/content".to_string(),
        index: 1,
        block: serde_json::json!({
            "type": "paragraph",
            "content": [{"type": "text", "text": "Inserted block"}]
        }),
    }];

    let summary = orchestrator
        .run(request)
        .expect("insert-only run should succeed");
    assert!(summary.success);
    assert_eq!(summary.applied_paths, vec!["/content/1".to_string()]);

    let patch_output = read_state_output(temp.path(), "run-insert-only", "patch");
    assert_eq!(
        patch_output["payload"]["candidate_page_adf"]["content"][1]["type"],
        serde_json::json!("paragraph")
    );
    assert_eq!(
        patch_output["payload"]["candidate_page_adf"]["content"][1]["content"][0]["text"],
        serde_json::json!("Inserted block")
    );
    assert_eq!(orchestrator.client().publish_attempts(), 1);
}

#[test]
fn remove_only_run_produces_correct_adf_and_publishes() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "prose_only_adf.json");

    let mut request = sample_request("run-remove-only");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::NoOp;
    request.block_ops = vec![BlockOp::Remove {
        target_path: "/content/1".to_string(),
    }];

    let summary = orchestrator
        .run(request)
        .expect("remove-only run should succeed");
    assert!(summary.success);
    assert_eq!(summary.applied_paths, vec!["/content/1".to_string()]);

    let patch_output = read_state_output(temp.path(), "run-remove-only", "patch");
    assert!(
        patch_output["payload"]["candidate_page_adf"]["content"]
            .as_array()
            .expect("content should be array")
            .iter()
            .all(|block| block["type"] != serde_json::json!("paragraph"))
    );
    assert_eq!(orchestrator.client().publish_attempts(), 1);
}

#[test]
fn mixed_insert_replace_remove_run_produces_expected_result() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "prose_only_adf.json");

    let mut request = sample_request("run-mixed-ops");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::SimpleScopedProseUpdate {
        target_path: Some("/content/1/content/0/text".to_string()),
        markdown: "Updated prose body".to_string(),
    };
    request.block_ops = vec![
        BlockOp::Insert {
            parent_path: "/content".to_string(),
            index: 2,
            block: serde_json::json!({
                "type": "paragraph",
                "content": [{"type": "text", "text": "Inserted sibling"}]
            }),
        },
        BlockOp::Remove {
            target_path: "/content/0".to_string(),
        },
    ];

    let summary = orchestrator
        .run(request)
        .expect("mixed operation run should succeed");
    assert!(summary.success);
    assert!(
        summary
            .applied_paths
            .contains(&"/content/1/content/0/text".to_string())
    );
    assert!(summary.applied_paths.contains(&"/content/2".to_string()));
    assert!(summary.applied_paths.contains(&"/content/0".to_string()));

    let patch_output = read_state_output(temp.path(), "run-mixed-ops", "patch");
    assert_eq!(
        patch_output["payload"]["candidate_page_adf"]["content"][0]["content"][0]["text"],
        serde_json::json!("Updated prose body")
    );
    assert_eq!(
        patch_output["payload"]["candidate_page_adf"]["content"][1]["content"][0]["text"],
        serde_json::json!("Inserted sibling")
    );
}

#[test]
fn replace_only_run_remains_backward_compatible() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "prose_only_adf.json");

    let mut request = sample_request("run-replace-only");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::SimpleScopedProseUpdate {
        target_path: Some("/content/1/content/0/text".to_string()),
        markdown: "Updated prose body".to_string(),
    };

    let summary = orchestrator
        .run(request)
        .expect("replace-only run should succeed");
    assert!(summary.success);
    assert_eq!(
        summary.applied_paths,
        vec!["/content/1/content/0/text".to_string()]
    );

    let patch_output = read_state_output(temp.path(), "run-replace-only", "patch");
    let patch_ops: Vec<Operation> =
        serde_json::from_value(patch_output["payload"]["patch_ops"].clone())
            .expect("patch_ops should deserialize");
    assert!(
        patch_ops
            .iter()
            .all(|operation| matches!(operation, Operation::Replace { .. }))
    );
}

#[test]
fn out_of_scope_insert_fails_before_patch() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "prose_only_adf.json");

    let mut request = sample_request("run-out-of-scope-insert");
    request.run_mode = RunMode::NoOp;
    request.block_ops = vec![BlockOp::Insert {
        parent_path: "/content".to_string(),
        index: 2,
        block: serde_json::json!({
            "type": "paragraph",
            "content": [{"type": "text", "text": "Out of scope"}]
        }),
    }];

    let error = orchestrator
        .run(request)
        .expect_err("out-of-scope insert should fail");
    assert_hard_error(
        error,
        PipelineState::AdfBlockOps,
        ErrorCode::OutOfScopeMutation.as_str(),
    );
}

#[test]
fn scope_anchor_remove_fails_with_remove_anchor_missing() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "prose_only_adf.json");

    let mut request = sample_request("run-remove-scope-anchor");
    request.run_mode = RunMode::NoOp;
    request.block_ops = vec![BlockOp::Remove {
        target_path: "/content/0".to_string(),
    }];

    let error = orchestrator
        .run(request)
        .expect_err("scope anchor remove should fail");
    assert_hard_error(
        error,
        PipelineState::Verify,
        ErrorCode::RemoveAnchorMissing.as_str(),
    );
}

#[test]
fn out_of_bounds_insert_fails_with_insert_position_invalid() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "prose_only_adf.json");

    let mut request = sample_request("run-insert-out-of-bounds");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::NoOp;
    request.block_ops = vec![BlockOp::Insert {
        parent_path: "/content".to_string(),
        index: 99,
        block: serde_json::json!({
            "type": "paragraph",
            "content": [{"type": "text", "text": "Too far"}]
        }),
    }];

    let error = orchestrator
        .run(request)
        .expect_err("out-of-bounds insert should fail");
    assert_hard_error(
        error,
        PipelineState::Patch,
        ErrorCode::InsertPositionInvalid.as_str(),
    );
}

#[test]
fn insert_section_run_inserts_heading_and_body_blocks() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "prose_only_adf.json");

    let mut request = sample_request("run-insert-section");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::NoOp;
    request.block_ops = vec![BlockOp::InsertSection {
        parent_path: "/content".to_string(),
        index: 2,
        heading_level: 2,
        heading_text: "FAQ".to_string(),
        body_blocks: vec![
            serde_json::json!({
                "type": "paragraph",
                "content": [{"type": "text", "text": "Q1 answer"}]
            }),
            serde_json::json!({
                "type": "paragraph",
                "content": [{"type": "text", "text": "Q2 answer"}]
            }),
        ],
    }];

    let summary = orchestrator
        .run(request)
        .expect("insert section run should succeed");
    assert!(summary.success);
    assert!(summary.applied_paths.contains(&"/content/2".to_string()));
    assert!(summary.applied_paths.contains(&"/content/3".to_string()));
    assert!(summary.applied_paths.contains(&"/content/4".to_string()));

    let patch_output = read_state_output(temp.path(), "run-insert-section", "patch");
    assert_eq!(
        patch_output["payload"]["candidate_page_adf"]["content"][2]["type"],
        serde_json::json!("heading")
    );
    assert_eq!(
        patch_output["payload"]["candidate_page_adf"]["content"][2]["content"][0]["text"],
        serde_json::json!("FAQ")
    );
    assert_eq!(
        patch_output["payload"]["candidate_page_adf"]["content"][3]["content"][0]["text"],
        serde_json::json!("Q1 answer")
    );
    assert_eq!(
        patch_output["payload"]["candidate_page_adf"]["content"][4]["content"][0]["text"],
        serde_json::json!("Q2 answer")
    );
}

#[test]
fn remove_section_run_removes_heading_and_body_preserving_adjacent_content() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "multi_section_adf.json");

    let mut request = sample_request("run-remove-section");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::NoOp;
    request.block_ops = vec![BlockOp::RemoveSection {
        heading_path: "/content/2".to_string(),
    }];

    let summary = orchestrator
        .run(request)
        .expect("remove section run should succeed");
    assert!(summary.success);
    assert_eq!(
        summary.applied_paths,
        vec![
            "/content/4".to_string(),
            "/content/3".to_string(),
            "/content/2".to_string(),
        ]
    );

    let patch_output = read_state_output(temp.path(), "run-remove-section", "patch");
    let content = patch_output["payload"]["candidate_page_adf"]["content"]
        .as_array()
        .expect("content should be array");
    assert_eq!(content.len(), 2);
    assert_eq!(content[0]["type"], serde_json::json!("heading"));
    assert_eq!(
        content[0]["content"][0]["text"],
        serde_json::json!("Overview")
    );
    assert_eq!(
        content[1]["content"][0]["text"],
        serde_json::json!("Overview body")
    );
}

#[test]
fn insert_table_run_produces_valid_table_and_publishes() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "prose_only_adf.json");

    let mut request = sample_request("run-insert-table");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::NoOp;
    request.block_ops = vec![BlockOp::InsertTable {
        parent_path: "/content".to_string(),
        index: 1,
        rows: 3,
        cols: 2,
        header_row: true,
    }];

    let summary = orchestrator
        .run(request)
        .expect("insert table run should succeed");
    assert!(summary.success);
    assert_eq!(orchestrator.client().publish_attempts(), 1);

    let patch_output = read_state_output(temp.path(), "run-insert-table", "patch");
    let table = &patch_output["payload"]["candidate_page_adf"]["content"][1];
    assert_eq!(table["type"], serde_json::json!("table"));
    assert_eq!(
        table["content"][0]["content"][0]["type"],
        serde_json::json!("tableHeader")
    );
    assert_eq!(
        table["content"][1]["content"][0]["type"],
        serde_json::json!("tableCell")
    );
}

#[test]
fn insert_list_run_produces_valid_list_and_publishes() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "prose_only_adf.json");

    let mut request = sample_request("run-insert-list");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::NoOp;
    request.block_ops = vec![BlockOp::InsertList {
        parent_path: "/content".to_string(),
        index: 2,
        ordered: false,
        items: vec![
            "Item A".to_string(),
            "Item B".to_string(),
            "Item C".to_string(),
        ],
    }];

    let summary = orchestrator
        .run(request)
        .expect("insert list run should succeed");
    assert!(summary.success);
    assert_eq!(orchestrator.client().publish_attempts(), 1);

    let patch_output = read_state_output(temp.path(), "run-insert-list", "patch");
    let list = &patch_output["payload"]["candidate_page_adf"]["content"][2];
    assert_eq!(list["type"], serde_json::json!("bulletList"));
    assert_eq!(list["content"].as_array().unwrap().len(), 3);
}

#[test]
fn insert_row_run_adds_row_with_matching_cell_count() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator =
        make_orchestrator_with_adf(temp.path(), table_with_header_and_rows_fixture());

    let mut request = sample_request("run-insert-row");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::NoOp;
    request.block_ops = vec![BlockOp::InsertRow {
        table_path: "/content/1".to_string(),
        index: 2,
        cells: vec!["X1".to_string(), "X2".to_string()],
    }];

    let summary = orchestrator
        .run(request)
        .expect("insert row run should succeed");
    assert!(summary.success);

    let patch_output = read_state_output(temp.path(), "run-insert-row", "patch");
    let table_rows = patch_output["payload"]["candidate_page_adf"]["content"][1]["content"]
        .as_array()
        .expect("table rows should be array");
    assert_eq!(table_rows.len(), 4);
    assert_eq!(
        table_rows[2]["content"][0]["content"][0]["content"][0]["text"],
        serde_json::json!("X1")
    );
    assert_eq!(
        table_rows[2]["content"][1]["content"][0]["content"][0]["text"],
        serde_json::json!("X2")
    );
}

#[test]
fn remove_row_run_removes_target_row_and_preserves_remaining_rows() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator =
        make_orchestrator_with_adf(temp.path(), table_with_header_and_rows_fixture());

    let mut request = sample_request("run-remove-row");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::NoOp;
    request.block_ops = vec![BlockOp::RemoveRow {
        table_path: "/content/1".to_string(),
        index: 1,
    }];

    let summary = orchestrator
        .run(request)
        .expect("remove row run should succeed");
    assert!(summary.success);

    let patch_output = read_state_output(temp.path(), "run-remove-row", "patch");
    let table_rows = patch_output["payload"]["candidate_page_adf"]["content"][1]["content"]
        .as_array()
        .expect("table rows should be array");
    assert_eq!(table_rows.len(), 2);
    assert_eq!(
        table_rows[1]["content"][0]["content"][0]["content"][0]["text"],
        serde_json::json!("B1")
    );
}

#[test]
fn insert_column_run_adds_cells_to_all_rows() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator =
        make_orchestrator_with_adf(temp.path(), table_with_header_and_rows_fixture());

    let mut request = sample_request("run-insert-column");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::NoOp;
    request.block_ops = vec![BlockOp::InsertColumn {
        table_path: "/content/1".to_string(),
        index: 1,
    }];

    let summary = orchestrator
        .run(request)
        .expect("insert column run should succeed");
    assert!(summary.success);

    let patch_output = read_state_output(temp.path(), "run-insert-column", "patch");
    let table_rows = patch_output["payload"]["candidate_page_adf"]["content"][1]["content"]
        .as_array()
        .expect("table rows should be array");

    for row in table_rows {
        assert_eq!(row["content"].as_array().unwrap().len(), 3);
    }
    assert_eq!(
        table_rows[0]["content"][1]["type"],
        serde_json::json!("tableHeader")
    );
    assert_eq!(
        table_rows[1]["content"][1]["type"],
        serde_json::json!("tableCell")
    );
}

#[test]
fn remove_column_run_removes_cells_from_all_rows() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator =
        make_orchestrator_with_adf(temp.path(), table_with_header_and_rows_fixture());

    let mut request = sample_request("run-remove-column");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::NoOp;
    request.block_ops = vec![BlockOp::RemoveColumn {
        table_path: "/content/1".to_string(),
        index: 1,
    }];

    let summary = orchestrator
        .run(request)
        .expect("remove column run should succeed");
    assert!(summary.success);

    let patch_output = read_state_output(temp.path(), "run-remove-column", "patch");
    let table_rows = patch_output["payload"]["candidate_page_adf"]["content"][1]["content"]
        .as_array()
        .expect("table rows should be array");
    for row in table_rows {
        assert_eq!(row["content"].as_array().unwrap().len(), 1);
    }
}

#[test]
fn update_attrs_run_updates_panel_attrs_without_touching_content() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_adf(temp.path(), panel_fixture());

    let mut request = sample_request("run-update-attrs-panel");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::NoOp;
    request.block_ops = vec![BlockOp::UpdateAttrs {
        target_path: "/content/1".to_string(),
        attrs: serde_json::json!({"panelType": "warning"}),
    }];

    let summary = orchestrator
        .run(request)
        .expect("update attrs on panel should succeed");
    assert!(summary.success);

    let patch_output = read_state_output(temp.path(), "run-update-attrs-panel", "patch");
    assert_eq!(
        patch_output["payload"]["candidate_page_adf"]["content"][1]["attrs"]["panelType"],
        serde_json::json!("warning")
    );
    assert_eq!(
        patch_output["payload"]["candidate_page_adf"]["content"][1]["content"][0]["content"][0]["text"],
        serde_json::json!("Panel body")
    );
}

#[test]
fn update_attrs_on_non_attr_editable_node_is_rejected() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_adf(temp.path(), extension_fixture());

    let mut request = sample_request("run-update-attrs-extension-rejected");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::NoOp;
    request.block_ops = vec![BlockOp::UpdateAttrs {
        target_path: "/content/1".to_string(),
        attrs: serde_json::json!({"title": "bad"}),
    }];

    let error = orchestrator
        .run(request)
        .expect_err("update attrs on extension should fail");
    assert_hard_error(
        error,
        PipelineState::AdfBlockOps,
        ErrorCode::AttrUpdateBlocked.as_str(),
    );
}

#[test]
fn update_attrs_with_disallowed_key_fails_attr_schema_validation() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_adf(temp.path(), panel_fixture());

    let mut request = sample_request("run-update-attrs-schema-invalid");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::NoOp;
    request.block_ops = vec![BlockOp::UpdateAttrs {
        target_path: "/content/1".to_string(),
        attrs: serde_json::json!({"dangerousKey": true}),
    }];

    let error = orchestrator
        .run(request)
        .expect_err("disallowed attr key should fail verify");
    assert_hard_error(
        error,
        PipelineState::Verify,
        ErrorCode::AttrSchemaViolation.as_str(),
    );
}

#[test]
fn replace_inside_panel_is_blocked_by_locked_boundary() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_adf(temp.path(), panel_fixture());

    let mut request = sample_request("run-replace-panel-blocked");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::SimpleScopedProseUpdate {
        target_path: Some("/content/1/content/0/content/0/text".to_string()),
        markdown: "Attempted replace".to_string(),
    };

    let error = orchestrator
        .run(request)
        .expect_err("replace inside panel should fail verify");
    assert_hard_error(
        error,
        PipelineState::Verify,
        ErrorCode::RouteViolation.as_str(),
    );
}

#[test]
fn mixed_insert_section_and_replace_run_succeeds() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "prose_only_adf.json");

    let mut request = sample_request("run-mixed-insert-section-replace");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::SimpleScopedProseUpdate {
        target_path: Some("/content/1/content/0/text".to_string()),
        markdown: "Updated prose body".to_string(),
    };
    request.block_ops = vec![BlockOp::InsertSection {
        parent_path: "/content".to_string(),
        index: 2,
        heading_level: 3,
        heading_text: "Follow-up".to_string(),
        body_blocks: vec![serde_json::json!({
            "type": "paragraph",
            "content": [{"type": "text", "text": "Inserted body"}]
        })],
    }];

    let summary = orchestrator
        .run(request)
        .expect("mixed replace + insert section should succeed");
    assert!(summary.success);
    assert!(
        summary
            .applied_paths
            .contains(&"/content/1/content/0/text".to_string())
    );
    assert!(summary.applied_paths.contains(&"/content/2".to_string()));
    assert!(summary.applied_paths.contains(&"/content/3".to_string()));

    let patch_output = read_state_output(temp.path(), "run-mixed-insert-section-replace", "patch");
    assert_eq!(
        patch_output["payload"]["candidate_page_adf"]["content"][1]["content"][0]["text"],
        serde_json::json!("Updated prose body")
    );
    assert_eq!(
        patch_output["payload"]["candidate_page_adf"]["content"][2]["type"],
        serde_json::json!("heading")
    );
}

#[test]
fn out_of_scope_insert_section_fails_before_patch() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "prose_only_adf.json");

    let mut request = sample_request("run-out-of-scope-insert-section");
    request.run_mode = RunMode::NoOp;
    request.block_ops = vec![BlockOp::InsertSection {
        parent_path: "/content".to_string(),
        index: 2,
        heading_level: 2,
        heading_text: "Blocked".to_string(),
        body_blocks: vec![],
    }];

    let error = orchestrator
        .run(request)
        .expect_err("out-of-scope insert section should fail");
    assert_hard_error(
        error,
        PipelineState::AdfBlockOps,
        ErrorCode::OutOfScopeMutation.as_str(),
    );
}

#[test]
fn remove_section_non_heading_target_fails_with_section_boundary_error() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut orchestrator = make_orchestrator_with_fixture(temp.path(), "prose_only_adf.json");

    let mut request = sample_request("run-remove-section-non-heading");
    request.scope_selectors = vec![];
    request.run_mode = RunMode::NoOp;
    request.block_ops = vec![BlockOp::RemoveSection {
        heading_path: "/content/1".to_string(),
    }];

    let error = orchestrator
        .run(request)
        .expect_err("remove section on paragraph should fail");
    assert_hard_error(
        error,
        PipelineState::AdfBlockOps,
        ErrorCode::SectionBoundaryInvalid.as_str(),
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
    assert_hard_error(
        error,
        PipelineState::Fetch,
        ErrorCode::BootstrapRequired.as_str(),
    );

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
    assert_hard_error(
        error,
        PipelineState::Fetch,
        ErrorCode::BootstrapInvalidState.as_str(),
    );

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
