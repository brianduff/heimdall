use std::ffi::OsString;

use anyhow::Result;
use rocket::Route;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};

use crate::config;
use crate::os;

use os::User;

#[derive(Serialize, Deserialize, Debug)]
struct Status {
    is_configured: bool,
    hostname: String,
}

#[get("/users")]
fn users() -> Result<Json<Vec<User>>> {
    Ok(Json(os::get_users()?))
}

#[get("/status")]
fn status() -> Result<Json<Status>> {
    let config = config::load()?;

    Ok(Json(Status {
        is_configured: !config.is_new(),
        hostname: hostname::get()?.to_str().unwrap().to_owned(),
    }))
}

#[get("/test")]
fn index() -> Result<String> {
    let config = config::load()?;

    Ok(match config.is_new() {
        true => "Welcome!",
        false => "Hello again!",
    }
    .to_string())
}

pub fn get_routes() -> Vec<Route> {
    routes![index, status, users]
}
