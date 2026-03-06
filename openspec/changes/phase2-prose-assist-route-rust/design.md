## Context

Phase 1 delivered deterministic orchestration, scoped fetch/indexing, patch guards, and replay artifacts, but prose route behavior remains effectively pass-through. Phase 2 adds practical prose editing while preserving v1 constraints: ADF-canonical pipeline, route isolation, mapped-path-only mutation, and verifier hard-fail on out-of-scope changes.

The core challenge is enabling markdown-assisted prose edits without introducing route leakage (table/locked nodes) or unstable mapping between markdown blocks and ADF paths. Existing contract and verifier rules already provide guardrails, so design should extend state behavior rather than alter state order.

## Goals / Non-Goals

**Goals:**
- Convert only `editable_prose` nodes into markdown assist blocks with stable `md_to_adf_map` path bindings.
- Apply prose edits only within mapped prose paths and enforce top-level type/boundary constraints.
- Preserve compatibility with existing `verify`/`patch` hard-safety checks.
- Add fixture coverage that proves non-regressive prose formatting and out-of-scope rejection.

**Non-Goals:**
- Table editing beyond existing v1 table route behavior.
- Any mutation path for `locked_structural` content.
- New pipeline states or state order changes.
- Multi-page orchestration, autonomous conflict resolution, or structural transformations.

## Decisions

1. Mapping identity uses canonical JSON Pointer ADF paths as `md_block_id` source of truth.
   - Decision: generate prose markdown blocks from classified `editable_prose` nodes and anchor each block to its ADF path in `md_to_adf_map`.
   - Rationale: path identity is already enforced elsewhere (`changed_paths`, scope checks), reducing translation complexity.
   - Alternative considered: synthetic markdown-only IDs with separate lookup table; rejected due to mismatch/debug risk across states.

2. `extract_prose` performs route-gated conversion, not generic subtree conversion.
   - Decision: only nodes explicitly labeled `editable_prose` are eligible for markdown extraction; table/locked nodes are skipped by construction.
   - Rationale: route gating at extraction stage prevents accidental downstream edit intent on ineligible nodes.
   - Alternative considered: convert broad content then filter later in `md_assist_edit`; rejected because leakage prevention should occur before transformation.

3. `md_assist_edit` emits candidate changes through mapped-path projection with boundary validation.
   - Decision: edited markdown blocks are projected back to original mapped paths only; block count/type expansion outside mapped paths is treated as hard error.
   - Rationale: keeps prose assist deterministic and aligned with verifier expectations for path-targeted changes.
   - Alternative considered: allowing structural markdown edits with later pruning; rejected due to ambiguity and high risk of non-prose drift.

4. Fidelity validation is fixture-first and path-scoped.
   - Decision: add fixtures covering prose-only and mixed-content pages, asserting non-regressive prose formatting and strict unchanged table/locked regions.
   - Rationale: fixture-first checks make regressions reproducible and measurable without depending on live systems.
   - Alternative considered: rely mostly on end-to-end smoke behavior; rejected because subtle formatting regressions are harder to isolate.

## Risks / Trade-offs

- [Markdown normalization alters benign whitespace/line wrapping] -> Mitigation: define canonical prose normalization expectations in fixtures and assert semantic equivalence where exact byte match is unreasonable.
- [Mapping instability when adjacent prose nodes are merged/split] -> Mitigation: enforce one-block-per-mapped-node behavior in v1 and fail fast on ambiguous remap scenarios.
- [Route classifier edge cases mislabel prose vs locked nodes] -> Mitigation: add negative fixtures with mixed macros/media/layout blocks and verify extraction exclusion.
- [Increased test matrix slows iteration] -> Mitigation: keep a minimal required fixture set for CI and separate extended fidelity suites for pre-release checks.

## Migration Plan

1. Tighten contract/state payload handling for `extract_prose` and `md_assist_edit` outputs if needed.
2. Implement route-gated prose extraction and deterministic mapping generation.
3. Implement mapped-path-only markdown edit projection with boundary/type constraints.
4. Extend integration fixtures and assertions for prose fidelity and out-of-scope safety.
5. Validate end-to-end no-op and scoped prose update flows with replay artifact inspection.

Rollback strategy:
- If prose route introduces unstable mapping or safety regressions, disable prose projection path and revert to Phase 1 pass-through behavior while preserving existing pipeline guardrails.

## Open Questions

- Should v1 prose fidelity checks use strict textual equality or semantic equivalence rules for whitespace-heavy markdown transforms?
- Do we need an explicit contract error code for ambiguous prose remap, or is `ERR_SCHEMA_INVALID` sufficient for Phase 2?
- What minimum mixed-content fixture set best represents real Confluence prose complexity without overfitting tests?
