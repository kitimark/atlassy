use serde_json::Value;

use crate::AdfError;

pub fn check_structural_validity(adf: &Value) -> Result<(), AdfError> {
    let content = adf
        .pointer("/content")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            AdfError::PostMutationInvalid("doc.content must be a non-empty array".to_string())
        })?;

    if content.is_empty() {
        return Err(AdfError::PostMutationInvalid(
            "doc.content must be a non-empty array".to_string(),
        ));
    }

    for (index, node) in content.iter().enumerate() {
        let node_type = node.get("type").and_then(Value::as_str).ok_or_else(|| {
            AdfError::PostMutationInvalid(format!(
                "doc.content[{index}] is missing a string type field"
            ))
        })?;

        if node_type == "heading" {
            let level = node
                .get("attrs")
                .and_then(Value::as_object)
                .and_then(|attrs| attrs.get("level"))
                .and_then(Value::as_u64);
            if !level.is_some_and(|value| (1..=6).contains(&value)) {
                return Err(AdfError::PostMutationInvalid(format!(
                    "heading at doc.content[{index}] must have attrs.level in [1, 6]"
                )));
            }
        }

        walk_for_table_validation(node)?;
    }

    Ok(())
}

fn walk_for_table_validation(node: &Value) -> Result<(), AdfError> {
    if node.get("type").and_then(Value::as_str) == Some("table") {
        check_table_column_consistency(node)?;
    }

    match node {
        Value::Object(map) => {
            for child in map.values() {
                walk_for_table_validation(child)?;
            }
        }
        Value::Array(items) => {
            for child in items {
                walk_for_table_validation(child)?;
            }
        }
        _ => {}
    }

    Ok(())
}

fn check_table_column_consistency(table: &Value) -> Result<(), AdfError> {
    let rows = table
        .get("content")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            AdfError::PostMutationInvalid("table.content must be an array".to_string())
        })?;

    if rows.is_empty() {
        return Err(AdfError::PostMutationInvalid(
            "table.content must contain at least one row".to_string(),
        ));
    }

    let mut expected_cols: Option<usize> = None;
    for (row_index, row) in rows.iter().enumerate() {
        if row.get("type").and_then(Value::as_str) != Some("tableRow") {
            return Err(AdfError::PostMutationInvalid(format!(
                "table row at index {row_index} must have type `tableRow`"
            )));
        }

        let cells = row
            .get("content")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                AdfError::PostMutationInvalid(format!(
                    "table row at index {row_index} must have content array"
                ))
            })?;

        if cells.is_empty() {
            return Err(AdfError::PostMutationInvalid(format!(
                "table row at index {row_index} must contain at least one cell"
            )));
        }

        match expected_cols {
            Some(expected) if expected != cells.len() => {
                return Err(AdfError::PostMutationInvalid(format!(
                    "table row at index {row_index} has {} cells; expected {expected}",
                    cells.len()
                )));
            }
            None => expected_cols = Some(cells.len()),
            _ => {}
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn check_structural_validity_accepts_valid_document() {
        let adf = json!({
            "type": "doc",
            "content": [
                {
                    "type": "heading",
                    "attrs": {"level": 2},
                    "content": [{"type": "text", "text": "Overview"}]
                },
                {
                    "type": "paragraph",
                    "content": [{"type": "text", "text": "Body"}]
                }
            ]
        });

        assert!(check_structural_validity(&adf).is_ok());
    }

    #[test]
    fn check_structural_validity_rejects_empty_content() {
        let adf = json!({"type": "doc", "content": []});
        assert!(matches!(
            check_structural_validity(&adf),
            Err(AdfError::PostMutationInvalid(_))
        ));
    }

    #[test]
    fn check_structural_validity_rejects_missing_type() {
        let adf = json!({
            "type": "doc",
            "content": [
                {
                    "content": [{"type": "text", "text": "Body"}]
                }
            ]
        });
        assert!(matches!(
            check_structural_validity(&adf),
            Err(AdfError::PostMutationInvalid(_))
        ));
    }

    #[test]
    fn check_structural_validity_rejects_bad_heading_level() {
        let adf = json!({
            "type": "doc",
            "content": [
                {
                    "type": "heading",
                    "attrs": {"level": 7},
                    "content": [{"type": "text", "text": "Oops"}]
                }
            ]
        });

        assert!(matches!(
            check_structural_validity(&adf),
            Err(AdfError::PostMutationInvalid(_))
        ));
    }

    #[test]
    fn check_structural_validity_accepts_consistent_table_column_counts() {
        let adf = json!({
            "type": "doc",
            "content": [
                {
                    "type": "table",
                    "content": [
                        {
                            "type": "tableRow",
                            "content": [
                                {"type": "tableHeader", "content": []},
                                {"type": "tableHeader", "content": []}
                            ]
                        },
                        {
                            "type": "tableRow",
                            "content": [
                                {"type": "tableCell", "content": []},
                                {"type": "tableCell", "content": []}
                            ]
                        }
                    ]
                }
            ]
        });

        assert!(check_structural_validity(&adf).is_ok());
    }

    #[test]
    fn check_structural_validity_rejects_inconsistent_table_column_counts() {
        let adf = json!({
            "type": "doc",
            "content": [
                {
                    "type": "table",
                    "content": [
                        {
                            "type": "tableRow",
                            "content": [
                                {"type": "tableHeader", "content": []},
                                {"type": "tableHeader", "content": []}
                            ]
                        },
                        {
                            "type": "tableRow",
                            "content": [
                                {"type": "tableCell", "content": []}
                            ]
                        }
                    ]
                }
            ]
        });

        assert!(matches!(
            check_structural_validity(&adf),
            Err(AdfError::PostMutationInvalid(_))
        ));
    }

    #[test]
    fn check_structural_validity_rejects_empty_table() {
        let adf = json!({
            "type": "doc",
            "content": [
                {
                    "type": "table",
                    "content": []
                }
            ]
        });

        assert!(matches!(
            check_structural_validity(&adf),
            Err(AdfError::PostMutationInvalid(_))
        ));
    }
}
