use std::{process::ExitCode, str::FromStr};

use clap::Parser;
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::Value;

#[derive(Parser, Clone)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(short, long, value_parser)]
    headers_json: String,

    #[clap(short, long, value_parser)]
    params_json: String,

    #[clap(value_parser)]
    url: String,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    let mut parsed_params: Value = serde_json::from_str(&cli.params_json).unwrap();

    let re = Regex::new(r"<NextMarker>(.*)</NextMarker>").unwrap();

    let client = reqwest::Client::new();

    loop {
        let req = client
            .get(&cli.url)
            .query(&parsed_params)
            .headers(parse_header_map(&cli.headers_json))
            .build()
            .unwrap();
        let response = client.execute(req).await.unwrap();
        let text = response.text().await.unwrap();

        for item in parse_listing(&text) {
            println!("{}", item);
        }
        if let Some(next_match) = re.captures(&text) {
            let token = next_match.get(1).unwrap().as_str().to_owned();
            parsed_params
                .as_object_mut()
                .unwrap()
                .insert("marker".into(), token.into());
        } else {
            break;
        }
    }

    ExitCode::SUCCESS
}

fn parse_header_map(headers_json: &str) -> HeaderMap {
    let parsed: Value = serde_json::from_str(headers_json).unwrap();
    let obj = parsed.as_object().unwrap();
    let mut res = HeaderMap::new();
    for (k, v) in obj {
        res.append(
            HeaderName::from_str(&k).unwrap(),
            HeaderValue::from_str(v.as_str().unwrap()).unwrap(),
        );
    }
    res
}

fn parse_listing(listing: &str) -> Vec<String> {
    let expr = Regex::new(r"<Name>(.*?)</Name>").unwrap();
    let mut res = Vec::new();
    for x in expr.captures_iter(listing) {
        res.push(x.get(1).unwrap().as_str().to_owned());
    }
    res
}
