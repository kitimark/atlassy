## ADDED Requirements

### Requirement: Ordering module exists with identity sort stub
The `atlassy-adf` crate MUST include an `ordering` module with a public `sort_operations()` function. In Phase 5.5, this function SHALL return operations in the same order they were received (identity sort).

#### Scenario: sort_operations returns input unchanged
- **WHEN** `sort_operations()` is called with a list of `Operation::Replace` values
- **THEN** it MUST return the operations in the same order as the input

#### Scenario: sort_operations handles empty input
- **WHEN** `sort_operations()` is called with an empty list
- **THEN** it MUST return an empty list without error

### Requirement: Ordering module is wired into the patch pipeline
The patch stage MUST call `sort_operations()` before applying operations. In Phase 5.5, this has no effect (identity sort) but ensures the module is integrated and tested.

#### Scenario: Patch stage calls sort before apply
- **WHEN** the patch stage processes operations
- **THEN** `sort_operations()` MUST be called before `apply_operations()`
- **AND** the result of `sort_operations()` is what gets applied
