## Context

Phase 1 establishes the executable foundation for Atlassy v1: a Rust pipeline that runs the defined state order with deterministic contracts, scoped ADF handling, strict patch safety, and replay artifacts. The proposal defines three new capabilities (`pipeline-state-orchestration`, `scoped-adf-fetch-and-index`, `patch-guard-and-replay-artifacts`) and aligns with roadmap constraints: ADF-canonical flow, route policy enforcement, verifier hard gates, and one-retry publish conflict policy.

The current state is planning-complete but runtime-empty. Without a stable skeleton, later phases (prose assist and table route) would duplicate orchestration and diagnostics logic, increasing delivery risk and making KPI comparisons noisy.

## Goals / Non-Goals

**Goals:**
- Provide a typed, deterministic orchestrator for v1 state transitions with fail-fast hard-error behavior.
- Establish reusable state envelopes and validation boundaries that match the AI contract.
- Implement scoped fetch/index and patch guard primitives needed by downstream routes.
- Persist replay artifacts per state and per run for reproducibility and triage.
- Keep the skeleton integration-ready for Phase 2 and Phase 3 without breaking v1 defaults.

**Non-Goals:**
- Implementing full prose transformation behavior in `md_assist_edit`.
- Implementing advanced table operations beyond `cell_text_update`.
- Expanding structural editing scope for locked nodes.
- Multi-page orchestration, autonomous conflict resolution, or policy expansion beyond v1.

## Decisions

1. Runtime decomposition follows the planned Rust workspace boundaries.
   - Decision: use separate crates for CLI surface, pipeline orchestration, ADF operations, Confluence integration, and contracts.
   - Rationale: keeps policy-heavy logic isolated from transport and command concerns; improves testability and ownership boundaries.
   - Alternative considered: single-crate runtime for faster initial coding; rejected because it would couple contracts, orchestration, and adapters, raising refactor cost in Phase 2/3.

2. Orchestration is modeled as a contract-driven state machine with explicit state IO types.
   - Decision: each pipeline state has typed input/output envelopes, with deterministic transition ordering and halt-on-hard-error semantics.
   - Rationale: enforces correctness at boundaries and makes replay artifacts structurally comparable across runs.
   - Alternative considered: loosely typed JSON pass-through between states; rejected due to weaker compile-time guarantees and higher schema drift risk.

3. JSON Pointer is the canonical path identity for scope, diffs, and patch guards.
   - Decision: represent node paths and `changed_paths` as RFC 6901 JSON Pointers across fetch, merge, patch, and verify.
   - Rationale: consistent addressing lowers ambiguity between indexing and patch generation and aligns with the v1 contract.
   - Alternative considered: internal path IDs with later conversion; rejected because dual identity systems increase mismatch and debugging overhead.

4. Patch safety uses explicit operation constraints instead of post-hoc best-effort filtering.
   - Decision: patch builder accepts only path-targeted operations inside `allowed_scope_paths` and explicitly rejects whole-body rewrite patterns.
   - Rationale: prevention is safer than cleanup and keeps out-of-scope mutations impossible by construction.
   - Alternative considered: allow broad candidate assembly then prune; rejected because pruning can miss edge cases and hide unsafe intent.

5. Replay artifact persistence is first-class in orchestrator execution.
   - Decision: every state writes `state_input.json`, `state_output.json`, and `diagnostics.json`, plus run-level `summary.json`.
   - Rationale: deterministic diagnostics and KPI analysis require complete per-state observability from day one.
   - Alternative considered: emit only final run summaries; rejected because it blocks root-cause analysis and weakens reproducibility claims.

## Risks / Trade-offs

- [Type and contract churn in early phases] -> Version state envelopes and centralize shared contract types in one crate to limit migration blast radius.
- [Stub-first development may diverge from live Confluence behavior] -> Keep scheduled live sandbox probes and update fixtures on drift signals.
- [Strict patch guards can initially reject borderline valid edits] -> Start conservative, log reject diagnostics with path evidence, and tune via explicit decision updates.
- [Workspace split increases setup complexity] -> Provide a minimal bootstrap path and shared lint/test commands from the root workspace.
- [Artifact volume increases storage and I/O overhead] -> Scope retention by run ID and apply lifecycle cleanup for old artifacts after KPI reporting.

## Migration Plan

1. Bootstrap Rust workspace and crates with shared contract and error foundations.
2. Implement orchestrator shell with state sequencing and hard-error halting.
3. Add scoped fetch/index and patch guard primitives with fixture-backed tests.
4. Add replay artifact persistence and run summary emission.
5. Wire CLI entrypoint for no-op and simple scoped update flows.
6. Validate against Phase 1 acceptance checks before starting Phase 2/3 route work.

Rollback strategy:
- If skeleton instability blocks downstream work, freeze new route logic, revert to the last passing orchestrator baseline, and restore from replay fixtures to isolate regressions.

## Open Questions

- Should Phase 1 include parallel state execution hooks now, or keep strictly sequential execution until Phase 4 telemetry stabilizes?
- Which run ID format and retention policy should be standardized for cross-phase KPI aggregation?
- How much live Confluence behavior probing is required before locking the first stable contract version tag?
