use atlassy_adf::{
    build_list, build_section, build_table, find_section_range, is_insertable_type,
    is_removable_type, is_within_allowed_scope, AdfError,
};
use atlassy_contracts::{
    BlockOp, Diagnostics, FetchOutput, Operation, PipelineState, StateEnvelope,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error_map::to_hard_error;
use crate::util::meta;
use crate::{ArtifactStore, PipelineError, RunRequest, StateTracker};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AdfBlockOpsInput {
    block_ops: Vec<BlockOp>,
    allowed_scope_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct AdfBlockOpsOutput {
    pub operations: Vec<Operation>,
}

pub(crate) fn run_adf_block_ops_state(
    artifact_store: &ArtifactStore,
    request: &RunRequest,
    tracker: &mut StateTracker,
    fetch: &StateEnvelope<FetchOutput>,
) -> Result<StateEnvelope<AdfBlockOpsOutput>, PipelineError> {
    tracker.transition_to(PipelineState::AdfBlockOps)?;

    let input = StateEnvelope {
        meta: meta(request, PipelineState::AdfBlockOps),
        payload: AdfBlockOpsInput {
            block_ops: request.block_ops.clone(),
            allowed_scope_paths: fetch.payload.allowed_scope_paths.clone(),
        },
    };

    let operations = request
        .block_ops
        .iter()
        .map(|block_op| {
            translate_block_op(
                block_op,
                &fetch.payload.allowed_scope_paths,
                &fetch.payload.scoped_adf,
            )
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|nested| nested.into_iter().flatten().collect::<Vec<_>>())
        .map_err(|error| to_hard_error(PipelineState::AdfBlockOps, error))?;

    let output = StateEnvelope {
        meta: meta(request, PipelineState::AdfBlockOps),
        payload: AdfBlockOpsOutput { operations },
    };

    artifact_store.persist_state(
        &request.request_id,
        PipelineState::AdfBlockOps,
        &input,
        &output,
        &Diagnostics::default(),
    )?;

    Ok(output)
}

fn translate_block_op(
    block_op: &BlockOp,
    allowed_scope_paths: &[String],
    scoped_adf: &Value,
) -> Result<Vec<Operation>, AdfError> {
    match block_op {
        BlockOp::Insert {
            parent_path,
            index,
            block,
        } => translate_insert(parent_path, *index, block, allowed_scope_paths).map(|op| vec![op]),
        BlockOp::Remove { target_path } => {
            translate_remove(target_path, allowed_scope_paths, scoped_adf).map(|op| vec![op])
        }
        BlockOp::InsertSection {
            parent_path,
            index,
            heading_level,
            heading_text,
            body_blocks,
        } => translate_insert_section(
            parent_path,
            *index,
            *heading_level,
            heading_text,
            body_blocks,
            allowed_scope_paths,
        ),
        BlockOp::RemoveSection { heading_path } => {
            translate_remove_section(heading_path, allowed_scope_paths, scoped_adf)
        }
        BlockOp::InsertTable {
            parent_path,
            index,
            rows,
            cols,
            header_row,
        } => translate_insert_table(
            parent_path,
            *index,
            *rows,
            *cols,
            *header_row,
            allowed_scope_paths,
        )
        .map(|op| vec![op]),
        BlockOp::InsertList {
            parent_path,
            index,
            ordered,
            items,
        } => translate_insert_list(parent_path, *index, *ordered, items, allowed_scope_paths)
            .map(|op| vec![op]),
    }
}

fn translate_insert(
    parent_path: &str,
    index: usize,
    block: &Value,
    allowed_scope_paths: &[String],
) -> Result<Operation, AdfError> {
    if !is_within_allowed_scope(parent_path, allowed_scope_paths) {
        return Err(AdfError::OutOfScope(parent_path.to_string()));
    }

    validate_insert_block_type(block)?;

    Ok(Operation::Insert {
        parent_path: parent_path.to_string(),
        index,
        block: block.clone(),
    })
}

fn translate_remove(
    target_path: &str,
    allowed_scope_paths: &[String],
    scoped_adf: &Value,
) -> Result<Operation, AdfError> {
    if !is_within_allowed_scope(target_path, allowed_scope_paths) {
        return Err(AdfError::OutOfScope(target_path.to_string()));
    }

    let target_node = scoped_adf
        .pointer(target_path)
        .ok_or_else(|| AdfError::RemoveTargetNotFound(format!("target path `{target_path}`")))?;
    let node_type = target_node.get("type").and_then(Value::as_str);
    if !node_type.is_some_and(is_removable_type) {
        return Err(AdfError::RemoveTargetNotFound(format!(
            "remove target `{target_path}` must be one of {:?}, got {:?}",
            atlassy_adf::REMOVABLE_BLOCK_TYPES,
            node_type
        )));
    }

    Ok(Operation::Remove {
        target_path: target_path.to_string(),
    })
}

fn translate_insert_section(
    parent_path: &str,
    index: usize,
    heading_level: u8,
    heading_text: &str,
    body_blocks: &[Value],
    allowed_scope_paths: &[String],
) -> Result<Vec<Operation>, AdfError> {
    if !is_within_allowed_scope(parent_path, allowed_scope_paths) {
        return Err(AdfError::OutOfScope(parent_path.to_string()));
    }

    let section_blocks = build_section(heading_level, heading_text, body_blocks)?;
    for block in &section_blocks {
        validate_insert_block_type(block)?;
    }

    section_blocks
        .into_iter()
        .enumerate()
        .map(|(offset, block)| {
            let position = index.checked_add(offset).ok_or_else(|| {
                AdfError::InsertPositionInvalid(format!(
                    "insert index overflow for `{parent_path}` at base index {index}"
                ))
            })?;

            Ok(Operation::Insert {
                parent_path: parent_path.to_string(),
                index: position,
                block,
            })
        })
        .collect()
}

fn translate_remove_section(
    heading_path: &str,
    allowed_scope_paths: &[String],
    scoped_adf: &Value,
) -> Result<Vec<Operation>, AdfError> {
    if !is_within_allowed_scope(heading_path, allowed_scope_paths) {
        return Err(AdfError::OutOfScope(heading_path.to_string()));
    }

    let section_range = find_section_range(scoped_adf, heading_path)?;
    for block_path in &section_range.block_paths {
        if !is_within_allowed_scope(block_path, allowed_scope_paths) {
            return Err(AdfError::OutOfScope(block_path.clone()));
        }
    }

    Ok(section_range
        .block_paths
        .into_iter()
        .rev()
        .map(|target_path| Operation::Remove { target_path })
        .collect())
}

fn translate_insert_table(
    parent_path: &str,
    index: usize,
    rows: usize,
    cols: usize,
    header_row: bool,
    allowed_scope_paths: &[String],
) -> Result<Operation, AdfError> {
    if !is_within_allowed_scope(parent_path, allowed_scope_paths) {
        return Err(AdfError::OutOfScope(parent_path.to_string()));
    }

    let block = build_table(rows, cols, header_row)?;
    validate_insert_block_type(&block)?;

    Ok(Operation::Insert {
        parent_path: parent_path.to_string(),
        index,
        block,
    })
}

fn translate_insert_list(
    parent_path: &str,
    index: usize,
    ordered: bool,
    items: &[String],
    allowed_scope_paths: &[String],
) -> Result<Operation, AdfError> {
    if !is_within_allowed_scope(parent_path, allowed_scope_paths) {
        return Err(AdfError::OutOfScope(parent_path.to_string()));
    }

    let item_refs = items.iter().map(String::as_str).collect::<Vec<_>>();
    let block = build_list(ordered, &item_refs)?;
    validate_insert_block_type(&block)?;

    Ok(Operation::Insert {
        parent_path: parent_path.to_string(),
        index,
        block,
    })
}

fn validate_insert_block_type(block: &Value) -> Result<(), AdfError> {
    let block_type = block.get("type").and_then(Value::as_str);
    if !block_type.is_some_and(is_insertable_type) {
        return Err(AdfError::InsertPositionInvalid(format!(
            "insert block type must be one of {:?}, got {:?}",
            atlassy_adf::INSERTABLE_BLOCK_TYPES,
            block_type
        )));
    }

    Ok(())
}
