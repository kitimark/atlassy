use std::collections::BTreeMap;

use serde_json::Value;

use crate::index::{collect_text, path_has_ancestor_type};
use crate::path::{document_order_sort, is_json_pointer, is_within_allowed_scope};
use crate::{AdfError, EDITABLE_PROSE_TYPES, SCOPE_ANCHOR_TYPES, TargetRoute};

pub fn discover_target_path(
    node_path_index: &BTreeMap<String, String>,
    allowed_scope_paths: &[String],
    route: TargetRoute,
    target_index: usize,
) -> Result<String, AdfError> {
    let mut candidates = node_path_index
        .iter()
        .filter(|(_, node_type)| *node_type == "text")
        .map(|(path, _)| path)
        .filter(|path| is_within_allowed_scope(path, allowed_scope_paths))
        .filter(|path| {
            let in_table =
                path_has_ancestor_type(path, node_path_index, &["table", "tableRow", "tableCell"]);
            match route {
                TargetRoute::Prose => {
                    path_has_ancestor_type(path, node_path_index, EDITABLE_PROSE_TYPES)
                        && !path_has_ancestor_type(path, node_path_index, SCOPE_ANCHOR_TYPES)
                        && !in_table
                }
                TargetRoute::TableCell => in_table,
            }
        })
        .cloned()
        .collect::<Vec<_>>();

    document_order_sort(&mut candidates);
    let found = candidates.len();
    let selected = candidates
        .get(target_index)
        .ok_or_else(|| AdfError::TargetDiscoveryFailed {
            route: route.to_string(),
            index: target_index,
            found,
        })?;

    Ok(format!("{selected}/text"))
}

pub fn is_table_cell_text_path(path: &str, node_path_index: &BTreeMap<String, String>) -> bool {
    path.ends_with("/text")
        && path_has_ancestor_type(path, node_path_index, &["table", "tableRow", "tableCell"])
}

pub fn is_table_shape_or_attr_path(path: &str, node_path_index: &BTreeMap<String, String>) -> bool {
    if !path_has_ancestor_type(path, node_path_index, &["table", "tableRow", "tableCell"]) {
        return false;
    }

    if path.contains("/attrs") {
        return true;
    }

    !path.ends_with("/text")
}

pub fn markdown_for_path(adf: &Value, path: &str) -> Result<String, AdfError> {
    if !is_json_pointer(path) {
        return Err(AdfError::InvalidPath(path.to_string()));
    }

    let node = if path == "/" {
        adf
    } else {
        adf.pointer(path)
            .ok_or_else(|| AdfError::MappingIntegrity(format!("path `{path}` does not resolve")))?
    };

    Ok(collect_text(node))
}
