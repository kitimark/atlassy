## Context

Phase 3 extends the existing v1 pipeline from prose-only practical edits to table support, but only within strict v1 safety bounds. Current architecture already provides route classification, path-targeted patching, verifier gates, and replay artifacts; the remaining gap is route-specific table behavior for `adf_table_edit` and related merge/verify checks.

Key constraints from roadmap and defaults:
- Table operations must remain `cell_text_update` only.
- Table topology and attribute mutations are out of scope for v1.
- Locked structural nodes remain immutable.
- Cross-route and out-of-scope mutations must fail before publish.

## Goals / Non-Goals

**Goals:**
- Implement a deterministic table edit route for table cell text updates only.
- Enforce explicit guardrails that reject table shape and table attribute changes.
- Ensure merge and verify logic detect cross-route and out-of-scope conflicts for table candidates.
- Add fixture-backed coverage proving no structural drift for allowed table edits.

**Non-Goals:**
- Row/column add-remove operations.
- Merge/split cell behavior or table attribute/layout changes.
- Changes to overall pipeline state order.
- Structural editing support for locked nodes.

## Decisions

1. Operation policy is allowlist-first (`cell_text_update` only).
   - Decision: represent table candidate actions as explicit op types and accept only `cell_text_update` in v1.
   - Rationale: strict allowlist makes unsupported behavior impossible by construction.
   - Alternative considered: broad op model with post-filtering; rejected due to risk of accidental drift.

2. Guardrails are enforced at both table-route construction and verifier checks.
   - Decision: reject forbidden table shape/attribute mutations early in `adf_table_edit`, and re-check in verify for defense in depth.
   - Rationale: dual enforcement reduces escape risk from edge cases in candidate generation.
   - Alternative considered: verify-only enforcement; rejected because failures would be later and harder to diagnose.

3. Route conflict handling remains fail-fast at merge.
   - Decision: if prose and table candidates overlap or violate path uniqueness/rule boundaries, merge fails immediately.
   - Rationale: keeps publish candidate deterministic and avoids ambiguous precedence rules.
   - Alternative considered: precedence-based auto-resolution; rejected as hidden behavior that can violate user intent.

4. Table fidelity validation is fixture-first with path-level assertions.
   - Decision: add mixed-content and table-focused fixtures that assert only targeted cell text paths change while table structure stays stable.
   - Rationale: fixture assertions provide reproducible structural drift detection.
   - Alternative considered: relying on ad hoc smoke checks; rejected due to weaker regression protection.

## Risks / Trade-offs

- [Over-restrictive guard rules may block legitimate cell-text edits] -> Mitigation: add explicit positive fixtures for known-safe table patterns and tune guard predicates with evidence.
- [Path instability in nested table content could cause false conflict or out-of-scope errors] -> Mitigation: canonicalize table paths and test nested table node patterns.
- [Merge fail-fast may reduce successful publish rate for mixed edits] -> Mitigation: provide deterministic diagnostics so retries/scope adjustments are straightforward.
- [Verifier strictness can increase initial failure volume] -> Mitigation: preserve detailed error codes and replay artifacts for rapid rule refinement.

## Migration Plan

1. Extend contracts for table route payloads and operation typing.
2. Implement `adf_table_edit` allowlisted operation generation and forbidden-op rejection.
3. Extend merge checks for table route path uniqueness and cross-route conflicts.
4. Extend verifier checks for table shape/attribute drift and route policy compliance.
5. Add fixture and integration test coverage for allowed and forbidden table scenarios.
6. Validate end-to-end with lint/test commands and replay artifact review.

Rollback strategy:
- If table route behavior is unstable, disable table candidate emission and retain existing prose-only behavior while preserving pipeline safety gates.

## Open Questions

- Should table attribute updates that are semantically neutral (for example formatting-only attrs) remain blocked in all cases for v1?
- Do we need a dedicated error code for cross-route path collision, or is existing schema/route violation signaling sufficient?
- What is the minimum fixture set needed to cover representative table complexity without overfitting to sample pages?
