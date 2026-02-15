use crate::client::MangaDexClient;
use serde::Deserialize;

/// A client for fetching image-related data from MangaDex @ Home servers.
pub struct ImageClient<'mangaclient> {
    /// Reference to the parent `MangaDexClient`.
    pub client: &'mangaclient MangaDexClient,
}
impl<'mangaclient> ImageClient<'mangaclient> {
    /// Fetches image filenames and server information for a specific chapter ID.
    pub async fn fetch_image_data(
        &self,
        chapter_id: &str,
    ) -> Result<ImageDataResponse, reqwest::Error> {
        let fetch_url = format!("{}/at-home/server/{}", self.client.base_url, chapter_id);
        let resp: reqwest::Response = self.client.http_client.get(fetch_url).send().await?;
        let resp_json = resp.json::<ImageDataResponse>().await?;
        Ok(resp_json)
    }
}

/// Response from the MangaDex API for a chapter's image data.
#[derive(Deserialize, Debug)]
pub struct ImageDataResponse {
    /// Result of the request (e.g., "ok").
    pub result: String,
    /// The base URL for fetching the images.
    #[serde(rename = "baseUrl")]
    pub base_url: String,
    /// Image data and hash for the chapter.
    pub chapter: ImageAttributes,
}
/// Image filenames and content hash for a chapter.
#[derive(Deserialize, Debug)]
pub struct ImageAttributes {
    /// The content hash used in the image URL path.
    pub hash: String,
    /// Filenames for high-quality images.
    pub data: Vec<String>,
    /// Filenames for compressed (data saver) images.
    #[serde(rename = "dataSaver")]
    pub data_saver: Vec<String>,
}
