use serde_json::{json, Value};

use crate::AdfError;

pub fn build_text(text: &str) -> Value {
    json!({
        "type": "text",
        "text": text,
    })
}

pub fn build_paragraph(text: &str) -> Value {
    json!({
        "type": "paragraph",
        "content": [build_text(text)],
    })
}

pub fn build_heading(level: u8, text: &str) -> Result<Value, AdfError> {
    if !(1..=6).contains(&level) {
        return Err(AdfError::StructuralCompositionFailed(format!(
            "heading level must be in [1, 6], got {level}"
        )));
    }

    Ok(json!({
        "type": "heading",
        "attrs": {
            "level": level,
        },
        "content": [build_text(text)],
    }))
}

pub fn build_table(rows: usize, cols: usize, header_row: bool) -> Result<Value, AdfError> {
    if rows == 0 || cols == 0 {
        return Err(AdfError::StructuralCompositionFailed(format!(
            "table dimensions must be non-zero (rows={rows}, cols={cols})"
        )));
    }

    let mut table_rows = Vec::with_capacity(rows);
    for row_index in 0..rows {
        let cell_type = if header_row && row_index == 0 {
            "tableHeader"
        } else {
            "tableCell"
        };
        let mut cells = Vec::with_capacity(cols);
        for _ in 0..cols {
            cells.push(json!({
                "type": cell_type,
                "content": [build_paragraph("")],
            }));
        }

        table_rows.push(json!({
            "type": "tableRow",
            "content": cells,
        }));
    }

    Ok(json!({
        "type": "table",
        "content": table_rows,
    }))
}

pub fn build_list(ordered: bool, items: &[&str]) -> Result<Value, AdfError> {
    if items.is_empty() {
        return Err(AdfError::StructuralCompositionFailed(
            "list items must be non-empty".to_string(),
        ));
    }

    let list_type = if ordered { "orderedList" } else { "bulletList" };
    let content = items
        .iter()
        .map(|item| {
            json!({
                "type": "listItem",
                "content": [build_paragraph(item)],
            })
        })
        .collect::<Vec<_>>();

    Ok(json!({
        "type": list_type,
        "content": content,
    }))
}

pub fn build_section(
    level: u8,
    heading_text: &str,
    body_blocks: &[Value],
) -> Result<Vec<Value>, AdfError> {
    let heading = build_heading(level, heading_text)?;
    let mut section = Vec::with_capacity(1 + body_blocks.len());
    section.push(heading);
    section.extend(body_blocks.iter().cloned());
    Ok(section)
}
