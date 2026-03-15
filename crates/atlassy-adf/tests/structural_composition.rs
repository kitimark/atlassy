use atlassy_adf::{
    AdfError, build_heading, build_list, build_paragraph, build_section, build_table,
    build_table_cell, build_table_header, build_table_row, build_text, check_structural_validity,
    find_section_range, is_attr_editable_type, is_editable_prose, is_insertable_type,
    is_removable_type,
};
use serde_json::json;

#[test]
fn build_text_paragraph_and_heading_work_and_validate_heading_level() {
    assert_eq!(
        build_text("Hello"),
        json!({"type": "text", "text": "Hello"})
    );
    assert_eq!(
        build_paragraph("Body"),
        json!({
            "type": "paragraph",
            "content": [{"type": "text", "text": "Body"}],
        })
    );

    assert_eq!(
        build_heading(2, "Title").unwrap(),
        json!({
            "type": "heading",
            "attrs": {"level": 2},
            "content": [{"type": "text", "text": "Title"}],
        })
    );

    assert!(matches!(
        build_heading(0, "bad"),
        Err(AdfError::StructuralCompositionFailed(_))
    ));
    assert!(matches!(
        build_heading(7, "bad"),
        Err(AdfError::StructuralCompositionFailed(_))
    ));
}

#[test]
fn build_table_supports_header_and_rejects_zero_dimensions() {
    let header_table = build_table(2, 3, true).unwrap();
    assert_eq!(header_table["type"], json!("table"));
    assert_eq!(header_table["content"].as_array().unwrap().len(), 2);
    assert_eq!(header_table["content"][0]["type"], json!("tableRow"));
    assert_eq!(
        header_table["content"][0]["content"][0]["type"],
        json!("tableHeader")
    );
    assert_eq!(
        header_table["content"][1]["content"][0]["type"],
        json!("tableCell")
    );

    let no_header_table = build_table(3, 2, false).unwrap();
    assert_eq!(no_header_table["content"].as_array().unwrap().len(), 3);
    assert_eq!(
        no_header_table["content"][0]["content"][0]["type"],
        json!("tableCell")
    );

    assert!(matches!(
        build_table(0, 2, false),
        Err(AdfError::StructuralCompositionFailed(_))
    ));
    assert!(matches!(
        build_table(2, 0, false),
        Err(AdfError::StructuralCompositionFailed(_))
    ));
}

#[test]
fn build_table_row_cell_and_header_construct_valid_shapes() {
    let header = build_table_header("H");
    let cell = build_table_cell("C");
    let row = build_table_row(&[header.clone(), cell.clone()]);

    assert_eq!(header["type"], json!("tableHeader"));
    assert_eq!(header["content"][0]["type"], json!("paragraph"));
    assert_eq!(header["content"][0]["content"][0]["text"], json!("H"));

    assert_eq!(cell["type"], json!("tableCell"));
    assert_eq!(cell["content"][0]["type"], json!("paragraph"));
    assert_eq!(cell["content"][0]["content"][0]["text"], json!("C"));

    assert_eq!(row["type"], json!("tableRow"));
    assert_eq!(row["content"][0], header);
    assert_eq!(row["content"][1], cell);
}

#[test]
fn build_list_supports_ordered_unordered_and_rejects_empty_items() {
    let unordered = build_list(false, &["Item A", "Item B"]).unwrap();
    assert_eq!(unordered["type"], json!("bulletList"));
    assert_eq!(unordered["content"].as_array().unwrap().len(), 2);

    let ordered = build_list(true, &["First", "Second"]).unwrap();
    assert_eq!(ordered["type"], json!("orderedList"));
    assert_eq!(ordered["content"].as_array().unwrap().len(), 2);

    assert!(matches!(
        build_list(false, &[]),
        Err(AdfError::StructuralCompositionFailed(_))
    ));
}

#[test]
fn build_section_supports_body_empty_body_and_invalid_level() {
    let para = build_paragraph("p1");
    let list = build_list(false, &["a", "b"]).unwrap();
    let section = build_section(2, "FAQ", &[para.clone(), list.clone()]).unwrap();
    assert_eq!(section.len(), 3);
    assert_eq!(section[0]["type"], json!("heading"));
    assert_eq!(section[1], para);
    assert_eq!(section[2], list);

    let heading_only = build_section(3, "Empty Section", &[]).unwrap();
    assert_eq!(heading_only.len(), 1);
    assert_eq!(heading_only[0]["type"], json!("heading"));

    assert!(matches!(
        build_section(8, "bad", &[]),
        Err(AdfError::StructuralCompositionFailed(_))
    ));
}

#[test]
fn find_section_range_handles_normal_end_of_doc_empty_and_invalid_targets() {
    let adf = json!({
        "type": "doc",
        "content": [
            {"type": "heading", "attrs": {"level": 1}, "content": [{"type": "text", "text": "Intro"}]},
            {"type": "paragraph", "content": [{"type": "text", "text": "intro body"}]},
            {"type": "heading", "attrs": {"level": 2}, "content": [{"type": "text", "text": "Details"}]},
            {"type": "paragraph", "content": [{"type": "text", "text": "d1"}]},
            {"type": "paragraph", "content": [{"type": "text", "text": "d2"}]},
            {"type": "heading", "attrs": {"level": 2}, "content": [{"type": "text", "text": "Summary"}]},
            {"type": "paragraph", "content": [{"type": "text", "text": "s1"}]}
        ]
    });

    let details = find_section_range(&adf, "/content/2").unwrap();
    assert_eq!(details.heading_index, 2);
    assert_eq!(details.end_index, 5);
    assert_eq!(details.block_count, 3);
    assert_eq!(
        details.block_paths,
        vec![
            "/content/2".to_string(),
            "/content/3".to_string(),
            "/content/4".to_string(),
        ]
    );

    let summary = find_section_range(&adf, "/content/5").unwrap();
    assert_eq!(summary.heading_index, 5);
    assert_eq!(summary.end_index, 7);
    assert_eq!(summary.block_count, 2);

    let empty = json!({
        "type": "doc",
        "content": [
            {"type": "heading", "attrs": {"level": 2}, "content": [{"type": "text", "text": "Empty"}]},
            {"type": "heading", "attrs": {"level": 2}, "content": [{"type": "text", "text": "Next"}]}
        ]
    });
    let empty_section = find_section_range(&empty, "/content/0").unwrap();
    assert_eq!(empty_section.end_index, 1);
    assert_eq!(empty_section.block_count, 1);

    assert!(matches!(
        find_section_range(&adf, "/content/1"),
        Err(AdfError::SectionBoundaryInvalid(_))
    ));
    assert!(matches!(
        find_section_range(&adf, "/content/99"),
        Err(AdfError::SectionBoundaryInvalid(_))
    ));
}

#[test]
fn type_policy_functions_cover_expected_categories() {
    for prose in [
        "paragraph",
        "heading",
        "bulletList",
        "orderedList",
        "listItem",
        "blockquote",
        "codeBlock",
    ] {
        assert!(
            is_editable_prose(prose),
            "expected editable prose for {prose}"
        );
        assert!(
            is_insertable_type(prose),
            "expected insertable type for {prose}"
        );
        assert!(
            is_removable_type(prose),
            "expected removable type for {prose}"
        );
    }

    assert!(!is_editable_prose("table"));
    assert!(is_insertable_type("table"));
    assert!(is_removable_type("table"));
    assert!(is_insertable_type("tableRow"));
    assert!(is_removable_type("tableRow"));

    assert!(!is_insertable_type("panel"));
    assert!(!is_removable_type("panel"));

    assert!(is_attr_editable_type("panel"));
    assert!(is_attr_editable_type("expand"));
    assert!(is_attr_editable_type("mediaSingle"));
    assert!(!is_attr_editable_type("paragraph"));
}

#[test]
fn builder_outputs_pass_structural_validity() {
    let table = build_table(3, 3, true).unwrap();
    let list = build_list(false, &["A", "B"]).unwrap();
    let heading = build_heading(2, "Overview").unwrap();
    let section = build_section(3, "FAQ", &[build_paragraph("answer")]).unwrap();

    let doc = json!({
        "type": "doc",
        "content": [
            heading,
            table,
            list,
            section[0].clone(),
            section[1].clone()
        ]
    });

    assert!(check_structural_validity(&doc).is_ok());
}
