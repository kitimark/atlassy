use std::collections::HashMap;

use serde_json::Value;

use crate::{
    ConfluenceClient, ConfluenceError, CreatePageResponse, FetchPageResponse, PublishPageResponse,
};

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
