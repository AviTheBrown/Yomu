pub mod chapter;
pub mod client;
pub mod image;
pub mod search;
pub use chapter::ChapterClient;
pub use client::MangaDexClient;
pub use image::ImageClient;
pub use search::SearchClient;

#[cfg(test)]
mod tests {
    use crate::client::MangaDexClient;

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
            .fetch_chapter("a77742b1-befd-49a4-bff5-1ad4e6b0ef7b".into(), Some("en"))
            .await
            .unwrap();
        println!("Found {} chapters", chpt_result.len());
        for chapter in chpt_result.iter().take(10) {
            if let Some(ref chpt_num) = chapter.attributes.chapter {
                println!(
                    "Chapter {} | Language: {:?} | Pages: {:?} | ID: {:?}",
                    chpt_num,
                    chapter.attributes.translated_language,
                    chapter.attributes.pages,
                    &chapter.id[..8]
                )
            }
        }
    }

    #[tokio::test]
    async fn fetch_image() {
        let client = MangaDexClient::new();
        let imgage_fetch_client = client.image_client();
        let img_results = imgage_fetch_client
            .fetch_image_data("73af4d8d-1532-4a72-b1b9-8f4e5cd295c9")
            .await
            .unwrap();
        println!("there are {} pages", img_results.chapter.data.len());

        for (i, filename) in img_results.chapter.data.iter().take(3).enumerate() {
            let url = format!(
                "{}/data/{}/{}",
                img_results.base_url, img_results.chapter.hash, filename
            );
            println!("Page {}: {}", i + 1, url);
        }
    }
}
