use std::fs;
use std::path::Path;

use atlassy_contracts::RunSummary;

use crate::DynError;

pub(crate) fn load_required_json<T>(path: &Path) -> Result<T, DynError>
where
    T: for<'de> serde::Deserialize<'de>,
{
    if !path.exists() {
        return Err(format!(
            "missing readiness evidence: {} (run `atlassy run-batch --manifest <file>` first)",
            path.display()
        )
        .into());
    }

    let text = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&text)?)
}

pub(crate) fn load_run_summary(
    artifacts_dir: &Path,
    run_id: &str,
) -> Result<Option<RunSummary>, DynError> {
    let summary_path = artifacts_dir
        .join("artifacts")
        .join(run_id)
        .join("summary.json");
    if !summary_path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(summary_path)?;
    let summary = serde_json::from_str::<RunSummary>(&text)?;
    Ok(Some(summary))
}
