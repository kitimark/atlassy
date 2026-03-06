use std::collections::HashMap;
use std::env;

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
    http_client: reqwest::blocking::Client,
    base_url: String,
    email: String,
    api_token: String,
    publish_attempts: usize,
}

impl LiveConfluenceClient {
    pub fn new(
        base_url: impl Into<String>,
        email: impl Into<String>,
        api_token: impl Into<String>,
    ) -> Self {
        Self {
            http_client: reqwest::blocking::Client::new(),
            base_url: base_url.into(),
            email: email.into(),
            api_token: api_token.into(),
            publish_attempts: 0,
        }
    }

    pub fn from_env() -> Result<Self, ConfluenceError> {
        let base_url = env::var("ATLASSY_CONFLUENCE_BASE_URL").map_err(|_| {
            ConfluenceError::Transport("missing ATLASSY_CONFLUENCE_BASE_URL".to_string())
        })?;
        let email = env::var("ATLASSY_CONFLUENCE_EMAIL").map_err(|_| {
            ConfluenceError::Transport("missing ATLASSY_CONFLUENCE_EMAIL".to_string())
        })?;
        let api_token = env::var("ATLASSY_CONFLUENCE_API_TOKEN").map_err(|_| {
            ConfluenceError::Transport("missing ATLASSY_CONFLUENCE_API_TOKEN".to_string())
        })?;

        Ok(Self::new(base_url, email, api_token))
    }

    fn content_endpoint(&self, page_id: &str) -> String {
        format!(
            "{}/wiki/rest/api/content/{page_id}",
            self.base_url.trim_end_matches('/')
        )
    }

    fn request(&self, method: reqwest::Method, url: String) -> reqwest::blocking::RequestBuilder {
        self.http_client
            .request(method, url)
            .basic_auth(&self.email, Some(&self.api_token))
            .header(reqwest::header::ACCEPT, "application/json")
    }

    fn parse_adf_value(value: &serde_json::Value) -> Result<serde_json::Value, ConfluenceError> {
        match value {
            serde_json::Value::Object(_) => Ok(value.clone()),
            serde_json::Value::String(raw) => serde_json::from_str(raw).map_err(|error| {
                ConfluenceError::Transport(format!("invalid ADF payload: {error}"))
            }),
            _ => Err(ConfluenceError::Transport(
                "unexpected atlas_doc_format.value payload type".to_string(),
            )),
        }
    }

    fn parse_http_error(status: reqwest::StatusCode, body: &str) -> ConfluenceError {
        let snippet = body.chars().take(240).collect::<String>();
        ConfluenceError::Transport(format!("http_status={} body={snippet}", status.as_u16()))
    }
}

impl ConfluenceClient for LiveConfluenceClient {
    fn fetch_page(&mut self, page_id: &str) -> Result<FetchPageResponse, ConfluenceError> {
        #[derive(Debug, Deserialize)]
        struct Version {
            number: u64,
        }
        #[derive(Debug, Deserialize)]
        struct AtlasDocFormat {
            value: serde_json::Value,
        }
        #[derive(Debug, Deserialize)]
        struct Body {
            atlas_doc_format: AtlasDocFormat,
        }
        #[derive(Debug, Deserialize)]
        struct FetchResponse {
            version: Version,
            body: Body,
        }

        let url = format!(
            "{}?expand=version,body.atlas_doc_format",
            self.content_endpoint(page_id)
        );

        let response = self
            .request(reqwest::Method::GET, url)
            .send()
            .map_err(|error| ConfluenceError::Transport(error.to_string()))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(ConfluenceError::NotFound(page_id.to_string()));
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(Self::parse_http_error(status, &body));
        }

        let payload = response
            .json::<FetchResponse>()
            .map_err(|error| ConfluenceError::Transport(error.to_string()))?;
        let adf = Self::parse_adf_value(&payload.body.atlas_doc_format.value)?;

        Ok(FetchPageResponse {
            page_version: payload.version.number,
            adf,
        })
    }

    fn publish_page(
        &mut self,
        page_id: &str,
        page_version: u64,
        candidate_adf: &Value,
    ) -> Result<PublishPageResponse, ConfluenceError> {
        #[derive(Debug, Deserialize)]
        struct Version {
            number: u64,
        }
        #[derive(Debug, Deserialize)]
        struct FetchMetadata {
            title: String,
            version: Version,
        }
        #[derive(Debug, Deserialize)]
        struct PublishResponse {
            version: Version,
        }

        self.publish_attempts += 1;

        let metadata_url = format!("{}?expand=version,title", self.content_endpoint(page_id));
        let metadata_response = self
            .request(reqwest::Method::GET, metadata_url)
            .send()
            .map_err(|error| ConfluenceError::Transport(error.to_string()))?;

        if metadata_response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(ConfluenceError::NotFound(page_id.to_string()));
        }

        if !metadata_response.status().is_success() {
            let status = metadata_response.status();
            let body = metadata_response.text().unwrap_or_default();
            return Err(Self::parse_http_error(status, &body));
        }

        let metadata = metadata_response
            .json::<FetchMetadata>()
            .map_err(|error| ConfluenceError::Transport(error.to_string()))?;

        if metadata.version.number != page_version {
            return Err(ConfluenceError::Conflict(page_id.to_string()));
        }

        let publish_payload = serde_json::json!({
            "id": page_id,
            "type": "page",
            "title": metadata.title,
            "version": { "number": page_version + 1 },
            "body": {
                "atlas_doc_format": {
                    "value": candidate_adf,
                    "representation": "atlas_doc_format"
                }
            }
        });

        let publish_response = self
            .request(reqwest::Method::PUT, self.content_endpoint(page_id))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(&publish_payload)
            .send()
            .map_err(|error| ConfluenceError::Transport(error.to_string()))?;

        if publish_response.status() == reqwest::StatusCode::CONFLICT {
            return Err(ConfluenceError::Conflict(page_id.to_string()));
        }
        if publish_response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(ConfluenceError::NotFound(page_id.to_string()));
        }
        if !publish_response.status().is_success() {
            let status = publish_response.status();
            let body = publish_response.text().unwrap_or_default();
            return Err(Self::parse_http_error(status, &body));
        }

        let payload = publish_response
            .json::<PublishResponse>()
            .map_err(|error| ConfluenceError::Transport(error.to_string()))?;

        Ok(PublishPageResponse {
            new_version: payload.version.number,
        })
    }

    fn publish_attempts(&self) -> usize {
        self.publish_attempts
    }
}
