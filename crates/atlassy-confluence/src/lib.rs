use std::collections::HashMap;

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

    fn publish_attempts(&self) -> usize;
}

#[derive(Debug, Clone)]
pub struct StubPage {
    pub version: u64,
    pub adf: Value,
}

#[derive(Debug, Clone)]
pub struct StubConfluenceClient {
    pages: HashMap<String, StubPage>,
    conflict_once: bool,
    always_conflict: bool,
    publish_attempts: usize,
}

impl StubConfluenceClient {
    pub fn new(pages: HashMap<String, StubPage>) -> Self {
        Self {
            pages,
            conflict_once: false,
            always_conflict: false,
            publish_attempts: 0,
        }
    }

    pub fn with_conflict_once(mut self) -> Self {
        self.conflict_once = true;
        self
    }

    pub fn with_always_conflict(mut self) -> Self {
        self.always_conflict = true;
        self
    }
}

impl ConfluenceClient for StubConfluenceClient {
    fn fetch_page(&mut self, page_id: &str) -> Result<FetchPageResponse, ConfluenceError> {
        let page = self
            .pages
            .get(page_id)
            .ok_or_else(|| ConfluenceError::NotFound(page_id.to_string()))?;
        Ok(FetchPageResponse {
            page_version: page.version,
            adf: page.adf.clone(),
        })
    }

    fn publish_page(
        &mut self,
        page_id: &str,
        page_version: u64,
        candidate_adf: &Value,
    ) -> Result<PublishPageResponse, ConfluenceError> {
        self.publish_attempts += 1;
        let page = self
            .pages
            .get_mut(page_id)
            .ok_or_else(|| ConfluenceError::NotFound(page_id.to_string()))?;

        if self.always_conflict {
            return Err(ConfluenceError::Conflict(page_id.to_string()));
        }

        if self.conflict_once {
            self.conflict_once = false;
            return Err(ConfluenceError::Conflict(page_id.to_string()));
        }

        if page.version != page_version {
            return Err(ConfluenceError::Conflict(page_id.to_string()));
        }

        page.version += 1;
        page.adf = candidate_adf.clone();
        Ok(PublishPageResponse {
            new_version: page.version,
        })
    }

    fn publish_attempts(&self) -> usize {
        self.publish_attempts
    }
}

pub struct LiveConfluenceClient {
    #[allow(dead_code)]
    http_client: reqwest::Client,
    #[allow(dead_code)]
    base_url: String,
}

impl LiveConfluenceClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            base_url: base_url.into(),
        }
    }
}

impl ConfluenceClient for LiveConfluenceClient {
    fn fetch_page(&mut self, _page_id: &str) -> Result<FetchPageResponse, ConfluenceError> {
        Err(ConfluenceError::NotImplemented)
    }

    fn publish_page(
        &mut self,
        _page_id: &str,
        _page_version: u64,
        _candidate_adf: &Value,
    ) -> Result<PublishPageResponse, ConfluenceError> {
        Err(ConfluenceError::NotImplemented)
    }

    fn publish_attempts(&self) -> usize {
        0
    }
}
