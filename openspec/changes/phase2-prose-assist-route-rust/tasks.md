## 1. Contracts and State Payload Tightening

- [ ] 1.1 Update `extract_prose` contract payload structures to fully support deterministic `markdown_blocks` and `md_to_adf_map` outputs
- [ ] 1.2 Update `md_assist_edit` payload structures to enforce mapped-path-only `prose_changed_paths`
- [ ] 1.3 Add/adjust contract validation rules for mapping integrity (one `md_block_id` to one mapped path)
- [ ] 1.4 Add contract tests for deterministic serialization and required-field validation for updated prose-route payloads

## 2. Route-Gated Prose Extraction

- [ ] 2.1 Implement extraction filtering that converts only `editable_prose` nodes and skips `table_adf` and `locked_structural`
- [ ] 2.2 Implement deterministic `md_to_adf_map` generation keyed by canonical JSON Pointer paths
- [ ] 2.3 Add integrity checks that fail when markdown blocks and mapping entries are missing, duplicated, or inconsistent
- [ ] 2.4 Add unit tests for mixed-route extraction behavior and mapping determinism

## 3. Markdown Assist Boundary Enforcement

- [ ] 3.1 Implement mapped-path projection in `md_assist_edit` so edits can only target paths present in `md_to_adf_map`
- [ ] 3.2 Enforce top-level type and prose-boundary constraints for mapped prose nodes
- [ ] 3.3 Ensure unmapped or cross-route candidate edits trigger hard validation failure before publish
- [ ] 3.4 Add tests covering unmapped-path attempts, boundary expansion attempts, and route-isolation guarantees

## 4. Merge, Verify, and Safety Integration

- [ ] 4.1 Wire prose-route changed-path outputs into merge flow while preserving uniqueness and lexicographic sort guarantees
- [ ] 4.2 Ensure verifier path-scope checks correctly block out-of-scope prose mutations
- [ ] 4.3 Add regression tests proving prose assist cannot mutate table or locked-structural paths
- [ ] 4.4 Validate hard-error propagation behavior for prose-route contract and boundary failures

## 5. Fixture Coverage and Fidelity Validation

- [ ] 5.1 Add prose-only fixture cases that validate non-regressive formatting for mapped prose paths
- [ ] 5.2 Add mixed-content fixtures (prose + table + locked) to assert route isolation and unchanged non-prose content
- [ ] 5.3 Add fixture assertions for out-of-scope formatting drift rejection
- [ ] 5.4 Document fixture expectations for strict-vs-semantic formatting checks used in Phase 2 tests

## 6. CLI and End-to-End Verification

- [ ] 6.1 Extend CLI-driven integration flow to exercise scoped prose update mode using updated prose-route states
- [ ] 6.2 Add end-to-end tests for no-op and simple scoped prose update runs with replay artifact validation
- [ ] 6.3 Verify replay artifacts include expected prose-route state inputs/outputs and diagnostics for pass/fail runs
- [ ] 6.4 Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` and resolve any failures
