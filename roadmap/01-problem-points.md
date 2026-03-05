# Problem Points (v1)

## Objective

State the concrete operational problems Atlassy v1 addresses for AI-assisted Confluence page updates.

## Context

- Confluence pages often contain mixed content (prose, tables, media, macros, layouts).
- AI editing workflows frequently use oversized prompts and broad update payloads.
- Teams need lower token cost without losing formatting fidelity or publish reliability.

## Primary Problem Points

### P-001 Context window pressure from full-page retrieval

- Full page bodies are often fetched for small, local edits.
- Large ADF payloads consume context quickly, reducing room for intent and policy.
- Context pressure raises truncation risk and inconsistent behavior across runs.

### P-002 Token waste from repeated boilerplate and retries

- Policy text, historical context, and unchanged content are repeatedly injected.
- Conflict retries replay large payloads, multiplying token usage.
- High token spend reduces throughput and increases operating cost.

### P-003 Update fragility from broad mutation scope

- Whole-body or coarse updates increase mutation blast radius.
- Unrelated blocks can drift when edits are not path-targeted.
- Broad changes amplify version conflict probability and retry overhead.

### P-004 Fidelity risk on structural content

- Tables, media, macros/extensions, and layouts are high-risk for lossy transforms.
- Naive markdown-first workflows can distort or drop structure.
- Repeated round-trips increase risk of unnoticed formatting regressions.

### P-005 Low observability for quality and efficiency decisions

- Teams cannot reliably compare baseline vs optimized flow performance.
- Missing per-state telemetry obscures where token or latency waste occurs.
- Weak diagnostics slow root-cause analysis for failed publishes.

## Root Causes

- Lack of strict scoped retrieval defaults.
- No consistent route policy separating prose-safe edits from structural risk.
- Insufficient verifier gates before publish.
- Incomplete run-level instrumentation and replay artifacts.

## User and Business Impact

- Slower turnaround for routine content maintenance.
- Higher token and compute cost per successful update.
- More manual cleanup after automated edits.
- Lower trust in AI-driven publish workflows.

## v1 Problem Boundaries

- Focus on single-page, scoped update reliability first.
- Treat structural edits as locked unless explicitly supported.
- Defer multi-page orchestration and advanced table restructuring.

## Success Signals

- Significant reduction in tokens per successful update.
- Significant reduction in full-page retrieval rate.
- Non-regressive fidelity and publish outcomes.
- Deterministic failure diagnostics with bounded retries.

## Links to Planning Artifacts

- Architecture and routing: `02-solution-architecture.md`
- Phase sequencing and delivery plan: `03-phased-roadmap.md`
- KPI and experiment protocol: `04-kpi-and-experiments.md`
- Risk controls: `05-risks-and-mitigations.md`
- Decisions and defaults: `06-decisions-and-defaults.md`
- Execution checklist and go/no-go: `07-execution-readiness.md`
