## Purpose

Define operation ordering behavior and integration points in the patch stage.

## Requirements

### Requirement: Ordering module implements reverse-document-order sorting
The `sort_operations()` function in `atlassy-adf/src/ordering.rs` MUST implement the reverse-document-order algorithm: partition replaces from structural ops, apply replaces first, then structural ops grouped by parent path in descending index order, with remove-before-insert tie-breaking at same index.

#### Scenario: sort_operations returns replaces before structural ops
- **WHEN** `sort_operations()` receives Replace and Insert operations
- **THEN** all Replace operations MUST appear before all Insert/Remove operations

#### Scenario: Structural ops sorted by descending index within parent
- **WHEN** `sort_operations()` receives Insert at index 1 and Remove at index 4 under the same parent
- **THEN** the index 4 operation MUST appear before the index 1 operation

#### Scenario: Same-index remove before insert
- **WHEN** `sort_operations()` receives Remove at index 2 and Insert at index 2 under the same parent
- **THEN** the Remove MUST appear before the Insert

#### Scenario: sort_operations handles empty input
- **WHEN** `sort_operations()` is called with an empty list
- **THEN** it MUST return an empty list without error

#### Scenario: Replace-only input returns unchanged order
- **WHEN** `sort_operations()` receives only Replace operations
- **THEN** the output order MUST match the input order

### Requirement: Ordering module is wired into the patch pipeline
The patch stage MUST call `sort_operations()` before applying operations. In Phase 5.5, this has no effect (identity sort) but ensures the module is integrated and tested.

#### Scenario: Patch stage calls sort before apply
- **WHEN** the patch stage processes operations
- **THEN** `sort_operations()` MUST be called before `apply_operations()`
- **AND** the result of `sort_operations()` is what gets applied
