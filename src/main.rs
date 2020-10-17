#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use chrono::prelude::*;
use rocket::response::content;
use rocket::response::NamedFile;
use std::path::{Path, PathBuf};

#[get("/")]
fn index() -> Option<NamedFile> {
    files(PathBuf::from("index.html"))
}

#[get("/<file..>")]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("./pkg/").join(file)).ok()
}

#[get("/weather?<lat>&<lon>")]
fn weather(lat: f32, lon: f32) -> Result<content::Json<String>, Box<dyn std::error::Error>> {
    let now: DateTime<Utc> = Utc::now();
    let three_days_from_now = now + chrono::Duration::days(3);

    let url = format!(
        "https://api.brightsky.dev/weather?lat={}&lon={}&date={}&last_date={}",
        lat,
        lon,
        now.format("%Y-%m-%d"),
        three_days_from_now.format("%Y-%m-%d")
    );

    let body = reqwest::blocking::get(&url)?.text()?;
    Ok(content::Json(body))
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, weather, files])
        .launch();
}
