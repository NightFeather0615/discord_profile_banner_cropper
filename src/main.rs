use std::time::{SystemTime, UNIX_EPOCH};
use std::convert::TryInto;
use image::{imageops, ImageFormat};
use clokwerk::{Scheduler, TimeUnits};
use std::thread;
use std::time::Duration;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use dotenv::dotenv;
use std::env;

const DISCORD_PROFILE_BANNER_ASPECT_RATIO: (f32, f32) = (5.0, 2.0);
const OFFSET_HOUR_CYCLE: u32 = 10;

fn get_hour_offset() -> u32 {
    let unix_timestamp_now: u32 = (
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Get Unix timestamp failed.")
            .as_secs() / 3600
    )
        .try_into()
        .expect("Try From Int failed.");
    return (unix_timestamp_now as f32 % OFFSET_HOUR_CYCLE as f32) as u32;
}

fn mapping_aspect_ratio(width: u32, _height: u32) -> (u32, u32) {
    let height_radio = DISCORD_PROFILE_BANNER_ASPECT_RATIO.1 / DISCORD_PROFILE_BANNER_ASPECT_RATIO.0;
    return (width, (width as f32 * height_radio) as u32);
}

fn crop_image(hour_offset: u32) {
    let mut source_image = image::open("./src/source.jpeg").expect("File not found.");
    let aspect_ratio: (u32, u32) = (source_image.width(), source_image.height());

    let mapped_aspect_ratio: (u32, u32) = mapping_aspect_ratio(aspect_ratio.0, aspect_ratio.1);
    let height_offset: u32 = (aspect_ratio.1 - mapped_aspect_ratio.1) / (OFFSET_HOUR_CYCLE - 1);

    imageops::crop(
        &mut source_image,
        0, height_offset * hour_offset,
        mapped_aspect_ratio.0, mapped_aspect_ratio.1
    )
        .to_image()
        .save_with_format("./src/cropped.jpeg", ImageFormat::Jpeg)
        .expect("Image process failed.");
}

fn change_profile_banner(discord_user_token: String) {
    let hour_offset = get_hour_offset();
    crop_image(hour_offset);

    let client = reqwest::blocking::Client::new();
    
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(discord_user_token.as_str()).unwrap());
    headers.insert("Content-Type", HeaderValue::from_str("application/json").unwrap());
    headers.insert("User-Agent", HeaderValue::from_str("discord/1.0.9005 Chrome/91.0.4472.164 Electron/13.6.6").unwrap());

    let profile_banner_data = reqwest::blocking::multipart::Form::new()
        .file("banner", "./src/cropped.jpeg")
        .expect("File load failed.");

    client.patch("https://discord.com/api/v9/users/@me/profile")
        .headers(headers)
        .multipart(profile_banner_data)
        .send()
        .expect("Profile edit failed.");

    println!("Change banner succeed, offset: {:}", hour_offset);
}

fn main() {
    println!("Starting...");
    dotenv().ok();
    println!(".ENV loaded.");
    let discord_user_token: String = env::var("DISCORD_USER_TOKEN").expect("Load .ENV failed.");
    println!("Discord user token loaded.");
    let mut scheduler = Scheduler::new();
    scheduler.every(1.hours()).run(move || change_profile_banner(discord_user_token.to_owned()));
    println!("Start scheduler loop.");
    loop {
        scheduler.run_pending();
        thread::sleep(Duration::from_secs(60));
    }
}