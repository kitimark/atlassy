use serde::Serialize;

pub const CONTRACT_VERSION: &str = "1.0.0";
pub const PIPELINE_VERSION: &str = "v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    ScopeMiss,
    RouteViolation,
    SchemaInvalid,
    InsertPositionInvalid,
    RemoveAnchorMissing,
    PostMutationSchemaInvalid,
    OutOfScopeMutation,
    LockedNodeMutation,
    TableShapeChange,
    ConflictRetryExhausted,
    RuntimeBackend,
    RuntimeUnmappedHard,
    BootstrapRequired,
    BootstrapInvalidState,
    TargetDiscoveryFailed,
    SectionBoundaryInvalid,
    StructuralCompositionFailed,
}

impl ErrorCode {
    pub const ALL: [Self; 17] = [
        Self::ScopeMiss,
        Self::RouteViolation,
        Self::SchemaInvalid,
        Self::InsertPositionInvalid,
        Self::RemoveAnchorMissing,
        Self::PostMutationSchemaInvalid,
        Self::OutOfScopeMutation,
        Self::LockedNodeMutation,
        Self::TableShapeChange,
        Self::ConflictRetryExhausted,
        Self::RuntimeBackend,
        Self::RuntimeUnmappedHard,
        Self::BootstrapRequired,
        Self::BootstrapInvalidState,
        Self::TargetDiscoveryFailed,
        Self::SectionBoundaryInvalid,
        Self::StructuralCompositionFailed,
    ];

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::ScopeMiss => "ERR_SCOPE_MISS",
            Self::RouteViolation => "ERR_ROUTE_VIOLATION",
            Self::SchemaInvalid => "ERR_SCHEMA_INVALID",
            Self::InsertPositionInvalid => "ERR_INSERT_POSITION_INVALID",
            Self::RemoveAnchorMissing => "ERR_REMOVE_ANCHOR_MISSING",
            Self::PostMutationSchemaInvalid => "ERR_POST_MUTATION_SCHEMA_INVALID",
            Self::OutOfScopeMutation => "ERR_OUT_OF_SCOPE_MUTATION",
            Self::LockedNodeMutation => "ERR_LOCKED_NODE_MUTATION",
            Self::TableShapeChange => "ERR_TABLE_SHAPE_CHANGE",
            Self::ConflictRetryExhausted => "ERR_CONFLICT_RETRY_EXHAUSTED",
            Self::RuntimeBackend => "ERR_RUNTIME_BACKEND",
            Self::RuntimeUnmappedHard => "ERR_RUNTIME_UNMAPPED_HARD",
            Self::BootstrapRequired => "ERR_BOOTSTRAP_REQUIRED",
            Self::BootstrapInvalidState => "ERR_BOOTSTRAP_INVALID_STATE",
            Self::TargetDiscoveryFailed => "ERR_TARGET_DISCOVERY_FAILED",
            Self::SectionBoundaryInvalid => "ERR_SECTION_BOUNDARY_INVALID",
            Self::StructuralCompositionFailed => "ERR_STRUCTURAL_COMPOSITION_FAILED",
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for ErrorCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

pub const FLOW_BASELINE: &str = "baseline";
pub const FLOW_OPTIMIZED: &str = "optimized";

pub const PATTERN_A: &str = "A";
pub const PATTERN_B: &str = "B";
pub const PATTERN_C: &str = "C";

pub const RUNTIME_STUB: &str = "stub";
pub const RUNTIME_LIVE: &str = "live";

#[cfg(test)]
mod tests {
    use super::ErrorCode;

    #[test]
    fn as_str_returns_expected_values_for_all_error_codes() {
        let cases = [
            (ErrorCode::ScopeMiss, "ERR_SCOPE_MISS"),
            (ErrorCode::RouteViolation, "ERR_ROUTE_VIOLATION"),
            (ErrorCode::SchemaInvalid, "ERR_SCHEMA_INVALID"),
            (
                ErrorCode::InsertPositionInvalid,
                "ERR_INSERT_POSITION_INVALID",
            ),
            (ErrorCode::RemoveAnchorMissing, "ERR_REMOVE_ANCHOR_MISSING"),
            (
                ErrorCode::PostMutationSchemaInvalid,
                "ERR_POST_MUTATION_SCHEMA_INVALID",
            ),
            (ErrorCode::OutOfScopeMutation, "ERR_OUT_OF_SCOPE_MUTATION"),
            (ErrorCode::LockedNodeMutation, "ERR_LOCKED_NODE_MUTATION"),
            (ErrorCode::TableShapeChange, "ERR_TABLE_SHAPE_CHANGE"),
            (
                ErrorCode::ConflictRetryExhausted,
                "ERR_CONFLICT_RETRY_EXHAUSTED",
            ),
            (ErrorCode::RuntimeBackend, "ERR_RUNTIME_BACKEND"),
            (ErrorCode::RuntimeUnmappedHard, "ERR_RUNTIME_UNMAPPED_HARD"),
            (ErrorCode::BootstrapRequired, "ERR_BOOTSTRAP_REQUIRED"),
            (
                ErrorCode::BootstrapInvalidState,
                "ERR_BOOTSTRAP_INVALID_STATE",
            ),
            (
                ErrorCode::TargetDiscoveryFailed,
                "ERR_TARGET_DISCOVERY_FAILED",
            ),
            (
                ErrorCode::SectionBoundaryInvalid,
                "ERR_SECTION_BOUNDARY_INVALID",
            ),
            (
                ErrorCode::StructuralCompositionFailed,
                "ERR_STRUCTURAL_COMPOSITION_FAILED",
            ),
        ];

        assert_eq!(cases.len(), ErrorCode::ALL.len());
        for (code, expected) in cases {
            assert_eq!(code.as_str(), expected);
        }
    }

    #[test]
    fn display_matches_as_str_for_all_error_codes() {
        for code in ErrorCode::ALL {
            assert_eq!(code.to_string(), code.as_str());
        }
    }
}
