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
    let v = vec![User{username: "foo".to_owned(), realname: "bar".to_owned(), id: 1, picture_base64: None, picture_mimetype: None}];
    Ok(Json(v))
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
    routes![index, status]
}
