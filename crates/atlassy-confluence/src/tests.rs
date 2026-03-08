use std::collections::HashMap;

use super::*;

#[test]
fn stub_create_page_inserts_into_store() {
    let mut pages = HashMap::new();
    pages.insert(
        "parent-1".to_string(),
        StubPage {
            version: 1,
            adf: serde_json::json!({"type": "doc", "version": 1, "content": []}),
        },
    );
    let mut client = StubConfluenceClient::new(pages);

    let result = client
        .create_page("Test Page", "parent-1", "SPACE")
        .unwrap();
    assert_eq!(result.page_version, 1);
    assert!(!result.page_id.is_empty());

    let fetched = client.fetch_page(&result.page_id).unwrap();
    assert_eq!(fetched.page_version, 1);
}

#[test]
fn stub_create_page_rejects_missing_parent() {
    let mut client = StubConfluenceClient::new(HashMap::new());
    let result = client.create_page("Test", "missing-parent", "SPACE");
    assert!(matches!(result, Err(ConfluenceError::NotFound(_))));
}

#[test]
fn stub_create_page_rejects_duplicate_title() {
    let mut pages = HashMap::new();
    pages.insert(
        "parent-1".to_string(),
        StubPage {
            version: 1,
            adf: serde_json::json!({"type": "doc", "version": 1, "content": []}),
        },
    );
    let mut client = StubConfluenceClient::new(pages);

    client.create_page("Dup Page", "parent-1", "SPACE").unwrap();
    let result = client.create_page("Dup Page", "parent-1", "SPACE");
    assert!(matches!(result, Err(ConfluenceError::Transport(_))));
}
