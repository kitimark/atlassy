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

### D-004: Locked structural policy

- Decision: unsupported structural blocks remain locked in v1.
- Default set: media, macros/extensions, layouts, mentions, status, panels, embeds, and non-whitelisted nodes.
- Rationale: avoid lossy or unsafe edits until dedicated support exists.

### D-005: Patch strategy

- Decision: path-targeted ADF patch operations only.
- Rationale: minimize mutation scope and reduce conflict amplification.

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

## Default Route Matrix

- `editable_prose`: paragraph, heading, bulletList, orderedList, listItem, blockquote, simple codeBlock.
- `table_adf`: table node family, cell text edits only.
- `locked_structural`: rule, and all other nodes by default.

## Change Control

- Any expansion of `table_adf` beyond cell text requires a new decision entry and updated verifier rules.
- Any expansion of Markdown conversion scope requires explicit fidelity test evidence.
- Any relaxation of explicit lifecycle controls (implicit create or implicit bootstrap) requires a new decision entry and readiness evidence update.
