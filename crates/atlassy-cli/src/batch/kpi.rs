use std::collections::BTreeMap;

use atlassy_contracts::{
    FLOW_BASELINE, FLOW_OPTIMIZED, PATTERN_A, PATTERN_B, PATTERN_C, RunSummary,
};

use crate::{
    FlowGroup, GateCheck, KpiMetricComparison, KpiReport, KpiRollup, KpiStats, OutlierRun,
    RecommendationSection, RegressionSummary, RunManifest,
};

pub(crate) fn collect_flow_groups(
    manifest: &RunManifest,
    summaries: &BTreeMap<String, RunSummary>,
) -> Vec<FlowGroup> {
    let mut grouped: BTreeMap<(String, String, String), FlowGroup> = BTreeMap::new();

    for run in &manifest.runs {
        let key = (
            run.page_id.clone(),
            run.pattern.clone(),
            run.edit_intent_hash.clone(),
        );
        let group = grouped.entry(key.clone()).or_insert_with(|| FlowGroup {
            page_id: key.0.clone(),
            pattern: key.1.clone(),
            edit_intent_hash: key.2.clone(),
            baseline: Vec::new(),
            optimized: Vec::new(),
        });

        if let Some(summary) = summaries.get(&run.run_id)
            && summary.telemetry_complete
        {
            if run.flow == FLOW_BASELINE {
                group.baseline.push(summary.clone());
            } else if run.flow == FLOW_OPTIMIZED {
                group.optimized.push(summary.clone());
            }
        }
    }

    grouped
        .into_values()
        .filter(|group| !group.baseline.is_empty() || !group.optimized.is_empty())
        .collect()
}

pub(crate) fn build_kpi_report(flow_groups: &[FlowGroup]) -> KpiReport {
    let global_rollup = build_kpi_rollup("global", flow_groups);
    let pattern_rollups = [PATTERN_A, PATTERN_B, PATTERN_C]
        .into_iter()
        .map(|pattern| {
            let groups = flow_groups
                .iter()
                .filter(|group| group.pattern == pattern)
                .cloned()
                .collect::<Vec<_>>();
            build_kpi_rollup(pattern, &groups)
        })
        .collect::<Vec<_>>();

    let checks = evaluate_kpi_checks(&global_rollup);

    KpiReport {
        global_rollup,
        pattern_rollups,
        checks,
    }
}

fn build_kpi_rollup(scope: &str, flow_groups: &[FlowGroup]) -> KpiRollup {
    let pair_count = flow_groups
        .iter()
        .filter(|group| !group.baseline.is_empty() && !group.optimized.is_empty())
        .count();

    let baseline = flow_groups
        .iter()
        .flat_map(|group| group.baseline.clone())
        .collect::<Vec<_>>();
    let optimized = flow_groups
        .iter()
        .flat_map(|group| group.optimized.clone())
        .collect::<Vec<_>>();

    let metrics = [
        "context_reduction_ratio",
        "scoped_section_tokens",
        "edit_success_rate",
        "structural_preservation",
        "conflict_rate",
        "publish_latency",
    ]
    .into_iter()
    .map(|kpi| {
        let baseline_values = kpi_values(&baseline, kpi);
        let optimized_values = kpi_values(&optimized, kpi);
        let baseline_stats = compute_stats(&baseline_values);
        let optimized_stats = compute_stats(&optimized_values);

        let delta_absolute = optimized_stats.median - baseline_stats.median;
        let delta_relative = if baseline_stats.median.abs() < f64::EPSILON {
            0.0
        } else {
            delta_absolute / baseline_stats.median
        };

        KpiMetricComparison {
            kpi: kpi.to_string(),
            baseline: baseline_stats,
            optimized: optimized_stats,
            delta_absolute: normalize_metric_value(delta_absolute),
            delta_relative: normalize_metric_value(delta_relative),
        }
    })
    .collect::<Vec<_>>();

    KpiRollup {
        scope: scope.to_string(),
        pair_count,
        metrics,
    }
}

fn kpi_values(summaries: &[RunSummary], kpi: &str) -> Vec<f64> {
    match kpi {
        "context_reduction_ratio" => summaries
            .iter()
            .map(|summary| summary.context_reduction_ratio * 100.0)
            .collect(),
        "scoped_section_tokens" => summaries
            .iter()
            .map(|summary| summary.scoped_adf_bytes as f64 / 4.0)
            .collect(),
        "edit_success_rate" => {
            if summaries.is_empty() {
                Vec::new()
            } else {
                let successful = summaries
                    .iter()
                    .filter(|summary| summary.publish_result == "published")
                    .count() as f64;
                vec![(successful / summaries.len() as f64) * 100.0]
            }
        }
        "structural_preservation" => {
            if summaries.is_empty() {
                Vec::new()
            } else {
                let preserved = summaries
                    .iter()
                    .filter(|summary| {
                        summary.verify_result == "pass"
                            && !summary.locked_node_mutation
                            && !summary.out_of_scope_mutation
                    })
                    .count() as f64;
                vec![(preserved / summaries.len() as f64) * 100.0]
            }
        }
        "conflict_rate" => {
            if summaries.is_empty() {
                Vec::new()
            } else {
                let conflicts = summaries
                    .iter()
                    .filter(|summary| summary.retry_count > 0)
                    .count() as f64;
                vec![(conflicts / summaries.len() as f64) * 100.0]
            }
        }
        "publish_latency" => summaries
            .iter()
            .map(|summary| summary.latency_ms as f64)
            .collect(),
        _ => Vec::new(),
    }
}

fn compute_stats(values: &[f64]) -> KpiStats {
    if values.is_empty() {
        return KpiStats {
            count: 0,
            median: 0.0,
            p90: 0.0,
            min: 0.0,
            max: 0.0,
        };
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|left, right| left.total_cmp(right));

    let median = if sorted.len() % 2 == 1 {
        sorted[sorted.len() / 2]
    } else {
        let right = sorted.len() / 2;
        let left = right - 1;
        (sorted[left] + sorted[right]) / 2.0
    };

    let p90_idx = ((sorted.len() as f64) * 0.9).ceil() as usize;
    let p90 = sorted[p90_idx.saturating_sub(1).min(sorted.len() - 1)];

    KpiStats {
        count: sorted.len(),
        median,
        p90,
        min: *sorted.first().unwrap_or(&0.0),
        max: *sorted.last().unwrap_or(&0.0),
    }
}

fn normalize_metric_value(value: f64) -> f64 {
    const SCALE: f64 = 1_000_000_000_000.0;
    (value * SCALE).round() / SCALE
}

fn evaluate_kpi_checks(global: &KpiRollup) -> Vec<GateCheck> {
    let metric = |name: &str| global.metrics.iter().find(|metric| metric.kpi == name);

    let context_reduction_check = metric("context_reduction_ratio")
        .map(|metric| metric.optimized.median >= 70.0)
        .unwrap_or(false);

    let edit_success_check = metric("edit_success_rate")
        .map(|metric| metric.optimized.median > 95.0)
        .unwrap_or(false);

    let structural_preservation_check = metric("structural_preservation")
        .map(|metric| metric.optimized.median >= 100.0)
        .unwrap_or(false);

    let conflict_rate_check = metric("conflict_rate")
        .map(|metric| metric.optimized.median < 10.0)
        .unwrap_or(false);

    let latency_check = metric("publish_latency")
        .map(|metric| {
            metric.optimized.median < 3000.0 && metric.optimized.p90 <= metric.baseline.p90
        })
        .unwrap_or(false);

    vec![
        GateCheck {
            name: "context_reduction_ratio".to_string(),
            target: "optimized median >= 70%".to_string(),
            pass: context_reduction_check,
        },
        GateCheck {
            name: "edit_success_rate".to_string(),
            target: "optimized median > 95%".to_string(),
            pass: edit_success_check,
        },
        GateCheck {
            name: "structural_preservation".to_string(),
            target: "optimized median = 100%".to_string(),
            pass: structural_preservation_check,
        },
        GateCheck {
            name: "conflict_rate".to_string(),
            target: "optimized median < 10%".to_string(),
            pass: conflict_rate_check,
        },
        GateCheck {
            name: "publish_latency".to_string(),
            target: "optimized median < 3000 ms and p90 <= baseline".to_string(),
            pass: latency_check,
        },
    ]
}

pub(crate) fn build_recommendation(
    gate_checks: &[GateCheck],
    kpi: Option<&KpiReport>,
    valid_summaries: &[RunSummary],
    flow_groups: &[FlowGroup],
    safety_failed: bool,
    drift_unresolved: bool,
) -> RecommendationSection {
    let kpi_failed = kpi
        .map(|report| report.checks.iter().any(|check| !check.pass))
        .unwrap_or(true);
    let gate_failed = gate_checks.iter().any(|check| !check.pass);

    let decision = if safety_failed || drift_unresolved {
        "stop"
    } else if gate_failed || kpi_failed {
        "iterate"
    } else {
        "go"
    }
    .to_string();

    let mut rationale = gate_checks
        .iter()
        .filter(|check| !check.pass)
        .map(|check| format!("gate failed: {} ({})", check.name, check.target))
        .collect::<Vec<_>>();
    if let Some(report) = kpi {
        rationale.extend(
            report
                .checks
                .iter()
                .filter(|check| !check.pass)
                .map(|check| format!("kpi target missed: {} ({})", check.name, check.target)),
        );
    }
    rationale.sort();
    rationale.dedup();

    RecommendationSection {
        decision,
        rationale,
        outliers: build_outliers(valid_summaries),
        regressions: build_regressions(flow_groups),
    }
}

fn build_outliers(summaries: &[RunSummary]) -> Vec<OutlierRun> {
    let mut outliers = Vec::new();

    let mut latency = summaries
        .iter()
        .map(|summary| OutlierRun {
            run_id: summary.run_id.clone(),
            kpi: "publish_latency".to_string(),
            value: summary.latency_ms as f64,
        })
        .collect::<Vec<_>>();
    latency.sort_by(|left, right| {
        right
            .value
            .total_cmp(&left.value)
            .then_with(|| left.run_id.cmp(&right.run_id))
    });
    outliers.extend(latency.into_iter().take(3));

    let mut context_reduction = summaries
        .iter()
        .map(|summary| OutlierRun {
            run_id: summary.run_id.clone(),
            kpi: "context_reduction_ratio".to_string(),
            value: summary.context_reduction_ratio * 100.0,
        })
        .collect::<Vec<_>>();
    context_reduction.sort_by(|left, right| {
        left.value
            .total_cmp(&right.value)
            .then_with(|| left.run_id.cmp(&right.run_id))
    });
    outliers.extend(context_reduction.into_iter().take(3));

    outliers
}

fn build_regressions(flow_groups: &[FlowGroup]) -> Vec<RegressionSummary> {
    let mut regressions = Vec::new();

    for group in flow_groups {
        if group.baseline.is_empty() || group.optimized.is_empty() {
            continue;
        }

        let baseline_context_reduction =
            compute_stats(&kpi_values(&group.baseline, "context_reduction_ratio")).median;
        let optimized_context_reduction =
            compute_stats(&kpi_values(&group.optimized, "context_reduction_ratio")).median;
        if optimized_context_reduction < baseline_context_reduction {
            regressions.push(RegressionSummary {
                page_id: group.page_id.clone(),
                pattern: group.pattern.clone(),
                edit_intent_hash: group.edit_intent_hash.clone(),
                kpi: "context_reduction_ratio".to_string(),
                baseline: baseline_context_reduction,
                optimized: optimized_context_reduction,
            });
        }

        let baseline_edit_success =
            compute_stats(&kpi_values(&group.baseline, "edit_success_rate")).median;
        let optimized_edit_success =
            compute_stats(&kpi_values(&group.optimized, "edit_success_rate")).median;
        if optimized_edit_success < baseline_edit_success {
            regressions.push(RegressionSummary {
                page_id: group.page_id.clone(),
                pattern: group.pattern.clone(),
                edit_intent_hash: group.edit_intent_hash.clone(),
                kpi: "edit_success_rate".to_string(),
                baseline: baseline_edit_success,
                optimized: optimized_edit_success,
            });
        }
    }

    regressions.sort_by(|left, right| {
        (
            left.page_id.as_str(),
            left.pattern.as_str(),
            left.edit_intent_hash.as_str(),
            left.kpi.as_str(),
        )
            .cmp(&(
                right.page_id.as_str(),
                right.pattern.as_str(),
                right.edit_intent_hash.as_str(),
                right.kpi.as_str(),
            ))
    });
    regressions
}
