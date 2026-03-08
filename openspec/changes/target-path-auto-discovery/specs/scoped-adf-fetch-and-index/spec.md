## ADDED Requirements

### Requirement: Editable prose types constant

The `atlassy-adf` crate SHALL expose a public constant `EDITABLE_PROSE_TYPES` containing the 7-type prose whitelist: `paragraph`, `heading`, `bulletList`, `orderedList`, `listItem`, `blockquote`, `codeBlock`. This constant SHALL be the single source of truth for editable prose type classification across all crates.

#### Scenario: Constant is accessible from pipeline crate

- **WHEN** `atlassy-pipeline` needs the prose type whitelist for `route_for_node()`
- **THEN** it references `EDITABLE_PROSE_TYPES` from `atlassy-adf` instead of an inline match pattern

#### Scenario: Constant matches existing route classification

- **WHEN** `route_for_node()` is refactored to use `EDITABLE_PROSE_TYPES`
- **THEN** the route classification behavior is identical to the previous inline match pattern for all 7 types
