use reqwest::get;
use image::{DynamicImage, GenericImageView, ImageReader, Rgba};
use std::io::Cursor;

pub async fn download_image(url: &str) -> Result<image::DynamicImage, Box<dyn std::error::Error>> {
    let response = get(url).await?.bytes().await?;
    let img = ImageReader::new(Cursor::new(response)).with_guessed_format()?.decode()?;
    Ok(img)
}

pub fn average_gb_blocks(image: &DynamicImage, x_segments: usize, y_segments: usize) -> Vec<[u8; 3]> {
    let (img_width, img_height) = image.dimensions();
    let block_width = img_width / x_segments as u32;
    let block_height = img_height / y_segments as u32;
    let mut block_averages = Vec::new();

    // Iterate over the blocks
    for y in 0..y_segments {
        for x in 0..x_segments {
            let mut sum_r = 0u64;
            let mut sum_g = 0u64;
            let mut sum_b = 0u64;
            let mut pixel_count = 0u64;

            // Determine the size of each block, handling the remainder for the last blocks
            let current_block_width = if x == x_segments - 1 {
                img_width - (block_width * (x_segments as u32 - 1))
            } else {
                block_width
            };
            let current_block_height = if y == y_segments - 1 {
                img_height - (block_height * (y_segments as u32 - 1))
            } else {
                block_height
            };

            for i in 0..current_block_width {
                for j in 0..current_block_height {
                    let pixel = image.get_pixel(x as u32 * block_width + i, y as u32 * block_height + j);
                    let Rgba([r, g, b, _]) = pixel;
                    sum_r += r as u64;
                    sum_g += g as u64;
                    sum_b += b as u64;
                    pixel_count += 1;
                }
            }

            // Compute average for the block
            block_averages.push([
                (sum_r / pixel_count) as u8,
                (sum_g / pixel_count) as u8,
                (sum_b / pixel_count) as u8,
            ]);
        }
    }
    block_averages
}

// Function to compare two block-averaged images using Chi-square
pub fn compare_images_chisquare(img1: &Vec<[u8; 3]>, img2: &Vec<[u8; 3]>) -> f64 {
    let mut chi_square = 0.0;
    let mut total_count = 0;

    for (block1, block2) in img1.iter().zip(img2.iter()) {
        for i in 0..3 {
            let expected = block1[i] as f64;
            let observed = block2[i] as f64;
            if expected > 0.0 {
                chi_square += (observed - expected).powi(2) / expected;
            }
            total_count += 1;
        }
    }

    // Normalize by the total number of comparisons
    chi_square / total_count as f64
}

fn save_image(image: &DynamicImage, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    image.save(path)?;
    Ok(())
}

pub fn smallest_dimensions(img1: &image::DynamicImage, img2: &image::DynamicImage) -> (i8, u32, u32) {
    let (width1, height1) = img1.dimensions();
    let (width2, height2) = img2.dimensions();
    if width1 * height1 < width2 * height2 {
        (0, width1, height1)
    } else {
        (1, width2, height2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const URLSMALL: &str = "https://cdn.discordapp.com/attachments/938538176841142362/1289993746829545534/20240926_032926.png?ex=6704124c&is=6702c0cc&hm=143dd6d5e1a026f465b9da2dfbc8b201ce8d16d419f9de961fab4f819f58e032&";
    const URLBIG: &str = "https://cdn.discordapp.com/attachments/835182477583581225/1288688973639585854/20240926_032926.jpg?ex=6703f061&is=67029ee1&hm=ab8bca2e2eb47af7227dab57683187b232087479ba85c9044a06fe1cebdfb280&";
    const URLOTHER: &str = "https://www.rust-lang.org/logos/rust-logo-512x512.png";
    const URLSOMEWHATSIMILAR: &str = "https://i.kym-cdn.com/photos/images/original/002/247/111/ee3.png";
    const URLSOMEWHATSIMILAR2: &str = "https://i.kym-cdn.com/photos/images/original/002/255/853/87e.jpg";

    #[tokio::test]
    async fn aspectratio_comparison() {
        let imgsmall = download_image(URLSMALL).await.unwrap();
        let imgbig = download_image(URLBIG).await.unwrap();
        assert_eq!(imgsmall.width()/imgsmall.height(), imgbig.width()/imgbig.height());
    }

    #[tokio::test]
    async fn test_average_gb_blocks() {
        let img = download_image(URLSMALL).await.unwrap();
        let blocks = average_gb_blocks(&img, 10, 10);
        assert_eq!(blocks.len(), 100);
    }

    #[tokio::test]
    async fn test_compare_images_chisquare() {
        let img1 = download_image(URLSMALL).await.unwrap();
        let img2 = download_image(URLBIG).await.unwrap();
        let blocks1 = average_gb_blocks(&img1, 10, 10);
        let blocks2 = average_gb_blocks(&img2, 10, 10);
        let result = compare_images_chisquare(&blocks1, &blocks2);
        assert!(result < 1.0);
    }

    #[tokio::test]
    async fn test_compare_images_chisquare_similar() {
        let img1 = download_image(URLSOMEWHATSIMILAR).await.unwrap();
        let img2 = download_image(URLSOMEWHATSIMILAR2).await.unwrap();
        let blocks1 = average_gb_blocks(&img1, 10, 10);
        let blocks2 = average_gb_blocks(&img2, 10, 10);
        let result = compare_images_chisquare(&blocks1, &blocks2);
        assert!(result > 1.0);
    }
}