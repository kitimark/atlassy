# Decisions and Defaults

## Decision Log (v1)

### D-001: Canonical representation

- Decision: ADF is canonical across fetch, patch, verify, and publish.
- Rationale: preserves Confluence-native fidelity and full feature coverage.

### D-002: Markdown usage policy

- Decision: Markdown is a transient assist format for `editable_prose` only.
- Rationale: improves human readability while avoiding structural feature drift.

### D-003: Table editing scope

- Decision: tables are editable in v1 via ADF-native path, cell text only.
- Rationale: balances usability gains with low structural risk.
- Deferred: row/column operations and full restructuring are tracked in `ideas/2026-03-advanced-table-editing-modes.md`.
- Structural phases note: table topology changes (row/column add/remove) are scheduled for Phase 9 (D-017, D-018).

### D-004: Locked structural policy

- Decision: unsupported structural blocks remain locked in v1.
- Default set: media, macros/extensions, layouts, mentions, status, panels, embeds, and non-whitelisted nodes.
- Rationale: avoid lossy or unsafe edits until dedicated support exists.
- Structural phases note: `locked_structural` policy will be relaxed for block-level insert/delete operations (not attribute editing) in Phase 7. Container nodes become insert/delete targets while preserving their wrapper structure.

### D-005: Patch strategy

- Decision: path-targeted ADF patch operations only.
- Rationale: minimize mutation scope and reduce conflict amplification.
- Structural phases note: patch strategy expands to include `Insert` and `Remove` operations in Phase 6 (D-017). Reverse-document-order processing maintains path stability (D-020).

### D-006: Verification gates

- Decision: publish is blocked unless all checks pass.
- Required checks: ADF schema validity, locked-node fingerprint preservation, no out-of-scope mutation, route-policy compliance.

### D-007: Conflict policy

- Decision: one scoped rebase retry on version conflict, then fail fast.
- Rationale: limits retry token waste and prevents hidden repeated mutation.

### D-008: v1 implementation stack

- Decision: implement v1 pipeline runtime in Rust as a CLI-first workspace with reusable core libraries.
- Default components: `clap` (CLI), `tokio` + `reqwest` (async Confluence integration), `serde` + `serde_json` (state contracts), `tracing` (structured diagnostics), `thiserror` (error taxonomy), and `cargo test` for fixture-backed verification.
- Rationale: strong type safety, deterministic state handling, and predictable performance for ADF-heavy patch/verify flows.
- Constraint: product defaults remain unchanged (ADF-canonical flow, route policy, verifier hard gates, and one-retry conflict policy).
- Note: Rust toolchain installation is a readiness prerequisite before Phase 1 implementation starts.

### D-009: Confluence testing mode policy

- Decision: use live Confluence research in a dedicated sandbox space to capture real API behavior, then run CI and regression suites against a deterministic stubbed Confluence service.
- Live scope: controlled read/write probes for fetch, publish, version conflicts, and representative error responses.
- Stub scope: scenario-driven simulation for happy path, retry exhaustion, schema errors, route/scope violations, and transient service failures.
- Rationale: preserve realism for behavior discovery while keeping automated test runs stable, fast, and reproducible.

### D-010: Runtime artifact retention policy

- Decision: `artifacts/` is a temporary execution output directory and is not versioned in git.
- Scope: applies to `run`, `run-batch`, and `run-readiness` outputs.
- Rationale: avoid repository noise and stale generated evidence.

### D-011: Evidence provenance policy

- Decision: decision-grade KPI/readiness evidence must include commit provenance.
- Required metadata: `git_commit_sha` (full 40-character SHA), `git_dirty`, and `pipeline_version`.
- Rationale: regenerated outputs can change across implementation revisions; provenance is required for defensible comparisons.

### D-012: Sub-page creation policy

- Decision: v1 includes command-first page creation via `create-subpage`.
- Default behavior: create a truly blank child page under an explicit parent page ID.
- Constraint: standard `run` flow must not create pages implicitly.
- Rationale: enables repeatable end-to-end release testing while keeping side effects explicit.

### D-013: Empty-page bootstrap policy

- Decision: first prose edit on an empty page requires explicit `--bootstrap-empty-page`.
- Default behavior matrix:
  - empty page + no bootstrap flag -> hard fail (`ERR_BOOTSTRAP_REQUIRED`)
  - empty page + bootstrap flag -> bootstrap minimal prose scaffold, then apply edit
  - non-empty page + bootstrap flag -> hard fail (`ERR_BOOTSTRAP_INVALID_STATE`)
  - non-empty page + no bootstrap flag -> unchanged flow
- Rationale: preserve deterministic safety behavior while enabling first-write lifecycle support.

### D-014: KPI framework revision for CLI-first PoC

- Decision: replace legacy v1 KPI framing with MCP-predictive metrics while remaining CLI-first for implementation.
- Retired metrics: `tokens_per_successful_update`, `full_page_retrieval_rate`, `retry_conflict_token_waste`, `formatting_fidelity_pass_rate`.
- Adopted metrics: `context_reduction_ratio`, `scoped_section_tokens`, `edit_success_rate`, `structural_preservation`, `conflict_rate`, `publish_latency`.
- Baseline rule: benchmark baseline uses empty `scope_selectors` (full-page fallback path).
- Optimized rule: benchmark optimized runs must include explicit heading/block selectors.
- Rationale: legacy metrics overemphasized internal pipeline payload accounting and underrepresented the real-world AI editing problem (context pressure, scoped reliability, and structure safety).

### D-015: Heading selector matching policy

- Decision: `heading:` scope selectors use exact text matching by default.
- Prior behavior: `find_heading_paths()` used substring matching (`text.contains(heading_text)`), which caused `heading:View` to match a heading titled "Overview".
- Rationale: enterprise content commonly has headings with shared prefixes (e.g., "Introduction", "Introduction to Setup"). Exact matching prevents silent incorrect scoping.
- Constraint: substring matching was not triggered during KPI revalidation (heading names were chosen to avoid overlaps), but remains a latent risk for uncontrolled content.
- Discovery: `ideas/2026-03-scope-resolution-quality.md`, `qa/investigations/2026-03-08-kpi-revalidation.md`.

### D-016: Page content seeding policy

- Decision: v1 includes a `seed-page` command for publishing arbitrary ADF JSON to an existing page, bypassing the pipeline safety envelope.
- Design: explicit opt-in command (not part of `run`), page must already exist, full-body ADF replacement, validate ADF syntax before publish, require `--runtime-backend live`.
- Constraint: `seed-page` is a setup/QA tool, not an editing tool. No verify gates, no route classification, no scope enforcement.
- Rationale: KPI experiment page setup requires structural variety (tables, expand macros, locked blocks). Manual Confluence UI editing does not scale and is non-reproducible. The underlying `publish_page()` capability exists but was unexposed.
- Discovery: `ideas/2026-03-raw-adf-page-seeding.md`.

## Decision Log (Structural Phases)

### D-017: Operation type consolidation strategy (revised)

- Decision: consolidate `PatchCandidate` (from `atlassy-adf`), `PatchOperation` (from `atlassy-adf`), and `PatchOp` (from `atlassy-contracts`) into a single `Operation` enum in `atlassy-contracts`. Phase 5.5 introduces the enum with `Operation::Replace { path, value }` only (zero behavior change). Phase 6 adds `Operation::Insert { parent_path, index, block }` and `Operation::Remove { target_path }` variants.
- Rationale: the three existing types carry the same concept but are scattered across two crates (Shotgun Surgery smell per refactoring.guru). A single typed enum eliminates primitive obsession (`op: String`), removes redundant type conversions, and ensures compile-time safety for different mutation semantics. Each variant carries exactly the data it needs — no `Option<Value>` hacks or unused fields.
- Constraint: `Insert` and `Remove` require post-mutation ADF schema validation. `Replace` does not (it preserves structure by construction). The `Operation` enum is the single type flowing from candidate construction through merge, patch, and verify.
- Phase: 5.5 (type introduction), 6 (Insert/Remove variants).

### D-018: Block insert/delete scope (Phase 6)

- Decision: Phase 6 enables insert/delete for all `editable_prose` types (paragraph, heading, bulletList, orderedList, listItem, blockquote, codeBlock). Table structure insert and locked structural block insert are deferred to Phase 7+.
- Rationale: start with the least risky structural changes. Prose blocks have the simplest ADF schema constraints and the most predictable insertion/deletion semantics. Phase 7 extends to tables and containers once the foundation is validated.
- Constraint: scope anchor headings (headings used in `scope_selectors`) are blocked from deletion unless explicit re-scoping is provided.

### D-019: Revised KPI framework for structural operations

- Decision: replace Foundation KPI framework with Structural metrics as the primary pass/fail gate. Retire `context_reduction_ratio` and `scoped_section_tokens` as hard targets. Adopt `operation_success_rate`, `schema_validity_rate`, `operation_precision`, `structural_integrity`, `conflict_rate`, `publish_latency`.
- Rationale: Foundation KPIs measured "how much less context do we fetch for text edits." Structural KPIs measure "do structural operations work correctly and safely." The context reduction metric becomes less meaningful when the goal is content modification rather than minimal-change editing.
- Phase: 6+.

### D-020: Reverse-order patch processing for insert/delete

- Decision: process insert and delete patch operations in reverse document order (highest index first) to prevent cascading index shifts.
- Rationale: avoids the complexity of a full stable-ID addressing refactor while enabling correct multi-operation scenarios. The current JSON Pointer addressing system is positional (array indices); inserting at `/content/2` would invalidate all subsequent sibling paths. Processing bottom-up eliminates this problem for non-overlapping operations.
- Constraint: operations within the same parent array must be sorted by descending index before application. Overlapping operations (insert and delete at the same position) require explicit ordering rules.
- Implementation: ordering logic lives in `atlassy-adf/src/ordering.rs` as a pure function `sort_operations()`. Algorithm: (1) partition into leaf replaces and structural ops; (2) replaces first (they don't shift indices); (3) structural ops grouped by parent path, descending index; (4) at same index: remove before insert. Conflict detection: reject if a remove path is a prefix of another operation's path.
- Upgrade path: if reverse-order processing proves insufficient for complex multi-operation scenarios, stable-ID-based node addressing can be introduced as a future decision.
- Phase: 5.5 (module stub with identity sort), 6 (full algorithm).

### D-021: Preparatory refactoring strategy

- Decision: apply structural refactoring before feature implementation, following refactoring.guru principles. Phase 5.5 performs type consolidation, method extraction, and interface preparation with zero behavior change, ensuring Phase 6 feature work builds on a clean type system.
- Rationale: the current codebase has three code smells that would compound if Phase 6 features were bolted on: Primitive Obsession (`op: String`), Shotgun Surgery (operation concept split across 3 types in 2 crates), and Divergent Change (`verify.rs` handling too many concerns). Refactoring first follows the principle: "when you need to add a feature, refactor the code first to make adding the feature easy, then add the feature."
- Refactoring techniques applied: Replace Type Code with Class (Operation enum), Inline Class (drop PatchCandidate/PatchOp), Extract Method (verify check functions), Add Parameter (block_ops on RunRequest).
- Constraint: Phase 5.5 must produce zero behavior change. All 159 existing tests must pass identically. `Operation::Replace` must produce byte-identical output to the previous `PatchOperation { op: "replace" }`.
- Phase: 5.5.

## Default Route Matrix

Authoritative ADF schema reference: [http://go.atlassian.com/adf-json-schema](http://go.atlassian.com/adf-json-schema) (43 node types, 16 mark types as of 2026-03).

### `editable_prose` (7 types)

paragraph, heading, bulletList, orderedList, listItem, blockquote, codeBlock.

### `table_adf` (4 types)

table, tableRow, tableCell, tableHeader.

v1 scope: cell text updates only. Row/column topology changes are forbidden (`ERR_TABLE_SHAPE_CHANGE`).

### `locked_structural` (32 types, catch-all)

All node types not listed above fall to `locked_structural` via the catch-all arm in `route_for_node()`.

| Category | Node Types |
|---|---|
| Containers | panel, expand, nestedExpand |
| Media | mediaSingle, mediaGroup, media, mediaInline, caption |
| Extensions / Macros | extension, bodiedExtension, inlineExtension |
| Smart Cards | blockCard, embedCard, inlineCard |
| Layouts | layoutSection, layoutColumn |
| Tasks / Decisions | taskList, taskItem, blockTaskItem, decisionList, decisionItem |
| Live Pages | syncBlock, bodiedSyncBlock |
| Inline | text, hardBreak, date, emoji, mention, status, placeholder |
| Divider | rule |
| Root | doc (never classified; structural root only) |

Structural phases note: `locked_structural` applies to text *edit* operations in Foundation phases. Phase 7 relaxes the lock for block-level *insert/delete* operations (not attribute editing). Container nodes become targets for inserting new child blocks or removing existing ones, while the container wrapper itself is preserved.

### Container routing note

Container nodes (panel, expand, nestedExpand, layoutSection, layoutColumn, bodiedExtension) are themselves `locked_structural`, but their child nodes are routed individually. A paragraph inside a panel is classified as `editable_prose`; a table inside an expand is classified as `table_adf`. The container wrapper is preserved unchanged; only the inner content is eligible for editing.

### Marks (16 types, all opaque)

alignment, annotation, backgroundColor, border, breakout, code, dataConsumer, em, fragment, indentation, link, strike, strong, subsup, textColor, underline.

Marks are never read, modified, or validated by the pipeline. Patch operations target `text` property values only, so marks on text nodes and block-level marks on containers are preserved by construction. This is correct for v1 text-replacement scope.

### Known gaps

- `tableHeader` is not yet explicitly referenced in the Rust code (works via `has_table_ancestor` fallback). Tracked for code fix.
- `taskList` / `decisionList` contain editable text but are locked in v1. Tracked in `ideas/2026-03-structural-block-editing-support.md`.
- Heading scope resolution only works for top-level headings in `doc.content[]`. Headings nested inside containers (panel, expand, layoutColumn, tableCell) trigger full-page fallback.

## Change Control

- Any expansion of `table_adf` beyond cell text requires a new decision entry and updated verifier rules.
- Any expansion of Markdown conversion scope requires explicit fidelity test evidence.
- Any relaxation of explicit lifecycle controls (implicit create or implicit bootstrap) requires a new decision entry and readiness evidence update.
- Any expansion of patch operations beyond `Replace` requires D-017 implementation and post-mutation schema validation (Gate 8).
- Any relaxation of `locked_structural` for block-level operations requires Phase 7 readiness gates (Gate 9).
- Any multi-page operation requires Phase 8 rollback infrastructure and orchestration safety gates (Gate 10).
