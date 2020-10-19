#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate log;

use chrono::prelude::*;
use dotenv::dotenv;
use frost::backend::*;
use frost::common::*;
use rocket::http::Cookies;
use rocket::response::content;
use rocket::response::NamedFile;
use rocket::State;
use std::env;
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
fn index(config: State<Config>) -> Option<NamedFile> {
    files(PathBuf::from("index.html"), config)
}

#[get("/<file..>")]
fn files(file: PathBuf, config: State<Config>) -> Option<NamedFile> {
    NamedFile::open(Path::new(&config.root_dir).join(file)).ok()
}

// TODO provide endpoints with query params and for use with cookies
// TODO use weather station ID cookie for cookie based api
// #[get("/weather?<lat>&<lon>&<warning_threshold>&<danger_threshold>")]
// fn weather(lat: f32,lon: f32,warning_threshold: f32,danger_threshold: f32,config: State<Config>,
#[get("/weather")]
fn weather(
    cookies: Cookies,
    config: State<Config>,
) -> Result<content::Json<String>, Box<dyn std::error::Error>> {
    let (lat, lon) = get_cookie_value(LOCATION_COOKIE, &cookies)?;
    let (warning_threshold, danger_threshold) = get_cookie_value(THRESHOLD_COOKIE, &cookies)?;

    let now: DateTime<Local> = Local::now();
    let noon_in_three_days: DateTime<Local> = (now + chrono::Duration::days(3))
        .with_hour(12)
        .and_then(|t| t.with_minute(0))
        .and_then(|t| t.with_second(0))
        .expect("always noon, can't be invalid");

    let api_endpoint = &config.brightsky_api_endpoint;

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

struct Config {
    brightsky_api_endpoint: String,
    root_dir: String,
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
        Err(_) => {
            let api_error: BrightskyApiError = serde_json::from_str(brightsky_response)?;
            Ok(Err(api_error.into()))
        }
    }
}

fn get_cookie_value(
    name: &str,
    cookies: &Cookies,
) -> Result<(f32, f32), Box<dyn std::error::Error>> {
    if let Some(value) = cookies.get(name) {
        Ok(serde_json::from_str(value.value())?)
    } else {
        Err(Box::new(CookieError("Location cookie not set!")))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env_logger::init();

    let brightsky_api_endpoint = env::var("FROST_BRIGHTSKY_ENDPOINT")?;
    let root_dir = env::var("FROST_APP_ROOT")?;

    let config = Config {
        brightsky_api_endpoint,
        root_dir,
    };

    rocket::ignite()
        .manage(config)
        .mount("/", routes![index, weather, files])
        .launch();

    Ok(())
}
