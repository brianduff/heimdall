use anyhow::{anyhow, Result};
use rocket::{Route, response::status};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use rocket::response::Debug;

use crate::config;
use crate::constants;
use crate::os;

use os::User;
use config::UserConfig;

#[derive(Serialize, Deserialize, Debug)]
struct Status {
    is_configured: bool,
    hostname: String,
}

// #[post("/say", data = "<message>")]
// fn say(message: String) -> Result<()> {
//     os::say(&message)?;

//     Ok(())
// }

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
    let mut new_config = config.into_inner();
    let mut loaded_config = config::load()?;

    if loaded_config.user_config.contains_key(&new_config.username) {
        Err(Debug(anyhow!("User {:?} already exists", &new_config.username)))
    } else {
        match (new_config.normal_password, new_config.lockdown_password) {
            (Some(normal_password), Some(lockdown_password)) => {
                let username = new_config.username.clone();
                println!("Attempting to change password for user {}...", username);
                os::change_password(&username, Some(&normal_password), &lockdown_password)?;

                println!("Changing password back for user {}...", username);
                os::change_password(&username, Some(&lockdown_password), &normal_password)?;

                println!("Storing passwords in keychain");
                os::store_password(&username, constants::KEYSTORE_NORMAL_PASSWORD_KEY, &normal_password)?;
                os::store_password(&username, constants::KEYSTORE_LOCKDOWN_PASSWORD_KEY, &lockdown_password)?;

                // Wipe passwords so they're not persisted in the config file
                new_config.normal_password = None;
                new_config.lockdown_password = None;

                loaded_config.user_config.insert(username, new_config);
                config::save(&loaded_config)?;
                Ok(status::Accepted(None))

            },
            _ => {
                Err(Debug(anyhow!("No passwords provided")))
            }
        }
    }
}

pub fn get_routes() -> Vec<Route> {
    routes![index, status, users, create_user_config]
}
