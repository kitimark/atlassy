use super::*;

#[test]
fn state_tracker_blocks_out_of_order_transitions() {
    let mut tracker = StateTracker::new();
    assert!(tracker.transition_to(PipelineState::Fetch).is_ok());
    let err = tracker.transition_to(PipelineState::Patch).unwrap_err();
    assert_eq!(
        err,
        ContractError::InvalidTransition {
            expected: "classify".to_string(),
            actual: "patch".to_string(),
        }
    );
}

#[test]
fn compute_section_bytes_sums_serialized_nodes_for_paths() {
    let adf = serde_json::json!({
        "type": "doc",
        "content": [
            {"type": "paragraph", "content": [{"type": "text", "text": "One"}]},
            {"type": "paragraph", "content": [{"type": "text", "text": "Two"}]}
        ]
    });

    let first = serde_json::to_vec(adf.pointer("/content/0").unwrap())
        .map(|value| value.len() as u64)
        .unwrap();
    let second = serde_json::to_vec(adf.pointer("/content/1").unwrap())
        .map(|value| value.len() as u64)
        .unwrap();

    let section_bytes =
        compute_section_bytes(&adf, &["/content/0".to_string(), "/content/1".to_string()]);

    assert_eq!(section_bytes, first + second);
}

#[test]
fn compute_section_bytes_returns_full_page_size_for_root_scope() {
    let adf = serde_json::json!({
        "type": "doc",
        "content": [
            {"type": "heading", "attrs": {"level": 2}, "content": [{"type": "text", "text": "Overview"}]},
            {"type": "paragraph", "content": [{"type": "text", "text": "Body"}]}
        ]
    });

    let full_page = serde_json::to_vec(&adf)
        .map(|value| value.len() as u64)
        .unwrap();
    let section_bytes = compute_section_bytes(&adf, &["/".to_string()]);

    assert_eq!(section_bytes, full_page);
}
