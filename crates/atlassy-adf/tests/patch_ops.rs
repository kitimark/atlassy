use atlassy_adf::*;
use atlassy_contracts::Operation;
use serde_json::Value;

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
fn applies_operations_to_candidate_payload() {
    let base = serde_json::json!({
        "type": "doc",
        "content": [
            {"type": "paragraph", "content": [{"type": "text", "text": "before"}]}
        ]
    });

    let operations = vec![Operation::Replace {
        path: "/content/0/content/0/text".to_string(),
        value: serde_json::Value::String("after".to_string()),
    }];

    let patched = apply_operations(&base, &operations).unwrap();
    assert_eq!(
        patched
            .pointer("/content/0/content/0/text")
            .and_then(Value::as_str),
        Some("after")
    );
}
