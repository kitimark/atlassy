## Context

Phase 6 added `Operation::Insert` and `Operation::Remove` as generic block-level primitives. The pipeline supports multi-op batches with reverse-document-order sorting. `BlockOp` has `Insert` and `Remove` variants that map 1:1 to `Operation` commands. The `adf_block_ops` state translates `BlockOp` → `Operation` with scope/type validation.

Phase 7 adds higher-level structural intents (sections, tables, lists) that expand into multiple Phase 6 primitives. The pipeline architecture (merge → sort → apply → verify) stays unchanged — the new logic lives entirely in `adf_block_ops` translation and new domain modules in `atlassy-adf`.

A Shotgun Surgery smell exists: `EDITABLE_PROSE_TYPES` is referenced in 6 files / 14 locations. Phase 7 needs to expand the type allowlist (add `table`), which would require changing all 14 locations without prior refactoring. The Rule of Three mandates extraction now.

## Goals / Non-Goals

**Goals:**

- Add InsertSection, RemoveSection, InsertTable, InsertList to BlockOp.
- Create ADF builder functions for constructing valid table, list, heading, and section ADF structures.
- Create section boundary detection logic for identifying all blocks in a section.
- Extract type policy into query functions to cure Shotgun Surgery smell before expanding allowlist.
- Add `table` to insertable/removable types.
- Prepare architecture for Phase 8 (multi-page) and Phase 9 (table topology) without building their features.

**Non-Goals:**

- New `Operation` enum variants — Phase 6's Insert/Remove are sufficient for Phase 7 composition.
- Template blocks system — builders ARE the templates (Speculative Generality prevention).
- Full locked_structural relaxation for container child insertion — first-time use, defer until needed.
- Multi-page orchestration (Phase 8).
- Table row/column topology operations (Phase 9).
- Stable-ID node addressing.

## Decisions

### D1: Builder Pattern for ADF node construction

ADF nodes are deeply nested JSON structures (table = 4 levels deep). Constructing them with raw `serde_json::json!` produces Duplicate Code and is error-prone. Builder functions follow Replace Constructor with Factory Method.

Builders are pure functions (Rust-idiomatic, not mutable builder classes) organized as atomic + composite:
- Atomic: `build_text(s)`, `build_paragraph(s)`, `build_heading(level, s)`
- Composite: `build_table(rows, cols, header_row)` → uses `build_text` internally
- Multi-node: `build_section(level, text, body_blocks)` → returns `Vec<Value>` (heading + body)

Phase 9 extends by adding `build_table_row()`, `build_table_cell()` composing existing atomics. No existing builder changes (Open/Closed).

### D2: Strategy Pattern for BlockOp expansion

Each BlockOp variant has its own `translate_*` function (Extract Method preventing Long Method smell). The match in `translate_block_op` dispatches to the right strategy:

- `translate_insert(...)` — existing, 1:1 mapping
- `translate_remove(...)` — existing, 1:1 mapping
- `translate_insert_section(...)` — uses builders, returns N Operation::Insert
- `translate_remove_section(...)` — uses section detection, returns N Operation::Remove
- `translate_insert_table(...)` — uses builders, returns 1 Operation::Insert
- `translate_insert_list(...)` — uses builders, returns 1 Operation::Insert

`translate_remove_section` requires `scoped_adf` access to detect section boundaries. The `run_adf_block_ops_state` function already receives `FetchOutput` — it passes `scoped_adf` to section-aware translators.

### D3: Section boundary detection as Extract Class

Section boundary detection is pure domain logic — "given an ADF doc and a heading path, find all blocks in that section." Extracted into `atlassy-adf/src/section.rs`.

Algorithm: parse heading_path → get parent array + index → read heading level from attrs.level → walk forward through siblings until next heading at same-or-higher level or end of array → return range.

Returns `SectionRange { heading_index, end_index, block_count, block_paths }`.

### D4: Type policy extraction (Rule of Three)

Replace 14 scattered `EDITABLE_PROSE_TYPES.contains(&t)` checks with query functions in a new `type_policy.rs` module (Replace Magic Number with Symbolic Constant + Separate Query from Modifier):

- `is_editable_prose(node_type) → bool` — the existing 7 prose types
- `is_insertable_type(node_type) → bool` — prose + table (Phase 7 expansion)
- `is_removable_type(node_type) → bool` — prose + table

All callers use functions instead of direct constant checks. Phase 9 adds `is_table_insertable()` / `is_table_removable()` without changing existing functions (Open/Closed).

This refactoring is Step 1 of Phase 7 execution — zero behavior change, all tests pass, BEFORE expanding the allowlist.

### D5: Locked boundary check extraction (Decompose Conditional)

Extract the locked structural check into `check_locked_boundary(operation, locked_paths)` function. Phase 7 implements basic rules (Replace overlap → blocked). The function signature is designed for Phase 8 (reuse per-page) and Phase 9 (add table-specific rules) extension.

Full op-type-aware relaxation (allowing child insertion inside containers) is deferred until there's a concrete use case. Phase 7's core operations target `doc.content`, not container children.

### D6: No Operation enum changes (Open/Closed Principle)

Phase 7 composes existing `Operation::Insert` and `Operation::Remove` into higher-level intents. "Insert section" = N Insert commands. "Remove section" = N Remove commands. The pipeline (merge → sort → apply → verify) handles multi-op batches correctly since Phase 6.

Phase 9 will add `InsertRow`, `RemoveColumn` etc. when table-internal semantics differ from block-level operations. Not needed now.

## Risks / Trade-offs

[Type policy refactoring blast radius] → 14 references across 6 files change from direct constant access to function calls. Mitigation: all existing tests validate behavioral equivalence. Execute as Step 1 with zero behavior change.

[Section boundary edge cases] → Sections with no body blocks (heading immediately followed by another heading), sections at the end of the document (no terminating heading), nested heading levels (H3 inside an H2 section). Mitigation: comprehensive test suite covering edge cases.

[BlockOp variant explosion] → 6 variants now, potentially more in Phase 9. Mitigation: each variant has its own translate function (Extract Method), preventing Long Method. The match statement is thin dispatch only.

[Builders may not cover all ADF schema constraints] → Tables need valid tableCell/tableHeader structure, lists need valid listItem nesting. Mitigation: structural validity check (from Phase 6) catches invalid ADF before publish. Builders are tested against known-valid ADF structures.
