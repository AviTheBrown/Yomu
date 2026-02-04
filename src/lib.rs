use serde::Deserialize;
use std::collections::HashMap;

pub struct MangaDexClient {
    pub http_client: reqwest::Client,
    pub base_url: String,
}
impl MangaDexClient {
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::builder()
                .user_agent("Yomu/0.1.0")
                .build()
                .unwrap(),
            base_url: "https://api.mangadex.org".into(),
        }
    }
    pub fn search_client<'mangaclient>(&'mangaclient self) -> SearchClient<'mangaclient> {
        return SearchClient { client: self };
    }
    pub fn chapter_client<'mangaclient>(&'mangaclient self) -> ChapterClient<'mangaclient> {
        return ChapterClient { client: self };
    }
}
pub struct ChapterClient<'mangaclient> {
    pub client: &'mangaclient MangaDexClient,
}
impl<'mangaclient> ChapterClient<'mangaclient> {
    pub async fn fetch_chapter(
        &self,
        manga_id: String,
    ) -> Result<Vec<ChapterData>, reqwest::Error> {
        let resp = self
            .client
            .http_client
            .get(&format!("{}/manga/{}/feed", self.client.base_url, manga_id))
            .send()
            .await?;
        let resp_json = resp.json::<ChapterResponse>().await?;
        Ok(resp_json.data)
    }
}
#[derive(Deserialize)]
pub struct ChapterResponse {
    pub result: String,
    pub response: String,
    pub data: Vec<ChapterData>,
    pub limit: usize,
    pub offset: usize,
    pub total: usize,
}
#[derive(Deserialize)]
pub struct ChapterData {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub attributes: ChapterAttributes,
}
#[derive(Deserialize, Debug)]
pub struct ChapterAttributes {
    pub volume: Option<String>,
    pub chapter: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "translatedLanguage")]
    pub translated_language: Option<String>,
    #[serde(rename = "isUnavailable")]
    pub is_unavailable: bool,
    pub pages: Option<usize>,
}
pub struct SearchClient<'mangaclient> {
    pub client: &'mangaclient MangaDexClient,
}
impl<'mangaclient> SearchClient<'mangaclient> {
    pub async fn search(&self, title: String) -> Result<Vec<MangaData>, reqwest::Error> {
        let resp = self
            .client
            .http_client
            .get(&format!("{}/manga", self.client.base_url))
            .query(&[("title", title)])
            .send()
            .await?;
        let resp_json = resp.json::<SearchResponse>().await?;
        Ok(resp_json.data)
    }
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
#[derive(Deserialize, Debug)]
pub struct MangaAttributes {
    pub title: Option<HashMap<String, String>>,
    pub description: Option<HashMap<String, String>>,
    pub publication_demographic: Option<String>,
    pub status: Option<String>,
    pub year: Option<usize>,
}

#[tokio::test]
async fn test_search() {
    let client = MangaDexClient::new();
    let search_client = client.search_client();

    let search_results = search_client
        .search("chainsaw man".to_string())
        .await
        .unwrap();

    println!("Found {} results", search_results.len());

    for (i, manga) in search_results.iter().take(3).enumerate() {
        println!("\n[{}] ID: {}", i + 1, manga.id);
        if let Some(ref titles) = manga.attributes.title {
            if let Some(en_title) = titles.get("en") {
                println!("    Title: {}", en_title);
            }
        }
    }

    assert!(!search_results.is_empty());
}
#[tokio::test]
async fn test_chapter() {
    let client = MangaDexClient::new();
    let chapter_client = client.chapter_client();
    let chpt_result = chapter_client
        .fetch_chapter("a77742b1-befd-49a4-bff5-1ad4e6b0ef7b".into())
        .await
        .unwrap();
    println!("Found {} chaptes", chpt_result.len());
    for chapter in chpt_result.iter().take(10) {
        if let Some(ref chpt_num) = chapter.attributes.chapter {
            println!(
                "Chapters: {} | Language: {:?}",
                chpt_num, chapter.attributes.translated_language
            )
        }
    }
    }







