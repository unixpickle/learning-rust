use reqwest::Version;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fmt::Display, time::Duration};
use tokio::time::sleep;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    HTTP(reqwest::Error),
    RetryLimitExceeded,
    ParseJSON(serde_json::Error),
    ProcessJSON(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HTTP(e) => write!(f, "error making request: {}", e),
            Self::RetryLimitExceeded => write!(f, "request retry limit exceeded"),
            Self::ParseJSON(e) => write!(f, "error parsing JSON: {}", e),
            Self::ProcessJSON(e) => write!(f, "error processing JSON structure: {}", e),
        }
    }
}

impl std::error::Error for Error {}

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

impl GeoCoord {
    pub fn neighborhood(&self, step_size: f64) -> (Self, Self) {
        (
            GeoCoord(self.0 - step_size, self.1 - step_size).clamp(),
            GeoCoord(self.0 + step_size, self.1 + step_size).clamp(),
        )
    }
    pub fn clamp(&self) -> Self {
        Self(self.0.clamp(-90.0, 90.0), self.1.clamp(-180.0, 180.0))
    }

    pub fn mid(&self, other: &Self) -> Self {
        Self((self.0 + other.0) / 2.0, (self.1 + other.1) / 2.0)
    }
}

pub struct GeoBounds(GeoCoord, GeoCoord);

impl GeoBounds {
    pub fn globe(step_size: f64) -> Vec<GeoBounds> {
        let mut all_regions = Vec::new();
        let mut lat = 0.0;
        while lat < 90.0 {
            let mut lon = -180.0;
            while lon < 180.0 {
                for lat_sign in [-1.0, 1.0] {
                    let coord = GeoCoord(lat * lat_sign, lon);
                    let (min, max) = coord.neighborhood(step_size);
                    all_regions.push(GeoBounds(min, max));
                    lon += step_size;
                }
            }
            lat += step_size;
        }
        all_regions
    }

    pub fn mid(&self) -> GeoCoord {
        return self.0.mid(&self.1);
    }

    pub fn split(&self) -> [GeoBounds; 4] {
        let x0 = self.0 .0;
        let y0 = self.0 .1;
        let x1 = self.mid().0;
        let y1 = self.mid().1;
        let x2 = self.1 .0;
        let y2 = self.1 .1;
        [
            GeoBounds(GeoCoord(x0, y0), GeoCoord(x1, y1)),
            GeoBounds(GeoCoord(x1, y0), GeoCoord(x2, y1)),
            GeoBounds(GeoCoord(x0, y1), GeoCoord(x1, y2)),
            GeoBounds(GeoCoord(x1, y1), GeoCoord(x2, y2)),
        ]
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MapItem {
    pub id: String,
    pub name: String,
    pub location: GeoCoord,
    pub address: Option<String>,
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
        bounds: &GeoBounds,
        max_retries: i32,
    ) -> Result<Vec<MapItem>> {
        let mut retry_count = 0;
        loop {
            let res = self.map_search_attempt(query, bounds).await;
            retry_count += 1;
            if res.is_ok() || retry_count > max_retries {
                break res;
            } else {
                eprintln!("retrying after error: {}", res.unwrap_err());
                sleep(Duration::from_secs(10)).await;
            }
        }
    }

    async fn map_search_attempt(&self, query: &str, bounds: &GeoBounds) -> Result<Vec<MapItem>> {
        for retry_timeout in [0.1, 1.0, 2.0, 4.0, 8.0, 10.0, 16.0, 32.0] {
            let response = self
                .client
                .get("https://www.bing.com/maps/overlaybfpr")
                .version(Version::HTTP_11)
                .query(&[
                    ("q", query),
                    ("filters", "direction_partner:\"maps\""),
                    ("mapcardtitle", ""),
                    ("p1", "[AplusAnswer]"),
                    ("count", "100"),
                    ("ecount", "100"),
                    ("first", "0"),
                    ("efirst", "1"),
                    (
                        "localMapView",
                        &format!(
                            "{:.15},{:.15},{:.15},{:.15}",
                            bounds.1 .0, bounds.0 .1, bounds.0 .0, bounds.1 .1
                        ),
                    ),
                    ("ads", "0"),
                    (
                        "cp",
                        &format!("{:.15}~{:.15}", bounds.mid().0, bounds.mid().1),
                    ),
                ])
                .send()
                .await?
                .text()
                .await?;
            // When overloaded, the server responds with messages of the form:
            // Ref A: DC5..................73B Ref B: AMB......06 Ref C: 2022-09-20T00:20:31Z
            if response.starts_with("Ref A:") {
                sleep(Duration::from_secs_f64(retry_timeout)).await;
                continue;
            }
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
                        address: read_object(&parsed, "entity.address").ok(),
                        phone: read_object(&parsed, "entity.phone").ok(),
                        chain_id: read_object(&parsed, "entity.chainId").ok(),
                    });
                }
            }
            return Ok(result);
        }
        Err(Error::RetryLimitExceeded)
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
    match T::from_json(cur_obj) {
        Ok(x) => Ok(x),
        Err(Error::ProcessJSON(x)) => Err(Error::ProcessJSON(format!(
            "error for object path {}: {}",
            path, x
        ))),
        other => other,
    }
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
