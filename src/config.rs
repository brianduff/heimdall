use std::{collections::HashMap, fs::File, io::{BufReader, BufWriter}, path::{Path, PathBuf}};
use serde::{Serialize, Deserialize};
use anyhow::Result;
use std::fs;
use anyhow::bail;

static CONFIG_FILE: &str = "/etc/heimdall/config.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
  user_config: HashMap<String, UserConfig>
}

impl Config {
  pub fn is_new(&self) -> bool {
    self.user_config.is_empty()
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserConfig {
  open_password: String,
  closed_password: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Schedule {
  open_periods: Vec<OpenPeriod>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenPeriod {
  start: Instant,
  end: Instant
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Instant {
  weekday: u8,
  hour: u8,
  minute: u8
}

pub fn load() -> Result<Config> {
  let path = Path::new(CONFIG_FILE);
  match path.exists() {
    true => {
      let file = File::open(CONFIG_FILE)?;
      let reader = BufReader::new(file);
      let config: Config = serde_json::from_reader(reader)?;

      Ok(config)
    },
    false =>  Ok(Config{ user_config: HashMap::new() })
  }
}

pub fn save(config: &Config) -> Result<()> {
  let path = Path::new(CONFIG_FILE);
  match path.parent() {
    Some(parent) => {
      if !parent.is_dir() {
        fs::create_dir_all(parent)?
      }
    },
    None => {
      bail!("Invalid config dir: {}", CONFIG_FILE)
    }
  }

  let file = File::open(path)?;
  let writer = BufWriter::new(file);
  serde_json::to_writer(writer, config)?;

  Ok(())
}