use std::collections::HashMap;

use atlassy_confluence::{ConfluenceClient, LiveConfluenceClient, StubConfluenceClient, StubPage};
use atlassy_contracts::{RUNTIME_LIVE, RUNTIME_STUB};

use crate::DynError;

pub fn create_subpage(
    parent_page_id: &str,
    space_key: &str,
    title: &str,
    runtime_mode: &str,
) -> Result<(), DynError> {
    let result = match runtime_mode {
        RUNTIME_STUB => {
            let mut pages = HashMap::new();
            pages.insert(
                parent_page_id.to_string(),
                StubPage {
                    version: 1,
                    adf: serde_json::json!({"type": "doc", "version": 1, "content": []}),
                },
            );
            let mut client = StubConfluenceClient::new(pages);
            client
                .create_page(title, parent_page_id, space_key)
                .map_err(|error| format!("{error}"))
        }
        RUNTIME_LIVE => {
            let mut client = LiveConfluenceClient::from_env()
                .map_err(|error| format!("live runtime startup failure: {error}"))?;
            client
                .create_page(title, parent_page_id, space_key)
                .map_err(|error| format!("{error}"))
        }
        _ => {
            return Err(format!("invalid runtime mode `{runtime_mode}`").into());
        }
    };

    match result {
        Ok(response) => {
            println!("{}", serde_json::to_string_pretty(&response)?);
            Ok(())
        }
        Err(error) => Err(format!("create-subpage failed: {error}").into()),
    }
}
