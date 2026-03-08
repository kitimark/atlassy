use std::collections::BTreeMap;

use serde_json::Value;

use crate::AdfError;
use crate::path::{escape_pointer_segment, parent_path};

pub fn build_node_path_index(adf: &Value) -> Result<BTreeMap<String, String>, AdfError> {
    let mut index = BTreeMap::new();
    build_node_path_index_inner(adf, String::new(), &mut index)?;
    Ok(index)
}

fn build_node_path_index_inner(
    value: &Value,
    path: String,
    index: &mut BTreeMap<String, String>,
) -> Result<(), AdfError> {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(node_type)) = map.get("type") {
                let effective_path = if path.is_empty() {
                    "/".to_string()
                } else {
                    path.clone()
                };
                if index
                    .insert(effective_path.clone(), node_type.clone())
                    .is_some()
                {
                    return Err(AdfError::DuplicatePath(effective_path));
                }
            }
            for (key, child) in map {
                let child_path = format!("{}/{}", path, escape_pointer_segment(key));
                build_node_path_index_inner(child, child_path, index)?;
            }
        }
        Value::Array(list) => {
            for (idx, child) in list.iter().enumerate() {
                let child_path = format!("{}/{}", path, idx);
                build_node_path_index_inner(child, child_path, index)?;
            }
        }
        _ => {}
    }
    Ok(())
}

pub fn path_has_ancestor_type(
    path: &str,
    node_path_index: &BTreeMap<String, String>,
    candidate_types: &[&str],
) -> bool {
    let mut current = path.to_string();
    while let Some(parent) = parent_path(&current) {
        if let Some(node_type) = node_path_index.get(&parent)
            && candidate_types
                .iter()
                .any(|candidate| *candidate == node_type)
        {
            return true;
        }
        current = parent;
    }
    false
}

pub(crate) fn collect_text(value: &Value) -> String {
    let mut text = String::new();
    if let Value::Object(map) = value {
        if map.get("type") == Some(&Value::String("text".to_string()))
            && let Some(Value::String(content)) = map.get("text")
        {
            text.push_str(content);
        }
        for child in map.values() {
            text.push_str(&collect_text(child));
        }
    } else if let Value::Array(items) = value {
        for child in items {
            text.push_str(&collect_text(child));
        }
    }
    text
}
