use atlassy_adf::*;

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

    assert_eq!(path, "/content/1/content/0/text");
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

    assert_eq!(path, "/content/2/content/0/text");
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

    assert_eq!(path, "/content/3/content/0/text");
}

#[test]
fn discovery_excludes_heading_text_nodes() {
    let adf = serde_json::json!({
        "type": "doc",
        "content": [
            {"type": "heading", "content": [{"type": "text", "text": "Overview"}]},
            {"type": "paragraph", "content": [{"type": "text", "text": "First body"}]},
            {"type": "paragraph", "content": [{"type": "text", "text": "Second body"}]}
        ]
    });

    let index = build_node_path_index(&adf).unwrap();
    let error = discover_target_path(
        &index,
        &[
            "/content/0".to_string(),
            "/content/1".to_string(),
            "/content/2".to_string(),
        ],
        TargetRoute::Prose,
        2,
    )
    .unwrap_err();

    assert_eq!(
        error,
        AdfError::TargetDiscoveryFailed {
            route: "prose".to_string(),
            index: 2,
            found: 2,
        }
    );
}

#[test]
fn discovery_fails_for_heading_only_section() {
    let adf = serde_json::json!({
        "type": "doc",
        "content": [
            {"type": "heading", "content": [{"type": "text", "text": "Overview"}]},
            {"type": "paragraph", "content": [{"type": "text", "text": "Outside"}]}
        ]
    });

    let index = build_node_path_index(&adf).unwrap();
    let error = discover_target_path(&index, &["/content/0".to_string()], TargetRoute::Prose, 0)
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
fn discovery_fails_on_empty_section() {
    let adf = serde_json::json!({
        "type": "doc",
        "content": [
            {"type": "heading", "attrs": {"level": 2}},
            {"type": "heading", "attrs": {"level": 2}, "content": [{"type": "text", "text": "Next"}]}
        ]
    });

    let index = build_node_path_index(&adf).unwrap();
    let error = discover_target_path(&index, &["/content/0".to_string()], TargetRoute::Prose, 0)
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
            found: 1,
        }
    );
}
