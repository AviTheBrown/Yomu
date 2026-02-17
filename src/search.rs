use crate::client::MangaDexClient;
use crate::error::Result;
use serde::Deserialize;
use std::collections::HashMap;

/// A client for searching manga.
pub struct SearchClient<'mangaclient> {
    /// Reference to the parent `MangaDexClient`.
    pub client: &'mangaclient MangaDexClient,
}

impl<'mangaclient> SearchClient<'mangaclient> {
    /// Searches for manga by title and filters out results with "Unknown Title".
    ///
    /// # Example
    ///
    /// ```rust
    /// # use yomu::MangaDexClient;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = MangaDexClient::new()?;
    /// let search_results = client.search_client().search("chainsaw man".to_string()).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search(&self, title: String) -> Result<Vec<MangaData>> {
        let resp: reqwest::Response = self
            .client
            .http_client
            .get(&format!("{}/manga", self.client.base_url))
            .query(&[("title", title)])
            .send()
            .await?
            .error_for_status()?;
        let resp_json = resp.json::<SearchResponse>().await?;
        let filtered = resp_json
            .data
            .into_iter()
            .filter(|manga| {
                manga
                    .attributes
                    .title
                    .as_ref()
                    .and_then(|titles| titles.get("en"))
                    .is_some_and(|t| t != "Unknown Title")
            })
            .collect();
        Ok(filtered)
    }
}
/// Response from the MangaDex API for a manga search request.
#[derive(Deserialize)]
pub struct SearchResponse {
    /// Result status of the request.
    pub result: String,
    /// Type of the response (if applicable).
    pub response: Option<String>,
    /// List of manga data entries.
    pub data: Vec<MangaData>,
    /// Number of items returned in this page.
    pub limit: usize,
    /// Number of items skipped.
    pub offset: usize,
    /// Total number of items matching the query.
    pub total: usize,
}
/// Data representation of a single manga entry.
#[derive(Deserialize)]
pub struct MangaData {
    /// Unique identifier for the manga.
    pub id: String,
    /// Resource type (usually "manga").
    #[serde(rename = "type")]
    pub type_: String,
    /// Attributes containing manga information.
    pub attributes: MangaAttributes,
}
/// Attributes associated with a manga.
#[derive(Deserialize, Debug)]
pub struct MangaAttributes {
    /// Map of titles in different languages.
    pub title: Option<HashMap<String, String>>,
    /// Map of descriptions in different languages.
    pub description: Option<HashMap<String, String>>,
    /// Targeted publication demographic.
    pub publication_demographic: Option<String>,
    /// Serialization status of the manga.
    pub status: Option<String>,
    /// Year of publication.
    pub year: Option<usize>,
}
