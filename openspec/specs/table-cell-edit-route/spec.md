## Purpose

Define v1 table route behavior for path-targeted table cell text updates while preserving table structure.

## Requirements

### Requirement: Table route supports cell text updates only
The `adf_table_edit` state SHALL emit table edit candidates only for `cell_text_update` operations and MUST NOT emit candidates for any other table operation type.

#### Scenario: Valid cell text update request
- **WHEN** edit intent targets text content within an existing table cell
- **THEN** `adf_table_edit` emits one or more candidates tagged as `cell_text_update`
- **AND** candidate paths resolve to table cell text nodes

### Requirement: Table candidates are path-targeted and deterministic
Table route candidates MUST be generated as path-targeted updates with deterministic ordering for the same input state and edit intent.

#### Scenario: Deterministic candidate emission
- **WHEN** the same table input and edit intent are processed repeatedly
- **THEN** emitted candidate path set is identical and lexicographically ordered across runs

### Requirement: Table updates preserve structure
Applying allowed table route candidates SHALL update cell text content only and MUST preserve table topology, row/column counts, and table-level attributes.

#### Scenario: Publish after allowed table edit
- **WHEN** only `cell_text_update` candidates are applied and verification passes
- **THEN** publish succeeds without structural drift in table nodes
