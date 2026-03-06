## ADDED Requirements

### Requirement: Required per-run telemetry completeness
Each PoC run summary MUST include all KPI-required telemetry fields, including run identity, flow metadata, token usage, scope outcomes, verify/publish results, retry data, and timing fields.

#### Scenario: Reject incomplete telemetry record
- **WHEN** a run summary is missing any required KPI field
- **THEN** the run is marked invalid for KPI aggregation
- **AND** batch reporting fails with telemetry-completeness diagnostics

### Requirement: Deterministic KPI aggregation and delta reporting
The reporting workflow SHALL compute deterministic median and p90 values per KPI for baseline and optimized flows, plus absolute and relative deltas and pass/fail checks against v1 targets.

#### Scenario: Generate aggregate KPI report
- **WHEN** paired run telemetry is complete for all required matrix keys
- **THEN** the system emits an aggregate report containing baseline and optimized KPI metrics, deltas, target thresholds, and pass/fail outcomes

### Requirement: Pattern-level and aggregate decision visibility
Reporting MUST provide both per-pattern (A/B/C) and global KPI summaries, including outlier listing and a recommendation-ready decision section.

#### Scenario: Produce decision review packet
- **WHEN** aggregate report generation completes
- **THEN** output includes pattern breakdowns, notable regressions, and a recommendation section suitable for `go | iterate | stop` review
