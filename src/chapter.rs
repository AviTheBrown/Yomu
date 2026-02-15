use crate::client::MangaDexClient;
use serde::Deserialize;

/// A client for fetching chapter-related information.
pub struct ChapterClient<'mangaclient> {
    /// Reference to the parent `MangaDexClient`.
    pub client: &'mangaclient MangaDexClient,
}

impl<'mangaclient> ChapterClient<'mangaclient> {
    /// Fetches the feed (chapters) for a specific manga ID.
    pub async fn fetch_chapter(
        &self,
        manga_id: &str,
        language: Option<&str>,
    ) -> Result<Vec<ChapterData>, reqwest::Error> {
        let resp: reqwest::Response = self
            .client
            .http_client
            .get(&format!("{}/manga/{}/feed", self.client.base_url, manga_id))
            .query(&[
                ("translatedLanguage[]", language.unwrap_or("en")),
                ("order[chapter]", "asc"),
            ])
            .send()
            .await?;
        let resp_json = resp.json::<ChapterResponse>().await?;
        let filtered_json: Vec<ChapterData> = resp_json
            .data
            .into_iter()
            .filter(|chapter| match chapter.attributes.pages {
                Some(pages) => pages > 0,
                None => false,
            })
            .collect();
        Ok(filtered_json)
    }
}
/// Response from the MangaDex API for a chapter feed request.
#[derive(Deserialize)]
pub struct ChapterResponse {
    /// Result status of the request.
    pub result: String,
    /// Type of the response.
    pub response: String,
    /// List of chapter data entries.
    pub data: Vec<ChapterData>,
    /// Number of items returned in this page.
    pub limit: usize,
    /// Number of items skipped.
    pub offset: usize,
    /// Total number of items matching the query.
    pub total: usize,
}
/// Data representation of a single chapter.
#[derive(Deserialize)]
pub struct ChapterData {
    /// Unique identifier for the chapter.
    pub id: String,
    /// Resource type (usually "chapter").
    #[serde(rename = "type")]
    pub type_: String,
    /// Attributes containing chapter information.
    pub attributes: ChapterAttributes,
}
/// Attributes associated with a chapter.
#[derive(Deserialize, Debug)]
pub struct ChapterAttributes {
    /// Volume number (if applicable).
    pub volume: Option<String>,
    /// Chapter number.
    pub chapter: Option<String>,
    /// Title of the chapter.
    pub title: Option<String>,
    /// The language the chapter was translated to.
    #[serde(rename = "translatedLanguage")]
    pub translated_language: Option<String>,
    /// Whether the chapter is currently unavailable.
    #[serde(rename = "isUnavailable")]
    pub is_unavailable: bool,
    /// Number of pages in the chapter.
    pub pages: Option<usize>,
}
