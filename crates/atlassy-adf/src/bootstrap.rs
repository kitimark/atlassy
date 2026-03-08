use serde_json::Value;

pub fn is_page_effectively_empty(adf: &Value) -> bool {
    let content = match adf.get("content") {
        Some(Value::Array(arr)) => arr,
        _ => return true,
    };

    if content.is_empty() {
        return true;
    }

    content.iter().all(|node| {
        let node_type = node.get("type").and_then(Value::as_str).unwrap_or_default();

        if node_type != "paragraph" {
            return false;
        }

        match node.get("content") {
            None => true,
            Some(Value::Array(children)) if children.is_empty() => true,
            Some(Value::Array(children)) => children.iter().all(|child| {
                let child_type = child
                    .get("type")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                if child_type != "text" {
                    return false;
                }
                match child.get("text").and_then(Value::as_str) {
                    None => true,
                    Some(text) => text.is_empty(),
                }
            }),
            _ => false,
        }
    })
}

pub fn bootstrap_scaffold() -> Value {
    serde_json::json!({
        "type": "doc",
        "version": 1,
        "content": [
            {
                "type": "heading",
                "attrs": {"level": 2},
                "content": [{"type": "text", "text": ""}]
            },
            {
                "type": "paragraph",
                "content": [{"type": "text", "text": ""}]
            }
        ]
    })
}
