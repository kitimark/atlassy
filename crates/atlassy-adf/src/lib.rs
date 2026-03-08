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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetRoute {
    Prose,
    TableCell,
}

impl std::fmt::Display for TargetRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            TargetRoute::Prose => "prose",
            TargetRoute::TableCell => "table_cell",
        };
        f.write_str(value)
    }
}

pub const EDITABLE_PROSE_TYPES: &[&str] = &[
    "paragraph",
    "heading",
    "bulletList",
    "orderedList",
    "listItem",
    "blockquote",
    "codeBlock",
];

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
    #[error("mapping integrity failure: {0}")]
    MappingIntegrity(String),
    #[error("no valid {route} target found in scope at index {index} (found {found})")]
    TargetDiscoveryFailed {
        route: String,
        index: usize,
        found: usize,
    },
}

pub fn resolve_scope(adf: &Value, selectors: &[String]) -> Result<ScopeResolution, AdfError> {
    if selectors.is_empty() {
        return full_page_resolution(adf, Some("no_scope_selectors".to_string()));
    }

    let mut matched_paths = Vec::new();
    let mut matched_heading_count = 0usize;
    let mut expanded_heading_found = false;
    for selector in selectors {
        let (kind, value) = selector
            .split_once(':')
            .ok_or_else(|| AdfError::InvalidSelector(selector.clone()))?;
        match kind {
            "heading" => {
                for heading_path in find_heading_paths(adf, value, String::new()) {
                    matched_heading_count += 1;
                    let expanded = expand_heading_to_section(adf, &heading_path);
                    if expanded.is_empty() {
                        continue;
                    }
                    expanded_heading_found = true;
                    matched_paths.extend(expanded);
                }
            }
            "block" => matched_paths.extend(find_block_paths(adf, value, String::new())),
            _ => return Err(AdfError::InvalidSelector(selector.clone())),
        }
    }

    if matched_heading_count > 0 && !expanded_heading_found && matched_paths.is_empty() {
        return full_page_resolution(adf, Some("nested_heading_scope_unsupported".to_string()));
    }

    if matched_paths.is_empty() {
        return full_page_resolution(adf, Some("scope_selector_not_found".to_string()));
    }

    matched_paths.sort();
    matched_paths.dedup();

    let node_path_index = build_node_path_index(adf)?;
    Ok(ScopeResolution {
        scoped_adf: adf.clone(),
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

pub fn apply_patch_ops(base: &Value, patch_ops: &[PatchOperation]) -> Result<Value, AdfError> {
    let mut candidate = base.clone();
    for op in patch_ops {
        if op.op != "replace" {
            return Err(AdfError::MappingIntegrity(format!(
                "unsupported patch operation `{}`",
                op.op
            )));
        }
        if op.path == "/" || op.path.is_empty() {
            return Err(AdfError::WholeBodyRewriteDisallowed);
        }
        let target = candidate.pointer_mut(&op.path).ok_or_else(|| {
            AdfError::MappingIntegrity(format!("path `{}` does not resolve", op.path))
        })?;
        *target = op.value.clone();
    }
    Ok(candidate)
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

pub fn canonicalize_mapped_path(
    path: &str,
    allowed_scope_paths: &[String],
) -> Result<String, AdfError> {
    if !is_json_pointer(path) {
        return Err(AdfError::InvalidPath(path.to_string()));
    }

    if allowed_scope_paths.iter().any(|allowed| allowed == "/") {
        return Ok(path.to_string());
    }

    if is_within_allowed_scope(path, allowed_scope_paths) {
        return Ok(path.to_string());
    }

    if allowed_scope_paths.len() == 1 {
        let root = allowed_scope_paths[0].trim_end_matches('/');
        if path == "/" {
            return Ok(root.to_string());
        }
        let tail = path.trim_start_matches('/');
        let canonical = format!("{root}/{tail}");
        if is_within_allowed_scope(&canonical, allowed_scope_paths) {
            return Ok(canonical);
        }
    }

    Err(AdfError::OutOfScope(path.to_string()))
}

pub fn is_path_within_or_descendant(path: &str, mapped_path: &str) -> bool {
    path == mapped_path
        || path
            .strip_prefix(mapped_path)
            .is_some_and(|suffix| suffix.starts_with('/'))
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
                    path_has_ancestor_type(path, node_path_index, EDITABLE_PROSE_TYPES) && !in_table
                }
                TargetRoute::TableCell => in_table,
            }
        })
        .cloned()
        .collect::<Vec<_>>();

    candidates.sort();
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

pub fn is_page_effectively_empty(adf: &Value) -> bool {
    let content = match adf.get("content") {
        Some(Value::Array(arr)) => arr,
        _ => return true,
    };

    if content.is_empty() {
        return true;
    }

    content.iter().all(|node| {
        let node_type = node.get("type").and_then(Value::as_str).unwrap_or_default();

        if node_type != "paragraph" {
            return false;
        }

        match node.get("content") {
            None => true,
            Some(Value::Array(children)) if children.is_empty() => true,
            Some(Value::Array(children)) => children.iter().all(|child| {
                let child_type = child
                    .get("type")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                if child_type != "text" {
                    return false;
                }
                match child.get("text").and_then(Value::as_str) {
                    None => true,
                    Some(text) => text.is_empty(),
                }
            }),
            _ => false,
        }
    })
}

pub fn bootstrap_scaffold() -> Value {
    serde_json::json!({
        "type": "doc",
        "version": 1,
        "content": [
            {
                "type": "heading",
                "attrs": {"level": 2},
                "content": [{"type": "text", "text": ""}]
            },
            {
                "type": "paragraph",
                "content": [{"type": "text", "text": ""}]
            }
        ]
    })
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

fn heading_level(node: &Value) -> u8 {
    node.get("attrs")
        .and_then(Value::as_object)
        .and_then(|attrs| attrs.get("level"))
        .and_then(Value::as_u64)
        .and_then(|level| u8::try_from(level).ok())
        .filter(|level| (1..=6).contains(level))
        .unwrap_or(6)
}

fn expand_heading_to_section(adf: &Value, heading_path: &str) -> Vec<String> {
    let Some(index_str) = heading_path.strip_prefix("/content/") else {
        return Vec::new();
    };
    if index_str.is_empty() || index_str.contains('/') {
        return Vec::new();
    }

    let Ok(start_index) = index_str.parse::<usize>() else {
        return Vec::new();
    };

    let Some(content) = adf.get("content").and_then(Value::as_array) else {
        return Vec::new();
    };
    let Some(heading_node) = content.get(start_index) else {
        return Vec::new();
    };
    if heading_node.get("type").and_then(Value::as_str) != Some("heading") {
        return Vec::new();
    }

    let base_level = heading_level(heading_node);
    let mut section_paths = vec![heading_path.to_string()];
    for (index, node) in content.iter().enumerate().skip(start_index + 1) {
        if node.get("type").and_then(Value::as_str) == Some("heading")
            && heading_level(node) <= base_level
        {
            break;
        }
        section_paths.push(format!("/content/{index}"));
    }

    section_paths
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
            if text == heading_text {
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

pub fn is_within_allowed_scope(path: &str, allowed_scope_paths: &[String]) -> bool {
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

fn escape_pointer_segment(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

fn parent_path(path: &str) -> Option<String> {
    if path == "/" {
        return None;
    }
    let (parent, _) = path.rsplit_once('/')?;
    if parent.is_empty() {
        return Some("/".to_string());
    }
    Some(parent.to_string())
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
        assert_eq!(
            resolution.allowed_scope_paths,
            vec!["/content/0", "/content/1"]
        );
        assert_eq!(resolution.scoped_adf, adf);
    }

    #[test]
    fn heading_selector_requires_exact_match() {
        let adf = serde_json::json!({
            "type": "doc",
            "content": [
                {"type": "heading", "content": [{"type":"text", "text":"Overview"}]},
                {"type": "paragraph", "content": [{"type":"text", "text":"Body"}]}
            ]
        });

        let resolution = resolve_scope(&adf, &["heading:View".to_string()]).unwrap();
        assert!(resolution.scope_resolution_failed);
        assert!(resolution.full_page_fetch);
        assert_eq!(resolution.allowed_scope_paths, vec!["/"]);
        assert!(
            resolution
                .fallback_reason
                .as_deref()
                .is_some_and(|reason| reason.contains("scope_selector_not_found"))
        );
    }

    #[test]
    fn heading_selector_exact_match_still_works() {
        let adf = serde_json::json!({
            "type": "doc",
            "content": [
                {"type": "heading", "content": [{"type":"text", "text":"Overview"}]},
                {"type": "paragraph", "content": [{"type":"text", "text":"Body"}]}
            ]
        });

        let resolution = resolve_scope(&adf, &["heading:Overview".to_string()]).unwrap();
        assert!(!resolution.scope_resolution_failed);
        assert!(!resolution.full_page_fetch);
        assert_eq!(
            resolution.allowed_scope_paths,
            vec!["/content/0", "/content/1"]
        );
    }

    #[test]
    fn duplicate_heading_text_matches_all_sections() {
        let adf = serde_json::json!({
            "type": "doc",
            "content": [
                {"type": "heading", "attrs": {"level": 2}, "content": [{"type":"text", "text":"Notes"}]},
                {"type": "paragraph", "content": [{"type":"text", "text":"First section"}]},
                {"type": "heading", "attrs": {"level": 2}, "content": [{"type":"text", "text":"Notes"}]},
                {"type": "paragraph", "content": [{"type":"text", "text":"Second section"}]}
            ]
        });

        let resolution = resolve_scope(&adf, &["heading:Notes".to_string()]).unwrap();
        assert_eq!(
            resolution.allowed_scope_paths,
            vec!["/content/0", "/content/1", "/content/2", "/content/3"]
        );
    }

    #[test]
    fn resolves_block_scope_by_attrs_id() {
        let adf = serde_json::json!({
            "type": "doc",
            "content": [
                {
                    "type": "panel",
                    "attrs": {"id": "panel-1"},
                    "content": [
                        {"type": "paragraph", "content": [{"type":"text", "text":"Panel body"}]}
                    ]
                }
            ]
        });

        let resolution = resolve_scope(&adf, &["block:panel-1".to_string()]).unwrap();
        assert!(!resolution.scope_resolution_failed);
        assert!(
            resolution
                .allowed_scope_paths
                .contains(&"/content/0".to_string())
        );
    }

    #[test]
    fn resolves_block_scope_by_attrs_local_id() {
        let adf = serde_json::json!({
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "attrs": {"localId": "local-abc"},
                    "content": [{"type":"text", "text":"Body"}]
                }
            ]
        });

        let resolution = resolve_scope(&adf, &["block:local-abc".to_string()]).unwrap();
        assert!(!resolution.scope_resolution_failed);
        assert!(
            resolution
                .allowed_scope_paths
                .contains(&"/content/0".to_string())
        );
    }

    #[test]
    fn block_selector_falls_back_when_no_match() {
        let adf = serde_json::json!({
            "type": "doc",
            "content": [
                {"type": "paragraph", "content": [{"type":"text", "text":"Body"}]}
            ]
        });

        let resolution = resolve_scope(&adf, &["block:nonexistent".to_string()]).unwrap();
        assert!(resolution.scope_resolution_failed);
        assert!(resolution.full_page_fetch);
        assert_eq!(resolution.allowed_scope_paths, vec!["/"]);
        assert!(
            resolution
                .fallback_reason
                .as_deref()
                .is_some_and(|reason| reason.contains("scope_selector_not_found"))
        );
    }

    #[test]
    fn resolves_heading_scope_until_next_same_level_heading() {
        let adf = serde_json::json!({
            "type": "doc",
            "content": [
                {"type": "heading", "attrs": {"level": 2}, "content": [{"type":"text", "text":"Overview"}]},
                {"type": "paragraph", "content": [{"type":"text", "text":"Body A"}]},
                {"type": "paragraph", "content": [{"type":"text", "text":"Body B"}]},
                {"type": "heading", "attrs": {"level": 2}, "content": [{"type":"text", "text":"Details"}]},
                {"type": "paragraph", "content": [{"type":"text", "text":"Body C"}]}
            ]
        });

        let resolution = resolve_scope(&adf, &["heading:Overview".to_string()]).unwrap();
        assert_eq!(
            resolution.allowed_scope_paths,
            vec!["/content/0", "/content/1", "/content/2"]
        );
    }

    #[test]
    fn resolves_heading_scope_for_heading_at_end_of_content() {
        let adf = serde_json::json!({
            "type": "doc",
            "content": [
                {"type": "paragraph", "content": [{"type":"text", "text":"Body"}]},
                {"type": "heading", "attrs": {"level": 2}, "content": [{"type":"text", "text":"Overview"}]}
            ]
        });

        let resolution = resolve_scope(&adf, &["heading:Overview".to_string()]).unwrap();
        assert_eq!(resolution.allowed_scope_paths, vec!["/content/1"]);
    }

    #[test]
    fn resolves_adjacent_same_level_headings_to_single_path() {
        let adf = serde_json::json!({
            "type": "doc",
            "content": [
                {"type": "heading", "attrs": {"level": 2}, "content": [{"type":"text", "text":"A"}]},
                {"type": "heading", "attrs": {"level": 2}, "content": [{"type":"text", "text":"B"}]}
            ]
        });

        let resolution = resolve_scope(&adf, &["heading:A".to_string()]).unwrap();
        assert_eq!(resolution.allowed_scope_paths, vec!["/content/0"]);
    }

    #[test]
    fn includes_nested_subheading_content_in_parent_section() {
        let adf = serde_json::json!({
            "type": "doc",
            "content": [
                {"type": "heading", "attrs": {"level": 2}, "content": [{"type":"text", "text":"Overview"}]},
                {"type": "paragraph", "content": [{"type":"text", "text":"Body A"}]},
                {"type": "heading", "attrs": {"level": 3}, "content": [{"type":"text", "text":"Subsection"}]},
                {"type": "paragraph", "content": [{"type":"text", "text":"Body B"}]},
                {"type": "heading", "attrs": {"level": 2}, "content": [{"type":"text", "text":"Next"}]}
            ]
        });

        let resolution = resolve_scope(&adf, &["heading:Overview".to_string()]).unwrap();
        assert_eq!(
            resolution.allowed_scope_paths,
            vec!["/content/0", "/content/1", "/content/2", "/content/3"]
        );
    }

    #[test]
    fn h1_section_includes_nested_h2_and_h3_until_next_h1() {
        let adf = serde_json::json!({
            "type": "doc",
            "content": [
                {"type": "heading", "attrs": {"level": 1}, "content": [{"type":"text", "text":"Overview"}]},
                {"type": "paragraph", "content": [{"type":"text", "text":"Top"}]},
                {"type": "heading", "attrs": {"level": 2}, "content": [{"type":"text", "text":"Details"}]},
                {"type": "paragraph", "content": [{"type":"text", "text":"Middle"}]},
                {"type": "heading", "attrs": {"level": 3}, "content": [{"type":"text", "text":"Deep"}]},
                {"type": "paragraph", "content": [{"type":"text", "text":"Bottom"}]},
                {"type": "heading", "attrs": {"level": 1}, "content": [{"type":"text", "text":"Next"}]}
            ]
        });

        let resolution = resolve_scope(&adf, &["heading:Overview".to_string()]).unwrap();
        assert_eq!(
            resolution.allowed_scope_paths,
            vec![
                "/content/0",
                "/content/1",
                "/content/2",
                "/content/3",
                "/content/4",
                "/content/5"
            ]
        );
    }

    #[test]
    fn unions_multiple_heading_selectors_with_sorted_deduped_paths() {
        let adf = serde_json::json!({
            "type": "doc",
            "content": [
                {"type": "heading", "attrs": {"level": 2}, "content": [{"type":"text", "text":"Alpha"}]},
                {"type": "paragraph", "content": [{"type":"text", "text":"A"}]},
                {"type": "heading", "attrs": {"level": 2}, "content": [{"type":"text", "text":"Beta"}]},
                {"type": "paragraph", "content": [{"type":"text", "text":"B"}]}
            ]
        });

        let resolution = resolve_scope(
            &adf,
            &[
                "heading:Beta".to_string(),
                "heading:Alpha".to_string(),
                "heading:Alpha".to_string(),
            ],
        )
        .unwrap();
        assert_eq!(
            resolution.allowed_scope_paths,
            vec!["/content/0", "/content/1", "/content/2", "/content/3"]
        );
    }

    #[test]
    fn heading_without_level_defaults_to_six() {
        let adf = serde_json::json!({
            "type": "doc",
            "content": [
                {"type": "heading", "content": [{"type":"text", "text":"Overview"}]},
                {"type": "paragraph", "content": [{"type":"text", "text":"Body"}]},
                {"type": "heading", "attrs": {"level": 3}, "content": [{"type":"text", "text":"Details"}]},
                {"type": "paragraph", "content": [{"type":"text", "text":"More"}]}
            ]
        });

        let resolution = resolve_scope(&adf, &["heading:Overview".to_string()]).unwrap();
        assert_eq!(
            resolution.allowed_scope_paths,
            vec!["/content/0", "/content/1"]
        );
    }

    #[test]
    fn nested_heading_falls_back_to_full_page() {
        let adf = serde_json::json!({
            "type": "doc",
            "content": [
                {
                    "type": "panel",
                    "content": [
                        {"type": "heading", "attrs": {"level": 2}, "content": [{"type":"text", "text":"Overview"}]},
                        {"type": "paragraph", "content": [{"type":"text", "text":"Inside panel"}]}
                    ]
                },
                {"type": "paragraph", "content": [{"type":"text", "text":"Outside"}]}
            ]
        });

        let resolution = resolve_scope(&adf, &["heading:Overview".to_string()]).unwrap();
        assert!(resolution.scope_resolution_failed);
        assert!(resolution.full_page_fetch);
        assert_eq!(resolution.allowed_scope_paths, vec!["/"]);
        assert!(
            resolution
                .fallback_reason
                .as_deref()
                .is_some_and(|reason| reason.contains("nested_heading_scope_unsupported"))
        );
        assert_eq!(resolution.scoped_adf, adf);
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

    #[test]
    fn canonicalizes_relative_path_to_scope_root() {
        let canonical = canonicalize_mapped_path("/content/0", &["/body/1".to_string()]).unwrap();
        assert_eq!(canonical, "/body/1/content/0");

        let root = canonicalize_mapped_path("/", &["/body/1".to_string()]).unwrap();
        assert_eq!(root, "/body/1");
    }

    #[test]
    fn extracts_markdown_for_resolved_path() {
        let adf = serde_json::json!({
            "type": "doc",
            "content": [
                {"type": "paragraph", "content": [{"type": "text", "text": "Hello prose"}]}
            ]
        });

        let markdown = markdown_for_path(&adf, "/content/0").unwrap();
        assert_eq!(markdown, "Hello prose");
    }

    #[test]
    fn detects_out_of_scope_paths() {
        let error = ensure_paths_in_scope(
            &["/content/2/content/0/text".to_string()],
            &["/content/0".to_string(), "/content/1".to_string()],
        )
        .unwrap_err();
        assert_eq!(
            error,
            AdfError::OutOfScope("/content/2/content/0/text".to_string())
        );
    }

    #[test]
    fn detects_table_cell_text_paths() {
        let adf = serde_json::json!({
            "type": "doc",
            "content": [{
                "type": "table",
                "content": [{
                    "type": "tableRow",
                    "content": [{
                        "type": "tableCell",
                        "content": [{
                            "type": "paragraph",
                            "content": [{"type": "text", "text": "Cell"}]
                        }]
                    }]
                }]
            }]
        });

        let index = build_node_path_index(&adf).unwrap();
        assert!(is_table_cell_text_path(
            "/content/0/content/0/content/0/content/0/content/0/text",
            &index
        ));
        assert!(is_table_shape_or_attr_path("/content/0/content/0", &index));
    }

    #[test]
    fn discovers_first_prose_text_in_section() {
        let adf = serde_json::json!({
            "type": "doc",
            "content": [
                {"type": "heading", "content": [{"type": "text", "text": "Overview"}]},
                {"type": "paragraph", "content": [{"type": "text", "text": "First body"}]},
                {"type": "paragraph", "content": [{"type": "text", "text": "Second body"}]}
            ]
        });

        let index = build_node_path_index(&adf).unwrap();
        let path = discover_target_path(
            &index,
            &[
                "/content/0".to_string(),
                "/content/1".to_string(),
                "/content/2".to_string(),
            ],
            TargetRoute::Prose,
            0,
        )
        .unwrap();

        assert_eq!(path, "/content/0/content/0/text");
    }

    #[test]
    fn discovers_nth_prose_text_with_index() {
        let adf = serde_json::json!({
            "type": "doc",
            "content": [
                {"type": "heading", "content": [{"type": "text", "text": "Overview"}]},
                {"type": "paragraph", "content": [{"type": "text", "text": "First body"}]},
                {"type": "paragraph", "content": [{"type": "text", "text": "Second body"}]}
            ]
        });

        let index = build_node_path_index(&adf).unwrap();
        let path = discover_target_path(
            &index,
            &[
                "/content/0".to_string(),
                "/content/1".to_string(),
                "/content/2".to_string(),
            ],
            TargetRoute::Prose,
            1,
        )
        .unwrap();

        assert_eq!(path, "/content/1/content/0/text");
    }

    #[test]
    fn discovers_table_cell_text() {
        let adf = serde_json::json!({
            "type": "doc",
            "content": [
                {"type": "heading", "content": [{"type": "text", "text": "Overview"}]},
                {
                    "type": "table",
                    "content": [{
                        "type": "tableRow",
                        "content": [{
                            "type": "tableCell",
                            "content": [{
                                "type": "paragraph",
                                "content": [{"type": "text", "text": "Cell"}]
                            }]
                        }]
                    }]
                }
            ]
        });

        let index = build_node_path_index(&adf).unwrap();
        let path = discover_target_path(
            &index,
            &["/content/0".to_string(), "/content/1".to_string()],
            TargetRoute::TableCell,
            0,
        )
        .unwrap();

        assert_eq!(
            path,
            "/content/1/content/0/content/0/content/0/content/0/text"
        );
    }

    #[test]
    fn discovery_respects_scope_boundary() {
        let adf = serde_json::json!({
            "type": "doc",
            "content": [
                {"type": "heading", "content": [{"type": "text", "text": "Overview"}]},
                {"type": "paragraph", "content": [{"type": "text", "text": "In scope"}]},
                {"type": "heading", "content": [{"type": "text", "text": "Outside"}]},
                {"type": "paragraph", "content": [{"type": "text", "text": "Out of scope"}]}
            ]
        });

        let index = build_node_path_index(&adf).unwrap();
        let path = discover_target_path(
            &index,
            &["/content/2".to_string(), "/content/3".to_string()],
            TargetRoute::Prose,
            0,
        )
        .unwrap();

        assert_eq!(path, "/content/2/content/0/text");
    }

    #[test]
    fn discovery_fails_on_empty_section() {
        let adf = serde_json::json!({
            "type": "doc",
            "content": [
                {"type": "heading", "attrs": {"level": 2}},
                {"type": "heading", "attrs": {"level": 2}, "content": [{"type": "text", "text": "Next"}]}
            ]
        });

        let index = build_node_path_index(&adf).unwrap();
        let error =
            discover_target_path(&index, &["/content/0".to_string()], TargetRoute::Prose, 0)
                .unwrap_err();

        assert_eq!(
            error,
            AdfError::TargetDiscoveryFailed {
                route: "prose".to_string(),
                index: 0,
                found: 0,
            }
        );
    }

    #[test]
    fn discovery_fails_on_out_of_bounds_index() {
        let adf = serde_json::json!({
            "type": "doc",
            "content": [
                {"type": "heading", "content": [{"type": "text", "text": "Overview"}]},
                {"type": "paragraph", "content": [{"type": "text", "text": "In scope"}]}
            ]
        });

        let index = build_node_path_index(&adf).unwrap();
        let error = discover_target_path(
            &index,
            &["/content/0".to_string(), "/content/1".to_string()],
            TargetRoute::Prose,
            5,
        )
        .unwrap_err();

        assert_eq!(
            error,
            AdfError::TargetDiscoveryFailed {
                route: "prose".to_string(),
                index: 5,
                found: 2,
            }
        );
    }

    #[test]
    fn applies_patch_ops_to_candidate_payload() {
        let base = serde_json::json!({
            "type": "doc",
            "content": [
                {"type": "paragraph", "content": [{"type": "text", "text": "before"}]}
            ]
        });

        let ops = vec![PatchOperation {
            op: "replace".to_string(),
            path: "/content/0/content/0/text".to_string(),
            value: serde_json::Value::String("after".to_string()),
        }];

        let patched = apply_patch_ops(&base, &ops).unwrap();
        assert_eq!(
            patched
                .pointer("/content/0/content/0/text")
                .and_then(Value::as_str),
            Some("after")
        );
    }

    #[test]
    fn empty_content_array_is_effectively_empty() {
        let adf = serde_json::json!({"type": "doc", "version": 1, "content": []});
        assert!(is_page_effectively_empty(&adf));
    }

    #[test]
    fn missing_content_is_effectively_empty() {
        let adf = serde_json::json!({"type": "doc", "version": 1});
        assert!(is_page_effectively_empty(&adf));
    }

    #[test]
    fn single_empty_paragraph_is_effectively_empty() {
        let adf = serde_json::json!({
            "type": "doc", "version": 1,
            "content": [{"type": "paragraph"}]
        });
        assert!(is_page_effectively_empty(&adf));
    }

    #[test]
    fn paragraph_with_empty_text_is_effectively_empty() {
        let adf = serde_json::json!({
            "type": "doc", "version": 1,
            "content": [{"type": "paragraph", "content": [{"type": "text", "text": ""}]}]
        });
        assert!(is_page_effectively_empty(&adf));
    }

    #[test]
    fn paragraph_with_local_id_but_no_text_is_effectively_empty() {
        let adf = serde_json::json!({
            "type": "doc", "version": 1,
            "content": [{
                "type": "paragraph",
                "attrs": {"localId": "abc123"},
                "content": []
            }]
        });
        assert!(is_page_effectively_empty(&adf));
    }

    #[test]
    fn paragraph_with_non_empty_text_is_not_empty() {
        let adf = serde_json::json!({
            "type": "doc", "version": 1,
            "content": [{"type": "paragraph", "content": [{"type": "text", "text": "Hello"}]}]
        });
        assert!(!is_page_effectively_empty(&adf));
    }

    #[test]
    fn heading_with_text_is_not_empty() {
        let adf = serde_json::json!({
            "type": "doc", "version": 1,
            "content": [{"type": "heading", "content": [{"type": "text", "text": "Title"}]}]
        });
        assert!(!is_page_effectively_empty(&adf));
    }

    #[test]
    fn table_node_is_not_empty() {
        let adf = serde_json::json!({
            "type": "doc", "version": 1,
            "content": [{"type": "table", "content": []}]
        });
        assert!(!is_page_effectively_empty(&adf));
    }

    #[test]
    fn panel_node_is_not_empty() {
        let adf = serde_json::json!({
            "type": "doc", "version": 1,
            "content": [{"type": "panel", "content": []}]
        });
        assert!(!is_page_effectively_empty(&adf));
    }

    #[test]
    fn bootstrap_scaffold_contains_only_prose_nodes() {
        let scaffold = bootstrap_scaffold();
        let content = scaffold["content"].as_array().unwrap();
        assert_eq!(content.len(), 2);

        assert_eq!(content[0]["type"], "heading");
        assert_eq!(content[0]["attrs"]["level"], 2);

        assert_eq!(content[1]["type"], "paragraph");

        // All nodes are editable_prose route types
        for node in content {
            let node_type = node["type"].as_str().unwrap();
            assert!(
                matches!(node_type, "heading" | "paragraph"),
                "unexpected node type in scaffold: {node_type}"
            );
        }
    }
}
