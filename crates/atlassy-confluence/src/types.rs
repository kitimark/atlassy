use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchPageResponse {
    pub page_version: u64,
    pub adf: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishPageResponse {
    pub new_version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePageResponse {
    pub page_id: String,
    pub page_version: u64,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfluenceError {
    #[error("page not found: {0}")]
    NotFound(String),
    #[error("version conflict on page: {0}")]
    Conflict(String),
    #[error("transport error: {0}")]
    Transport(String),
    #[error("operation is not implemented")]
    NotImplemented,
}

pub trait ConfluenceClient {
    fn fetch_page(&mut self, page_id: &str) -> Result<FetchPageResponse, ConfluenceError>;

    fn publish_page(
        &mut self,
        page_id: &str,
        page_version: u64,
        candidate_adf: &Value,
    ) -> Result<PublishPageResponse, ConfluenceError>;

    fn create_page(
        &mut self,
        title: &str,
        parent_page_id: &str,
        space_key: &str,
    ) -> Result<CreatePageResponse, ConfluenceError>;

    fn publish_attempts(&self) -> usize;
}
