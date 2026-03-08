use std::fs;
use std::path::{Path, PathBuf};

use atlassy_contracts::{Diagnostics, PipelineState, RunSummary, StateEnvelope};

use crate::PipelineError;

pub struct ArtifactStore {
    root: PathBuf,
}

impl ArtifactStore {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    pub fn persist_state<TInput: serde::Serialize, TOutput: serde::Serialize>(
        &self,
        run_id: &str,
        state: PipelineState,
        input: &StateEnvelope<TInput>,
        output: &StateEnvelope<TOutput>,
        diagnostics: &Diagnostics,
    ) -> Result<(), PipelineError> {
        let state_dir = self
            .root
            .join("artifacts")
            .join(run_id)
            .join(state.as_str());
        fs::create_dir_all(&state_dir)?;

        let input_file = state_dir.join("state_input.json");
        let output_file = state_dir.join("state_output.json");
        let diag_file = state_dir.join("diagnostics.json");

        fs::write(input_file, serde_json::to_string_pretty(input)?)?;
        fs::write(output_file, serde_json::to_string_pretty(output)?)?;
        fs::write(diag_file, serde_json::to_string_pretty(diagnostics)?)?;
        Ok(())
    }

    pub fn persist_summary(&self, run_id: &str, summary: &RunSummary) -> Result<(), PipelineError> {
        let run_dir = self.root.join("artifacts").join(run_id);
        fs::create_dir_all(&run_dir)?;
        fs::write(
            run_dir.join("summary.json"),
            serde_json::to_string_pretty(summary)?,
        )?;
        Ok(())
    }
}
