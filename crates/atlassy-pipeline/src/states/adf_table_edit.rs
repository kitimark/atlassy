use std::collections::BTreeMap;

use atlassy_adf::{
    TargetRoute, canonicalize_mapped_path, discover_target_path, is_table_cell_text_path,
    is_table_shape_or_attr_path, normalize_changed_paths, path_has_ancestor_type,
};
use atlassy_contracts::{
    AdfTableEditInput, AdfTableEditOutput, ClassifyOutput, Diagnostics, ErrorCode, FetchOutput,
    PipelineState, RunSummary, StateEnvelope, TableChangeCandidate, TableOperation,
    validate_table_candidates,
};

use crate::error_map::to_hard_error;
use crate::util::meta;
use crate::{ArtifactStore, PipelineError, RunMode, RunRequest, StateTracker};

pub(crate) fn run_adf_table_edit_state(
    artifact_store: &ArtifactStore,
    request: &RunRequest,
    tracker: &mut StateTracker,
    fetch: &StateEnvelope<FetchOutput>,
    classify: &StateEnvelope<ClassifyOutput>,
    summary: &mut RunSummary,
) -> Result<StateEnvelope<AdfTableEditOutput>, PipelineError> {
    tracker.transition_to(PipelineState::AdfTableEdit)?;
    let table_nodes = classify
        .payload
        .node_manifest
        .iter()
        .filter(|node| node.route == "table_adf")
        .cloned()
        .collect::<Vec<_>>();

    let input = StateEnvelope {
        meta: meta(request, PipelineState::AdfTableEdit),
        payload: AdfTableEditInput {
            table_nodes,
            allowed_scope_paths: fetch.payload.allowed_scope_paths.clone(),
            edit_intent: request.edit_intent.clone(),
        },
    };

    let allowed_ops = vec![TableOperation::CellTextUpdate];
    let mut table_candidates = Vec::new();

    match &request.run_mode {
        RunMode::NoOp | RunMode::SimpleScopedProseUpdate { .. } => {}
        RunMode::SimpleScopedTableCellUpdate { target_path, text } => {
            let resolved_target_path = if let Some(path) = target_path {
                path.clone()
            } else {
                let discovered_path = discover_target_path(
                    &fetch.payload.node_path_index,
                    &fetch.payload.allowed_scope_paths,
                    TargetRoute::TableCell,
                    request.target_index,
                )
                .map_err(|error| PipelineError::Hard {
                    state: PipelineState::AdfTableEdit,
                    code: ErrorCode::TargetDiscoveryFailed,
                    message: error.to_string(),
                })?;
                summary.discovered_target_path = Some(discovered_path.clone());
                discovered_path
            };

            project_table_candidate(
                &resolved_target_path,
                text,
                &fetch.payload.allowed_scope_paths,
                &fetch.payload.node_path_index,
                &allowed_ops,
                &mut table_candidates,
            )?;
        }
        RunMode::ForbiddenTableOperation {
            target_path,
            operation,
        } => {
            if *operation != TableOperation::CellTextUpdate {
                return Err(PipelineError::Hard {
                    state: PipelineState::AdfTableEdit,
                    code: ErrorCode::TableShapeChange,
                    message: format!(
                        "forbidden table operation requested: {} at {}",
                        operation.as_str(),
                        target_path
                    ),
                });
            }

            project_table_candidate(
                target_path,
                "Allowed table operation",
                &fetch.payload.allowed_scope_paths,
                &fetch.payload.node_path_index,
                &allowed_ops,
                &mut table_candidates,
            )?;
        }
        RunMode::SyntheticRouteConflict { table_path, .. } => {
            project_table_candidate(
                table_path,
                "Synthetic table conflict candidate",
                &fetch.payload.allowed_scope_paths,
                &fetch.payload.node_path_index,
                &allowed_ops,
                &mut table_candidates,
            )?;
        }
        RunMode::SyntheticTableShapeDrift { path } => {
            table_candidates.push(TableChangeCandidate {
                op: TableOperation::CellTextUpdate,
                path: path.clone(),
                value: serde_json::Value::String("Synthetic drift".to_string()),
                source_route: "table_adf".to_string(),
            });
        }
    }

    table_candidates.sort_by(|left, right| left.path.cmp(&right.path));
    validate_table_candidates(&table_candidates, &allowed_ops)?;

    let table_changed_paths = normalize_changed_paths(
        &table_candidates
            .iter()
            .map(|candidate| candidate.path.clone())
            .collect::<Vec<_>>(),
    )
    .map_err(|error| to_hard_error(PipelineState::AdfTableEdit, error))?;

    let output = StateEnvelope {
        meta: meta(request, PipelineState::AdfTableEdit),
        payload: AdfTableEditOutput {
            table_candidates,
            table_changed_paths,
            allowed_ops,
        },
    };

    artifact_store.persist_state(
        &request.request_id,
        PipelineState::AdfTableEdit,
        &input,
        &output,
        &Diagnostics::default(),
    )?;
    Ok(output)
}

fn project_table_candidate(
    target_path: &str,
    text: &str,
    allowed_scope_paths: &[String],
    node_path_index: &BTreeMap<String, String>,
    allowed_ops: &[TableOperation],
    table_candidates: &mut Vec<TableChangeCandidate>,
) -> Result<(), PipelineError> {
    let canonical_path = canonicalize_mapped_path(target_path, allowed_scope_paths)
        .map_err(|error| to_hard_error(PipelineState::AdfTableEdit, error))?;

    if !path_has_ancestor_type(
        &canonical_path,
        node_path_index,
        &["table", "tableRow", "tableCell"],
    ) {
        return Err(PipelineError::Hard {
            state: PipelineState::AdfTableEdit,
            code: ErrorCode::RouteViolation,
            message: format!("target path `{canonical_path}` is not in table route"),
        });
    }

    if !is_table_cell_text_path(&canonical_path, node_path_index)
        || is_table_shape_or_attr_path(&canonical_path, node_path_index)
    {
        return Err(PipelineError::Hard {
            state: PipelineState::AdfTableEdit,
            code: ErrorCode::TableShapeChange,
            message: format!(
                "target path `{canonical_path}` is not an allowed table cell text update path"
            ),
        });
    }

    let candidate = TableChangeCandidate {
        op: TableOperation::CellTextUpdate,
        path: canonical_path,
        value: serde_json::Value::String(text.to_string()),
        source_route: "table_adf".to_string(),
    };

    validate_table_candidates(std::slice::from_ref(&candidate), allowed_ops).map_err(|error| {
        PipelineError::Hard {
            state: PipelineState::AdfTableEdit,
            code: ErrorCode::TableShapeChange,
            message: error.to_string(),
        }
    })?;

    table_candidates.push(candidate);
    Ok(())
}
