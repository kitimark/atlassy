use std::cmp::Ordering;

use crate::AdfError;

pub fn document_order_sort(paths: &mut [String]) {
    paths.sort_by(|left, right| compare_path_segments(left, right));
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

pub fn is_within_allowed_scope(path: &str, allowed_scope_paths: &[String]) -> bool {
    allowed_scope_paths.iter().any(|allowed| {
        if allowed == "/" {
            return true;
        }
        path == allowed
            || path
                .strip_prefix(allowed)
                .is_some_and(|suffix| suffix.starts_with('/'))
    })
}

pub fn is_path_within_or_descendant(path: &str, mapped_path: &str) -> bool {
    path == mapped_path
        || path
            .strip_prefix(mapped_path)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

pub fn canonicalize_mapped_path(
    path: &str,
    allowed_scope_paths: &[String],
) -> Result<String, AdfError> {
    if !is_json_pointer(path) {
        return Err(AdfError::InvalidPath(path.to_string()));
    }

    if allowed_scope_paths.iter().any(|allowed| allowed == "/") {
        return Ok(path.to_string());
    }

    if is_within_allowed_scope(path, allowed_scope_paths) {
        return Ok(path.to_string());
    }

    if allowed_scope_paths.len() == 1 {
        let root = allowed_scope_paths[0].trim_end_matches('/');
        if path == "/" {
            return Ok(root.to_string());
        }
        let tail = path.trim_start_matches('/');
        let canonical = format!("{root}/{tail}");
        if is_within_allowed_scope(&canonical, allowed_scope_paths) {
            return Ok(canonical);
        }
    }

    Err(AdfError::OutOfScope(path.to_string()))
}

pub(crate) fn is_json_pointer(path: &str) -> bool {
    path.starts_with('/')
}

pub(crate) fn escape_pointer_segment(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

pub(crate) fn parent_path(path: &str) -> Option<String> {
    if path == "/" {
        return None;
    }
    let (parent, _) = path.rsplit_once('/')?;
    if parent.is_empty() {
        return Some("/".to_string());
    }
    Some(parent.to_string())
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use super::*;

    #[test]
    fn compare_path_segments_orders_numeric_segments_numerically() {
        assert_eq!(
            compare_path_segments("/content/2", "/content/10"),
            Ordering::Less
        );
    }

    #[test]
    fn compare_path_segments_orders_prefix_shorter_first() {
        assert_eq!(
            compare_path_segments("/content/2", "/content/2/0"),
            Ordering::Less
        );
    }

    #[test]
    fn compare_path_segments_reports_equal_for_identical_paths() {
        assert_eq!(
            compare_path_segments("/content/2", "/content/2"),
            Ordering::Equal
        );
    }

    #[test]
    fn is_json_pointer_accepts_slash_prefixed_paths() {
        assert!(is_json_pointer("/content/0"));
    }

    #[test]
    fn is_json_pointer_rejects_non_slash_prefixed_paths() {
        assert!(!is_json_pointer("content/0"));
    }

    #[test]
    fn is_json_pointer_rejects_empty_string() {
        assert!(!is_json_pointer(""));
    }

    #[test]
    fn escape_pointer_segment_escapes_tilde() {
        assert_eq!(escape_pointer_segment("a~b"), "a~0b");
    }

    #[test]
    fn escape_pointer_segment_escapes_slash() {
        assert_eq!(escape_pointer_segment("a/b"), "a~1b");
    }

    #[test]
    fn escape_pointer_segment_keeps_clean_segments() {
        assert_eq!(escape_pointer_segment("clean"), "clean");
    }

    #[test]
    fn parent_path_returns_none_for_root() {
        assert_eq!(parent_path("/"), None);
    }

    #[test]
    fn parent_path_returns_parent_for_nested_path() {
        assert_eq!(parent_path("/content/0"), Some("/content".to_string()));
    }

    #[test]
    fn parent_path_returns_root_for_single_segment() {
        assert_eq!(parent_path("/content"), Some("/".to_string()));
    }
}
