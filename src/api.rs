use anyhow::Result;
use rocket::Route;

use crate::config;

struct Status {
  is_configured: bool,
  hostname: String,
}

#[get("/test")]
fn index() -> Result<String> {
    let config = config::load()?;

    Ok(match config.is_new() {
        true => "Welcome!",
        false => "Hello again!"
    }.to_string())
}

pub fn get_routes() -> Vec<Route> {
  routes![index]
}