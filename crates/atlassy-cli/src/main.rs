use std::collections::HashMap;
use std::path::PathBuf;

use atlassy_confluence::{StubConfluenceClient, StubPage};
use atlassy_pipeline::{Orchestrator, RunMode, RunRequest};
use clap::{Parser, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "atlassy")]
#[command(about = "Atlassy CLI for v1 pipeline execution")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
    Run {
        #[arg(long)]
        request_id: String,
        #[arg(long)]
        page_id: String,
        #[arg(long)]
        edit_intent: String,
        #[arg(long = "scope")]
        scope_selectors: Vec<String>,
        #[arg(long, default_value = ".")]
        artifacts_dir: PathBuf,
        #[arg(long, value_enum, default_value_t = CliMode::NoOp)]
        mode: CliMode,
        #[arg(long)]
        target_path: Option<String>,
        #[arg(long)]
        new_value: Option<String>,
        #[arg(long)]
        force_verify_fail: bool,
    },
}

#[derive(Debug, Clone, ValueEnum)]
enum CliMode {
    NoOp,
    SimpleScopedUpdate,
    SimpleScopedProseUpdate,
    SimpleScopedTableCellUpdate,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            request_id,
            page_id,
            edit_intent,
            scope_selectors,
            artifacts_dir,
            mode,
            target_path,
            new_value,
            force_verify_fail,
        } => {
            let run_mode = match mode {
                CliMode::NoOp => RunMode::NoOp,
                CliMode::SimpleScopedUpdate => {
                    let path =
                        target_path.unwrap_or_else(|| "/content/1/content/0/text".to_string());
                    let value =
                        serde_json::json!(new_value.unwrap_or_else(|| "Updated text".to_string()));
                    RunMode::SimpleScopedUpdate {
                        target_path: path,
                        new_value: value,
                    }
                }
                CliMode::SimpleScopedProseUpdate => {
                    let path =
                        target_path.unwrap_or_else(|| "/content/1/content/0/text".to_string());
                    let markdown = new_value.unwrap_or_else(|| "Updated prose body".to_string());
                    RunMode::SimpleScopedProseUpdate {
                        target_path: path,
                        markdown,
                    }
                }
                CliMode::SimpleScopedTableCellUpdate => {
                    let path = target_path.unwrap_or_else(|| {
                        "/content/2/content/0/content/0/content/0/content/0/text".to_string()
                    });
                    let text = new_value.unwrap_or_else(|| "Updated table cell".to_string());
                    RunMode::SimpleScopedTableCellUpdate {
                        target_path: path,
                        text,
                    }
                }
            };

            let mut pages = HashMap::new();
            pages.insert(
                page_id.clone(),
                StubPage {
                    version: 1,
                    adf: demo_page(),
                },
            );

            let mut orchestrator =
                Orchestrator::new(StubConfluenceClient::new(pages), artifacts_dir);
            let request = RunRequest {
                request_id,
                page_id,
                edit_intent,
                scope_selectors,
                timestamp: "2026-03-06T10:00:00Z".to_string(),
                run_mode,
                force_verify_fail,
            };

            match orchestrator.run(request) {
                Ok(summary) => println!("{}", serde_json::to_string_pretty(&summary)?),
                Err(error) => {
                    eprintln!("pipeline failed: {error}");
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}

fn demo_page() -> serde_json::Value {
    serde_json::json!({
      "type": "doc",
      "version": 1,
      "content": [
        {
          "type": "heading",
          "attrs": {"level": 2, "id": "intro-heading"},
          "content": [{"type": "text", "text": "Overview"}]
        },
        {
          "type": "paragraph",
          "attrs": {"id": "intro-paragraph"},
          "content": [{"type": "text", "text": "Initial paragraph"}]
        },
        {
          "type": "table",
          "content": [
            {
              "type": "tableRow",
              "content": [
                {
                  "type": "tableCell",
                  "content": [
                    {
                      "type": "paragraph",
                      "content": [{"type": "text", "text": "Initial table cell"}]
                    }
                  ]
                }
              ]
            }
          ]
        }
      ]
    })
}
