use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{convert::TryInto, path::Path, process::Command};
use std::str;
use std::fs;

// Mac os X specific things.
#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub username: String,
    pub realname: String,
    pub id: u64,
    pub picture_base64: Option<String>,
    pub picture_mimetype: Option<String>
}

#[derive(Deserialize, Debug)]
struct DsclPlistUser {
    #[serde(rename = "dsAttrTypeStandard:RealName")]
    realname: Vec<String>,

    #[serde(rename = "dsAttrTypeStandard:UniqueID")]
    id: Vec<String>,

    #[serde(rename = "dsAttrTypeStandard:Picture")]
    picture: Vec<String>,
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

    Ok(str::from_utf8(&output.stdout)?
        .split('\n')
        .filter(|u| is_normal_user(u) && !is_special_account(u))
        .map(|x| x.to_owned())
        .collect())
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
        Some(os_str) => {
            match os_str.to_str() {
                None => None,
                Some(ext) => {
                    match ext.to_lowercase().as_str(){
                        "tif" => Some("image/tiff"),
                        "gif" => Some("image/gif"),
                        "png" => Some("image/png"),
                        "jpg" | "jpeg" | "jfif" | "pjpeg" | "pjp" => Some("image/jpeg"),
                        "webp" => Some("image/webp"),
                        _ => None
                    }
                }
            }
        }
    };

    match result {
        Some(string) => Some(string.to_owned()),
        None => None
    }
}

fn get_image_base64(path: &Path) -> Result<Option<String>> {
    match path.exists() {
        false => Ok(None),
        true => Ok(Some(base64::encode_config(fs::read(path)?, base64::URL_SAFE)))
    }
}

fn get_user(username: &str) -> Result<User> {
    let output = Command::new("dscl")
        .args(&["-plist", ".", "read", &format!("/Users/{}", username)])
        .output()?;

    let user: DsclPlistUser = plist::from_bytes(&output.stdout)?;
    let picture = get_only(user.picture)?;
    let picture_path = Path::new(&picture);

    Ok(User {
        realname: get_only(user.realname)?,
        id: get_only(user.id)?.parse::<u64>()?,
        picture_base64: get_image_base64(&picture_path)?,
        picture_mimetype: get_image_mimetype(&picture_path),
        username: username.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_usernames() {
        println!("{:?}", get_usernames());
        println!("{:?}", get_user("bduff"));
    }
}
