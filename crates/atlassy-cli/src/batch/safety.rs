use std::collections::{BTreeMap, BTreeSet};

use atlassy_contracts::{ErrorCode, RunSummary};

use crate::{
    DriftAssessment, DriftStatusInput, RunManifest, SafetyAssessment, ScenarioCoverageAssessment,
};

pub(crate) fn assess_drift(input: &DriftStatusInput) -> DriftAssessment {
    let unresolved_material_drift =
        !input.scoped_fetch_parity || !input.publish_conflict_parity || !input.error_payload_parity;
    DriftAssessment {
        scoped_fetch_parity: input.scoped_fetch_parity,
        publish_conflict_parity: input.publish_conflict_parity,
        error_payload_parity: input.error_payload_parity,
        unresolved_material_drift,
    }
}

pub(crate) fn assess_scenario_coverage(manifest: &RunManifest) -> ScenarioCoverageAssessment {
    let mut required = manifest.batch.required_scenario_ids.clone();
    required.sort();
    required.dedup();

    let observed = observed_scenario_ids(manifest);
    let observed_set = observed.iter().cloned().collect::<BTreeSet<_>>();
    let missing = required
        .iter()
        .filter(|scenario| !observed_set.contains(*scenario))
        .cloned()
        .collect::<Vec<_>>();

    ScenarioCoverageAssessment {
        required_scenario_ids: required,
        observed_scenario_ids: observed,
        missing_scenario_ids: missing.clone(),
        complete: missing.is_empty(),
    }
}

pub(crate) fn observed_scenario_ids(manifest: &RunManifest) -> Vec<String> {
    let mut observed = manifest.batch.observed_scenario_ids.clone();
    for run in &manifest.runs {
        observed.extend(run.scenario_ids.clone());
    }
    observed.sort();
    observed.dedup();
    observed
}

pub(crate) fn assess_safety(summaries: &BTreeMap<String, RunSummary>) -> SafetyAssessment {
    let mut locked = Vec::new();
    let mut out_of_scope = Vec::new();
    let mut table_shape = Vec::new();

    for (run_id, summary) in summaries {
        if summary
            .error_codes
            .iter()
            .any(|code| code == ErrorCode::LockedNodeMutation.as_str())
            || summary.locked_node_mutation
        {
            locked.push(run_id.clone());
        }
        if summary
            .error_codes
            .iter()
            .any(|code| code == ErrorCode::OutOfScopeMutation.as_str())
        {
            out_of_scope.push(run_id.clone());
        }
        if summary
            .error_codes
            .iter()
            .any(|code| code == ErrorCode::TableShapeChange.as_str())
        {
            table_shape.push(run_id.clone());
        }
    }

    locked.sort();
    out_of_scope.sort();
    table_shape.sort();

    SafetyAssessment {
        locked_node_violation_runs: locked.clone(),
        out_of_scope_violation_runs: out_of_scope.clone(),
        table_shape_violation_runs: table_shape.clone(),
        safety_failed: !(locked.is_empty() && out_of_scope.is_empty() && table_shape.is_empty()),
    }
}
