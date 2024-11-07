use screenshots::Screen;
use std::{thread, time::Duration};
use rusty_tesseract::{Args, Image};
use image::{RgbaImage, DynamicImage};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let search_phrase = "Enter Dungeon";
    let screens = Screen::all()?;

    println!("Starting screen monitor...");
    println!("Looking for phrase: {}", search_phrase);

    loop {
        for screen in &screens {
            // Capture screenshot
            let screenshot = screen.capture()?;

            // Create RgbaImage from raw pixels
            let rgba_image = RgbaImage::from_raw(
                screenshot.width(),
                screenshot.height(),
                screenshot.as_raw().to_vec(),
            ).expect("Failed to create image from raw data");

            // Convert to DynamicImage
            let dynamic_img = DynamicImage::ImageRgba8(rgba_image);

            // Create rusty-tesseract Image
            let img = Image::from_dynamic_image(&dynamic_img)?;

            let args = Args::default();

            // Perform OCR
            if let Ok(text) = rusty_tesseract::image_to_string(&img, &args) {
                if text.contains(search_phrase) {
                    println!("Found matching phrase!");
                    println!("Screen: {}", screen.display_info.id);
                    println!("Time: {}", chrono::Local::now());
                }
            }
        }

        thread::sleep(Duration::from_secs(1));
    }
}
