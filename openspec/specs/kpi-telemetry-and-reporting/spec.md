## Purpose

Define required PoC telemetry completeness and deterministic KPI reporting outputs for baseline versus optimized comparisons.

## Requirements

### Requirement: Required per-run telemetry completeness
Each PoC run summary MUST include all KPI-required telemetry fields, including run identity, flow metadata, scope outcomes, payload sizing fields (`full_page_adf_bytes`, `scoped_adf_bytes`, `context_reduction_ratio`, `patch_ops_bytes`), verify/publish results, retry data, timing fields, out-of-scope mutation flag, and decision-grade provenance fields (`git_commit_sha`, `git_dirty`, `pipeline_version`).

#### Scenario: Reject incomplete telemetry record
- **WHEN** a run summary is missing any required KPI or provenance field
- **THEN** the run is marked invalid for KPI aggregation
- **AND** batch reporting fails with telemetry-completeness diagnostics

#### Scenario: Reject placeholder operational telemetry
- **WHEN** non-synthetic runs emit static placeholder values for timing, scope retrieval, or payload sizing metrics
- **THEN** the run is marked non-evaluable for KPI reporting
- **AND** readiness output is blocked until telemetry quality is restored

### Requirement: Deterministic KPI aggregation and delta reporting
The reporting workflow SHALL compute deterministic median and p90 values for the six revised KPIs (`context_reduction_ratio`, `scoped_section_tokens`, `edit_success_rate`, `structural_preservation`, `conflict_rate`, `publish_latency`) for baseline and optimized flows, plus absolute and relative deltas and pass/fail checks against revised v1 targets, using only telemetry-complete and provenance-stamped runs.

#### Scenario: Generate aggregate KPI report
- **WHEN** paired run telemetry is complete for all required matrix keys
- **THEN** the system emits an aggregate report containing baseline and optimized KPI metrics, deltas, target thresholds, and pass/fail outcomes for all six revised KPIs

#### Scenario: Exclude non-decision-grade runs from aggregation
- **WHEN** a run lacks telemetry completeness or provenance validity
- **THEN** the run is excluded from aggregate KPI calculations
- **AND** diagnostics identify why it was excluded

### Requirement: Revised KPI gate checks enforce absolute thresholds
The gate evaluation logic SHALL apply the following pass/fail rules to the optimized flow's aggregated statistics, replacing all legacy relative-delta checks.

#### Scenario: Context reduction ratio gate
- **WHEN** the optimized flow's `context_reduction_ratio` median is evaluated
- **THEN** the gate passes if the median is greater than or equal to 70%

#### Scenario: Edit success rate gate
- **WHEN** the optimized flow's `edit_success_rate` median is evaluated
- **THEN** the gate passes if the median is greater than 95%

#### Scenario: Structural preservation gate
- **WHEN** the optimized flow's `structural_preservation` median is evaluated
- **THEN** the gate passes if the median is 100%

#### Scenario: Conflict rate gate
- **WHEN** the optimized flow's `conflict_rate` median is evaluated
- **THEN** the gate passes if the median is less than 10%

#### Scenario: Publish latency gate
- **WHEN** the optimized flow's `publish_latency` statistics are evaluated
- **THEN** the gate passes if the median is less than 3000 ms and the p90 is less than or equal to the baseline flow's p90

### Requirement: Out-of-scope mutation flag is a first-class run summary field
The run summary SHALL include `out_of_scope_mutation` as a boolean field, derived from the presence of `ERR_OUT_OF_SCOPE_MUTATION` in the run's error codes.

#### Scenario: Out-of-scope mutation detected
- **WHEN** a run's error codes include `ERR_OUT_OF_SCOPE_MUTATION`
- **THEN** the run summary sets `out_of_scope_mutation` to `true`

#### Scenario: No out-of-scope mutation
- **WHEN** a run's error codes do not include `ERR_OUT_OF_SCOPE_MUTATION`
- **THEN** the run summary sets `out_of_scope_mutation` to `false`

### Requirement: Structural preservation uses three-condition formula
The `structural_preservation` KPI SHALL evaluate to 100% (pass) for a run only when all three conditions hold: `verify_result` is `"pass"`, `locked_node_mutation` is `false`, and `out_of_scope_mutation` is `false`.

#### Scenario: All three conditions pass
- **WHEN** a run has `verify_result == "pass"` and `locked_node_mutation == false` and `out_of_scope_mutation == false`
- **THEN** the run contributes 100% to `structural_preservation`

#### Scenario: Any condition fails
- **WHEN** a run has `verify_result != "pass"` or `locked_node_mutation == true` or `out_of_scope_mutation == true`
- **THEN** the run contributes 0% to `structural_preservation`

### Requirement: Outlier detection uses revised KPI names
Outlier tagging SHALL use `context_reduction_ratio` (lowest values) and `publish_latency` (highest values) as the outlier axes, replacing the retired `tokens_per_successful_update` axis.

#### Scenario: Context reduction outliers tagged
- **WHEN** outlier detection runs
- **THEN** the top 3 runs with the lowest `context_reduction_ratio` are tagged as outliers

#### Scenario: Latency outliers tagged
- **WHEN** outlier detection runs
- **THEN** the top 3 runs with the highest `publish_latency` are tagged as outliers

### Requirement: Regression detection uses revised KPI names
Per-group regression tagging SHALL compare baseline and optimized statistics using `context_reduction_ratio` and `edit_success_rate`, replacing the retired `tokens_per_successful_update` and `full_page_retrieval_rate` axes.

#### Scenario: Regression flagged on context reduction
- **WHEN** the optimized flow's `context_reduction_ratio` for a group is worse than the baseline's
- **THEN** the group is flagged as a notable regression for `context_reduction_ratio`

### Requirement: Pattern-level and aggregate decision visibility
Reporting MUST provide both per-pattern (A/B/C) and global KPI summaries, including outlier listing and a recommendation-ready decision section.

#### Scenario: Produce decision review packet
- **WHEN** aggregate report generation completes
- **THEN** output includes pattern breakdowns, notable regressions, and a recommendation section suitable for `go | iterate | stop` review
