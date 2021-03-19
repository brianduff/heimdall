use anyhow::{anyhow, Result};
use rocket::{Route, response::status};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use rocket::response::Debug;

use crate::config;
use crate::os;

use os::User;
use config::UserConfig;

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

#[post("/userconfig", data = "<config>")]
fn create_user_config(config: Json<UserConfig>) -> std::result::Result<status::Accepted<String>, Debug<anyhow::Error>> {
    let new_config = config.into_inner();
    let mut loaded_config = config::load()?;

    if loaded_config.user_config.contains_key(&new_config.username) {
        Err(Debug(anyhow!("User {:?} already exists", &new_config.username)))
    } else {
        loaded_config.user_config.insert(new_config.username.clone(), new_config);
        config::save(&loaded_config)?;
        Ok(status::Accepted(None))
    }
}

pub fn get_routes() -> Vec<Route> {
    routes![index, status, users, create_user_config]
}
