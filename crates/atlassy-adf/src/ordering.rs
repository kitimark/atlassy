use std::cmp::Ordering;

use atlassy_contracts::Operation;

use crate::AdfError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StructuralOpKind {
    Insert,
    Remove,
}

#[derive(Debug, Clone)]
struct StructuralOp {
    operation: Operation,
    parent_path: String,
    index: usize,
    kind: StructuralOpKind,
}

pub fn sort_operations(operations: &[Operation]) -> Result<Vec<Operation>, AdfError> {
    detect_remove_prefix_conflicts(operations)?;

    let mut replaces = Vec::new();
    let mut structural = Vec::new();

    for operation in operations {
        if let Some((parent_path, index, kind)) = extract_path_info(operation)? {
            structural.push(StructuralOp {
                operation: operation.clone(),
                parent_path,
                index,
                kind,
            });
        } else {
            replaces.push(operation.clone());
        }
    }

    structural.sort_by(compare_structural_ops);

    let mut sorted = replaces;
    sorted.extend(structural.into_iter().map(|entry| entry.operation));
    Ok(sorted)
}

fn extract_path_info(
    operation: &Operation,
) -> Result<Option<(String, usize, StructuralOpKind)>, AdfError> {
    match operation {
        Operation::Replace { .. } | Operation::UpdateAttrs { .. } => Ok(None),
        Operation::Insert {
            parent_path, index, ..
        } => Ok(Some((
            parent_path.clone(),
            *index,
            StructuralOpKind::Insert,
        ))),
        Operation::Remove { target_path } => {
            let (parent_path, index) = split_parent_index(target_path)?;
            Ok(Some((parent_path, index, StructuralOpKind::Remove)))
        }
    }
}

fn compare_structural_ops(left: &StructuralOp, right: &StructuralOp) -> Ordering {
    let parent_order = compare_path_segments(&right.parent_path, &left.parent_path);
    if parent_order != Ordering::Equal {
        return parent_order;
    }

    if left.kind == right.kind {
        let index_order = match left.kind {
            StructuralOpKind::Insert => left.index.cmp(&right.index),
            StructuralOpKind::Remove => right.index.cmp(&left.index),
        };
        if index_order != Ordering::Equal {
            return index_order;
        }
    } else {
        let index_order = right.index.cmp(&left.index);
        if index_order != Ordering::Equal {
            return index_order;
        }
    }

    match (left.kind, right.kind) {
        (StructuralOpKind::Remove, StructuralOpKind::Insert) => Ordering::Less,
        (StructuralOpKind::Insert, StructuralOpKind::Remove) => Ordering::Greater,
        _ => Ordering::Equal,
    }
}

fn detect_remove_prefix_conflicts(operations: &[Operation]) -> Result<(), AdfError> {
    for (remove_index, remove_path) in
        operations
            .iter()
            .enumerate()
            .filter_map(|(index, operation)| match operation {
                Operation::Remove { target_path } => Some((index, target_path.as_str())),
                _ => None,
            })
    {
        for (index, operation) in operations.iter().enumerate() {
            if index == remove_index {
                continue;
            }
            let operation_path = operation_conflict_path(operation)?;
            if is_strict_path_prefix(remove_path, &operation_path) {
                return Err(AdfError::OperationConflict(format!(
                    "remove `{remove_path}` conflicts with operation path `{operation_path}`"
                )));
            }
        }
    }

    Ok(())
}

fn operation_conflict_path(operation: &Operation) -> Result<String, AdfError> {
    match operation {
        Operation::Replace { path, .. } => Ok(path.clone()),
        Operation::UpdateAttrs { target_path, .. } => Ok(target_path.clone()),
        Operation::Insert {
            parent_path, index, ..
        } => Ok(format!("{parent_path}/{index}")),
        Operation::Remove { target_path } => split_parent_index(target_path)
            .map(|(parent_path, index)| format!("{parent_path}/{index}")),
    }
}

fn is_strict_path_prefix(prefix: &str, path: &str) -> bool {
    path.strip_prefix(prefix)
        .is_some_and(|suffix| suffix.starts_with('/'))
}

fn split_parent_index(path: &str) -> Result<(String, usize), AdfError> {
    let (parent, index_segment) = path
        .rsplit_once('/')
        .ok_or_else(|| AdfError::InvalidPath(path.to_string()))?;
    let index = index_segment
        .parse::<usize>()
        .map_err(|_| AdfError::InvalidPath(path.to_string()))?;
    let parent_path = if parent.is_empty() {
        "/".to_string()
    } else {
        parent.to_string()
    };
    Ok((parent_path, index))
}

fn compare_path_segments(left: &str, right: &str) -> Ordering {
    let mut left_segments = left.split('/');
    let mut right_segments = right.split('/');

    loop {
        match (left_segments.next(), right_segments.next()) {
            (Some(left_segment), Some(right_segment)) => {
                let ordering = match (
                    left_segment.parse::<usize>(),
                    right_segment.parse::<usize>(),
                ) {
                    (Ok(left_number), Ok(right_number)) => left_number.cmp(&right_number),
                    _ => left_segment.cmp(right_segment),
                };
                if ordering != Ordering::Equal {
                    return ordering;
                }
            }
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (None, None) => return Ordering::Equal,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn sort_operations_returns_empty_for_empty_input() {
        assert!(sort_operations(&[]).unwrap().is_empty());
    }

    #[test]
    fn sort_operations_keeps_replace_only_input_order() {
        let operations = vec![
            Operation::Replace {
                path: "/content/0/content/0/text".to_string(),
                value: json!("a"),
            },
            Operation::UpdateAttrs {
                target_path: "/content/1".to_string(),
                attrs: json!({"panelType": "warning"}),
            },
            Operation::Replace {
                path: "/content/1/content/0/text".to_string(),
                value: json!("b"),
            },
        ];

        assert_eq!(sort_operations(&operations).unwrap(), operations);
    }

    #[test]
    fn sort_operations_orders_replaces_before_structural_with_insert_ascending() {
        let operations = vec![
            Operation::Insert {
                parent_path: "/content".to_string(),
                index: 1,
                block: json!({"type": "paragraph"}),
            },
            Operation::Replace {
                path: "/content/0/content/0/text".to_string(),
                value: json!("replace"),
            },
            Operation::Remove {
                target_path: "/content/2".to_string(),
            },
            Operation::Insert {
                parent_path: "/content".to_string(),
                index: 2,
                block: json!({"type": "paragraph"}),
            },
        ];

        let sorted = sort_operations(&operations).unwrap();
        assert!(matches!(sorted[0], Operation::Replace { .. }));
        assert!(matches!(
            sorted[1],
            Operation::Remove { ref target_path } if target_path == "/content/2"
        ));
        assert!(matches!(
            sorted[2],
            Operation::Insert { index, .. } if index == 1
        ));
        assert!(matches!(
            sorted[3],
            Operation::Insert { index, .. } if index == 2
        ));
    }

    #[test]
    fn sort_operations_detects_remove_prefix_conflict() {
        let operations = vec![
            Operation::Remove {
                target_path: "/content/2".to_string(),
            },
            Operation::Replace {
                path: "/content/2/content/0/text".to_string(),
                value: json!("x"),
            },
        ];

        assert!(matches!(
            sort_operations(&operations),
            Err(AdfError::OperationConflict(_))
        ));
    }

    #[test]
    fn sort_operations_sorts_independently_across_parents() {
        let operations = vec![
            Operation::Insert {
                parent_path: "/content".to_string(),
                index: 1,
                block: json!({"type": "paragraph"}),
            },
            Operation::Insert {
                parent_path: "/content/3/content".to_string(),
                index: 4,
                block: json!({"type": "paragraph"}),
            },
            Operation::Insert {
                parent_path: "/content/3/content".to_string(),
                index: 2,
                block: json!({"type": "paragraph"}),
            },
        ];

        let sorted = sort_operations(&operations).unwrap();
        assert!(matches!(
            sorted[0],
            Operation::Insert {
                ref parent_path,
                index,
                ..
            } if parent_path == "/content/3/content" && index == 2
        ));
        assert!(matches!(
            sorted[1],
            Operation::Insert {
                ref parent_path,
                index,
                ..
            } if parent_path == "/content/3/content" && index == 4
        ));
    }

    #[test]
    fn sort_operations_places_update_attrs_with_replace_before_structural_ops() {
        let operations = vec![
            Operation::Insert {
                parent_path: "/content".to_string(),
                index: 1,
                block: json!({"type": "paragraph"}),
            },
            Operation::UpdateAttrs {
                target_path: "/content/0".to_string(),
                attrs: json!({"panelType": "note"}),
            },
            Operation::Replace {
                path: "/content/0/content/0/text".to_string(),
                value: json!("updated"),
            },
        ];

        let sorted = sort_operations(&operations).unwrap();
        assert!(matches!(sorted[0], Operation::UpdateAttrs { .. }));
        assert!(matches!(sorted[1], Operation::Replace { .. }));
        assert!(matches!(sorted[2], Operation::Insert { .. }));
    }
}
