## 1. Readiness Models and Evidence Loading

- [x] 1.1 Add readiness-domain data models for gate results, runbook entries, risk deltas, and decision packet outputs in CLI reporting code
- [x] 1.2 Add evidence loader(s) that ingest normalized manifest, artifact index, run summaries, and prior batch report data from `artifacts/`
- [x] 1.3 Add deterministic ordering/canonicalization for loaded evidence (gate order, run ordering, stable sort keys)
- [x] 1.4 Add validation that required readiness evidence is present before gate evaluation starts

## 2. Gate Checklist Evaluation

- [x] 2.1 Implement Gate 1-6 checklist evaluator with fixed execution order and normalized pass/fail records
- [x] 2.2 Implement blocking behavior when any mandatory gate fails or required evidence is missing
- [x] 2.3 Add owner-role attribution, generation timestamp, and source artifact references to checklist outputs
- [x] 2.4 Persist readiness checklist output as a deterministic artifact file in batch artifacts

## 3. Operator Runbook Synthesis

- [x] 3.1 Implement deterministic mapping from priority failure classes to runbook templates (verify hard fail, retry exhaustion, safety violations, drift unresolved, telemetry incomplete)
- [x] 3.2 Include severity, primary owner role, escalation owner role, and escalation trigger fields in each generated runbook section
- [x] 3.3 Implement fallback runbook generation for unknown failure classes with manual-review routing
- [x] 3.4 Ensure unknown-class fallback marks readiness as non-passing until mapping coverage is restored

## 4. Decision Packet Governance

- [x] 4.1 Implement decision packet assembler that includes gate outcomes, KPI summaries, risk-status deltas, top failure classes, and recommendation rationale
- [x] 4.2 Implement safety-first recommendation precedence (`safety/drift blockers -> incomplete mandatory gates -> KPI misses -> pass`)
- [x] 4.3 Add explicit blocking-condition capture in packet output when recommendation is non-go
- [x] 4.4 Persist decision packet artifact with deterministic formatting and stable field ordering

## 5. Reproducibility and CLI Workflow Integration

- [x] 5.1 Add CLI command path(s) for generating readiness checklist, runbook bundle, and final decision packet from batch artifacts
- [x] 5.2 Add replay/rebuild path that regenerates decision packet from persisted artifacts and compares with stored output
- [x] 5.3 Fail readiness workflow when regenerated packet diverges from stored packet
- [x] 5.4 Add operator-facing error messages for missing evidence, replay mismatch, and blocked readiness outcomes

## 6. Verification and Regression Coverage

- [x] 6.1 Add fixture-backed tests for deterministic Gate 1-6 evaluation and mandatory-gate blocking behavior
- [x] 6.2 Add tests for mapped runbook generation, escalation metadata presence, and unknown-class fallback handling
- [x] 6.3 Add tests for decision packet required sections, recommendation precedence, and blocking-condition traceability
- [x] 6.4 Add reproducibility tests proving decision packet rebuild from stored artifacts is equivalent and mismatch fails readiness
- [x] 6.5 Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` and fix any issues
