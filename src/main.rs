use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use ctrlc::set_handler;
use screenshots::Screen;
use rusty_tesseract::{Image, Args};
use tempfile::Builder;
use tokio::time::{sleep, Duration};
use futures::future::try_join_all;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let search_phrase = "Enter Dungeon";
    let screens = Screen::all()?;
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");
    println!("Starting screen monitor...");
    // println!("Looking for phrase: {}", search_phrase);

    let temp_dir = Builder::new()
        .prefix("screenshot_monitor")
        .tempdir()?;

    while running.load(Ordering::SeqCst) {
        let search_futures: Vec<_> = screens.iter().map(|screen| {
            let temp_path = temp_dir.path().join(format!("temp_screenshot_{}.png", screen.display_info.id));
            let search_phrase = search_phrase.to_string();

            async move {
                // Capture and save screenshot
                let image = screen.capture()?;
                image.save(&temp_path).expect("Failed to save image");

                // Set up OCR
                let img = Image::from_path(&temp_path).expect("Failed to load image");
                let args = Args::default();

                // Perform OCR
                let text = tokio::task::spawn_blocking(move || {
                    rusty_tesseract::image_to_string(&img, &args)
                }).await??;

                let found = text.contains(&search_phrase);
                if found {
                    println!("Found matching phrase!");
                    println!("Screen: {}", screen.display_info.id);
                    println!("Time: {}", chrono::Local::now());
                }

                if temp_path.exists() {
                    tokio::fs::remove_file(&temp_path).await?;
                }

                Ok::<bool, anyhow::Error>(found)
            }
        }).collect();

        let results = try_join_all(search_futures).await?;
        let found = results.into_iter().reduce(|acc, x| acc || x).unwrap_or(false);

        let secs = if found { 2 * 60 } else { 8 };
        println!("sleeping for {}s", &secs);
        sleep(Duration::from_secs(secs)).await;
    }

    println!("Exiting...");
    temp_dir.close()?;
    Ok(())
}
