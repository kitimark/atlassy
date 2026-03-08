mod kpi;
mod report;
mod safety;

pub use report::rebuild_batch_report_from_artifacts;

pub(crate) use kpi::{build_kpi_report, build_recommendation, collect_flow_groups};
pub(crate) use report::{build_artifact_index, summarize_failure_classes};
pub(crate) use safety::{assess_drift, assess_safety, assess_scenario_coverage};
