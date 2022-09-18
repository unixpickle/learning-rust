mod bing_maps;

use bing_maps::{Client, GeoCoord};
use clap::Parser;
use std::process::ExitCode;

#[derive(Parser, Clone)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(value_parser)]
    store_name: String,

    #[clap(value_parser)]
    output_path: String,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    let client = Client::new();
    let response = client
        .map_search(
            &cli.store_name,
            GeoCoord(37.0, -122.0),
            GeoCoord(38.0, -121.0),
            GeoCoord(37.5, -121.5),
        )
        .await;
    println!(
        "got {:?} results",
        serde_json::to_string(&response.unwrap())
    );

    ExitCode::SUCCESS
}
