use crate::client::MangaDexClient;
use crate::error::{Result, YomuError};
use image::DynamicImage;
use serde::Deserialize;

/// Maximum bytes accepted for a single image download (50 MB).
const MAX_IMAGE_BYTES: u64 = 50 * 1024 * 1024;

/// A client for fetching image-related data from MangaDex @ Home servers.
pub struct ImageClient<'mangaclient> {
    /// Reference to the parent `MangaDexClient`.
    pub client: &'mangaclient MangaDexClient,
}
impl<'mangaclient> ImageClient<'mangaclient> {
    /// Fetches image filenames and server information for a specific chapter ID.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use yomu::MangaDexClient;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = MangaDexClient::new()?;
    /// let chapter_id = "73af4d8d-1532-4a72-b1b9-8f4e5cd295c9";
    /// let image_data = client.image_client().fetch_image_data(chapter_id).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fetch_image_data(&self, chapter_id: &str) -> Result<ImageDataResponse> {
        let fetch_url = format!("{}/at-home/server/{}", self.client.base_url, chapter_id);
        let resp: reqwest::Response = self
            .client
            .http_client()
            .get(fetch_url)
            .send()
            .await?
            .error_for_status()?;
        let resp_json = resp.json::<ImageDataResponse>().await?;
        if !resp_json.base_url.starts_with("https://") {
            return Err(YomuError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "unsafe base_url from API: expected https://, got \"{}\"",
                    resp_json.base_url
                ),
            )));
        }
        Ok(resp_json)
    }
    /// Downloads an image from the given URL and returns the raw bytes.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use yomu::MangaDexClient;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = MangaDexClient::new()?;
    /// let url = "https://example.com/image.jpg";
    /// let bytes = client.image_client().download_image_bytes(url).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn download_image_bytes(&self, url: &str) -> Result<Vec<u8>> {
        let resp = self.client.http_client().get(url).send().await?;
        if resp.content_length().map_or(false, |len| len > MAX_IMAGE_BYTES) {
            return Err(YomuError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "image response exceeds size limit",
            )));
        }
        let bytes = resp.bytes().await?;
        if bytes.len() as u64 > MAX_IMAGE_BYTES {
            return Err(YomuError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "image response exceeds size limit",
            )));
        }
        Ok(bytes.into())
    }
    /// Downloads an image from the given URL and decodes it into a `DynamicImage`.
    pub async fn download_image(&self, url: &str) -> Result<DynamicImage> {
        let bytes: Vec<u8> = self.download_image_bytes(url).await?;
        let img = image::load_from_memory(&bytes)?;
        Ok(img)
    }
}

/// Response from the MangaDex API for a chapter's image data.
#[derive(Deserialize, Debug, Clone)]
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
#[derive(Deserialize, Debug, Clone)]
pub struct ImageAttributes {
    /// The content hash used in the image URL path.
    pub hash: String,
    /// Filenames for high-quality images.
    pub data: Vec<String>,
    /// Filenames for compressed (data saver) images.
    #[serde(rename = "dataSaver")]
    pub data_saver: Vec<String>,
}

#[cfg(test)]
mod test {
    use crate::client::MangaDexClient;

    #[tokio::test]
    #[ignore = "requires live network access"]
    async fn test_img_download() {
        let client = MangaDexClient::new().unwrap();
        let img_client = client.image_client();
        let img_dwbl = img_client.download_image_bytes("https://cmdxd98sb0x3yprd.mangadex.network/data/d828fa5fffd26b264ad400b3b0fdffe8/A1-612f24d412cc157e7221bd8a051d5d564adcd539931b8c0bd58b691c07bf8c90.jpg").await.unwrap();
        println!("the image has: {} bytes", img_dwbl.len());
    }
}
