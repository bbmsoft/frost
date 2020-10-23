#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate log;

use chrono::prelude::*;
use dotenv::dotenv;
use frost::backend::*;
use frost::common::*;
use rocket::fairing::AdHoc;
use rocket::response::content;
use rocket::response::NamedFile;
use rocket::State;
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct CookieError(&'static str);
impl fmt::Display for CookieError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for CookieError {}

#[get("/")]
fn index(root: State<RootDir>) -> Option<NamedFile> {
    files(PathBuf::from("index.html"), root)
}

#[get("/<file..>")]
fn files(file: PathBuf, root: State<RootDir>) -> Option<NamedFile> {
    NamedFile::open(Path::new(&root.0).join(file)).ok()
}

#[get("/weather?<lat>&<lon>&<warning_threshold>&<danger_threshold>")]
fn weather(
    lat: f32,
    lon: f32,
    warning_threshold: f32,
    danger_threshold: f32,
    brightsky_api_endpoint: State<BrightSkyEndpoint>,
) -> Result<content::Json<String>, Box<dyn std::error::Error>> {
    let now: DateTime<Local> = Local::now();
    let noon_in_three_days: DateTime<Local> = (now + chrono::Duration::days(3))
        .with_hour(12)
        .and_then(|t| t.with_minute(0))
        .and_then(|t| t.with_second(0))
        .expect("always noon, can't be invalid");

    let api_endpoint = &brightsky_api_endpoint.0;

    let url = format!(
        "{}?lat={}&lon={}&date={}&last_date={}",
        api_endpoint,
        lat,
        lon,
        now.to_rfc3339(),
        noon_in_three_days.to_rfc3339()
    )
    // TODO escape forbidden characters in a more robust way!
    .replace("+", "%2b");

    debug!("Pulling weather data from {}", url);

    let body = reqwest::blocking::get(&url)?.text()?;

    debug!("Received data:\n{}", body);

    let response = parse_response(&body, warning_threshold, danger_threshold)?;
    let json = serde_json::to_string(&response)?;

    Ok(content::Json(json))
}

fn parse_response(
    brightsky_response: &str,
    warning_threshold: f32,
    danger_threshold: f32,
) -> Result<BackendResult, Box<dyn std::error::Error>> {
    let data = serde_json::from_str(brightsky_response);
    match data {
        Ok(data) => {
            let cold_phases = accumulate_cold_phases(warning_threshold, danger_threshold, &data);
            Ok(Ok(cold_phases))
        }
        Err(e) => {
            error!("Error parsing brightsky data: {}", e);
            let api_error: BrightskyApiError = serde_json::from_str(brightsky_response)?;
            Ok(Err(api_error.into()))
        }
    }
}

struct RootDir(String);
struct BrightSkyEndpoint(String);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env_logger::init();

    rocket::ignite()
        .mount("/", routes![index, weather, files])
        .attach(AdHoc::on_attach("Root Dir", |rocket| {
            let root_dir = rocket
                .config()
                .get_str("frost_app_root")
                .unwrap_or("./dist")
                .to_string();

            Ok(rocket.manage(RootDir(root_dir)))
        }))
        .attach(AdHoc::on_attach("BrightSky Api Endpoint", |rocket| {
            let brightsky_endpoint = rocket
                .config()
                .get_str("frost_brightsky_endpoint")
                .unwrap_or("https://api.brightsky.dev/weather")
                .to_string();

            Ok(rocket.manage(BrightSkyEndpoint(brightsky_endpoint)))
        }))
        .launch();

    Ok(())
}
