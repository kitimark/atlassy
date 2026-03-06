use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeResolution {
    pub scoped_adf: Value,
    pub allowed_scope_paths: Vec<String>,
    pub node_path_index: BTreeMap<String, String>,
    pub scope_resolution_failed: bool,
    pub full_page_fetch: bool,
    pub fallback_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchCandidate {
    pub path: String,
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchOperation {
    pub op: String,
    pub path: String,
    pub value: Value,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AdfError {
    #[error("scope resolution failed")]
    ScopeResolutionFailed,
    #[error("invalid selector format: {0}")]
    InvalidSelector(String),
    #[error("invalid JSON pointer path: {0}")]
    InvalidPath(String),
    #[error("duplicate path in node index: {0}")]
    DuplicatePath(String),
    #[error("whole-body rewrite is not allowed")]
    WholeBodyRewriteDisallowed,
    #[error("path `{0}` is outside allowed scope")]
    OutOfScope(String),
}

pub fn resolve_scope(adf: &Value, selectors: &[String]) -> Result<ScopeResolution, AdfError> {
    if selectors.is_empty() {
        return full_page_resolution(adf, Some("no_scope_selectors".to_string()));
    }

    let mut matched_paths = Vec::new();
    for selector in selectors {
        let (kind, value) = selector
            .split_once(':')
            .ok_or_else(|| AdfError::InvalidSelector(selector.clone()))?;
        match kind {
            "heading" => matched_paths.extend(find_heading_paths(adf, value, String::new())),
            "block" => matched_paths.extend(find_block_paths(adf, value, String::new())),
            _ => return Err(AdfError::InvalidSelector(selector.clone())),
        }
    }

    if matched_paths.is_empty() {
        return full_page_resolution(adf, Some("scope_selector_not_found".to_string()));
    }

    matched_paths.sort();
    matched_paths.dedup();

    let scoped = if matched_paths.len() == 1 {
        pointer_get(adf, &matched_paths[0])
            .cloned()
            .unwrap_or_else(|| adf.clone())
    } else {
        let mut nodes = Vec::new();
        for path in &matched_paths {
            if let Some(value) = pointer_get(adf, path) {
                nodes.push(value.clone());
            }
        }
        serde_json::json!({ "type": "doc", "content": nodes })
    };

    let node_path_index = build_node_path_index(&scoped)?;
    Ok(ScopeResolution {
        scoped_adf: scoped,
        allowed_scope_paths: matched_paths,
        node_path_index,
        scope_resolution_failed: false,
        full_page_fetch: false,
        fallback_reason: None,
    })
}

pub fn build_node_path_index(adf: &Value) -> Result<BTreeMap<String, String>, AdfError> {
    let mut index = BTreeMap::new();
    build_node_path_index_inner(adf, String::new(), &mut index)?;
    Ok(index)
}

pub fn normalize_changed_paths(paths: &[String]) -> Result<Vec<String>, AdfError> {
    let mut unique = BTreeSet::new();
    for path in paths {
        if !is_json_pointer(path) {
            return Err(AdfError::InvalidPath(path.clone()));
        }
        unique.insert(path.clone());
    }
    Ok(unique.into_iter().collect())
}

pub fn build_patch_ops(
    candidates: &[PatchCandidate],
    allowed_scope_paths: &[String],
) -> Result<Vec<PatchOperation>, AdfError> {
    let mut ops = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        if candidate.path == "/" || candidate.path.is_empty() {
            return Err(AdfError::WholeBodyRewriteDisallowed);
        }
        if !is_within_allowed_scope(&candidate.path, allowed_scope_paths) {
            return Err(AdfError::OutOfScope(candidate.path.clone()));
        }
        ops.push(PatchOperation {
            op: "replace".to_string(),
            path: candidate.path.clone(),
            value: candidate.value.clone(),
        });
    }
    Ok(ops)
}

pub fn ensure_paths_in_scope(
    paths: &[String],
    allowed_scope_paths: &[String],
) -> Result<(), AdfError> {
    for path in paths {
        if !is_within_allowed_scope(path, allowed_scope_paths) {
            return Err(AdfError::OutOfScope(path.clone()));
        }
    }
    Ok(())
}

fn full_page_resolution(adf: &Value, reason: Option<String>) -> Result<ScopeResolution, AdfError> {
    let node_path_index = build_node_path_index(adf)?;
    Ok(ScopeResolution {
        scoped_adf: adf.clone(),
        allowed_scope_paths: vec!["/".to_string()],
        node_path_index,
        scope_resolution_failed: true,
        full_page_fetch: true,
        fallback_reason: reason,
    })
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

fn find_heading_paths(value: &Value, heading_text: &str, path: String) -> Vec<String> {
    let mut matches = Vec::new();
    if let Value::Object(map) = value {
        if map.get("type") == Some(&Value::String("heading".to_string())) {
            let text = collect_text(value);
            if text.contains(heading_text) {
                matches.push(if path.is_empty() {
                    "/".to_string()
                } else {
                    path.clone()
                });
            }
        }
        for (key, child) in map {
            let child_path = format!("{}/{}", path, escape_pointer_segment(key));
            matches.extend(find_heading_paths(child, heading_text, child_path));
        }
    } else if let Value::Array(items) = value {
        for (idx, child) in items.iter().enumerate() {
            let child_path = format!("{}/{}", path, idx);
            matches.extend(find_heading_paths(child, heading_text, child_path));
        }
    }
    matches
}

fn find_block_paths(value: &Value, block_id: &str, path: String) -> Vec<String> {
    let mut matches = Vec::new();
    if let Value::Object(map) = value {
        if let Some(Value::Object(attrs)) = map.get("attrs") {
            let has_match = attrs
                .get("id")
                .and_then(Value::as_str)
                .is_some_and(|id| id == block_id)
                || attrs
                    .get("localId")
                    .and_then(Value::as_str)
                    .is_some_and(|id| id == block_id);
            if has_match {
                matches.push(if path.is_empty() {
                    "/".to_string()
                } else {
                    path.clone()
                });
            }
        }
        for (key, child) in map {
            let child_path = format!("{}/{}", path, escape_pointer_segment(key));
            matches.extend(find_block_paths(child, block_id, child_path));
        }
    } else if let Value::Array(items) = value {
        for (idx, child) in items.iter().enumerate() {
            let child_path = format!("{}/{}", path, idx);
            matches.extend(find_block_paths(child, block_id, child_path));
        }
    }
    matches
}

fn collect_text(value: &Value) -> String {
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

fn is_within_allowed_scope(path: &str, allowed_scope_paths: &[String]) -> bool {
    allowed_scope_paths.iter().any(|allowed| {
        if allowed == "/" {
            return true;
        }
        path == allowed
            || path
                .strip_prefix(allowed)
                .is_some_and(|suffix| suffix.starts_with('/'))
    })
}

fn is_json_pointer(path: &str) -> bool {
    path.starts_with('/')
}

fn pointer_get<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    if path == "/" {
        return Some(value);
    }
    value.pointer(path)
}

fn escape_pointer_segment(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_heading_scope() {
        let adf = serde_json::json!({
            "type": "doc",
            "content": [
                {"type": "heading", "content": [{"type":"text", "text":"Overview"}]},
                {"type": "paragraph", "content": [{"type":"text", "text":"Body"}]}
            ]
        });

        let resolution = resolve_scope(&adf, &["heading:Overview".to_string()]).unwrap();
        assert!(!resolution.scope_resolution_failed);
        assert_eq!(resolution.allowed_scope_paths, vec!["/content/0"]);
    }

    #[test]
    fn rejects_whole_body_patch() {
        let candidates = vec![PatchCandidate {
            path: "/".to_string(),
            value: serde_json::json!({}),
        }];

        let error = build_patch_ops(&candidates, &["/content/0".to_string()]).unwrap_err();
        assert_eq!(error, AdfError::WholeBodyRewriteDisallowed);
    }
}
