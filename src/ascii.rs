use crate::error::Result;
use image::{imageops::FilterType, load_from_memory};
/// Converts raw image bytes into an ASCII art string.
///
/// This function resizes the image to the specified width and height,
/// converts it to grayscale, and then maps brightness levels to ASCII characters.
///
/// # Example
///
/// ```rust
/// use yomu::ascii::convert_to_ascii;
///
/// let bytes = include_bytes!("../tests/test_image.jpg");
/// let ascii = convert_to_ascii(bytes, 80, 40)?;
/// println!("{}", ascii);
/// # Ok::<(), yomu::error::YomuError>(())
/// ```
pub fn convert_to_ascii(bytes: &[u8], width: u32, height: u32) -> Result<String> {
    // Professional 70-character grayscale set (ordered from dark to light)
    let ascii_chars = "$@B%8&WM#*oahkbdpqwmZO0QLCJUYXzcvunxrjft/\\|()1{}[]?-_+~<>i!lI;:,\"^`'. ";
    
    let img = load_from_memory(bytes)?
        .resize(width, height / 2, FilterType::Triangle)
        .to_luma8();

    let mut ascii_art = String::with_capacity((img.len() as usize + height as usize) * 2);
    let char_list: Vec<char> = ascii_chars.chars().collect();
    let num_chars = char_list.len();

    for (i, &brightness) in img.iter().enumerate() {
        // Map brightness such that 255 (white) is ' ' and 0 (black) is '$' (darkest)
        // Since the set is dark-to-light, we use the brightness directly to pick from the end
        let index = (brightness as usize * (num_chars - 1)) / 255;
        ascii_art.push(char_list[index]);

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
