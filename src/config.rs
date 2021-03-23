use anyhow::bail;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::Path,
};

#[cfg(not(debug_assertions))]
static CONFIG_FILE: &str = "/usr/local/etc/heimdall/config.json";
#[cfg(debug_assertions)]
static CONFIG_FILE: &str = "/tmp/heimdall/config.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub user_config: HashMap<String, UserConfig>,
}

impl Config {
    pub fn is_new(&self) -> bool {
        self.user_config.is_empty()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserConfig {
    pub username: String,
    pub normal_password: Option<String>,
    pub lockdown_password: Option<String>,
    pub schedule: Schedule,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Schedule {
    pub open_periods: Vec<OpenPeriod>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenPeriod {
    pub start: Instant,
    pub end: Instant,
    pub note: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Instant {
    pub weekday: u8,
    pub hour: u8,
    pub minute: u8,
}

pub fn get_config_path() -> PathBuf {
    PathBuf::from(CONFIG_FILE)
}

pub fn load() -> Result<Config> {
    let path = get_config_path();
    match path.exists() {
        true => {
            println!("Loading config from {}", CONFIG_FILE);
            let file = File::open(CONFIG_FILE)?;
            let reader = BufReader::new(file);
            let config: Config = serde_json::from_reader(reader)?;

            Ok(config)
        }
        false => {
            println!("Creating new config");
            Ok(Config { user_config: HashMap::new()})
        },
    }
}

pub fn save(config: &Config) -> Result<()> {
    let path = get_config_path();
    match path.parent() {
        Some(parent) => {
            if !parent.is_dir() {
                fs::create_dir_all(parent)?
            }
        }
        None => {
            bail!("Invalid config dir: {}", CONFIG_FILE)
        }
    }

    serde_json::to_writer_pretty(&File::create(path)?, &config)?;

    Ok(())
}
