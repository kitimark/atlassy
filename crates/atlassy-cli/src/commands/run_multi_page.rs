use std::collections::HashMap;
use std::fs;
use std::path::Path;

use atlassy_confluence::{ConfluenceError, LiveConfluenceClient, StubConfluenceClient, StubPage};
use atlassy_contracts::{ErrorCode, MultiPageRequest, RUNTIME_LIVE, RUNTIME_STUB};
use atlassy_pipeline::{MultiPageOrchestrator, PipelineError};

use crate::{demo_page, DynError};

fn map_live_startup_error(error: ConfluenceError) -> PipelineError {
    PipelineError::Hard {
        state: atlassy_contracts::PipelineState::Fetch,
        code: ErrorCode::RuntimeBackend,
        message: format!("live runtime startup failure: {error}"),
    }
}

pub fn execute_multi_page_from_manifest_file_with_backend(
    manifest_path: &Path,
    artifacts_dir: &Path,
    runtime_mode: &str,
) -> Result<(), DynError> {
    let request = read_multi_page_manifest(manifest_path)?;

    match runtime_mode {
        RUNTIME_STUB => {
            let pages = seed_stub_pages(&request);
            let mut orchestrator =
                MultiPageOrchestrator::new(StubConfluenceClient::new(pages), artifacts_dir);
            match orchestrator.run(request) {
                Ok(summary) => {
                    println!("{}", serde_json::to_string_pretty(&summary)?);
                    if !summary.success {
                        eprintln!(
                            "multi-page run completed with failures: {}",
                            ErrorCode::MultiPagePartialFailure.as_str()
                        );
                        std::process::exit(1);
                    }
                }
                Err(error) => {
                    eprintln!("run-multi-page failed: {error}");
                    std::process::exit(1);
                }
            }
        }
        RUNTIME_LIVE => {
            let client = match LiveConfluenceClient::from_env() {
                Ok(client) => client,
                Err(error) => {
                    eprintln!("run-multi-page failed: {}", map_live_startup_error(error));
                    std::process::exit(1);
                }
            };
            let mut orchestrator = MultiPageOrchestrator::new(client, artifacts_dir);
            match orchestrator.run(request) {
                Ok(summary) => {
                    println!("{}", serde_json::to_string_pretty(&summary)?);
                    if !summary.success {
                        eprintln!(
                            "multi-page run completed with failures: {}",
                            ErrorCode::MultiPagePartialFailure.as_str()
                        );
                        std::process::exit(1);
                    }
                }
                Err(error) => {
                    eprintln!("run-multi-page failed: {error}");
                    std::process::exit(1);
                }
            }
        }
        _ => {
            return Err(format!("invalid runtime mode `{runtime_mode}`").into());
        }
    }

    Ok(())
}

fn read_multi_page_manifest(manifest_path: &Path) -> Result<MultiPageRequest, DynError> {
    let manifest_text = fs::read_to_string(manifest_path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            format!("multi-page manifest not found: {}", manifest_path.display())
        } else {
            format!(
                "failed to read multi-page manifest {}: {error}",
                manifest_path.display()
            )
        }
    })?;

    serde_json::from_str(&manifest_text).map_err(|error| {
        format!(
            "failed to parse multi-page manifest {} as MultiPageRequest: {error}",
            manifest_path.display()
        )
        .into()
    })
}

fn seed_stub_pages(request: &MultiPageRequest) -> HashMap<String, StubPage> {
    let mut pages = HashMap::new();

    for target in &request.pages {
        if let Some(page_id) = target.page_id.as_deref() {
            pages.entry(page_id.to_string()).or_insert(StubPage {
                version: 7,
                adf: demo_page(),
            });
        }
        if let Some(create) = &target.create {
            pages
                .entry(create.parent_page_id.clone())
                .or_insert(StubPage {
                    version: 7,
                    adf: demo_page(),
                });
        }
    }

    pages
}
