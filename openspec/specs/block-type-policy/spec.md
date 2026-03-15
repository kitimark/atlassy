## Purpose

Define centralized block type policy helpers for insert/remove/edit legality checks.

## Requirements

### Requirement: Type policy module centralizes block type checking
The `atlassy-adf` crate SHALL provide a `type_policy` module with query functions that replace all scattered `EDITABLE_PROSE_TYPES.contains()` checks in the codebase.

#### Scenario: is_editable_prose returns true for prose types
- **WHEN** `is_editable_prose("paragraph")` is called
- **THEN** it MUST return `true`
- **AND** the same for: heading, bulletList, orderedList, listItem, blockquote, codeBlock

#### Scenario: is_editable_prose returns false for non-prose types
- **WHEN** `is_editable_prose("table")` is called
- **THEN** it MUST return `false`

#### Scenario: is_insertable_type includes prose and structural types
- **WHEN** `is_insertable_type("paragraph")` is called
- **THEN** it MUST return `true`
- **AND** `is_insertable_type("table")` MUST return `true`

#### Scenario: is_insertable_type rejects locked types
- **WHEN** `is_insertable_type("panel")` is called
- **THEN** it MUST return `false`

#### Scenario: is_removable_type matches insertable types
- **WHEN** `is_removable_type(t)` is called for any type
- **THEN** it MUST return the same result as `is_insertable_type(t)`

### Requirement: All callers use type policy functions instead of direct constant checks
After Phase 7 refactoring, no source file outside of `type_policy.rs` SHALL directly reference `EDITABLE_PROSE_TYPES` for type checking. All type decisions MUST go through `is_editable_prose()`, `is_insertable_type()`, or `is_removable_type()`.

#### Scenario: No direct EDITABLE_PROSE_TYPES.contains outside type_policy
- **WHEN** the codebase is searched for `EDITABLE_PROSE_TYPES.contains`
- **THEN** only `type_policy.rs` MUST contain such references
- **AND** all other files MUST use the query functions

### Requirement: EDITABLE_PROSE_TYPES constant remains available for classification
The `EDITABLE_PROSE_TYPES` constant SHALL remain publicly exported from `atlassy-adf` for use in route classification (`classify.rs`). The constant itself does not change - only the type-checking pattern changes from direct `.contains()` to function calls.

#### Scenario: classify.rs continues to use EDITABLE_PROSE_TYPES for routing
- **WHEN** route classification assigns `editable_prose` route
- **THEN** it MAY use `EDITABLE_PROSE_TYPES` directly or use `is_editable_prose()` - both are valid for classification

### Requirement: Type policy supports table-internal and attr-editable types
The type policy module MUST provide additional query functions for Phase 9 operations.

#### Scenario: tableRow is insertable/removable
- **WHEN** `is_insertable_type("tableRow")` is called
- **THEN** it MUST return `true` (for row operations within tables)

#### Scenario: is_attr_editable_type for panels
- **WHEN** `is_attr_editable_type("panel")` is called
- **THEN** it MUST return `true`

#### Scenario: is_attr_editable_type for expand
- **WHEN** `is_attr_editable_type("expand")` is called
- **THEN** it MUST return `true`

#### Scenario: is_attr_editable_type for mediaSingle
- **WHEN** `is_attr_editable_type("mediaSingle")` is called
- **THEN** it MUST return `true`

#### Scenario: is_attr_editable_type rejects non-allowed types
- **WHEN** `is_attr_editable_type("paragraph")` is called
- **THEN** it MUST return `false`
