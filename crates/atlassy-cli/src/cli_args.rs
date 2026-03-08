use std::path::PathBuf;

use clap::Args;

use super::{CliMode, RuntimeBackend};

#[derive(Debug, Args)]
pub struct RunArgs {
    #[arg(long)]
    pub request_id: String,
    #[arg(long)]
    pub page_id: String,
    #[arg(long)]
    pub edit_intent: String,
    #[arg(long = "scope")]
    pub scope_selectors: Vec<String>,
    #[arg(long, default_value = ".")]
    pub artifacts_dir: PathBuf,
    #[arg(long, value_enum, default_value_t = CliMode::NoOp)]
    pub mode: CliMode,
    #[arg(long)]
    pub target_path: Option<String>,
    #[arg(long)]
    pub target_index: Option<usize>,
    #[arg(long)]
    pub new_value: Option<String>,
    #[arg(long)]
    pub force_verify_fail: bool,
    #[arg(long)]
    pub bootstrap_empty_page: bool,
    #[arg(long, value_enum, default_value_t = RuntimeBackend::Stub)]
    pub runtime_backend: RuntimeBackend,
}

#[derive(Debug, Args)]
pub struct RunBatchArgs {
    #[arg(long)]
    pub manifest: PathBuf,
    #[arg(long, default_value = ".")]
    pub artifacts_dir: PathBuf,
    #[arg(long, value_enum, default_value_t = RuntimeBackend::Stub)]
    pub runtime_backend: RuntimeBackend,
}

#[derive(Debug, Args)]
pub struct RunReadinessArgs {
    #[arg(long, default_value = ".")]
    pub artifacts_dir: PathBuf,
    #[arg(long)]
    pub verify_replay: bool,
}

#[derive(Debug, Args)]
pub struct CreateSubpageArgs {
    #[arg(long)]
    pub parent_page_id: String,
    #[arg(long)]
    pub space_key: String,
    #[arg(long)]
    pub title: String,
    #[arg(long, value_enum, default_value_t = RuntimeBackend::Stub)]
    pub runtime_backend: RuntimeBackend,
}
