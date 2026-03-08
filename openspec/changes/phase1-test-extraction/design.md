## Context

The workspace has 5 crates, each with a single `src/` file containing an inline `mod tests { ... }` block. Private-access analysis determined that 3 crates require `src/`-resident test files (private API or binary crate barrier), while 2 crates can use external `tests/` directories (public API only). The workspace dependency graph (leaf crates → pipeline → CLI) does not constrain this phase since test extraction has no cross-crate coupling.

Current state:

| Crate | File | Inline tests | Lines | Private access? |
|---|---|---|---|---|
| `atlassy-adf` | `src/lib.rs` | 42 | 622-1408 | No |
| `atlassy-contracts` | `src/lib.rs` | 10 | 662-997 | No |
| `atlassy-confluence` | `src/lib.rs` | 7 | 494-643 | Yes (2 private methods) |
| `atlassy-pipeline` | `src/lib.rs` | 3 | 1529-1587 | Yes (1 private fn) |
| `atlassy-cli` | `src/main.rs` | 15 | 2789-3380 | Yes (binary crate + ~13 private items) |

## Goals / Non-Goals

**Goals:**

- Remove all long inline `mod tests { ... }` blocks from production files.
- Place tests in the correct location based on access requirements (private → `src/`, public → `tests/`).
- Preserve exact test behavior — identical assertions, identical test names.
- Maintain all quality gates (`fmt`, `clippy`, `test`).

**Non-Goals:**

- No module restructuring (Phase 2-4 scope).
- No `pub` visibility widening to accommodate test moves.
- No shared test helper abstractions for ADF fixture construction.
- No new tests. No deleted tests. No modified assertions.
- No CI gate enforcement (grep-based manual check for now).

## Decisions

### 1. Placement: `src/tests.rs` vs `tests/` per crate

**Decision:** Place tests based on whether they access private items.

| Crate | Destination | Rationale |
|---|---|---|
| `atlassy-pipeline` | `src/tests.rs` | `compute_section_bytes` is private. 3 tests too small to split. |
| `atlassy-confluence` | `src/tests.rs` | `build_publish_payload` and `build_create_payload` are private methods. 7 tests too small to split public from private. |
| `atlassy-contracts` | `tests/contract_validation.rs` | All 10 tests use only `pub` API. |
| `atlassy-adf` | `tests/*.rs` (5 files) | All 42 tests use only `pub` API. Domain grouping aids navigation. |
| `atlassy-cli` | `src/tests.rs` | Binary crate — `tests/` cannot import from `main.rs` at all. All 15 tests use private internals. |

**Alternative considered:** Make private items `pub(crate)` to allow `tests/` placement. Rejected — violates the "no visibility widening" constraint and creates tech debt that Phase 2-4 would inherit.

### 2. ADF test grouping (42 tests → 5 files)

**Decision:** Group by functional domain, not by the function under test.

| File | Count | Domain |
|---|---|---|
| `scope_resolution.rs` | 15 | `resolve_scope` — heading selectors, block selectors, fallbacks, multi-selector union |
| `target_discovery.rs` | 8 | `discover_target_path` — prose/table discovery, scope boundary, error cases |
| `patch_ops.rs` | 3 | `build_patch_ops`, `apply_patch_ops`, `canonicalize_mapped_path` |
| `path_classification.rs` | 4 | `is_table_cell_text_path`, `is_table_shape_or_attr_path`, `document_order_sort`, type-set invariant |
| `emptiness_bootstrap.rs` | 12 | `is_page_effectively_empty` (9 variants), `bootstrap_scaffold` |

**Alternative considered:** Single `tests/adf_tests.rs` file. Rejected — 42 tests in one file recreates the readability problem we're solving.

### 3. No shared ADF fixture helpers

**Decision:** Keep inline `serde_json::json!(...)` construction duplicated across test files.

**Rationale:** ADF tests are testing operations on specific document structures. The inline JSON IS the test specification — abstracting it behind builder functions (`heading("Overview")`, `paragraph("Body")`) would hide the structure being tested and force readers to mentally reconstruct it. Only 2 tests use the deeply-nested table structure; duplication cost is low.

### 4. CLI test helpers extract to `src/test_helpers.rs`

**Decision:** Move `fixture_path` and `execute_batch_from_manifest_file` to a separate `#[cfg(test)]` module file.

**Rationale:** These are logic (path construction, stub backend wiring), not test data. They're called by multiple tests and would clutter `tests.rs` if inlined. The `#[cfg(test)]` gate at `main.rs:664` for `execute_batch_from_manifest_file` moves cleanly to this file.

`main.rs` will reference both:
```rust
#[cfg(test)] mod test_helpers;
#[cfg(test)] mod tests;
```

### 5. Extraction order

**Decision:** pipeline → confluence → contracts → adf → cli

**Rationale:** Smallest-first. Pipeline (3 tests) and confluence (7 tests) are trivial warm-ups that prove the pattern. Contracts (10) is a clean public-API move. ADF (42) is the largest but mechanically straightforward. CLI (15) is last because it has the most structural complexity (binary crate, separate helpers file, `#[cfg(test)]` function relocation).

### 6. One commit per crate, one PR

**Decision:** Single PR with 5 atomic commits (one per crate extraction).

**Rationale:** Each commit is independently revertible if a crate's extraction causes issues. Single PR keeps review cohesive for a mechanical change. Verification runs after each commit and at PR level.

## Risks / Trade-offs

- **Risk:** Test accidentally dropped during move (wrong count after extraction).
  **Mitigation:** `grep -r '#\[test\]' crates/ --include='*.rs' | wc -l` must equal 107 before and after. Run per-commit.

- **Risk:** `use super::*` import breaks when test moves to new file location.
  **Mitigation:** `src/tests.rs` files keep `use super::*`. `tests/` files switch to explicit `use atlassy_<crate>::*` imports. Compiler catches any missing import immediately.

- **Risk:** CLI `#[cfg(test)]` helper function at `main.rs:664` is missed during extraction.
  **Mitigation:** Explicitly tracked — moves to `src/test_helpers.rs`. Verified by `grep -rn 'mod tests {' crates/*/src/main.rs` returning 0 matches.

- **Risk:** ADF test grouping puts a test in the wrong domain file.
  **Mitigation:** Low consequence — test still runs, just lives in a less intuitive file. Can be moved later without any code change.
