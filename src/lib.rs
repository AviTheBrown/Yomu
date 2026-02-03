use std::collections::HashMap;

use serde::Deserialize;

pub struct MangaDexClient {
    pub http_client: reqwest::Client,
    pub base_url: String,
}
pub struct SearchClient<'mangaclient> {
    pub client: &'mangaclient MangaDexClient,
}
impl SearchClient {
    pub fn search(&self, title: String) {}
}
#[derive(Deserialize)]
pub struct SearchResponse {
    pub result: String,
    pub response: String,
    pub data: Vec<MangaData>,
    pub limit: usize,
    pub offset: usize,
    pub total: usize,
}
#[derive(Deserialize)]
pub struct MangaData {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub attributes: MangaAttributes,
}
#[derive(Deserialize)]
pub struct MangaAttributes {
    pub title: Option<HashMap<String, String>>,
    pub description: Option<HashMap<String, String>>,
    pub publication_demographic: Option<String>,
    pub status: Option<String>,
    pub year: Option<usize>,
}

impl MangaDexClient {
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::new(),
            base_url: "https://api.mangadex.org".into(),
        }
    }
    pub fn search_client<'mangaclient>(&'mangaclient self) -> SearchClient<'mangaclient> {
        return SearchClient { client: self };
    }
}
