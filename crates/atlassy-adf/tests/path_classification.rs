use atlassy_adf::*;

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
fn document_order_sort_numeric_segments() {
    let mut paths = vec![
        "/content/10/content/0".to_string(),
        "/content/2/content/0".to_string(),
    ];

    document_order_sort(&mut paths);

    assert_eq!(
        paths,
        vec![
            "/content/2/content/0".to_string(),
            "/content/10/content/0".to_string(),
        ]
    );
}

#[test]
fn document_order_sort_shared_prefix() {
    let mut paths = vec!["/content/0/content/0".to_string(), "/content/0".to_string()];

    document_order_sort(&mut paths);

    assert_eq!(
        paths,
        vec!["/content/0".to_string(), "/content/0/content/0".to_string()]
    );
}

#[test]
fn scope_anchor_types_is_subset_of_editable_prose() {
    for scope_anchor_type in SCOPE_ANCHOR_TYPES {
        assert!(
            EDITABLE_PROSE_TYPES.contains(scope_anchor_type),
            "scope anchor type must be editable prose: {scope_anchor_type}"
        );
    }
}
