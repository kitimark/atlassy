use atlassy_cli::*;
use atlassy_contracts::{RUNTIME_LIVE, RUNTIME_STUB};
use clap::{Parser, Subcommand, ValueEnum};

mod cli_args;

use cli_args::{CreateSubpageArgs, RunArgs, RunBatchArgs, RunReadinessArgs};

#[derive(Debug, Parser)]
#[command(name = "atlassy")]
#[command(about = "Atlassy CLI for v1 pipeline execution and readiness checks")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Run(RunArgs),
    RunBatch(RunBatchArgs),
    RunReadiness(RunReadinessArgs),
    CreateSubpage(CreateSubpageArgs),
}

#[derive(Debug, Clone, ValueEnum)]
enum CliMode {
    NoOp,
    SimpleScopedProseUpdate,
    SimpleScopedTableCellUpdate,
}

impl CliMode {
    fn as_str(&self) -> &'static str {
        match self {
            CliMode::NoOp => "no-op",
            CliMode::SimpleScopedProseUpdate => "simple-scoped-prose-update",
            CliMode::SimpleScopedTableCellUpdate => "simple-scoped-table-cell-update",
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
enum RuntimeBackend {
    Stub,
    Live,
}

impl RuntimeBackend {
    fn as_str(&self) -> &'static str {
        match self {
            RuntimeBackend::Stub => RUNTIME_STUB,
            RuntimeBackend::Live => RUNTIME_LIVE,
        }
    }
}

fn main() -> Result<(), DynError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run(args) => {
            execute_run_command(
                args.request_id,
                args.page_id,
                args.edit_intent,
                args.scope_selectors,
                args.artifacts_dir,
                args.mode.as_str(),
                args.target_path,
                args.target_index,
                args.new_value,
                args.force_verify_fail,
                args.bootstrap_empty_page,
                args.runtime_backend.as_str(),
            )?;
        }
        Commands::RunBatch(args) => {
            let report = execute_batch_from_manifest_file_with_backend(
                &args.manifest,
                &args.artifacts_dir,
                args.runtime_backend.as_str(),
            )?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Commands::RunReadiness(args) => {
            let readiness = generate_readiness_outputs_from_artifacts(&args.artifacts_dir)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&readiness.decision_packet)?
            );
            if args.verify_replay {
                verify_decision_packet_replay(&args.artifacts_dir)?;
                println!("readiness replay verification passed");
            }
            ensure_readiness_unblocked(&readiness.decision_packet)?;
        }
        Commands::CreateSubpage(args) => {
            create_subpage(
                &args.parent_page_id,
                &args.space_key,
                &args.title,
                args.runtime_backend.as_str(),
            )?;
        }
    }

    Ok(())
}
