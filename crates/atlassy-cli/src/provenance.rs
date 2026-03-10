use atlassy_contracts::{
    PIPELINE_VERSION, ProvenanceStamp, RUNTIME_LIVE, RUNTIME_STUB, RunSummary,
    validate_provenance_stamp, validate_run_summary_telemetry,
};

use crate::DynError;

const GIT_COMMIT_SHA: &str = env!("GIT_COMMIT_SHA");
const GIT_DIRTY: &str = env!("GIT_DIRTY");

pub fn collect_provenance(runtime_mode: &str) -> Result<ProvenanceStamp, DynError> {
    if !matches!(runtime_mode, RUNTIME_STUB | RUNTIME_LIVE) {
        return Err(format!(
            "invalid runtime mode `{runtime_mode}`: expected `{}` or `{}`",
            RUNTIME_STUB, RUNTIME_LIVE
        )
        .into());
    }

    let git_dirty = match GIT_DIRTY {
        "true" => true,
        "false" => false,
        value => {
            return Err(format!(
                "embedded git dirty flag `{value}` is invalid: expected `true` or `false`"
            )
            .into());
        }
    };

    let provenance = ProvenanceStamp {
        git_commit_sha: GIT_COMMIT_SHA.to_string(),
        git_dirty,
        pipeline_version: PIPELINE_VERSION.to_string(),
        runtime_mode: runtime_mode.to_string(),
    };
    validate_provenance_stamp(&provenance)?;
    Ok(provenance)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_git_commit_sha_passes_validation() {
        let provenance = ProvenanceStamp {
            git_commit_sha: env!("GIT_COMMIT_SHA").to_string(),
            git_dirty: false,
            pipeline_version: PIPELINE_VERSION.to_string(),
            runtime_mode: RUNTIME_STUB.to_string(),
        };

        assert!(validate_provenance_stamp(&provenance).is_ok());
    }
}
