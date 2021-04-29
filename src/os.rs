use anyhow::{anyhow, Result};
use itertools::{cloned, Itertools};
use keyring::Keyring;
use log::info;
use serde::{Deserialize, Serialize};
use std::fs;
use std::str;
use std::{ffi::OsStr, fmt::Display, path::Path, process::Command};

// Mac os X specific things.
#[derive(Serialize, Deserialize, Debug)]
pub struct User {
  pub username: String,
  pub realname: String,
  pub id: u64,
  pub picture_base64: Option<String>,
  pub picture_mimetype: Option<String>,
}

#[derive(Deserialize, Debug)]
struct DsclPlistUser {
  #[serde(rename = "dsAttrTypeStandard:RealName")]
  realname: Vec<String>,

  #[serde(rename = "dsAttrTypeStandard:UniqueID")]
  id: Vec<String>,

  #[serde(rename = "dsAttrTypeStandard:Picture")]
  picture: Option<Vec<String>>,
}

/// Is this a normal username, and not a special built in account (starting with _)
fn is_normal_user(s: &str) -> bool {
  match s.chars().next() {
    Some(c) => c != '_',
    None => false,
  }
}

/// Is this a special account (e.g. root, daemon)?
fn is_special_account(s: &str) -> bool {
  ["daemon", "nobody", "root", "sysadmin"].contains(&s)
}

fn get_usernames() -> Result<Vec<String>> {
  let output = Command::new("dscl")
    .args(&[".", "-list", "/Users"])
    .output()?;

  Ok(
    str::from_utf8(&output.stdout)?
      .split('\n')
      .filter(|u| is_normal_user(u) && !is_special_account(u))
      .map(|x| x.to_owned())
      .collect(),
  )
}

fn get_only(vec: Vec<String>) -> Result<String> {
  match vec.get(0) {
    Some(item) => Ok(item.to_owned()),
    None => Err(anyhow!("No element")),
  }
}

fn get_image_mimetype(path: &Path) -> Option<String> {
  let result = match path.extension() {
    None => None,
    Some(os_str) => match os_str.to_str() {
      None => None,
      Some(ext) => match ext.to_lowercase().as_str() {
        "tif" => Some("image/tiff"),
        "gif" => Some("image/gif"),
        "png" => Some("image/png"),
        "jpg" | "jpeg" | "jfif" | "pjpeg" | "pjp" => Some("image/jpeg"),
        "webp" => Some("image/webp"),
        _ => None,
      },
    },
  };

  result.map(|x| x.to_owned())
}

fn get_image_base64(path: &Path) -> Result<Option<String>> {
  match path.exists() {
    false => Ok(None),
    true => Ok(Some(base64::encode(fs::read(path)?))),
  }
}

fn get_user(username: &str) -> Result<User> {
  let output = Command::new("dscl")
    .args(&["-plist", ".", "read", &format!("/Users/{}", username)])
    .output()?;

  let user: DsclPlistUser = plist::from_bytes(&output.stdout)?;
  // let picture = user.picture.map_or_else(|p| p.get(0), None);
  // let (picture_base64, picture_mimetype) = match picture {
  //     Some(picture) => {
  //         let picture_path = Path::new(picture);
  //         (get_image_base64(picture_path)?, get_image_mimetype(picture_path))
  //     },
  //     None => (None, None)
  // };

  Ok(User {
    realname: get_only(user.realname)?,
    id: get_only(user.id)?.parse::<u64>()?,
    picture_base64: None,
    picture_mimetype: None,
    username: username.to_owned(),
  })
}

pub fn get_users() -> Result<Vec<User>> {
  Ok(
    get_usernames()?
      .iter()
      .map(|username| get_user(username).unwrap())
      .collect(),
  )
}

pub fn store_password(username: &str, name: &str, password: &str) -> Result<()> {
  info!(
    "Storing password in keychain {} for user {}",
    name, username
  );
  let keyring = Keyring::new(name, username);
  keyring.set_password(password)?;
  info!("Stored password for {}", username);

  Ok(())
}

pub fn retrieve_password(username: &str, name: &str) -> Result<String> {
  let keyring = Keyring::new(name, username);

  Ok(keyring.get_password()?)
}

pub fn change_password(
  username: &str,
  old_password: Option<&str>,
  new_password: &str,
) -> Result<()> {
  // TODO: use the stdin version of dscl.
  let user_path = format!("/Users/{}", username);

  let mut options = vec![".", "passwd", &user_path];
  if let Some(old_password) = old_password {
    options.push(old_password);
  }
  options.push(new_password);

  run_command("dscl", &options)?;

  Ok(())
}

/// Logs the given user out of this computer immediately.
pub fn boot_user_out(username: &str) -> Result<()> {
  // First get the uid of this user.
  let output = Command::new("id").args(&["-u", username]).output()?;
  let user_id = str::from_utf8(&output.stdout)?;

  // Now issue the launchctl command that kicks out a user
  let user_string = format!("user/{}", user_id);
  run_command("launchctl", &["bootout", &user_string])?;

  Ok(())
}

fn santize_for_quotes(s: &str) -> String {
  s.chars().filter(|c| *c != '"').collect()
}

fn run_command<I, S>(program: &str, args: I) -> Result<String>
where
  I: IntoIterator<Item = S>,
  S: AsRef<OsStr> + Display,
{
  // Temporary logging.
  let vec: Vec<S> = args.into_iter().collect();
  info!("Command: {} {}", program, vec.iter().join(" "));

  let output = Command::new(program).args(vec).output()?;

  Ok(String::from_utf8(output.stdout)?)
}

/// Shows a user notification to the currently logged in user. Note that
/// this only transiently appears on the screen.
pub fn show_notification(title: &str, message: &str) -> Result<()> {
  let message_s = santize_for_quotes(message);
  let title_s = santize_for_quotes(title);
  let script_command = format!(
    "display notification \"{}\" sound name \"Submarine\" with title \"{}\"",
    message_s, title_s
  );

  run_command("osascript", &["-e", &script_command])?;

  Ok(())
}

/// Say something
pub fn say(message: &str) -> Result<()> {
  run_command("say", &[message])?;

  Ok(())
}

pub fn show_alert(title: &str, message: &str) -> Result<()> {
  let message_s = santize_for_quotes(message);
  let title_s = santize_for_quotes(title);
  let script_command = format!(
    "display alert\"{}\" message \"{}\" as critical",
    title_s, message_s
  );

  run_command("osascript", &["-e", &script_command])?;

  Ok(())
}

/// Shows a message on the login screen.
pub fn show_loginscreen_message(message: &str, force: bool) -> Result<()> {
  run_command(
    "defaults",
    &[
      "write",
      "/Library/Preferences/com.apple.loginwindow",
      "LoginwindowText",
      message,
    ],
  )?;

  if force {
    // It won't update unless we force kill the login window.
    // Need to check if this is going to force a log out.
    run_command("killall", &["-9", "loginwindow"])?;
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_get_usernames() {
    println!("{:?}", get_usernames());
    println!("{:?}", get_user("bduff"));
    println!("{:?}", get_users());
  }

  #[test]
  fn test_notifications() -> Result<()> {
    show_notification("Hello", "This is a long message!")?;
    show_notification("Hello", "This is a shifty\"notification\"")?;

    Ok(())
  }

  #[test]
  fn test_say() -> Result<()> {
    say("Rust is cool! Do you want to try something new?")?;

    Ok(())
  }

  // #[test]
  // fn test_alert() -> Result<()> {
  //     show_alert("Your time is up!", "Please prepare to be logged out")?;

  //     Ok(())
  // }
}
