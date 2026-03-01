use crate::chapter::ChapterClient;
use crate::error::Result;
use crate::image::ImageClient;
use crate::search::SearchClient;
use std::time::Duration;

/// A client for interacting with the MangaDex API.
///
/// This client serves as the entry point for searching manga,
/// listing chapters, and fetching image data.
///
/// # Example
///
/// ```rust
/// use yomu::MangaDexClient;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = MangaDexClient::new()?;
///     let search_results = client.search_client().search("chainsaw man".to_string()).await?;
///     println!("Found {} results", search_results.len());
///     Ok(())
/// }
/// ```
pub struct MangaDexClient {
    http_client: reqwest::Client,
    pub(crate) base_url: String,
}
impl MangaDexClient {
    /// Creates a new `MangaDexClient` with default settings.
    pub fn new() -> Result<Self> {
        Ok(Self {
            http_client: reqwest::Client::builder()
                .user_agent("Yomu/0.1.0")
                .timeout(Duration::from_secs(30))
                .build()?,
            base_url: "https://api.mangadex.org".into(),
        })
    }

    /// Returns a reference to the underlying HTTP client.
    ///
    /// Provides read-only access to the shared `reqwest::Client` for making
    /// raw HTTP requests, such as downloading images directly from CDN servers.
    ///
    /// # Example
    ///
    /// ```rust
    /// use yomu::MangaDexClient;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = MangaDexClient::new()?;
    /// let http = client.http_client();
    /// let resp = http.get("https://example.com").send().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }

    /// Returns a `SearchClient` for searching manga.
    pub fn search_client<'mangaclient>(&'mangaclient self) -> SearchClient<'mangaclient> {
        return SearchClient { client: self };
    }

    /// Returns a `ChapterClient` for fetching chapter data.
    pub fn chapter_client<'mangaclient>(&'mangaclient self) -> ChapterClient<'mangaclient> {
        return ChapterClient { client: self };
    }
    /// Returns an `ImageClient` for fetching image data and URLs.
    pub fn image_client<'mangaclient>(&'mangaclient self) -> ImageClient<'mangaclient> {
        return ImageClient { client: self };
    }
}
