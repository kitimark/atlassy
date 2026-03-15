## ADDED Requirements

### Requirement: Operations are sorted in reverse document order before application
The `sort_operations()` function MUST sort operations so that higher-index operations are applied before lower-index operations within the same parent array. Replace operations (leaf-path) MUST be applied before structural operations (Insert/Remove).

#### Scenario: Two inserts at same parent sorted by descending index
- **WHEN** operations include Insert at `/content/1` and Insert at `/content/4`
- **THEN** the insert at index 4 MUST appear before the insert at index 1 in the sorted output

#### Scenario: Replace operations precede structural operations
- **WHEN** operations include `Operation::Replace` and `Operation::Insert`
- **THEN** all Replace operations MUST appear before all Insert/Remove operations in the sorted output

#### Scenario: Same-index tie-break: remove before insert
- **WHEN** operations include Remove at `/content/2` and Insert at parent `/content` index 2
- **THEN** the Remove MUST appear before the Insert in the sorted output

#### Scenario: Operations across different parents are independent
- **WHEN** operations target different parent paths (e.g., `/content` and `/content/3/content`)
- **THEN** operations within each parent group MUST be independently sorted by descending index

### Requirement: Conflict detection rejects overlapping remove paths
The sort/validation step MUST detect when a Remove path is a prefix of another operation's path and reject the batch.

#### Scenario: Remove is prefix of another operation
- **WHEN** operations include Remove at `/content/2` and Replace at `/content/2/content/0/text`
- **THEN** the batch MUST be rejected as a conflict

#### Scenario: Non-overlapping remove and other operations
- **WHEN** operations include Remove at `/content/2` and Replace at `/content/0/content/0/text`
- **THEN** the batch MUST proceed without conflict

### Requirement: Empty operation list is valid
The sort function MUST handle an empty operation list without error.

#### Scenario: Empty operations
- **WHEN** `sort_operations()` is called with an empty list
- **THEN** it MUST return an empty list without error
