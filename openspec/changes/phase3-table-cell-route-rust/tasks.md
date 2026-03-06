## 1. Contracts and Table Route Types

- [ ] 1.1 Extend table-route contract payloads to model allowlisted table operations and candidate metadata
- [ ] 1.2 Add explicit operation typing that permits `cell_text_update` only for v1 table route outputs
- [ ] 1.3 Add contract validation rules for table candidate path structure and deterministic ordering
- [ ] 1.4 Add contract tests for table payload serialization stability and required-field validation

## 2. `adf_table_edit` Allowlisted Candidate Generation

- [ ] 2.1 Implement `adf_table_edit` candidate generation for table cell text paths only
- [ ] 2.2 Enforce deterministic candidate ordering for identical input state and edit intent
- [ ] 2.3 Ensure candidate generation excludes non-table and locked-structural node paths
- [ ] 2.4 Add unit tests for valid table cell text update candidate emission

## 3. Table Shape and Attribute Guardrails

- [ ] 3.1 Implement guard logic that rejects row/column add-remove requests with `ERR_TABLE_SHAPE_CHANGE`
- [ ] 3.2 Implement guard logic that rejects merge/split or topology-changing table mutations
- [ ] 3.3 Implement guard logic that rejects table-level and structural attribute updates in v1
- [ ] 3.4 Add negative tests proving forbidden table operations fail before publish

## 4. Merge and Conflict Safety

- [ ] 4.1 Extend merge logic to enforce uniqueness across prose and table changed paths
- [ ] 4.2 Add fast-fail checks for cross-route collisions between table candidates and prohibited boundaries
- [ ] 4.3 Ensure merge output remains lexicographically sorted and verifier-compatible when conflict-free
- [ ] 4.4 Add merge tests for duplicate path collisions and valid mixed-route merges

## 5. Verify and Structural Drift Protection

- [ ] 5.1 Extend verify checks to detect unauthorized table topology changes and emit `ERR_TABLE_SHAPE_CHANGE`
- [ ] 5.2 Extend verify checks to detect forbidden table attribute drift and block publish
- [ ] 5.3 Add fixture assertions that allowed table cell edits preserve row/column counts and table structure
- [ ] 5.4 Add regression tests proving locked-structural paths remain unchanged in table edit flows

## 6. End-to-End Coverage and Validation

- [ ] 6.1 Add table-focused fixtures for allowed cell text updates and forbidden shape/attribute operations
- [ ] 6.2 Add integration tests covering successful table cell updates and blocked forbidden operations
- [ ] 6.3 Validate replay artifacts include table-route state inputs, outputs, and diagnostics for pass/fail runs
- [ ] 6.4 Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` and resolve issues
