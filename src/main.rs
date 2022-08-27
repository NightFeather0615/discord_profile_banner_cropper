use std::env;
use std::thread;
use std::io::Cursor;
use std::convert::TryInto;
use std::collections::HashMap;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use image::{imageops, ImageFormat};
use clokwerk::{Scheduler, TimeUnits};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use dotenv::dotenv;

const DISCORD_PROFILE_BANNER_ASPECT_RATIO: AspectRatio = AspectRatio::new(
    5.0,
    2.0
);
const OFFSET_CYCLE: u32 = 10;
const MINUTES_PER_CYCLE: u32 = 15;

struct AspectRatio {
    width: f32,
    height: f32
}

impl AspectRatio {
    const fn new(width: f32, height: f32) -> AspectRatio {
        return AspectRatio { width, height };
    }

    pub fn height_rate(self: &Self) -> f32 {
        return self.height / self.width;
    }

    pub fn width_rate(self: &Self) -> f32 {
        return self.width / self.height;
    }
}

struct ImageSizeData {
    width: u32,
    height: u32
}

impl ImageSizeData {
    const fn new(width: u32, height: u32) -> ImageSizeData {
        return ImageSizeData { width, height };
    }

    fn from_width(width: u32, aspect_ratio: AspectRatio) -> ImageSizeData {
        return ImageSizeData::new(
            width,
            (width as f32 * aspect_ratio.height_rate()) as u32
        );
    }

    fn from_height(height: u32, aspect_ratio: AspectRatio) -> ImageSizeData {
        return ImageSizeData::new(
            (height as f32 * aspect_ratio.width_rate()) as u32,
            height
        );
    }

    pub fn map_from_aspect_ratio(self: &Self, aspect_ratio: AspectRatio) -> ImageSizeData {
        if self.width > self.height {
            return ImageSizeData::from_height(self.height, aspect_ratio);
        } else {
            return ImageSizeData::from_width(self.width, aspect_ratio);
        }
    }
}

fn get_time_offset() -> u32 {
    let unix_timestamp_now: u32 = (
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Get Unix timestamp failed.")
            .as_secs() / (MINUTES_PER_CYCLE * 60) as u64
    )
        .try_into()
        .expect("Try From Int failed.");
    return (unix_timestamp_now as f32 % OFFSET_CYCLE as f32) as u32;
}

fn crop_image(time_offset: u32) {
    let mut source_image = image::open("./src/source.jpeg").expect("File not found.");
    let source_image_size: ImageSizeData = ImageSizeData::new(source_image.width(), source_image.height());

    let mapped_image_size: ImageSizeData = source_image_size.map_from_aspect_ratio(DISCORD_PROFILE_BANNER_ASPECT_RATIO);
    let height_offset: u32 = (source_image_size.height - mapped_image_size.height) / (OFFSET_CYCLE - 1);

    imageops::crop(
        &mut source_image,
        0, height_offset * time_offset,
        mapped_image_size.width, mapped_image_size.height
    )
        .to_image()
        .save_with_format("./src/cropped.jpeg", ImageFormat::Jpeg)
        .expect("Image process failed.");
}

fn read_image_as_base64(path: &str) -> String {
    let image = image::open(path).expect("File not found.");
    let mut image_buffer: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    image.write_to(&mut image_buffer, image::ImageOutputFormat::Png).expect("Write image to bytes failed.");
    let image_base64 = base64::encode(&image_buffer.get_ref());
    return format!("data:image/png;base64,{}", image_base64);
}

fn change_profile_banner(discord_user_token: String) {
    let time_offset = get_time_offset();
    crop_image(time_offset);

    let client = reqwest::blocking::Client::new();
    
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(discord_user_token.as_str()).unwrap());
    headers.insert("Content-Type", HeaderValue::from_str("application/json").unwrap());
    headers.insert("User-Agent", HeaderValue::from_str("discord/1.0.9005 Chrome/91.0.4472.164 Electron/13.6.6").unwrap());

    let mut data = HashMap::new();
    data.insert("banner", read_image_as_base64("./src/cropped.jpeg"));

    client.patch("https://discord.com/api/v9/users/@me/profile")
        .headers(headers)
        .json(&data)
        .send()
        .expect("Profile edit failed.");

    println!("Change banner succeed, offset: {time_offset}");
}

fn main() {
    println!("Starting...");
    if MINUTES_PER_CYCLE < 10 {
        println!("Minutes per cycle lower then 10 mins will hit Discord's ratelimit.");
        std::process::exit(1);
    }
    dotenv().ok();
    println!(".ENV loaded.");
    let discord_user_token: String = env::var("DISCORD_USER_TOKEN").expect("Load .ENV failed.");
    println!("Discord user token loaded.");
    let mut scheduler = Scheduler::new();
    scheduler.every(MINUTES_PER_CYCLE.minutes()).run(move || change_profile_banner(discord_user_token.to_owned()));
    println!("Start scheduler loop.");
    loop {
        scheduler.run_pending();
        thread::sleep(Duration::from_secs(60));
    }
}