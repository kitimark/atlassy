## Why

Phase 2 enabled safe prose edits, but table updates are still effectively unavailable beyond prose-adjacent paths. We need Phase 3 to support practical table cell text edits while preserving v1 safety defaults that forbid structural table mutations.

## What Changes

- Implement `adf_table_edit` so table updates are limited to `cell_text_update` operations only.
- Add table candidate merge logic with path uniqueness and cross-route conflict checks.
- Enforce rejection of table topology and table attribute changes (`row/column add-remove`, `merge/split`, table-level attrs/layout).
- Ensure forbidden table operations fail with deterministic `ERR_TABLE_SHAPE_CHANGE` behavior.
- Add fixture-backed verification that table edits publish without structural drift and without locked-node mutation.

## Capabilities

### New Capabilities
- `table-cell-edit-route`: Enables ADF-native table cell text updates with strict operation allowlist (`cell_text_update` only).
- `table-shape-change-guards`: Detects and rejects forbidden table shape/attribute mutations with explicit failure signaling.
- `table-route-merge-conflict-safety`: Enforces path uniqueness and fast-fail behavior for cross-route conflicts involving table candidates.

### Modified Capabilities
- None.

## Impact

- Affected code: `crates/atlassy-pipeline` (`adf_table_edit`, merge checks, verifier integration), `crates/atlassy-adf` (table candidate helpers and guard utilities), and `crates/atlassy-contracts` (table-route payload and error surface tightening as needed).
- Test coverage: expanded fixture and integration scenarios in `crates/atlassy-pipeline/tests` for allowed table cell edits and forbidden shape mutations.
- APIs/contracts: no state-order change; extends v1 route behavior under existing ADF-canonical and verifier-gated pipeline contracts.
- Scope policy: keeps advanced table operations deferred per v1 defaults and roadmap boundaries.
