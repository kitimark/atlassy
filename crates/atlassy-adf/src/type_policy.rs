use crate::EDITABLE_PROSE_TYPES;

pub const INSERTABLE_BLOCK_TYPES: &[&str] = &[
    "paragraph",
    "heading",
    "bulletList",
    "orderedList",
    "listItem",
    "blockquote",
    "codeBlock",
    "table",
    "tableRow",
];

pub const REMOVABLE_BLOCK_TYPES: &[&str] = INSERTABLE_BLOCK_TYPES;

pub fn is_editable_prose(node_type: &str) -> bool {
    EDITABLE_PROSE_TYPES.contains(&node_type)
}

pub fn is_insertable_type(node_type: &str) -> bool {
    INSERTABLE_BLOCK_TYPES.contains(&node_type)
}

pub fn is_removable_type(node_type: &str) -> bool {
    REMOVABLE_BLOCK_TYPES.contains(&node_type)
}

pub fn is_attr_editable_type(node_type: &str) -> bool {
    matches!(node_type, "panel" | "expand" | "mediaSingle")
}
