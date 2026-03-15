use atlassy_adf::*;
use atlassy_contracts::Operation;
use serde_json::{json, Value};

fn base_adf() -> Value {
    json!({
        "type": "doc",
        "content": [
            {
                "type": "heading",
                "attrs": {"level": 2},
                "content": [{"type": "text", "text": "Heading"}]
            },
            {
                "type": "paragraph",
                "content": [{"type": "text", "text": "before"}]
            },
            {
                "type": "paragraph",
                "content": [{"type": "text", "text": "after"}]
            }
        ]
    })
}

fn paragraph(text: &str) -> Value {
    json!({
        "type": "paragraph",
        "content": [{"type": "text", "text": text}]
    })
}

#[test]
fn rejects_whole_body_patch() {
    let operations = vec![Operation::Replace {
        path: "/".to_string(),
        value: serde_json::json!({}),
    }];

    let error = validate_operations(&operations, &["/content/0".to_string()]).unwrap_err();
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
fn applies_replace_operation_to_candidate_payload() {
    let base = base_adf();
    let operations = vec![Operation::Replace {
        path: "/content/1/content/0/text".to_string(),
        value: Value::String("updated".to_string()),
    }];

    let patched = apply_operations(&base, &operations).unwrap();
    assert_eq!(
        patched
            .pointer("/content/1/content/0/text")
            .and_then(Value::as_str),
        Some("updated")
    );
}

#[test]
fn apply_insert_supports_beginning_middle_and_end_positions() {
    let base = base_adf();

    let beginning = apply_operations(
        &base,
        &[Operation::Insert {
            parent_path: "/content".to_string(),
            index: 0,
            block: paragraph("beginning"),
        }],
    )
    .unwrap();
    assert_eq!(
        beginning
            .pointer("/content/0/content/0/text")
            .and_then(Value::as_str),
        Some("beginning")
    );

    let middle = apply_operations(
        &base,
        &[Operation::Insert {
            parent_path: "/content".to_string(),
            index: 1,
            block: paragraph("middle"),
        }],
    )
    .unwrap();
    assert_eq!(
        middle
            .pointer("/content/1/content/0/text")
            .and_then(Value::as_str),
        Some("middle")
    );

    let len = base
        .pointer("/content")
        .and_then(Value::as_array)
        .map(|array| array.len())
        .unwrap();
    let end = apply_operations(
        &base,
        &[Operation::Insert {
            parent_path: "/content".to_string(),
            index: len,
            block: paragraph("end"),
        }],
    )
    .unwrap();
    assert_eq!(
        end.pointer(&format!("/content/{len}/content/0/text"))
            .and_then(Value::as_str),
        Some("end")
    );
}

#[test]
fn apply_insert_rejects_out_of_bounds_non_array_and_empty_path() {
    let mut candidate = base_adf();

    let out_of_bounds = apply_insert(&mut candidate, "/content", 99, &paragraph("x"));
    assert!(matches!(
        out_of_bounds,
        Err(AdfError::InsertPositionInvalid(_))
    ));

    let non_array_parent = apply_insert(&mut candidate, "/content/0/type", 0, &paragraph("x"));
    assert!(matches!(
        non_array_parent,
        Err(AdfError::InsertPositionInvalid(_))
    ));

    let empty_path = apply_insert(&mut candidate, "", 0, &paragraph("x"));
    assert!(matches!(
        empty_path,
        Err(AdfError::InsertPositionInvalid(_))
    ));
}

#[test]
fn apply_remove_supports_valid_and_last_element_removal_and_rejects_missing_target() {
    let mut candidate = base_adf();

    apply_remove(&mut candidate, "/content/1").unwrap();
    assert_eq!(
        candidate
            .pointer("/content/1/content/0/text")
            .and_then(Value::as_str),
        Some("after")
    );

    apply_remove(&mut candidate, "/content/1").unwrap();
    let content_len = candidate
        .pointer("/content")
        .and_then(Value::as_array)
        .map(|array| array.len())
        .unwrap();
    assert_eq!(content_len, 1);

    let missing = apply_remove(&mut candidate, "/content/99");
    assert!(matches!(missing, Err(AdfError::RemoveTargetNotFound(_))));
}

#[test]
fn validate_operations_checks_insert_remove_scope_and_whole_body_rewrite() {
    let allowed_scope_paths = vec!["/content".to_string()];

    let valid = vec![
        Operation::Insert {
            parent_path: "/content".to_string(),
            index: 0,
            block: paragraph("new"),
        },
        Operation::Remove {
            target_path: "/content/1".to_string(),
        },
    ];
    assert!(validate_operations(&valid, &allowed_scope_paths).is_ok());

    let out_of_scope_insert = vec![Operation::Insert {
        parent_path: "/attrs".to_string(),
        index: 0,
        block: paragraph("x"),
    }];
    assert_eq!(
        validate_operations(&out_of_scope_insert, &allowed_scope_paths),
        Err(AdfError::OutOfScope("/attrs".to_string()))
    );

    let out_of_scope_remove = vec![Operation::Remove {
        target_path: "/attrs/title".to_string(),
    }];
    assert_eq!(
        validate_operations(&out_of_scope_remove, &allowed_scope_paths),
        Err(AdfError::OutOfScope("/attrs/title".to_string()))
    );

    let whole_body = vec![Operation::Replace {
        path: "/".to_string(),
        value: json!({}),
    }];
    assert_eq!(
        validate_operations(&whole_body, &allowed_scope_paths),
        Err(AdfError::WholeBodyRewriteDisallowed)
    );
}
