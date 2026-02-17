use crate::error::Result;
use image::{imageops::FilterType, load_from_memory};
/// Converts raw image bytes into an ASCII art string.
///
/// This function resizes the image to the specified width and height,
/// converts it to grayscale, and then maps brightness levels to ASCII characters.
pub fn convert_to_ascii(bytes: &[u8], width: u32, height: u32) -> Result<String> {
    let ascii_chars = " .:-=+*#@";
    let img = load_from_memory(bytes)?
        .resize(width, height / 2, FilterType::Nearest)
        .grayscale()
        .into_bytes();
    let mut ascii_art = String::with_capacity((img.len() + height as usize) * 2);
    for (i, &brightness) in img.iter().enumerate() {
        let index = (brightness as usize * 8 / 255).min(8);
        ascii_art.push(ascii_chars.chars().nth(index).unwrap_or('@'));
        if (i + 1) % width as usize == 0 && i + 1 < img.len() {
            ascii_art.push('\n');
        }
    }
    Ok(ascii_art)
}
#[cfg(test)]
mod test {
    use crate::{ascii::convert_to_ascii, client::MangaDexClient};

    #[tokio::test]
    async fn test_img_download() {
        let client = MangaDexClient::new().unwrap();
        let img_client = client.image_client();
        let img_dwbl = img_client.download_image_bytes("https://cmdxd98sb0x3yprd.mangadex.network/data/d828fa5fffd26b264ad400b3b0fdffe8/A1-612f24d412cc157e7221bd8a051d5d564adcd539931b8c0bd58b691c07bf8c90.jpg").await.unwrap();
        println!("the image has: {} bytes", img_dwbl.len());
        let img = convert_to_ascii(&img_dwbl, 200, 100).unwrap();
        println!("{}", img);
    }
}
