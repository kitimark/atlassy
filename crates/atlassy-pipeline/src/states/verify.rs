use atlassy_adf::{
    SCOPE_ANCHOR_TYPES, check_structural_validity, is_attr_editable_type, is_editable_prose,
    is_insertable_type, is_removable_type, is_table_cell_text_path, is_within_allowed_scope,
    path_has_ancestor_type,
};
use atlassy_contracts::{
    BlockOp, Diagnostics, ErrorCode, ErrorInfo, FetchOutput, MergeCandidatesOutput, Operation,
    PatchOutput, PipelineState, StateEnvelope, VerifyInput, VerifyOutput, VerifyResult,
};
use serde_json::Value;

use super::locked_boundary::{LockedPath, check_locked_boundary};
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
                &request.block_ops,
                &fetch.payload.node_path_index,
                &mut diagnostics,
            )
        })
        .or_else(|| {
            check_locked_boundary_violations(
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
    operation_manifest: &[BlockOp],
    node_path_index: &std::collections::BTreeMap<String, String>,
    diagnostics: &mut Diagnostics,
) -> Option<VerifyResult> {
    let declared_table_topology_change = operation_manifest.iter().any(|operation| {
        matches!(
            operation,
            BlockOp::InsertRow { .. }
                | BlockOp::RemoveRow { .. }
                | BlockOp::InsertColumn { .. }
                | BlockOp::RemoveColumn { .. }
        )
    });

    let violating_path = operations.iter().flat_map(operation_paths).find(|path| {
        let under_table = path_is_table_related(path, node_path_index);
        if !under_table {
            return false;
        }

        if is_table_cell_text_path(path, node_path_index) {
            return false;
        }

        !declared_table_topology_change
    })?;

    diagnostics.errors.push(ErrorInfo {
        code: ErrorCode::TableShapeChange.to_string(),
        message: format!("forbidden table shape or attribute mutation at `{violating_path}`"),
        recovery:
            "declare row/column block operations for table topology changes or limit edits to cell text"
                .to_string(),
    });
    Some(VerifyResult::Fail)
}

fn check_locked_boundary_violations(
    operations: &[Operation],
    node_path_index: &std::collections::BTreeMap<String, String>,
    diagnostics: &mut Diagnostics,
) -> Option<VerifyResult> {
    let locked_paths = node_path_index
        .iter()
        .filter(|(path, node_type)| is_locked_structural_path(path, node_type, node_path_index))
        .map(|(path, node_type)| LockedPath {
            path: path.as_str(),
            node_type: node_type.as_str(),
        })
        .collect::<Vec<_>>();

    for operation in operations {
        if let Some(error) = check_locked_boundary(operation, &locked_paths)
            && let PipelineError::Hard { code, message, .. } = error
        {
            diagnostics.errors.push(ErrorInfo {
                code: code.to_string(),
                message,
                recovery: "avoid mutations that overlap locked structural boundaries".to_string(),
            });
            return Some(VerifyResult::Fail);
        }
    }

    None
}

fn is_locked_structural_path(
    path: &str,
    node_type: &str,
    node_path_index: &std::collections::BTreeMap<String, String>,
) -> bool {
    if path == "/" || node_type == "doc" || node_type == "text" {
        return false;
    }

    if is_editable_prose(node_type) {
        return false;
    }

    if matches!(node_type, "table" | "tableRow" | "tableCell") {
        return false;
    }

    !path_has_ancestor_type(path, node_path_index, &["table", "tableRow", "tableCell"])
}

fn path_is_table_related(
    path: &str,
    node_path_index: &std::collections::BTreeMap<String, String>,
) -> bool {
    if let Some(node_type) = node_path_index.get(path)
        && matches!(node_type.as_str(), "table" | "tableRow" | "tableCell")
    {
        return true;
    }

    path_has_ancestor_type(path, node_path_index, &["table", "tableRow", "tableCell"])
}

fn check_operation_legality(
    operations: &[Operation],
    original_scoped_adf: &Value,
    allowed_scope_paths: &[String],
    diagnostics: &mut Diagnostics,
) -> Option<VerifyResult> {
    let mut insert_offsets: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();

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
                let is_table_cell_insert =
                    block_type.is_some_and(|node_type| matches!(node_type, "tableCell" | "tableHeader"));
                if !block_type.is_some_and(is_insertable_type) && !is_table_cell_insert {
                    diagnostics.errors.push(ErrorInfo {
                        code: ErrorCode::InsertPositionInvalid.to_string(),
                        message: format!(
                            "insert block type must be one of {:?} (or tableCell/tableHeader for table column ops), got {:?}",
                            atlassy_adf::INSERTABLE_BLOCK_TYPES,
                            block_type
                        ),
                        recovery: "insert only allowed block types".to_string(),
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

                let offset = insert_offsets.get(parent_path).copied().unwrap_or(0);
                if *index > parent_array.len() + offset {
                    diagnostics.errors.push(ErrorInfo {
                        code: ErrorCode::InsertPositionInvalid.to_string(),
                        message: format!(
                            "insert index {index} out of bounds for `{parent_path}` with effective length {}",
                            parent_array.len() + offset
                        ),
                        recovery: "use an index in [0, parent_len]".to_string(),
                    });
                    return Some(VerifyResult::Fail);
                }

                insert_offsets
                    .entry(parent_path.clone())
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
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
                let is_table_cell_remove =
                    node_type.is_some_and(|value| matches!(value, "tableCell" | "tableHeader"));
                if !node_type.is_some_and(is_removable_type) && !is_table_cell_remove {
                    diagnostics.errors.push(ErrorInfo {
                        code: ErrorCode::RemoveAnchorMissing.to_string(),
                        message: format!(
                            "remove target `{target_path}` must be one of {:?} (or tableCell/tableHeader for table column ops), got {:?}",
                            atlassy_adf::REMOVABLE_BLOCK_TYPES,
                            node_type
                        ),
                        recovery: "remove only allowed block types".to_string(),
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
            Operation::UpdateAttrs { target_path, attrs } => {
                if !is_within_allowed_scope(target_path, allowed_scope_paths) {
                    diagnostics.errors.push(ErrorInfo {
                        code: ErrorCode::OutOfScopeMutation.to_string(),
                        message: format!(
                            "update attrs target path `{target_path}` is outside allowed scope"
                        ),
                        recovery: "restrict update attrs operations to allowed_scope_paths"
                            .to_string(),
                    });
                    return Some(VerifyResult::Fail);
                }

                let Some(target_node) = original_scoped_adf.pointer(target_path) else {
                    diagnostics.errors.push(ErrorInfo {
                        code: ErrorCode::AttrUpdateBlocked.to_string(),
                        message: format!(
                            "update attrs target path `{target_path}` does not resolve"
                        ),
                        recovery: "target an existing attr-editable node".to_string(),
                    });
                    return Some(VerifyResult::Fail);
                };

                let Some(node_type) = target_node.get("type").and_then(Value::as_str) else {
                    diagnostics.errors.push(ErrorInfo {
                        code: ErrorCode::AttrUpdateBlocked.to_string(),
                        message: format!(
                            "update attrs target `{target_path}` is missing a node type"
                        ),
                        recovery: "target a valid ADF node with a type field".to_string(),
                    });
                    return Some(VerifyResult::Fail);
                };

                if !is_attr_editable_type(node_type) {
                    diagnostics.errors.push(ErrorInfo {
                        code: ErrorCode::AttrUpdateBlocked.to_string(),
                        message: format!(
                            "update attrs target `{target_path}` has non-editable node type `{node_type}`"
                        ),
                        recovery: "target panel, expand, or mediaSingle nodes only".to_string(),
                    });
                    return Some(VerifyResult::Fail);
                }

                let Some(attr_map) = attrs.as_object() else {
                    diagnostics.errors.push(ErrorInfo {
                        code: ErrorCode::AttrSchemaViolation.to_string(),
                        message: format!(
                            "update attrs payload for `{target_path}` must be an object"
                        ),
                        recovery: "provide attrs as a JSON object".to_string(),
                    });
                    return Some(VerifyResult::Fail);
                };

                let allowed_keys = allowed_attr_keys_for_node_type(node_type).unwrap_or(&[]);
                if let Some(disallowed) = attr_map
                    .keys()
                    .find(|key| !allowed_keys.iter().any(|allowed| allowed == &key.as_str()))
                {
                    diagnostics.errors.push(ErrorInfo {
                        code: ErrorCode::AttrSchemaViolation.to_string(),
                        message: format!(
                            "attr `{disallowed}` is not allowed on node type `{node_type}`"
                        ),
                        recovery:
                            "use only allowed attrs (panel: panelType, expand: title, mediaSingle: alt/title/width/height)"
                                .to_string(),
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
            Operation::UpdateAttrs { target_path, .. }
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

fn allowed_attr_keys_for_node_type(node_type: &str) -> Option<&'static [&'static str]> {
    const PANEL_ATTRS: &[&str] = &["panelType"];
    const EXPAND_ATTRS: &[&str] = &["title"];
    const MEDIA_SINGLE_ATTRS: &[&str] = &["alt", "title", "width", "height"];

    match node_type {
        "panel" => Some(PANEL_ATTRS),
        "expand" => Some(EXPAND_ATTRS),
        "mediaSingle" => Some(MEDIA_SINGLE_ATTRS),
        _ => None,
    }
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
        Operation::UpdateAttrs { target_path, .. } => vec![target_path.clone()],
    }
}
