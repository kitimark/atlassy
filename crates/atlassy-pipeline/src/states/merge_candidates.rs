use atlassy_contracts::{
    AdfTableEditOutput, ClassifyOutput, Diagnostics, ErrorCode, MdAssistEditOutput,
    MergeCandidatesInput, MergeCandidatesOutput, Operation, PipelineState, StateEnvelope,
};

use super::adf_block_ops::AdfBlockOpsOutput;
use crate::util::meta;
use crate::{ArtifactStore, PipelineError, RunRequest, StateTracker};

pub(crate) fn run_merge_candidates_state(
    artifact_store: &ArtifactStore,
    request: &RunRequest,
    tracker: &mut StateTracker,
    classify: &StateEnvelope<ClassifyOutput>,
    md_edit: &StateEnvelope<MdAssistEditOutput>,
    table_edit: &StateEnvelope<AdfTableEditOutput>,
    adf_block_ops: &StateEnvelope<AdfBlockOpsOutput>,
) -> Result<StateEnvelope<MergeCandidatesOutput>, PipelineError> {
    tracker.transition_to(PipelineState::MergeCandidates)?;

    let input = StateEnvelope {
        meta: meta(request, PipelineState::MergeCandidates),
        payload: MergeCandidatesInput {
            prose_changed_paths: md_edit.payload.prose_changed_paths.clone(),
            table_changed_paths: table_edit.payload.table_changed_paths.clone(),
            block_operations: adf_block_ops.payload.operations.clone(),
        },
    };

    let prose_operations = md_edit
        .payload
        .prose_change_candidates
        .iter()
        .map(|candidate| Operation::Replace {
            path: candidate.path.clone(),
            value: serde_json::Value::String(candidate.markdown.clone()),
        })
        .collect::<Vec<_>>();
    let table_operations = table_edit
        .payload
        .table_candidates
        .iter()
        .map(|candidate| Operation::Replace {
            path: candidate.path.clone(),
            value: candidate.value.clone(),
        })
        .collect::<Vec<_>>();
    let block_operations = input.payload.block_operations.clone();

    for prose_operation in &prose_operations {
        for table_operation in &table_operations {
            if operations_overlap(prose_operation, table_operation) {
                return Err(PipelineError::Hard {
                    state: PipelineState::MergeCandidates,
                    code: ErrorCode::RouteViolation,
                    message: format!(
                        "cross-route conflict: prose operation {:?} overlaps table operation {:?}",
                        prose_operation, table_operation
                    ),
                });
            }
        }
    }

    for prose_operation in &prose_operations {
        for block_operation in &block_operations {
            if operations_overlap(prose_operation, block_operation) {
                return Err(PipelineError::Hard {
                    state: PipelineState::MergeCandidates,
                    code: ErrorCode::RouteViolation,
                    message: format!(
                        "cross-route conflict: prose operation {:?} overlaps block operation {:?}",
                        prose_operation, block_operation
                    ),
                });
            }
        }
    }

    for table_operation in &table_operations {
        for block_operation in &block_operations {
            if operations_overlap(table_operation, block_operation) {
                return Err(PipelineError::Hard {
                    state: PipelineState::MergeCandidates,
                    code: ErrorCode::RouteViolation,
                    message: format!(
                        "cross-route conflict: table operation {:?} overlaps block operation {:?}",
                        table_operation, block_operation
                    ),
                });
            }
        }
    }

    let locked_paths = classify
        .payload
        .node_manifest
        .iter()
        .filter(|node| {
            node.route == "locked_structural"
                && node.path != "/"
                && node.node_type.as_str() != "doc"
        })
        .map(|node| node.path.as_str())
        .collect::<Vec<_>>();

    for operation in table_operations.iter().chain(block_operations.iter()) {
        for path in operation_paths(operation) {
            if locked_paths
                .iter()
                .any(|locked_path| path_is_within_locked_boundary(&path, locked_path))
            {
                return Err(PipelineError::Hard {
                    state: PipelineState::MergeCandidates,
                    code: ErrorCode::RouteViolation,
                    message: format!("operation path `{path}` overlaps locked structural boundary"),
                });
            }
        }
    }

    let mut operations = prose_operations;
    operations.extend(table_operations);
    operations.extend(block_operations);
    let output = StateEnvelope {
        meta: meta(request, PipelineState::MergeCandidates),
        payload: MergeCandidatesOutput { operations },
    };

    artifact_store.persist_state(
        &request.request_id,
        PipelineState::MergeCandidates,
        &input,
        &output,
        &Diagnostics::default(),
    )?;
    Ok(output)
}

fn path_is_within_locked_boundary(path: &str, locked_path: &str) -> bool {
    path == locked_path
        || path
            .strip_prefix(locked_path)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

fn paths_overlap(left: &str, right: &str) -> bool {
    left == right
        || left
            .strip_prefix(right)
            .is_some_and(|suffix| suffix.starts_with('/'))
        || right
            .strip_prefix(left)
            .is_some_and(|suffix| suffix.starts_with('/'))
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

fn operations_overlap(left: &Operation, right: &Operation) -> bool {
    operation_paths(left).iter().any(|left_path| {
        operation_paths(right)
            .iter()
            .any(|right_path| paths_overlap(left_path, right_path))
    })
}
