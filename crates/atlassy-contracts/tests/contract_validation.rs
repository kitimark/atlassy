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
            "adf_block_ops",
            "merge_candidates",
            "patch",
            "verify",
            "publish",
        ]
    );
}

#[test]
fn patch_output_serializes_operation_replace_with_patch_ops_key() {
    let output = PatchOutput {
        patch_ops: vec![Operation::Replace {
            path: "/content/0/content/0/text".to_string(),
            value: serde_json::json!("after"),
        }],
        candidate_page_adf: serde_json::json!({"type": "doc", "content": []}),
        patch_ops_bytes: 0,
    };

    let serialized = serde_json::to_value(&output).unwrap();
    assert!(serialized.get("patch_ops").is_some());
    assert_eq!(
        serialized["patch_ops"][0]["op"],
        serde_json::json!("replace")
    );
    assert_eq!(
        serialized["patch_ops"][0]["path"],
        serde_json::json!("/content/0/content/0/text")
    );

    let decoded: PatchOutput = serde_json::from_value(serialized).unwrap();
    assert!(matches!(
        &decoded.patch_ops[0],
        Operation::Replace { path, .. } if path == "/content/0/content/0/text"
    ));
}

#[test]
fn merge_patch_and_verify_payloads_use_operations_field() {
    let operations = vec![
        Operation::Replace {
            path: "/content/0/content/0/text".to_string(),
            value: serde_json::json!("after"),
        },
        Operation::Insert {
            parent_path: "/content".to_string(),
            index: 1,
            block: serde_json::json!({"type": "paragraph", "content": []}),
        },
        Operation::Remove {
            target_path: "/content/2".to_string(),
        },
    ];

    let merge = MergeCandidatesOutput {
        operations: operations.clone(),
    };
    let patch = PatchInput {
        scoped_adf: serde_json::json!({"type": "doc", "content": []}),
        operations: operations.clone(),
    };
    let verify = VerifyInput {
        original_scoped_adf: serde_json::json!({"type": "doc", "content": []}),
        candidate_page_adf: serde_json::json!({"type": "doc", "content": []}),
        allowed_scope_paths: vec!["/content".to_string()],
        operations,
    };

    let merge_json = serde_json::to_value(&merge).unwrap();
    let patch_json = serde_json::to_value(&patch).unwrap();
    let verify_json = serde_json::to_value(&verify).unwrap();

    assert!(merge_json.get("operations").is_some());
    assert!(patch_json.get("operations").is_some());
    assert!(verify_json.get("operations").is_some());
}

#[test]
fn operation_insert_and_remove_round_trip_serde() {
    let insert = Operation::Insert {
        parent_path: "/content".to_string(),
        index: 2,
        block: serde_json::json!({"type": "paragraph", "content": []}),
    };
    let remove = Operation::Remove {
        target_path: "/content/2".to_string(),
    };

    let insert_json = serde_json::to_value(&insert).unwrap();
    let remove_json = serde_json::to_value(&remove).unwrap();

    assert_eq!(insert_json["op"], serde_json::json!("insert"));
    assert_eq!(remove_json["op"], serde_json::json!("remove"));

    let insert_decoded: Operation = serde_json::from_value(insert_json).unwrap();
    let remove_decoded: Operation = serde_json::from_value(remove_json).unwrap();
    assert_eq!(insert_decoded, insert);
    assert_eq!(remove_decoded, remove);
}

#[test]
fn block_op_enum_round_trip_serde() {
    let cases = vec![
        (
            BlockOp::Insert {
                parent_path: "/content".to_string(),
                index: 0,
                block: serde_json::json!({"type": "paragraph"}),
            },
            "insert",
        ),
        (
            BlockOp::Remove {
                target_path: "/content/0".to_string(),
            },
            "remove",
        ),
        (
            BlockOp::InsertSection {
                parent_path: "/content".to_string(),
                index: 1,
                heading_level: 2,
                heading_text: "FAQ".to_string(),
                body_blocks: vec![
                    serde_json::json!({"type": "paragraph", "content": [{"type": "text", "text": "A"}]}),
                    serde_json::json!({"type": "paragraph", "content": [{"type": "text", "text": "B"}]}),
                ],
            },
            "insert_section",
        ),
        (
            BlockOp::RemoveSection {
                heading_path: "/content/1".to_string(),
            },
            "remove_section",
        ),
        (
            BlockOp::InsertTable {
                parent_path: "/content".to_string(),
                index: 2,
                rows: 2,
                cols: 3,
                header_row: true,
            },
            "insert_table",
        ),
        (
            BlockOp::InsertList {
                parent_path: "/content".to_string(),
                index: 3,
                ordered: false,
                items: vec!["One".to_string(), "Two".to_string()],
            },
            "insert_list",
        ),
    ];

    for (block_op, expected_op_tag) in cases {
        let encoded = serde_json::to_value(&block_op).unwrap();
        assert_eq!(encoded["op"], serde_json::json!(expected_op_tag));

        let decoded: BlockOp = serde_json::from_value(encoded).unwrap();
        assert_eq!(decoded, block_op);
    }
}

#[test]
fn existing_block_op_variants_remain_unchanged() {
    let insert = BlockOp::Insert {
        parent_path: "/content".to_string(),
        index: 0,
        block: serde_json::json!({"type": "paragraph"}),
    };
    let remove = BlockOp::Remove {
        target_path: "/content/0".to_string(),
    };

    let insert_json = serde_json::to_value(&insert).unwrap();
    let remove_json = serde_json::to_value(&remove).unwrap();

    assert_eq!(insert_json["op"], serde_json::json!("insert"));
    assert_eq!(insert_json["parent_path"], serde_json::json!("/content"));
    assert_eq!(insert_json["index"], serde_json::json!(0));

    assert_eq!(remove_json["op"], serde_json::json!("remove"));
    assert_eq!(remove_json["target_path"], serde_json::json!("/content/0"));
}

#[test]
fn operation_replace_serialization_remains_unchanged() {
    let replace = Operation::Replace {
        path: "/content/1/content/0/text".to_string(),
        value: serde_json::json!("updated"),
    };

    let serialized = serde_json::to_value(&replace).unwrap();
    assert_eq!(
        serialized,
        serde_json::json!({
            "op": "replace",
            "path": "/content/1/content/0/text",
            "value": "updated"
        })
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

    assert!(validate_markdown_mapping(
        &markdown_blocks,
        &valid_map,
        &["/content/1".to_string(), "/content/2".to_string()],
        &["/content".to_string()]
    )
    .is_ok());

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

#[test]
fn multi_page_types_round_trip_serde() {
    let provenance = ProvenanceStamp {
        git_commit_sha: "0123456789abcdef0123456789abcdef01234567".to_string(),
        git_dirty: false,
        pipeline_version: PIPELINE_VERSION.to_string(),
        runtime_mode: RUNTIME_STUB.to_string(),
    };

    let run_summary = RunSummary {
        success: true,
        run_id: "run-1".to_string(),
        request_id: "req-1".to_string(),
        page_id: "page-1".to_string(),
        flow: FLOW_OPTIMIZED.to_string(),
        pattern: PATTERN_A.to_string(),
        edit_intent_hash: "hash-1".to_string(),
        scope_selectors: vec!["heading:Overview".to_string()],
        scope_resolution_failed: false,
        full_page_fetch: false,
        full_page_adf_bytes: 1024,
        scoped_adf_bytes: 512,
        context_reduction_ratio: 0.5,
        pipeline_version: PIPELINE_VERSION.to_string(),
        git_commit_sha: provenance.git_commit_sha.clone(),
        git_dirty: provenance.git_dirty,
        runtime_mode: provenance.runtime_mode.clone(),
        state_token_usage: BTreeMap::new(),
        total_tokens: 0,
        retry_count: 0,
        retry_tokens: 0,
        patch_ops_bytes: 64,
        verify_result: "pass".to_string(),
        verify_error_codes: vec![],
        publish_result: "published".to_string(),
        publish_error_code: None,
        new_version: Some(2),
        start_ts: "2026-03-15T00:00:00Z".to_string(),
        verify_end_ts: "2026-03-15T00:00:01Z".to_string(),
        publish_end_ts: "2026-03-15T00:00:02Z".to_string(),
        latency_ms: 42,
        locked_node_mutation: false,
        out_of_scope_mutation: false,
        telemetry_complete: true,
        discovered_target_path: None,
        applied_paths: vec![],
        blocked_paths: vec![],
        error_codes: vec![],
        token_metrics: BTreeMap::new(),
        failure_state: None,
        empty_page_detected: false,
        bootstrap_applied: false,
    };

    let request = MultiPageRequest {
        plan_id: "plan-1".to_string(),
        pages: vec![PageTarget {
            page_id: None,
            create: Some(CreatePageTarget {
                title: "Report".to_string(),
                parent_page_id: "123".to_string(),
                space_key: "PROJ".to_string(),
            }),
            edit_intent: "Create report section".to_string(),
            scope_selectors: vec![],
            run_mode: PageRunMode::SimpleScopedProseUpdate {
                target_path: Some("/content/0/content/0/text".to_string()),
                markdown: "Hello".to_string(),
            },
            block_ops: vec![BlockOp::InsertSection {
                parent_path: "/content".to_string(),
                index: 0,
                heading_level: 2,
                heading_text: "Overview".to_string(),
                body_blocks: vec![serde_json::json!({
                    "type": "paragraph",
                    "content": [{"type": "text", "text": "Body"}]
                })],
            }],
            bootstrap_empty_page: true,
            depends_on: vec!["@0".to_string()],
        }],
        rollback_on_failure: true,
        provenance: provenance.clone(),
        timestamp: "2026-03-15T00:00:00Z".to_string(),
    };

    let snapshot = PageSnapshot {
        page_id: "page-1".to_string(),
        version_before: 4,
        adf_before: serde_json::json!({"type": "doc", "content": []}),
        version_after: Some(5),
    };

    let summary = MultiPageSummary {
        plan_id: "plan-1".to_string(),
        success: false,
        page_results: vec![PageResult {
            page_id: "page-1".to_string(),
            created: false,
            summary: run_summary,
        }],
        rollback_results: vec![RollbackResult {
            page_id: "page-1".to_string(),
            success: true,
            conflict: false,
            error: None,
        }],
        total_pages: 1,
        succeeded_pages: 0,
        failed_pages: 1,
        rolled_back_pages: 1,
        latency_ms: 123,
    };

    let request_round_trip: MultiPageRequest =
        serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
    let snapshot_round_trip: PageSnapshot =
        serde_json::from_str(&serde_json::to_string(&snapshot).unwrap()).unwrap();
    let summary_round_trip: MultiPageSummary =
        serde_json::from_str(&serde_json::to_string(&summary).unwrap()).unwrap();

    assert_eq!(request_round_trip.plan_id, request.plan_id);
    assert_eq!(request_round_trip.pages.len(), 1);
    assert!(matches!(
        request_round_trip.pages[0].run_mode,
        PageRunMode::SimpleScopedProseUpdate { .. }
    ));
    assert_eq!(snapshot_round_trip.version_after, Some(5));
    assert_eq!(summary_round_trip.plan_id, summary.plan_id);
    assert_eq!(summary_round_trip.rollback_results.len(), 1);
}
