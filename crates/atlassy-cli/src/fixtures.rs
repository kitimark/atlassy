pub fn demo_page() -> serde_json::Value {
    serde_json::json!({
      "type": "doc",
      "version": 1,
      "content": [
        {
          "type": "heading",
          "attrs": {"level": 2, "id": "intro-heading"},
          "content": [{"type": "text", "text": "Overview"}]
        },
        {
          "type": "paragraph",
          "attrs": {"id": "intro-paragraph"},
          "content": [{"type": "text", "text": "Initial paragraph"}]
        },
        {
          "type": "table",
          "content": [
            {
              "type": "tableRow",
              "content": [
                {
                  "type": "tableCell",
                  "content": [
                    {
                      "type": "paragraph",
                      "content": [{"type": "text", "text": "Initial table cell"}]
                    }
                  ]
                }
              ]
            }
          ]
        }
      ]
    })
}

pub fn empty_page() -> serde_json::Value {
    serde_json::json!({
      "type": "doc",
      "version": 1,
      "content": [
        {
          "type": "paragraph",
          "content": []
        }
      ]
    })
}
