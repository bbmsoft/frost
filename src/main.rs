#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use chrono::prelude::*;
use dotenv::dotenv;
use frost::backend::*;
use frost::common::*;
use rocket::response::content;
use rocket::response::NamedFile;
use rocket::State;
use std::env;
use std::path::{Path, PathBuf};

#[get("/")]
fn index() -> Option<NamedFile> {
    files(PathBuf::from("index.html"))
}

#[get("/<file..>")]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("./pkg/").join(file)).ok()
}

#[get("/weather?<lat>&<lon>&<warning_threshold>&<danger_threshold>")]
fn weather(
    lat: f32,
    lon: f32,
    warning_threshold: f32,
    danger_threshold: f32,
    config: State<Config>,
) -> Result<content::Json<String>, Box<dyn std::error::Error>> {
    let now: DateTime<Utc> = Utc::now();
    let three_days_from_now = now + chrono::Duration::days(3);

    let api_endpoint = &config.brightsky_api_endpoint;

    let url = format!(
        "{}?lat={}&lon={}&date={}&last_date={}",
        api_endpoint,
        lat,
        lon,
        now.format("%Y-%m-%d"),
        three_days_from_now.format("%Y-%m-%d")
    );

    let body = reqwest::blocking::get(&url)?.text()?;

    let response = parse_response(&body, warning_threshold, danger_threshold, &now)?;
    let json = serde_json::to_string(&response)?;

    Ok(content::Json(json))
}

struct Config {
    brightsky_api_endpoint: String,
}

fn parse_response(
    brightsky_response: &str,
    warning_threshold: f32,
    danger_threshold: f32,
    now: &DateTime<Utc>,
) -> Result<BackendResult, Box<dyn std::error::Error>> {
    let data = serde_json::from_str(brightsky_response);
    match data {
        Ok(data) => {
            let cold_phases =
                accumulate_cold_phases(warning_threshold, danger_threshold, &data, &now);
            Ok(Ok(cold_phases))
        }
        Err(_) => {
            let api_error: BrightskyApiError = serde_json::from_str(brightsky_response)?;
            Ok(Err(api_error.into()))
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let brightsky_api_endpoint = env::var("FROST_BRIGHTSKY_ENDPOINT")?;

    let config = Config {
        brightsky_api_endpoint,
    };

    rocket::ignite()
        .manage(config)
        .mount("/", routes![index, weather, files])
        .launch();

    Ok(())
}
