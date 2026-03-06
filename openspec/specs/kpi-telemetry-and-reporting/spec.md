## Purpose

Define required PoC telemetry completeness and deterministic KPI reporting outputs for baseline versus optimized comparisons.

## Requirements

### Requirement: Required per-run telemetry completeness
Each PoC run summary MUST include all KPI-required telemetry fields, including run identity, flow metadata, token usage, scope outcomes, verify/publish results, retry data, timing fields, and decision-grade provenance fields (`git_commit_sha`, `git_dirty`, `pipeline_version`).

#### Scenario: Reject incomplete telemetry record
- **WHEN** a run summary is missing any required KPI or provenance field
- **THEN** the run is marked invalid for KPI aggregation
- **AND** batch reporting fails with telemetry-completeness diagnostics

#### Scenario: Reject placeholder operational telemetry
- **WHEN** non-synthetic runs emit static placeholder values for timing, token, or scope retrieval metrics
- **THEN** the run is marked non-evaluable for KPI reporting
- **AND** readiness output is blocked until telemetry quality is restored

### Requirement: Deterministic KPI aggregation and delta reporting
The reporting workflow SHALL compute deterministic median and p90 values per KPI for baseline and optimized flows, plus absolute and relative deltas and pass/fail checks against v1 targets, using only telemetry-complete and provenance-stamped runs.

#### Scenario: Generate aggregate KPI report
- **WHEN** paired run telemetry is complete for all required matrix keys
- **THEN** the system emits an aggregate report containing baseline and optimized KPI metrics, deltas, target thresholds, and pass/fail outcomes

#### Scenario: Exclude non-decision-grade runs from aggregation
- **WHEN** a run lacks telemetry completeness or provenance validity
- **THEN** the run is excluded from aggregate KPI calculations
- **AND** diagnostics identify why it was excluded

### Requirement: Pattern-level and aggregate decision visibility
Reporting MUST provide both per-pattern (A/B/C) and global KPI summaries, including outlier listing and a recommendation-ready decision section.

#### Scenario: Produce decision review packet
- **WHEN** aggregate report generation completes
- **THEN** output includes pattern breakdowns, notable regressions, and a recommendation section suitable for `go | iterate | stop` review
