use atlassy_adf::*;

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
