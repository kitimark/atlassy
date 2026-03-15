use atlassy_adf::{
    check_structural_validity, is_table_cell_text_path, is_table_shape_or_attr_path,
    is_within_allowed_scope, path_has_ancestor_type, EDITABLE_PROSE_TYPES, SCOPE_ANCHOR_TYPES,
};
use atlassy_contracts::{
    Diagnostics, ErrorCode, ErrorInfo, FetchOutput, MergeCandidatesOutput, Operation, PatchOutput,
    PipelineState, StateEnvelope, VerifyInput, VerifyOutput, VerifyResult,
};
use serde_json::Value;

use crate::util::meta;
use crate::{ArtifactStore, PipelineError, RunRequest, StateTracker};

pub(crate) fn run_verify_state(
    artifact_store: &ArtifactStore,
    request: &RunRequest,
    tracker: &mut StateTracker,
    fetch: &StateEnvelope<FetchOutput>,
    patch: &StateEnvelope<PatchOutput>,
    merged: &StateEnvelope<MergeCandidatesOutput>,
) -> Result<StateEnvelope<VerifyOutput>, PipelineError> {
    tracker.transition_to(PipelineState::Verify)?;

    let input = StateEnvelope {
        meta: meta(request, PipelineState::Verify),
        payload: VerifyInput {
            original_scoped_adf: fetch.payload.scoped_adf.clone(),
            candidate_page_adf: patch.payload.candidate_page_adf.clone(),
            allowed_scope_paths: fetch.payload.allowed_scope_paths.clone(),
            operations: merged.payload.operations.clone(),
        },
    };

    let mut diagnostics = Diagnostics::default();
    let verify_result = check_forced_fail(request.force_verify_fail, &mut diagnostics)
        .or_else(|| {
            check_table_shape_integrity(
                &input.payload.operations,
                &fetch.payload.node_path_index,
                &mut diagnostics,
            )
        })
        .or_else(|| {
            check_operation_legality(
                &input.payload.operations,
                &input.payload.original_scoped_adf,
                &input.payload.allowed_scope_paths,
                &mut diagnostics,
            )
        })
        .or_else(|| {
            check_scope_containment(
                &input.payload.operations,
                &input.payload.allowed_scope_paths,
                &mut diagnostics,
            )
        })
        .or_else(|| {
            check_post_mutation_structural_validity(
                &input.payload.operations,
                &input.payload.candidate_page_adf,
                &mut diagnostics,
            )
        })
        .unwrap_or(VerifyResult::Pass);

    let output = StateEnvelope {
        meta: meta(request, PipelineState::Verify),
        payload: VerifyOutput {
            verify_result,
            diagnostics: diagnostics.clone(),
        },
    };

    artifact_store.persist_state(
        &request.request_id,
        PipelineState::Verify,
        &input,
        &output,
        &diagnostics,
    )?;
    Ok(output)
}

fn check_forced_fail(
    force_verify_fail: bool,
    diagnostics: &mut Diagnostics,
) -> Option<VerifyResult> {
    if !force_verify_fail {
        return None;
    }

    diagnostics.errors.push(ErrorInfo {
        code: ErrorCode::SchemaInvalid.to_string(),
        message: "forced verify failure".to_string(),
        recovery: "fix candidate payload".to_string(),
    });
    Some(VerifyResult::Fail)
}

fn check_table_shape_integrity(
    operations: &[Operation],
    node_path_index: &std::collections::BTreeMap<String, String>,
    diagnostics: &mut Diagnostics,
) -> Option<VerifyResult> {
    let violating_path = operations.iter().flat_map(operation_paths).find(|path| {
        is_table_shape_or_attr_path(path, node_path_index)
            || (path_has_ancestor_type(path, node_path_index, &["table", "tableRow", "tableCell"])
                && !is_table_cell_text_path(path, node_path_index))
    })?;

    diagnostics.errors.push(ErrorInfo {
        code: ErrorCode::TableShapeChange.to_string(),
        message: format!("forbidden table shape or attribute mutation at `{violating_path}`"),
        recovery: "limit table updates to cell text paths only".to_string(),
    });
    Some(VerifyResult::Fail)
}

fn check_operation_legality(
    operations: &[Operation],
    original_scoped_adf: &Value,
    allowed_scope_paths: &[String],
    diagnostics: &mut Diagnostics,
) -> Option<VerifyResult> {
    for operation in operations {
        match operation {
            Operation::Replace { path, .. } => {
                if !is_within_allowed_scope(path, allowed_scope_paths) {
                    diagnostics.errors.push(ErrorInfo {
                        code: ErrorCode::OutOfScopeMutation.to_string(),
                        message: format!("replace path `{path}` is outside allowed scope"),
                        recovery: "restrict replace operations to allowed_scope_paths".to_string(),
                    });
                    return Some(VerifyResult::Fail);
                }
            }
            Operation::Insert {
                parent_path,
                index,
                block,
            } => {
                if !is_within_allowed_scope(parent_path, allowed_scope_paths) {
                    diagnostics.errors.push(ErrorInfo {
                        code: ErrorCode::OutOfScopeMutation.to_string(),
                        message: format!(
                            "insert parent path `{parent_path}` is outside allowed scope"
                        ),
                        recovery: "restrict insert operations to allowed_scope_paths".to_string(),
                    });
                    return Some(VerifyResult::Fail);
                }

                let block_type = block.get("type").and_then(Value::as_str);
                if !block_type.is_some_and(|value| EDITABLE_PROSE_TYPES.contains(&value)) {
                    diagnostics.errors.push(ErrorInfo {
                        code: ErrorCode::InsertPositionInvalid.to_string(),
                        message: format!(
                            "insert block type must be one of {:?}, got {:?}",
                            EDITABLE_PROSE_TYPES, block_type
                        ),
                        recovery: "insert only editable prose block types".to_string(),
                    });
                    return Some(VerifyResult::Fail);
                }

                let Some(parent) = original_scoped_adf.pointer(parent_path) else {
                    diagnostics.errors.push(ErrorInfo {
                        code: ErrorCode::InsertPositionInvalid.to_string(),
                        message: format!("insert parent path `{parent_path}` does not resolve"),
                        recovery: "use a valid parent array path for insert".to_string(),
                    });
                    return Some(VerifyResult::Fail);
                };
                let Some(parent_array) = parent.as_array() else {
                    diagnostics.errors.push(ErrorInfo {
                        code: ErrorCode::InsertPositionInvalid.to_string(),
                        message: format!("insert parent path `{parent_path}` is not an array"),
                        recovery: "target an array parent path for insert".to_string(),
                    });
                    return Some(VerifyResult::Fail);
                };
                if *index > parent_array.len() {
                    diagnostics.errors.push(ErrorInfo {
                        code: ErrorCode::InsertPositionInvalid.to_string(),
                        message: format!(
                            "insert index {index} out of bounds for `{parent_path}` with length {}",
                            parent_array.len()
                        ),
                        recovery: "use an index in [0, parent_len]".to_string(),
                    });
                    return Some(VerifyResult::Fail);
                }
            }
            Operation::Remove { target_path } => {
                if !is_within_allowed_scope(target_path, allowed_scope_paths) {
                    diagnostics.errors.push(ErrorInfo {
                        code: ErrorCode::OutOfScopeMutation.to_string(),
                        message: format!(
                            "remove target path `{target_path}` is outside allowed scope"
                        ),
                        recovery: "restrict remove operations to allowed_scope_paths".to_string(),
                    });
                    return Some(VerifyResult::Fail);
                }

                let Some(target_node) = original_scoped_adf.pointer(target_path) else {
                    diagnostics.errors.push(ErrorInfo {
                        code: ErrorCode::RemoveAnchorMissing.to_string(),
                        message: format!("remove target path `{target_path}` does not resolve"),
                        recovery: "remove only existing, in-scope block nodes".to_string(),
                    });
                    return Some(VerifyResult::Fail);
                };

                let node_type = target_node.get("type").and_then(Value::as_str);
                if !node_type.is_some_and(|value| EDITABLE_PROSE_TYPES.contains(&value)) {
                    diagnostics.errors.push(ErrorInfo {
                        code: ErrorCode::RemoveAnchorMissing.to_string(),
                        message: format!(
                            "remove target `{target_path}` must be one of {:?}, got {:?}",
                            EDITABLE_PROSE_TYPES, node_type
                        ),
                        recovery: "remove only editable prose blocks".to_string(),
                    });
                    return Some(VerifyResult::Fail);
                }

                if node_type.is_some_and(|value| SCOPE_ANCHOR_TYPES.contains(&value))
                    && allowed_scope_paths.iter().any(|path| path == target_path)
                {
                    diagnostics.errors.push(ErrorInfo {
                        code: ErrorCode::RemoveAnchorMissing.to_string(),
                        message: format!(
                            "remove target `{target_path}` is a protected scope anchor"
                        ),
                        recovery: "remove non-anchor block nodes only".to_string(),
                    });
                    return Some(VerifyResult::Fail);
                }
            }
        }
    }

    None
}

fn check_scope_containment(
    operations: &[Operation],
    allowed_scope_paths: &[String],
    diagnostics: &mut Diagnostics,
) -> Option<VerifyResult> {
    for operation in operations {
        let violating_path = match operation {
            Operation::Replace { path, .. }
                if !is_within_allowed_scope(path, allowed_scope_paths) =>
            {
                Some(path.clone())
            }
            Operation::Insert {
                parent_path, index, ..
            } if !is_within_allowed_scope(parent_path, allowed_scope_paths) => {
                Some(format!("{parent_path}/{index}"))
            }
            Operation::Remove { target_path }
                if !is_within_allowed_scope(target_path, allowed_scope_paths) =>
            {
                Some(target_path.clone())
            }
            _ => None,
        };

        if let Some(path) = violating_path {
            diagnostics.errors.push(ErrorInfo {
                code: ErrorCode::OutOfScopeMutation.to_string(),
                message: format!("operation path `{path}` is outside allowed scope"),
                recovery: "restrict changes to allowed_scope_paths".to_string(),
            });
            return Some(VerifyResult::Fail);
        }
    }

    None
}

fn check_post_mutation_structural_validity(
    operations: &[Operation],
    candidate_page_adf: &Value,
    diagnostics: &mut Diagnostics,
) -> Option<VerifyResult> {
    if !operations.iter().any(|operation| {
        matches!(
            operation,
            Operation::Insert { .. } | Operation::Remove { .. }
        )
    }) {
        return None;
    }

    let error = check_structural_validity(candidate_page_adf).err()?;
    diagnostics.errors.push(ErrorInfo {
        code: ErrorCode::PostMutationSchemaInvalid.to_string(),
        message: error.to_string(),
        recovery: "ensure structural operations produce valid ADF".to_string(),
    });
    Some(VerifyResult::Fail)
}

fn operation_paths(operation: &Operation) -> Vec<String> {
    match operation {
        Operation::Replace { path, .. } => vec![path.clone()],
        Operation::Insert {
            parent_path, index, ..
        } => vec![format!("{parent_path}/{index}")],
        Operation::Remove { target_path } => vec![target_path.clone()],
    }
}
