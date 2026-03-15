use atlassy_adf::is_attr_editable_type;
use atlassy_contracts::{ErrorCode, Operation, PipelineState};

use crate::PipelineError;

#[derive(Debug, Clone, Copy)]
pub(crate) struct LockedPath<'a> {
    pub path: &'a str,
    pub node_type: &'a str,
}

pub(crate) fn check_locked_boundary(
    operation: &Operation,
    locked_paths: &[LockedPath<'_>],
) -> Option<PipelineError> {
    let violating_path = locked_paths
        .iter()
        .find_map(|locked| locked_path_violation(operation, *locked))?;

    Some(PipelineError::Hard {
        state: PipelineState::MergeCandidates,
        code: ErrorCode::RouteViolation,
        message: format!("operation path `{violating_path}` overlaps locked structural boundary"),
    })
}

fn locked_path_violation(operation: &Operation, locked: LockedPath<'_>) -> Option<String> {
    match operation {
        Operation::Replace { path, .. } => {
            path_is_within_locked_boundary(path, locked.path).then(|| path.clone())
        }
        Operation::UpdateAttrs { target_path, .. } => {
            if target_path == locked.path {
                if is_attr_editable_type(locked.node_type) {
                    None
                } else {
                    Some(target_path.clone())
                }
            } else {
                path_is_descendant_of(target_path, locked.path).then(|| target_path.clone())
            }
        }
        Operation::Insert {
            parent_path, index, ..
        } => {
            let inserted_path = format!("{parent_path}/{index}");
            if !path_is_within_locked_boundary(&inserted_path, locked.path) {
                return None;
            }

            if parent_path == locked.path || path_is_descendant_of(parent_path, locked.path) {
                None
            } else {
                Some(inserted_path)
            }
        }
        Operation::Remove { target_path } => {
            (target_path == locked.path).then(|| target_path.clone())
        }
    }
}

fn path_is_within_locked_boundary(path: &str, locked_path: &str) -> bool {
    path == locked_path || path_is_descendant_of(path, locked_path)
}

fn path_is_descendant_of(path: &str, parent: &str) -> bool {
    path.strip_prefix(parent)
        .is_some_and(|suffix| suffix.starts_with('/'))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn allows_update_attrs_on_attr_editable_locked_node() {
        let operation = Operation::UpdateAttrs {
            target_path: "/content/0".to_string(),
            attrs: json!({"panelType": "warning"}),
        };
        let locked_paths = [LockedPath {
            path: "/content/0",
            node_type: "panel",
        }];

        assert!(check_locked_boundary(&operation, &locked_paths).is_none());
    }

    #[test]
    fn blocks_replace_on_locked_structural_node() {
        let operation = Operation::Replace {
            path: "/content/0/attrs/title".to_string(),
            value: json!("new"),
        };
        let locked_paths = [LockedPath {
            path: "/content/0",
            node_type: "panel",
        }];

        assert!(check_locked_boundary(&operation, &locked_paths).is_some());
    }

    #[test]
    fn allows_insert_and_remove_children_inside_locked_container() {
        let insert = Operation::Insert {
            parent_path: "/content/0/content".to_string(),
            index: 0,
            block: json!({"type": "paragraph", "content": []}),
        };
        let remove = Operation::Remove {
            target_path: "/content/0/content/0".to_string(),
        };
        let locked_paths = [LockedPath {
            path: "/content/0",
            node_type: "panel",
        }];

        assert!(check_locked_boundary(&insert, &locked_paths).is_none());
        assert!(check_locked_boundary(&remove, &locked_paths).is_none());
    }

    #[test]
    fn blocks_remove_of_locked_container_itself() {
        let remove = Operation::Remove {
            target_path: "/content/0".to_string(),
        };
        let locked_paths = [LockedPath {
            path: "/content/0",
            node_type: "panel",
        }];

        assert!(check_locked_boundary(&remove, &locked_paths).is_some());
    }
}
