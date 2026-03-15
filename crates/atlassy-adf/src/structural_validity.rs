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
}
