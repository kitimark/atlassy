use super::*;

#[test]
fn publish_payload_includes_required_contract_fields() {
    let candidate_adf = serde_json::json!({
        "type": "doc",
        "version": 1,
        "content": []
    });

    let payload =
        LiveConfluenceClient::build_publish_payload("18841604", "Sandbox page", 7, &candidate_adf)
            .expect("payload should build");

    assert_eq!(payload["id"], serde_json::json!("18841604"));
    assert_eq!(payload["type"], serde_json::json!("page"));
    assert_eq!(payload["status"], serde_json::json!("current"));
    assert_eq!(payload["version"]["number"], serde_json::json!(8));
    assert_eq!(
        payload["body"]["atlas_doc_format"]["representation"],
        serde_json::json!("atlas_doc_format")
    );
}

#[test]
fn publish_payload_encodes_candidate_adf_as_json_string_value() {
    let candidate_adf = serde_json::json!({
        "type": "doc",
        "version": 1,
        "content": [
            {
                "type": "paragraph",
                "content": [
                    { "type": "text", "text": "hello" }
                ]
            }
        ]
    });

    let payload =
        LiveConfluenceClient::build_publish_payload("18841604", "Sandbox page", 1, &candidate_adf)
            .expect("payload should build");

    let encoded = payload["body"]["atlas_doc_format"]["value"]
        .as_str()
        .expect("atlas_doc_format.value should be a string");
    let decoded: Value = serde_json::from_str(encoded).expect("encoded value should be JSON");
    assert_eq!(decoded, candidate_adf);
}

#[test]
fn create_payload_includes_space_key_and_ancestors() {
    let payload = LiveConfluenceClient::build_create_payload("Child Page", "parent-123", "DEV")
        .expect("payload should build");

    assert_eq!(payload["type"], serde_json::json!("page"));
    assert_eq!(payload["status"], serde_json::json!("current"));
    assert_eq!(payload["title"], serde_json::json!("Child Page"));
    assert_eq!(payload["space"]["key"], serde_json::json!("DEV"));
    assert_eq!(
        payload["ancestors"][0]["id"],
        serde_json::json!("parent-123")
    );
}

#[test]
fn create_payload_encodes_empty_adf_and_has_no_version() {
    let payload = LiveConfluenceClient::build_create_payload("New Page", "parent-1", "SPACE")
        .expect("payload should build");

    assert!(
        payload.get("version").is_none(),
        "create payload should not include version"
    );
    assert!(
        payload.get("id").is_none(),
        "create payload should not include id"
    );

    let encoded = payload["body"]["atlas_doc_format"]["value"]
        .as_str()
        .expect("atlas_doc_format.value should be a string");
    let decoded: Value = serde_json::from_str(encoded).expect("encoded value should be JSON");
    assert_eq!(
        decoded,
        serde_json::json!({"type": "doc", "version": 1, "content": []})
    );
    assert_eq!(
        payload["body"]["atlas_doc_format"]["representation"],
        serde_json::json!("atlas_doc_format")
    );
}

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
