use atlassy_contracts::{
    PIPELINE_VERSION, ProvenanceStamp, RUNTIME_LIVE, RUNTIME_STUB, RunSummary,
    validate_provenance_stamp, validate_run_summary_telemetry,
};

use crate::DynError;

pub fn collect_provenance(runtime_mode: &str) -> Result<ProvenanceStamp, DynError> {
    if !matches!(runtime_mode, RUNTIME_STUB | RUNTIME_LIVE) {
        return Err(format!(
            "invalid runtime mode `{runtime_mode}`: expected `{}` or `{}`",
            RUNTIME_STUB, RUNTIME_LIVE
        )
        .into());
    }

    let git_commit_sha = resolve_git_commit_sha()?;
    let git_dirty = resolve_git_dirty()?;
    let provenance = ProvenanceStamp {
        git_commit_sha,
        git_dirty,
        pipeline_version: PIPELINE_VERSION.to_string(),
        runtime_mode: runtime_mode.to_string(),
    };
    validate_provenance_stamp(&provenance)?;
    Ok(provenance)
}

fn resolve_git_commit_sha() -> Result<String, DynError> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()?;
    if !output.status.success() {
        return Err("failed to collect git commit SHA via `git rev-parse HEAD`".into());
    }

    let value = String::from_utf8(output.stdout)?.trim().to_string();
    if value.len() != 40 || !value.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(
            "git commit SHA is malformed; expected 40 lowercase/uppercase hex chars".into(),
        );
    }
    Ok(value)
}

fn resolve_git_dirty() -> Result<bool, DynError> {
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .output()?;
    if !output.status.success() {
        return Err("failed to inspect git dirty state via `git status --porcelain`".into());
    }
    Ok(!String::from_utf8(output.stdout)?.trim().is_empty())
}

pub(crate) fn provenance_matches(summary: &RunSummary, provenance: &ProvenanceStamp) -> bool {
    summary.git_commit_sha == provenance.git_commit_sha
        && summary.git_dirty == provenance.git_dirty
        && summary.pipeline_version == provenance.pipeline_version
        && summary.runtime_mode == provenance.runtime_mode
}

pub(crate) fn summary_telemetry_complete(summary: &RunSummary) -> bool {
    summary.telemetry_complete && validate_run_summary_telemetry(summary).is_ok()
}
