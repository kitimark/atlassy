## Why

Phase 1 established the v1 pipeline skeleton, but prose-route states are still placeholder behavior. We need Phase 2 now to make prose edits actually useful while preserving v1 safety boundaries (scope, route policy, and non-regressive formatting fidelity).

## What Changes

- Implement `extract_prose` so only `editable_prose` nodes are converted into markdown assist blocks.
- Implement stable markdown block to ADF path mapping and enforce deterministic mapping behavior across a run.
- Implement `md_assist_edit` to produce prose-only candidate changes constrained to mapped prose paths.
- Enforce prose boundary and top-level type constraints so markdown assist cannot expand into table or locked structural routes.
- Add fixture-backed validation for Phase 2 acceptance criteria (path safety, route isolation, and prose formatting non-regression).

## Capabilities

### New Capabilities
- `prose-extraction-and-mapping`: Extract markdown blocks from `editable_prose` only, with stable `md_to_adf_map` path bindings.
- `prose-assist-boundary-enforcement`: Apply markdown-assisted prose edits while enforcing mapped-path-only mutations and top-level type/boundary constraints.
- `prose-formatting-fidelity-checks`: Validate prose-route edits against fixtures to prevent formatting regressions in in-scope prose content.

### Modified Capabilities
- None.

## Impact

- Affected code: `crates/atlassy-pipeline` (`extract_prose`, `md_assist_edit`, merge inputs), `crates/atlassy-adf` (prose extraction helpers and mapping utilities), `crates/atlassy-contracts` (state payload tightening if needed), and CLI integration paths in `crates/atlassy-cli`.
- Tests and fixtures: expanded integration fixtures for prose-only and mixed-content pages in `crates/atlassy-pipeline/tests`.
- APIs/contracts: no state-order change; extends practical behavior of existing v1 `extract_prose` and `md_assist_edit` contract outputs.
- Systems/dependencies: no new external systems expected; continues Rust workspace defaults and existing replay artifact pipeline.
