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

impl StubConfluenceClient {
    fn synthetic_page_id(title: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        title.hash(&mut hasher);
        format!("stub-{}", hasher.finish())
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

    fn create_page(
        &mut self,
        title: &str,
        parent_page_id: &str,
        _space_key: &str,
    ) -> Result<CreatePageResponse, ConfluenceError> {
        if !self.pages.contains_key(parent_page_id) {
            return Err(ConfluenceError::NotFound(parent_page_id.to_string()));
        }

        let page_id = Self::synthetic_page_id(title);
        if self.pages.contains_key(&page_id) {
            return Err(ConfluenceError::Transport(format!(
                "http_status=400 body=A page with title '{title}' already exists in this space"
            )));
        }

        let empty_adf = serde_json::json!({
            "type": "doc",
            "version": 1,
            "content": []
        });

        self.pages.insert(
            page_id.clone(),
            StubPage {
                version: 1,
                adf: empty_adf,
            },
        );

        Ok(CreatePageResponse {
            page_id,
            page_version: 1,
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

    fn content_collection_endpoint(&self) -> String {
        format!(
            "{}/wiki/rest/api/content",
            self.base_url.trim_end_matches('/')
        )
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

    fn build_create_payload(
        title: &str,
        parent_page_id: &str,
        space_key: &str,
    ) -> Result<Value, ConfluenceError> {
        let empty_adf = serde_json::json!({
            "type": "doc",
            "version": 1,
            "content": []
        });
        let adf_value = serde_json::to_string(&empty_adf)
            .map_err(|error| ConfluenceError::Transport(error.to_string()))?;

        Ok(serde_json::json!({
            "type": "page",
            "status": "current",
            "title": title,
            "ancestors": [{ "id": parent_page_id }],
            "space": { "key": space_key },
            "body": {
                "atlas_doc_format": {
                    "value": adf_value,
                    "representation": "atlas_doc_format"
                }
            }
        }))
    }

    fn build_publish_payload(
        page_id: &str,
        title: &str,
        page_version: u64,
        candidate_adf: &Value,
    ) -> Result<Value, ConfluenceError> {
        let adf_value = serde_json::to_string(candidate_adf)
            .map_err(|error| ConfluenceError::Transport(error.to_string()))?;

        Ok(serde_json::json!({
            "id": page_id,
            "type": "page",
            "status": "current",
            "title": title,
            "version": { "number": page_version + 1 },
            "body": {
                "atlas_doc_format": {
                    "value": adf_value,
                    "representation": "atlas_doc_format"
                }
            }
        }))
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

        let publish_payload =
            Self::build_publish_payload(page_id, &metadata.title, page_version, candidate_adf)?;

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

    fn create_page(
        &mut self,
        title: &str,
        parent_page_id: &str,
        space_key: &str,
    ) -> Result<CreatePageResponse, ConfluenceError> {
        #[derive(Debug, Deserialize)]
        struct Version {
            number: u64,
        }
        #[derive(Debug, Deserialize)]
        struct CreateResponse {
            id: String,
            version: Version,
        }

        let payload = Self::build_create_payload(title, parent_page_id, space_key)?;

        let url = self.content_collection_endpoint();
        let response = self
            .request(reqwest::Method::POST, url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(&payload)
            .send()
            .map_err(|error| ConfluenceError::Transport(error.to_string()))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(ConfluenceError::NotFound(parent_page_id.to_string()));
        }
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(Self::parse_http_error(status, &body));
        }

        let result = response
            .json::<CreateResponse>()
            .map_err(|error| ConfluenceError::Transport(error.to_string()))?;

        Ok(CreatePageResponse {
            page_id: result.id,
            page_version: result.version.number,
        })
    }

    fn publish_attempts(&self) -> usize {
        self.publish_attempts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publish_payload_includes_required_contract_fields() {
        let candidate_adf = serde_json::json!({
            "type": "doc",
            "version": 1,
            "content": []
        });

        let payload = LiveConfluenceClient::build_publish_payload(
            "18841604",
            "Sandbox page",
            7,
            &candidate_adf,
        )
        .expect("payload should build");

        assert_eq!(payload["id"], serde_json::json!("18841604"));
        assert_eq!(payload["type"], serde_json::json!("page"));
        assert_eq!(payload["status"], serde_json::json!("current"));
        assert_eq!(payload["version"]["number"], serde_json::json!(8));
        assert_eq!(
            payload["body"]["atlas_doc_format"]["representation"],
            serde_json::json!("atlas_doc_format")
        );
    }

    #[test]
    fn publish_payload_encodes_candidate_adf_as_json_string_value() {
        let candidate_adf = serde_json::json!({
            "type": "doc",
            "version": 1,
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        { "type": "text", "text": "hello" }
                    ]
                }
            ]
        });

        let payload = LiveConfluenceClient::build_publish_payload(
            "18841604",
            "Sandbox page",
            1,
            &candidate_adf,
        )
        .expect("payload should build");

        let encoded = payload["body"]["atlas_doc_format"]["value"]
            .as_str()
            .expect("atlas_doc_format.value should be a string");
        let decoded: Value = serde_json::from_str(encoded).expect("encoded value should be JSON");
        assert_eq!(decoded, candidate_adf);
    }

    #[test]
    fn create_payload_includes_space_key_and_ancestors() {
        let payload = LiveConfluenceClient::build_create_payload("Child Page", "parent-123", "DEV")
            .expect("payload should build");

        assert_eq!(payload["type"], serde_json::json!("page"));
        assert_eq!(payload["status"], serde_json::json!("current"));
        assert_eq!(payload["title"], serde_json::json!("Child Page"));
        assert_eq!(payload["space"]["key"], serde_json::json!("DEV"));
        assert_eq!(
            payload["ancestors"][0]["id"],
            serde_json::json!("parent-123")
        );
    }

    #[test]
    fn create_payload_encodes_empty_adf_and_has_no_version() {
        let payload = LiveConfluenceClient::build_create_payload("New Page", "parent-1", "SPACE")
            .expect("payload should build");

        assert!(
            payload.get("version").is_none(),
            "create payload should not include version"
        );
        assert!(
            payload.get("id").is_none(),
            "create payload should not include id"
        );

        let encoded = payload["body"]["atlas_doc_format"]["value"]
            .as_str()
            .expect("atlas_doc_format.value should be a string");
        let decoded: Value = serde_json::from_str(encoded).expect("encoded value should be JSON");
        assert_eq!(
            decoded,
            serde_json::json!({"type": "doc", "version": 1, "content": []})
        );
        assert_eq!(
            payload["body"]["atlas_doc_format"]["representation"],
            serde_json::json!("atlas_doc_format")
        );
    }

    #[test]
    fn stub_create_page_inserts_into_store() {
        let mut pages = HashMap::new();
        pages.insert(
            "parent-1".to_string(),
            StubPage {
                version: 1,
                adf: serde_json::json!({"type": "doc", "version": 1, "content": []}),
            },
        );
        let mut client = StubConfluenceClient::new(pages);

        let result = client
            .create_page("Test Page", "parent-1", "SPACE")
            .unwrap();
        assert_eq!(result.page_version, 1);
        assert!(!result.page_id.is_empty());

        // New page is fetchable
        let fetched = client.fetch_page(&result.page_id).unwrap();
        assert_eq!(fetched.page_version, 1);
    }

    #[test]
    fn stub_create_page_rejects_missing_parent() {
        let mut client = StubConfluenceClient::new(HashMap::new());
        let result = client.create_page("Test", "missing-parent", "SPACE");
        assert!(matches!(result, Err(ConfluenceError::NotFound(_))));
    }

    #[test]
    fn stub_create_page_rejects_duplicate_title() {
        let mut pages = HashMap::new();
        pages.insert(
            "parent-1".to_string(),
            StubPage {
                version: 1,
                adf: serde_json::json!({"type": "doc", "version": 1, "content": []}),
            },
        );
        let mut client = StubConfluenceClient::new(pages);

        client.create_page("Dup Page", "parent-1", "SPACE").unwrap();
        let result = client.create_page("Dup Page", "parent-1", "SPACE");
        assert!(matches!(result, Err(ConfluenceError::Transport(_))));
    }
}
