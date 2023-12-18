use anyhow::{Context, Result};
use reqwest;
use serde_json::{Value};
use std::env;
use std::process::Command;
use chrono::Utc;
use image::{DynamicImage,  Rgba};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};
use dotenv::dotenv;
use std::path::Path;



fn overlay_image_on_top(base: &mut DynamicImage, overlay: DynamicImage, text: &str) -> Result<DynamicImage> {
    // Define the position where the overlay image will be placed on the base image
     let _overlay_pos = image::imageops::overlay(base, &overlay, 50, 50);

    // Load a font
    let font_data = include_bytes!("../data/font/helvetica.ttf");
    let font = Font::try_from_bytes(font_data as &[u8]).expect("Error constructing Font");

    // Set the size of the text
    let scale = Scale {
        x: 20.0,
        y: 20.0,
    };

    // Set the color of the text (black)
    let color = Rgba([0u8, 0u8, 0u8, 255u8]);

    // Combine the current date with the Instagram text
    let current_date = Utc::now().format("%Y-%m-%d").to_string();
    let combined_text = format!("{} - {}", current_date, text);

    // Choose the position where the text will be drawn
    let text_pos = (50, 100); // You can adjust this as needed

    // Draw the text on the base image
    draw_text_mut(base, color, text_pos.0, text_pos.1, scale, &font, &combined_text);

    Ok(base.clone())
}


fn print_image(file_path: &str) -> Result<()> {
    Command::new("lp")
        .arg(file_path)
        .spawn()?
        .wait()?;
    Ok(())
}


async fn fetch_hashtag_data(hashtag_id: &str, user_id: &str, fields: &str, access_token: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://graph.facebook.com/{}/recent_media?user_id={}&fields={}&access_token={}",
        hashtag_id, user_id, fields, access_token, 
    );
println!("url {}",url);
    let resp = client.get(url).send().await?;
    let body = resp.text().await?;
    let json: Value = serde_json::from_str(&body)?;


    // Extract the image URL from the JSON response
    // This is a placeholder - adjust the extraction based on the actual response structure
    let image_url = json["data"][0]["media_url"].as_str()
        .ok_or_else(|| anyhow::anyhow!("Failed to extract image URL"))?
        .to_string();

    Ok(image_url)
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();



    let image_path = Path::new("./data/overlay/overlay_image.png");

    // Check if the file exists
    if !image_path.exists() {
        eprintln!("Error: 'overlay_image' not found in the current directory.");
        return Ok(());
    }





    let hashtag_id = env::var("HASHTAG_ID").context("Missing HASHTAG_ID")?;
    let user_id = env::var("USER_ID").context("Missing USER_ID")?;
    let fields = env::var("FIELDS").context("Missing FIELDS")?;
    let access_token = env::var("ACCESS_TOKEN").context("Missing ACCESS_TOKEN")?;

    let instagram_image_url = fetch_hashtag_data(&hashtag_id, &user_id, &fields, &access_token).await?;
    let instagram_image = reqwest::get(&instagram_image_url).await?.bytes().await?;
    let mut image = image::load_from_memory(&instagram_image).context("Error loading image from memory")?;

    let overlay_image = image::open(image_path).context("Error opening overlay image")?;
    let final_image = overlay_image_on_top(&mut image, overlay_image, "Some Text Here - Lorem ipsum?")?;

    let file_path = "data/final_image.jpg";
    final_image.save(file_path).context("Error saving final image")?;

    //don't print it now, we're testing
    //print_image(file_path)?;

    Ok(())
}