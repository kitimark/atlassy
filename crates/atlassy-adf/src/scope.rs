use serde_json::Value;

use crate::index::{build_node_path_index, collect_text};
use crate::path::{document_order_sort, escape_pointer_segment};
use crate::{AdfError, ScopeResolution};

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

    document_order_sort(&mut matched_paths);
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn heading_level_extracts_from_attrs() {
        let heading = json!({"type": "heading", "attrs": {"level": 2}});
        assert_eq!(heading_level(&heading), 2);
    }

    #[test]
    fn heading_level_defaults_to_six_when_attrs_missing() {
        let heading = json!({"type": "heading"});
        assert_eq!(heading_level(&heading), 6);
    }

    #[test]
    fn heading_level_defaults_to_six_when_out_of_range() {
        let heading = json!({"type": "heading", "attrs": {"level": 7}});
        assert_eq!(heading_level(&heading), 6);
    }

    #[test]
    fn find_heading_paths_matches_exact_heading_text() {
        let adf = json!({
            "type": "doc",
            "content": [
                {
                    "type": "heading",
                    "attrs": {"level": 2},
                    "content": [{"type": "text", "text": "Exact Match"}]
                },
                {
                    "type": "heading",
                    "attrs": {"level": 2},
                    "content": [{"type": "text", "text": "Exact Match Extra"}]
                }
            ]
        });

        assert_eq!(
            find_heading_paths(&adf, "Exact Match", String::new()),
            vec!["/content/0".to_string()]
        );
    }

    #[test]
    fn find_heading_paths_returns_empty_when_not_found() {
        let adf = json!({
            "type": "doc",
            "content": [
                {
                    "type": "heading",
                    "attrs": {"level": 2},
                    "content": [{"type": "text", "text": "Something Else"}]
                }
            ]
        });

        assert!(find_heading_paths(&adf, "Missing", String::new()).is_empty());
    }

    #[test]
    fn find_heading_paths_recurses_into_nested_content() {
        let adf = json!({
            "type": "doc",
            "content": [
                {
                    "type": "expand",
                    "content": [
                        {
                            "type": "heading",
                            "attrs": {"level": 3},
                            "content": [{"type": "text", "text": "Nested Heading"}]
                        }
                    ]
                }
            ]
        });

        assert_eq!(
            find_heading_paths(&adf, "Nested Heading", String::new()),
            vec!["/content/0/content/0".to_string()]
        );
    }

    #[test]
    fn find_block_paths_matches_attrs_id() {
        let adf = json!({
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "attrs": {"id": "block-1"},
                    "content": [{"type": "text", "text": "Body"}]
                }
            ]
        });

        assert_eq!(
            find_block_paths(&adf, "block-1", String::new()),
            vec!["/content/0".to_string()]
        );
    }

    #[test]
    fn find_block_paths_matches_attrs_local_id() {
        let adf = json!({
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "attrs": {"localId": "local-1"},
                    "content": [{"type": "text", "text": "Body"}]
                }
            ]
        });

        assert_eq!(
            find_block_paths(&adf, "local-1", String::new()),
            vec!["/content/0".to_string()]
        );
    }

    #[test]
    fn find_block_paths_returns_empty_when_not_found() {
        let adf = json!({
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "attrs": {"id": "other"},
                    "content": [{"type": "text", "text": "Body"}]
                }
            ]
        });

        assert!(find_block_paths(&adf, "missing", String::new()).is_empty());
    }

    #[test]
    fn expand_heading_to_section_includes_until_same_level_heading() {
        let adf = json!({
            "type": "doc",
            "content": [
                {
                    "type": "heading",
                    "attrs": {"level": 2},
                    "content": [{"type": "text", "text": "Section"}]
                },
                {"type": "paragraph", "content": [{"type": "text", "text": "A"}]},
                {
                    "type": "heading",
                    "attrs": {"level": 3},
                    "content": [{"type": "text", "text": "Subsection"}]
                },
                {"type": "paragraph", "content": [{"type": "text", "text": "B"}]},
                {
                    "type": "heading",
                    "attrs": {"level": 2},
                    "content": [{"type": "text", "text": "Next"}]
                }
            ]
        });

        assert_eq!(
            expand_heading_to_section(&adf, "/content/0"),
            vec![
                "/content/0".to_string(),
                "/content/1".to_string(),
                "/content/2".to_string(),
                "/content/3".to_string(),
            ]
        );
    }

    #[test]
    fn expand_heading_to_section_returns_empty_for_non_content_paths() {
        let adf = json!({"type": "doc", "content": []});
        assert!(expand_heading_to_section(&adf, "/attrs/title").is_empty());
    }

    #[test]
    fn expand_heading_to_section_returns_empty_for_nested_heading_paths() {
        let adf = json!({
            "type": "doc",
            "content": [
                {
                    "type": "heading",
                    "attrs": {"level": 2},
                    "content": [{"type": "text", "text": "Heading"}]
                }
            ]
        });

        assert!(expand_heading_to_section(&adf, "/content/0/content/0").is_empty());
    }

    #[test]
    fn full_page_resolution_sets_fallback_fields_and_reason() {
        let adf = json!({"type": "doc", "version": 1, "content": []});

        let resolution =
            full_page_resolution(&adf, Some("scope_selector_not_found".to_string())).unwrap();

        assert_eq!(resolution.scoped_adf, adf);
        assert_eq!(resolution.allowed_scope_paths, vec!["/".to_string()]);
        assert!(resolution.scope_resolution_failed);
        assert!(resolution.full_page_fetch);
        assert_eq!(
            resolution.fallback_reason,
            Some("scope_selector_not_found".to_string())
        );
        assert_eq!(
            resolution.node_path_index.get("/"),
            Some(&"doc".to_string())
        );
    }
}
