use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Display;
type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    HTTP(reqwest::Error),
    ParseJSON(serde_json::Error),
    ProcessJSON(String),
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::HTTP(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::ParseJSON(e)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeoCoord(pub f64, pub f64);

impl Display for GeoCoord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.15},{:.15}", self.0, self.1)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MapItem {
    pub id: String,
    pub name: String,
    pub location: GeoCoord,
    pub address: String,
    pub phone: Option<String>,
    pub chain_id: Option<String>,
}

pub struct Client {
    client: reqwest::Client,
}

impl Client {
    pub fn new() -> Client {
        Client {
            client: reqwest::Client::new(),
        }
    }

    pub async fn map_search(
        &self,
        query: &str,
        min: GeoCoord,
        max: GeoCoord,
        center: GeoCoord,
    ) -> Result<Vec<MapItem>> {
        let response = self
            .client
            .get("https://www.bing.com/maps/overlaybfpr")
            .query(&[
                ("q", query),
                ("filters", "direction_partner:\"maps\""),
                ("mapcardtitle", ""),
                ("p1", "[AplusAnswer]"),
                ("count", "18"),
                ("ecount", "18"),
                ("first", "0"),
                ("efirst", "1"),
                ("localMapView", &format!("{},{}", min, max)),
                ("ads", "0"),
                ("cp", &format!("{}", center)),
            ])
            .send()
            .await?
            .text()
            .await?;
        let doc = Html::parse_fragment(&response);
        let mut result = Vec::new();
        for obj in doc.select(&Selector::parse("a.listings-item").unwrap()) {
            if let Some(info_json) = obj.value().attr("data-entity") {
                let parsed: Value = serde_json::from_str(info_json)?;
                result.push(MapItem {
                    id: read_object(&parsed, "entity.id")?,
                    name: read_object(&parsed, "entity.title")?,
                    location: GeoCoord(
                        read_object(&parsed, "geometry.x")?,
                        read_object(&parsed, "geometry.y")?,
                    ),
                    address: read_object(&parsed, "entity.address")?,
                    phone: read_object(&parsed, "entity.phone").ok(),
                    chain_id: read_object(&parsed, "entity.chainId").ok(),
                });
            }
        }
        Ok(result)
    }
}

fn read_object<T: FromJSON>(root: &Value, path: &str) -> Result<T> {
    let mut cur_obj = root;
    for part in path.split(".") {
        if let Value::Object(obj) = cur_obj {
            if let Some(x) = obj.get(part) {
                cur_obj = x;
            } else {
                return Err(Error::ProcessJSON(format!(
                    "object path not found: {}",
                    path
                )));
            }
        } else {
            return Err(Error::ProcessJSON(format!(
                "incorrect type in object path: {}",
                path
            )));
        }
    }
    T::from_json(cur_obj)
}

trait FromJSON
where
    Self: Sized,
{
    fn from_json(value: &Value) -> Result<Self>;
}

impl FromJSON for f64 {
    fn from_json(value: &Value) -> Result<Self> {
        match value {
            Value::Number(x) => {
                if let Some(f) = x.as_f64() {
                    Ok(f)
                } else {
                    Err(Error::ProcessJSON(format!("{} is not an f64", x)))
                }
            }
            _ => Err(Error::ProcessJSON(format!("{} is not a number", value))),
        }
    }
}

impl FromJSON for String {
    fn from_json(value: &Value) -> Result<Self> {
        match value {
            Value::String(x) => Ok(x.clone()),
            _ => Err(Error::ProcessJSON(format!("{} is not a string", value))),
        }
    }
}
