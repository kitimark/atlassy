use atlassy_contracts::{EnvelopeMeta, PipelineState};

use crate::{PipelineError, RunRequest};

pub(crate) fn meta(request: &RunRequest, state: PipelineState) -> EnvelopeMeta {
    EnvelopeMeta {
        request_id: request.request_id.clone(),
        page_id: request.page_id.clone(),
        state,
        timestamp: request.timestamp.clone(),
    }
}

pub(crate) fn estimate_tokens<T: serde::Serialize>(value: &T) -> Result<u64, PipelineError> {
    let bytes = serde_json::to_vec(value)?.len();
    let tokens = bytes.div_ceil(4) as u64;
    Ok(tokens.max(1))
}

pub(crate) fn compute_section_bytes(adf: &serde_json::Value, section_paths: &[String]) -> u64 {
    if section_paths.iter().any(|path| path == "/") {
        return serde_json::to_vec(adf)
            .map(|value| value.len() as u64)
            .unwrap_or(0);
    }

    section_paths
        .iter()
        .filter_map(|path| adf.pointer(path))
        .map(|node| {
            serde_json::to_vec(node)
                .map(|value| value.len() as u64)
                .unwrap_or(0)
        })
        .sum()
}

pub(crate) fn add_duration_suffix(timestamp: &str, elapsed_ms: u64) -> String {
    format!("{timestamp}+{elapsed_ms}ms")
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
