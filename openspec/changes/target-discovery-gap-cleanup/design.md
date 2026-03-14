## Context

The target path auto-discovery feature (implemented in Phase 5) has three minor gaps identified during code review. The `SimpleScopedUpdate` RunMode variant is dead code — unused in any integration test or QA manifest, fully superseded by route-specific variants. The `to_hard_error()` error mapping contains a dead arm that maps `TargetDiscoveryFailed` to the wrong error code. The table cell auto-discovery path lacks end-to-end integration test coverage despite being structurally identical to the tested prose path.

All three gaps are low-risk and independent. The code changes span `atlassy-pipeline` and `atlassy-cli` crates.

## Goals / Non-Goals

**Goals:**
- Remove `SimpleScopedUpdate` variant from `RunMode` and all associated plumbing (Dead Code removal per refactoring.guru)
- Fix `to_hard_error()` fallthrough mapping so `AdfError::TargetDiscoveryFailed` maps to `ErrorCode::TargetDiscoveryFailed` (Replace Magic Number with Symbolic Constant per refactoring.guru)
- Add integration test for table cell auto-discovery to achieve parity with prose route coverage

**Non-Goals:**
- Refactoring the `From<AdfError> for PipelineError` impl (it hardcodes `PipelineState::Patch` — fixing that would require a larger redesign of error origin tracking)
- Adding new RunMode variants or capabilities
- Changing any runtime behavior for existing routes

## Decisions

### Decision 1: Remove `SimpleScopedUpdate` entirely rather than extracting the magic string to a constant

**Choice**: Remove the variant, its match arms, CLI/manifest plumbing.

**Alternatives considered**:
- *Extract `"/content/1/content/0/text"` to a named constant*: Addresses Duplicate Code and Primitive Obsession but preserves dead code. The variant has no integration tests, no QA manifest usage, and is fully superseded.
- *Deprecate with a comment, remove later*: Delays cleanup without benefit — no consumers exist.

**Rationale**: The route-specific variants (`SimpleScopedProseUpdate`, `SimpleScopedTableCellUpdate`) cover all production use cases and support auto-discovery. `SimpleScopedUpdate` cannot support auto-discovery due to route ambiguity. Removing it simplifies match arms in both pipeline states and eliminates the only hardcoded default path in the codebase.

### Decision 2: Fix `to_hard_error()` match arm to use correct ErrorCode rather than marking it unreachable

**Choice**: Change `AdfError::TargetDiscoveryFailed { .. } => ErrorCode::SchemaInvalid` to `AdfError::TargetDiscoveryFailed { .. } => ErrorCode::TargetDiscoveryFailed`.

**Alternatives considered**:
- *Use `unreachable!()` macro*: Would panic at runtime if the dead arm is ever reached. Too aggressive for a mapping function — better to produce a correct (if incomplete) error than crash.
- *Leave as-is with a comment*: Preserves the misleading mapping. Future contributors reading `to_hard_error()` would reasonably assume `SchemaInvalid` is the canonical mapping for discovery errors.
- *Remove `From<AdfError>` impl entirely*: Would force all `AdfError` handling to use explicit `map_err`. Correct but high-churn — every `?` on an `AdfError` in pipeline states would need rewriting.

**Rationale**: The fix is one line. Even though the arm is unreachable (discovery errors use explicit `map_err`), having the correct code makes the mapping self-documenting and defensively correct. If a future code path accidentally routes through `From<AdfError>`, it gets the right error code (though wrong state — that's the `From` impl's limitation).

### Decision 3: Model table cell integration test on existing prose auto-discovery test

**Choice**: Add a single test `pipeline_auto_discovers_table_cell_and_patches` modeled on the existing `pipeline_auto_discovers_and_patches` test, using the existing `table_allowed_cell_update_adf.json` fixture.

**Rationale**: The fixture already contains a heading + 2-cell table row. The expected discovered path is `/content/1/content/0/content/0/content/0/content/0/text` (first cell's text node). This is the same path used by the explicit-path test `table_cell_update_run_succeeds`, confirming fixture suitability.

## Risks / Trade-offs

**[Risk: Manifest backward compatibility]** Any existing manifest JSON files using `"mode": "simple_scoped_update"` would fail at serde deserialization after `ManifestMode::SimpleScopedUpdate` is removed. **Mitigation**: Grep confirms zero manifest files use this mode across `qa/`, `openspec/`, and the entire repository.

**[Risk: CLI backward compatibility]** Users running `--mode simple-scoped-update` would get an "invalid CLI mode" error. **Mitigation**: The CLI mode string is only wired through `CliMode` enum and `execute_run_command()` match — both are internal. No documented external CLI contracts reference this mode.

**[Trade-off: `From<AdfError>` still hardcodes `PipelineState::Patch`]** Fixing the error code in `to_hard_error()` is a partial fix. If discovery errors ever flow through `From<AdfError>`, they'd get the right code but wrong state (`Patch` instead of `MdAssistEdit`/`AdfTableEdit`). This is accepted — the explicit `map_err` pattern remains the correct approach and is mandated by the spec.
