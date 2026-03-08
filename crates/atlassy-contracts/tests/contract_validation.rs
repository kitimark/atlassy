use std::collections::BTreeMap;

use atlassy_contracts::*;

#[test]
fn pipeline_state_enum_values_are_stable() {
    let serialized = serde_json::to_string(&PipelineState::MdAssistEdit).unwrap();
    assert_eq!(serialized, "\"md_assist_edit\"");

    let order: Vec<&str> = PipelineState::ORDER
        .iter()
        .map(|state| state.as_str())
        .collect();
    assert_eq!(
        order,
        vec![
            "fetch",
            "classify",
            "extract_prose",
            "md_assist_edit",
            "adf_table_edit",
            "merge_candidates",
            "patch",
            "verify",
            "publish",
        ]
    );
}

#[test]
fn changed_paths_must_be_unique_and_sorted() {
    let valid = vec!["/a".to_string(), "/b".to_string()];
    assert!(validate_changed_paths(&valid).is_ok());

    let duplicate = vec!["/a".to_string(), "/a".to_string()];
    assert_eq!(
        validate_changed_paths(&duplicate),
        Err(ContractError::InvalidChangedPaths)
    );

    let unsorted = vec!["/b".to_string(), "/a".to_string()];
    assert_eq!(
        validate_changed_paths(&unsorted),
        Err(ContractError::InvalidChangedPaths)
    );
}

#[test]
fn envelope_serialization_and_required_fields() {
    let envelope = StateEnvelope {
        meta: EnvelopeMeta {
            request_id: "req-1".to_string(),
            page_id: "18841604".to_string(),
            state: PipelineState::Fetch,
            timestamp: "2026-03-06T10:00:00Z".to_string(),
        },
        payload: FetchInput {
            page_id: "18841604".to_string(),
            edit_intent: "update section".to_string(),
            scope_selectors: vec!["heading:Overview".to_string()],
        },
    };

    envelope.validate_meta().unwrap();
    let serialized = serde_json::to_string(&envelope).unwrap();
    assert!(serialized.contains("\"request_id\":\"req-1\""));
    assert!(serialized.contains("\"state\":\"fetch\""));

    let invalid = StateEnvelope {
        meta: EnvelopeMeta {
            request_id: String::new(),
            page_id: "p".to_string(),
            state: PipelineState::Fetch,
            timestamp: "ts".to_string(),
        },
        payload: serde_json::json!({}),
    };

    assert_eq!(
        invalid.validate_meta(),
        Err(ContractError::MissingField("request_id"))
    );
}

#[test]
fn markdown_mapping_must_be_one_to_one_and_in_scope() {
    let markdown_blocks = vec![
        MarkdownBlock {
            md_block_id: "/content/1".to_string(),
            markdown: "Hello".to_string(),
        },
        MarkdownBlock {
            md_block_id: "/content/2".to_string(),
            markdown: "World".to_string(),
        },
    ];

    let valid_map = vec![
        MarkdownMapEntry {
            md_block_id: "/content/1".to_string(),
            adf_path: "/content/1".to_string(),
        },
        MarkdownMapEntry {
            md_block_id: "/content/2".to_string(),
            adf_path: "/content/2".to_string(),
        },
    ];

    assert!(
        validate_markdown_mapping(
            &markdown_blocks,
            &valid_map,
            &["/content/1".to_string(), "/content/2".to_string()],
            &["/content".to_string()]
        )
        .is_ok()
    );

    let duplicate_block_map = vec![
        MarkdownMapEntry {
            md_block_id: "/content/1".to_string(),
            adf_path: "/content/1".to_string(),
        },
        MarkdownMapEntry {
            md_block_id: "/content/1".to_string(),
            adf_path: "/content/2".to_string(),
        },
    ];

    assert!(matches!(
        validate_markdown_mapping(
            &markdown_blocks,
            &duplicate_block_map,
            &["/content/1".to_string(), "/content/2".to_string()],
            &["/content".to_string()]
        ),
        Err(ContractError::InvalidMarkdownMapping(_))
    ));
}

#[test]
fn prose_changed_paths_must_stay_within_mapped_paths() {
    let mapped_paths = vec!["/content/1".to_string(), "/content/2".to_string()];

    assert!(validate_prose_changed_paths(&["/content/1/text".to_string()], &mapped_paths).is_ok());
    assert_eq!(
        validate_prose_changed_paths(&["/content/99".to_string()], &mapped_paths),
        Err(ContractError::UnmappedProsePath("/content/99".to_string()))
    );
}

#[test]
fn prose_route_payload_serialization_is_deterministic() {
    let input = MdAssistEditInput {
        markdown_blocks: vec![MarkdownBlock {
            md_block_id: "/content/1".to_string(),
            markdown: "Initial prose".to_string(),
        }],
        md_to_adf_map: vec![MarkdownMapEntry {
            md_block_id: "/content/1".to_string(),
            adf_path: "/content/1".to_string(),
        }],
        editable_prose_paths: vec!["/content/1".to_string()],
        allowed_scope_paths: vec!["/content".to_string()],
        edit_intent: "Update one section".to_string(),
    };

    let first = serde_json::to_string(&input).unwrap();
    let second = serde_json::to_string(&input).unwrap();
    assert_eq!(first, second);
    assert!(first.contains("\"md_to_adf_map\""));
    assert!(first.contains("\"allowed_scope_paths\""));
}

#[test]
fn table_candidates_allowlist_and_order_are_enforced() {
    let allowed = vec![TableOperation::CellTextUpdate];
    let valid = vec![
        TableChangeCandidate {
            op: TableOperation::CellTextUpdate,
            path: "/content/2/content/0/content/0/content/0/text".to_string(),
            value: serde_json::json!("A"),
            source_route: "table_adf".to_string(),
        },
        TableChangeCandidate {
            op: TableOperation::CellTextUpdate,
            path: "/content/2/content/0/content/0/content/1/text".to_string(),
            value: serde_json::json!("B"),
            source_route: "table_adf".to_string(),
        },
    ];
    assert!(validate_table_candidates(&valid, &allowed).is_ok());

    let forbidden = vec![TableChangeCandidate {
        op: TableOperation::RowAdd,
        path: "/content/2/content/0".to_string(),
        value: serde_json::json!({}),
        source_route: "table_adf".to_string(),
    }];
    assert_eq!(
        validate_table_candidates(&forbidden, &allowed),
        Err(ContractError::TableOperationNotAllowed(
            "row_add".to_string()
        ))
    );

    let unsorted = vec![
        TableChangeCandidate {
            op: TableOperation::CellTextUpdate,
            path: "/content/2/content/0/content/0/content/1/text".to_string(),
            value: serde_json::json!("B"),
            source_route: "table_adf".to_string(),
        },
        TableChangeCandidate {
            op: TableOperation::CellTextUpdate,
            path: "/content/2/content/0/content/0/content/0/text".to_string(),
            value: serde_json::json!("A"),
            source_route: "table_adf".to_string(),
        },
    ];
    assert_eq!(
        validate_table_candidates(&unsorted, &allowed),
        Err(ContractError::TableCandidateOrder)
    );
}

#[test]
fn table_payload_serialization_is_deterministic() {
    let payload = AdfTableEditOutput {
        table_candidates: vec![TableChangeCandidate {
            op: TableOperation::CellTextUpdate,
            path: "/content/2/content/0/content/0/content/0/text".to_string(),
            value: serde_json::json!("Updated"),
            source_route: "table_adf".to_string(),
        }],
        table_changed_paths: vec!["/content/2/content/0/content/0/content/0/text".to_string()],
        allowed_ops: vec![TableOperation::CellTextUpdate],
    };

    let first = serde_json::to_string(&payload).unwrap();
    let second = serde_json::to_string(&payload).unwrap();
    assert_eq!(first, second);
    assert!(first.contains("\"cell_text_update\""));
}

#[test]
fn run_summary_telemetry_validation_requires_kpi_fields() {
    let mut summary = RunSummary {
        success: true,
        run_id: "run-1".to_string(),
        request_id: "req-1".to_string(),
        page_id: "18841604".to_string(),
        flow: FLOW_OPTIMIZED.to_string(),
        pattern: PATTERN_A.to_string(),
        edit_intent_hash: "hash-1".to_string(),
        scope_selectors: vec!["heading:Overview".to_string()],
        scope_resolution_failed: false,
        full_page_fetch: false,
        full_page_adf_bytes: 2048,
        scoped_adf_bytes: 512,
        context_reduction_ratio: 0.75,
        pipeline_version: PIPELINE_VERSION.to_string(),
        git_commit_sha: "0123456789abcdef0123456789abcdef01234567".to_string(),
        git_dirty: false,
        runtime_mode: RUNTIME_STUB.to_string(),
        state_token_usage: BTreeMap::from([
            ("fetch".to_string(), 0_u64),
            ("verify".to_string(), 0_u64),
            ("publish".to_string(), 0_u64),
        ]),
        total_tokens: 0,
        retry_count: 0,
        retry_tokens: 0,
        patch_ops_bytes: 128,
        verify_result: "pass".to_string(),
        verify_error_codes: Vec::new(),
        publish_result: "published".to_string(),
        publish_error_code: None,
        new_version: Some(2),
        start_ts: "2026-03-06T10:00:00Z".to_string(),
        verify_end_ts: "2026-03-06T10:00:01Z".to_string(),
        publish_end_ts: "2026-03-06T10:00:02Z".to_string(),
        latency_ms: 200,
        locked_node_mutation: false,
        out_of_scope_mutation: false,
        telemetry_complete: true,
        discovered_target_path: None,
        applied_paths: Vec::new(),
        blocked_paths: Vec::new(),
        error_codes: Vec::new(),
        token_metrics: BTreeMap::new(),
        failure_state: None,
        empty_page_detected: false,
        bootstrap_applied: false,
    };

    assert!(validate_run_summary_telemetry(&summary).is_ok());

    summary.flow = "unknown".to_string();
    assert_eq!(
        validate_run_summary_telemetry(&summary),
        Err(ContractError::TelemetryIncomplete("flow".to_string()))
    );
}

#[test]
fn provenance_stamp_validation_requires_sha_and_runtime_mode() {
    let mut stamp = ProvenanceStamp {
        git_commit_sha: "0123456789abcdef0123456789abcdef01234567".to_string(),
        git_dirty: false,
        pipeline_version: PIPELINE_VERSION.to_string(),
        runtime_mode: RUNTIME_STUB.to_string(),
    };

    assert!(validate_provenance_stamp(&stamp).is_ok());

    stamp.git_commit_sha = "short".to_string();
    assert_eq!(
        validate_provenance_stamp(&stamp),
        Err(ContractError::TelemetryIncomplete(
            "git_commit_sha".to_string()
        ))
    );

    stamp.git_commit_sha = "0123456789abcdef0123456789abcdef01234567".to_string();
    stamp.runtime_mode = "invalid".to_string();
    assert_eq!(
        validate_provenance_stamp(&stamp),
        Err(ContractError::TelemetryIncomplete(
            "runtime_mode".to_string()
        ))
    );
}
