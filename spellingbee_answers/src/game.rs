use regex::Regex;
use serde_json::Value;
use std::time::{Duration, SystemTime};

#[derive(Debug)]
pub enum Error {
    HTTP(reqwest::Error),
    Serde(serde_json::Error),
    DecodePage(String),
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Error {
        Error::HTTP(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        Error::Serde(e)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::HTTP(ref e) => write!(f, "error making HTTP request: {}", e),
            Error::Serde(ref e) => write!(f, "error parsing JSON: {}", e),
            Error::DecodePage(ref e) => write!(f, "error decoding page: {}", e),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct GameInfo {
    pub expiration: SystemTime,
    pub center_letter: String,
    pub outer_letters: Vec<String>,
    pub pangrams: Vec<String>,
    pub answers: Vec<String>,
}

impl GameInfo {
    pub async fn fetch() -> Result<GameInfo> {
        let resp = reqwest::get("https://www.nytimes.com/puzzles/spelling-bee").await?;
        let contents = resp.text().await?;
        let expr = Regex::new("window.gameData = (.*?)</script>").unwrap();
        if let Some(caps) = expr.captures(&contents) {
            let v: Value = serde_json::from_str(caps.get(1).unwrap().as_str())?;
            Ok(GameInfo {
                expiration: parse_expiration(&v)?,
                center_letter: parse_center_letter(&v)?,
                outer_letters: parse_string_vec(&v, &["today", "outerLetters"])?,
                pangrams: parse_string_vec(&v, &["today", "pangrams"])?,
                answers: parse_string_vec(&v, &["today", "answers"])?,
            })
        } else {
            Err(Error::DecodePage(String::from(
                "could not find game data block",
            )))
        }
    }
}

fn parse_expiration(obj: &Value) -> Result<SystemTime> {
    if let Value::Number(ref x) = value_at_path(obj, &["today", "expiration"])? {
        if let Some(epoch) = x.as_u64() {
            if let Some(time) = SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(epoch)) {
                return Ok(time);
            }
        }
    }
    Err(Error::DecodePage(String::from("could not parse timestamp")))
}

fn parse_center_letter(obj: &Value) -> Result<String> {
    if let Value::String(ref x) = value_at_path(obj, &["today", "centerLetter"])? {
        Ok(x.clone())
    } else {
        Err(Error::DecodePage(String::from(
            "invalid type for center letter",
        )))
    }
}

fn parse_string_vec(obj: &Value, path: &[&str]) -> Result<Vec<String>> {
    let val = value_at_path(obj, path)?;
    let err = Err(Error::DecodePage(format!(
        "value at {} was not list of strings",
        path.join("."),
    )));
    if let Value::Array(ref arr) = val {
        let mut res = Vec::new();
        for item in arr {
            if let Value::String(ref str_item) = item {
                res.push(str_item.clone());
            } else {
                return err;
            }
        }
        return Ok(res);
    }
    return err;
}

fn value_at_path<'a>(obj: &'a Value, path: &[&str]) -> Result<&'a Value> {
    let mut sub_obj = obj;
    for x in path {
        if let Some(x) = sub_obj.get(x) {
            sub_obj = x;
        } else {
            return Err(Error::DecodePage(format!(
                "no attribute at path: {}",
                path.join(".")
            )));
        }
    }
    Ok(sub_obj)
}
