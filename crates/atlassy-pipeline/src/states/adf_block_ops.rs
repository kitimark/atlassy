use atlassy_adf::{
    AdfError, build_list, build_section, build_table, build_table_cell, build_table_header,
    build_table_row, find_section_range, is_attr_editable_type, is_insertable_type,
    is_removable_type, is_within_allowed_scope,
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
        BlockOp::InsertRow {
            table_path,
            index,
            cells,
        } => translate_insert_row(table_path, *index, cells, allowed_scope_paths, scoped_adf)
            .map(|op| vec![op]),
        BlockOp::RemoveRow { table_path, index } => {
            translate_remove_row(table_path, *index, allowed_scope_paths, scoped_adf)
                .map(|op| vec![op])
        }
        BlockOp::InsertColumn { table_path, index } => {
            translate_insert_column(table_path, *index, allowed_scope_paths, scoped_adf)
        }
        BlockOp::RemoveColumn { table_path, index } => {
            translate_remove_column(table_path, *index, allowed_scope_paths, scoped_adf)
        }
        BlockOp::UpdateAttrs { target_path, attrs } => {
            translate_update_attrs(target_path, attrs, allowed_scope_paths, scoped_adf)
                .map(|op| vec![op])
        }
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

fn translate_insert_row(
    table_path: &str,
    index: usize,
    cells: &[String],
    allowed_scope_paths: &[String],
    scoped_adf: &Value,
) -> Result<Operation, AdfError> {
    if !is_within_allowed_scope(table_path, allowed_scope_paths) {
        return Err(AdfError::OutOfScope(table_path.to_string()));
    }

    let table = scoped_adf.pointer(table_path).ok_or_else(|| {
        AdfError::TableRowInvalid(format!("table path `{table_path}` does not resolve"))
    })?;
    if table.get("type").and_then(Value::as_str) != Some("table") {
        return Err(AdfError::TableRowInvalid(format!(
            "target `{table_path}` is not a table"
        )));
    }

    let rows = table
        .get("content")
        .and_then(Value::as_array)
        .ok_or_else(|| AdfError::TableRowInvalid("table content must be an array".to_string()))?;

    if rows.is_empty() {
        return Err(AdfError::TableRowInvalid(
            "cannot insert row into empty table".to_string(),
        ));
    }
    if index > rows.len() {
        return Err(AdfError::TableRowInvalid(format!(
            "row index {index} out of bounds for table with {} rows",
            rows.len()
        )));
    }

    let expected_cols = rows
        .first()
        .and_then(|row| row.get("content"))
        .and_then(Value::as_array)
        .map(|cells| cells.len())
        .filter(|count| *count > 0)
        .ok_or_else(|| {
            AdfError::TableRowInvalid("table rows must contain at least one cell".to_string())
        })?;

    for (row_index, row) in rows.iter().enumerate() {
        let row_cells = row
            .get("content")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                AdfError::TableRowInvalid(format!(
                    "table row {row_index} is missing a content array"
                ))
            })?;
        if row_cells.len() != expected_cols {
            return Err(AdfError::TableRowInvalid(format!(
                "table row {row_index} has {} cells; expected {expected_cols}",
                row_cells.len()
            )));
        }
    }

    if cells.len() != expected_cols {
        return Err(AdfError::TableRowInvalid(format!(
            "new row has {} cells; expected {expected_cols}",
            cells.len()
        )));
    }

    let row_cells = cells
        .iter()
        .map(|cell| build_table_cell(cell))
        .collect::<Vec<_>>();
    let row = build_table_row(&row_cells);

    Ok(Operation::Insert {
        parent_path: format!("{table_path}/content"),
        index,
        block: row,
    })
}

fn translate_remove_row(
    table_path: &str,
    index: usize,
    allowed_scope_paths: &[String],
    scoped_adf: &Value,
) -> Result<Operation, AdfError> {
    if !is_within_allowed_scope(table_path, allowed_scope_paths) {
        return Err(AdfError::OutOfScope(table_path.to_string()));
    }

    let table = scoped_adf.pointer(table_path).ok_or_else(|| {
        AdfError::TableRowInvalid(format!("table path `{table_path}` does not resolve"))
    })?;
    if table.get("type").and_then(Value::as_str) != Some("table") {
        return Err(AdfError::TableRowInvalid(format!(
            "target `{table_path}` is not a table"
        )));
    }

    let rows = table
        .get("content")
        .and_then(Value::as_array)
        .ok_or_else(|| AdfError::TableRowInvalid("table content must be an array".to_string()))?;

    if rows.len() <= 1 {
        return Err(AdfError::TableRowInvalid(
            "cannot remove last remaining row from table".to_string(),
        ));
    }
    if index >= rows.len() {
        return Err(AdfError::TableRowInvalid(format!(
            "row index {index} out of bounds for table with {} rows",
            rows.len()
        )));
    }

    Ok(Operation::Remove {
        target_path: format!("{table_path}/content/{index}"),
    })
}

fn translate_insert_column(
    table_path: &str,
    index: usize,
    allowed_scope_paths: &[String],
    scoped_adf: &Value,
) -> Result<Vec<Operation>, AdfError> {
    if !is_within_allowed_scope(table_path, allowed_scope_paths) {
        return Err(AdfError::OutOfScope(table_path.to_string()));
    }

    let table = scoped_adf.pointer(table_path).ok_or_else(|| {
        AdfError::TableColumnInvalid(format!("table path `{table_path}` does not resolve"))
    })?;
    if table.get("type").and_then(Value::as_str) != Some("table") {
        return Err(AdfError::TableColumnInvalid(format!(
            "target `{table_path}` is not a table"
        )));
    }

    let rows = table
        .get("content")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            AdfError::TableColumnInvalid("table content must be an array".to_string())
        })?;

    if rows.is_empty() {
        return Err(AdfError::TableColumnInvalid(
            "cannot insert column into empty table".to_string(),
        ));
    }

    let expected_cols = rows
        .first()
        .and_then(|row| row.get("content"))
        .and_then(Value::as_array)
        .map(|cells| cells.len())
        .filter(|count| *count > 0)
        .ok_or_else(|| {
            AdfError::TableColumnInvalid("table rows must contain at least one cell".to_string())
        })?;

    if index > expected_cols {
        return Err(AdfError::TableColumnInvalid(format!(
            "column index {index} out of bounds for table with {expected_cols} columns"
        )));
    }

    let mut operations = Vec::with_capacity(rows.len());
    for (row_index, row) in rows.iter().enumerate() {
        let row_cells = row
            .get("content")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                AdfError::TableColumnInvalid(format!(
                    "table row {row_index} is missing a content array"
                ))
            })?;
        if row_cells.len() != expected_cols {
            return Err(AdfError::TableColumnInvalid(format!(
                "table row {row_index} has {} cells; expected {expected_cols}",
                row_cells.len()
            )));
        }

        let is_header_row = row_cells
            .first()
            .and_then(|cell| cell.get("type"))
            .and_then(Value::as_str)
            == Some("tableHeader");
        let block = if is_header_row {
            build_table_header("")
        } else {
            build_table_cell("")
        };

        operations.push(Operation::Insert {
            parent_path: format!("{table_path}/content/{row_index}/content"),
            index,
            block,
        });
    }

    Ok(operations)
}

fn translate_remove_column(
    table_path: &str,
    index: usize,
    allowed_scope_paths: &[String],
    scoped_adf: &Value,
) -> Result<Vec<Operation>, AdfError> {
    if !is_within_allowed_scope(table_path, allowed_scope_paths) {
        return Err(AdfError::OutOfScope(table_path.to_string()));
    }

    let table = scoped_adf.pointer(table_path).ok_or_else(|| {
        AdfError::TableColumnInvalid(format!("table path `{table_path}` does not resolve"))
    })?;
    if table.get("type").and_then(Value::as_str) != Some("table") {
        return Err(AdfError::TableColumnInvalid(format!(
            "target `{table_path}` is not a table"
        )));
    }

    let rows = table
        .get("content")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            AdfError::TableColumnInvalid("table content must be an array".to_string())
        })?;

    if rows.is_empty() {
        return Err(AdfError::TableColumnInvalid(
            "cannot remove column from empty table".to_string(),
        ));
    }

    let expected_cols = rows
        .first()
        .and_then(|row| row.get("content"))
        .and_then(Value::as_array)
        .map(|cells| cells.len())
        .filter(|count| *count > 0)
        .ok_or_else(|| {
            AdfError::TableColumnInvalid("table rows must contain at least one cell".to_string())
        })?;

    if expected_cols <= 1 {
        return Err(AdfError::TableColumnInvalid(
            "cannot remove last remaining column".to_string(),
        ));
    }
    if index >= expected_cols {
        return Err(AdfError::TableColumnInvalid(format!(
            "column index {index} out of bounds for table with {expected_cols} columns"
        )));
    }

    let mut operations = Vec::with_capacity(rows.len());
    for (row_index, row) in rows.iter().enumerate() {
        let row_cells = row
            .get("content")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                AdfError::TableColumnInvalid(format!(
                    "table row {row_index} is missing a content array"
                ))
            })?;
        if row_cells.len() != expected_cols {
            return Err(AdfError::TableColumnInvalid(format!(
                "table row {row_index} has {} cells; expected {expected_cols}",
                row_cells.len()
            )));
        }

        operations.push(Operation::Remove {
            target_path: format!("{table_path}/content/{row_index}/content/{index}"),
        });
    }

    Ok(operations)
}

fn translate_update_attrs(
    target_path: &str,
    attrs: &Value,
    allowed_scope_paths: &[String],
    scoped_adf: &Value,
) -> Result<Operation, AdfError> {
    if !is_within_allowed_scope(target_path, allowed_scope_paths) {
        return Err(AdfError::OutOfScope(target_path.to_string()));
    }
    if !attrs.is_object() {
        return Err(AdfError::AttrSchemaViolation(format!(
            "attrs payload for `{target_path}` must be an object"
        )));
    }

    let target = scoped_adf.pointer(target_path).ok_or_else(|| {
        AdfError::AttrUpdateBlocked(format!("target path `{target_path}` does not resolve"))
    })?;
    let node_type = target.get("type").and_then(Value::as_str).ok_or_else(|| {
        AdfError::AttrUpdateBlocked(format!("target `{target_path}` is missing node type"))
    })?;
    if !is_attr_editable_type(node_type) {
        return Err(AdfError::AttrUpdateBlocked(format!(
            "node type `{node_type}` at `{target_path}` is not attr-editable"
        )));
    }

    Ok(Operation::UpdateAttrs {
        target_path: target_path.to_string(),
        attrs: attrs.clone(),
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
