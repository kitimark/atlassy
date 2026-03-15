## MODIFIED Requirements

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
