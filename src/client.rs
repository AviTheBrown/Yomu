use crate::chapter::ChapterClient;
use crate::error::Result;
use crate::image::ImageClient;
use crate::search::SearchClient;

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
    /// The underlying HTTP client used for making requests.
    pub http_client: reqwest::Client,
    /// The base URL for the MangaDex API.
    pub base_url: String,
}
impl MangaDexClient {
    /// Creates a new `MangaDexClient` with default settings.
    pub fn new() -> Result<Self> {
        Ok(Self {
            http_client: reqwest::Client::builder()
                .user_agent("Yomu/0.1.0")
                .build()?,
            base_url: "https://api.mangadex.org".into(),
        })
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
