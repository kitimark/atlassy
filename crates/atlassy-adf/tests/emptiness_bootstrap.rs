use atlassy_adf::*;

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

    for node in content {
        let node_type = node["type"].as_str().unwrap();
        assert!(
            matches!(node_type, "heading" | "paragraph"),
            "unexpected node type in scaffold: {node_type}"
        );
    }
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
