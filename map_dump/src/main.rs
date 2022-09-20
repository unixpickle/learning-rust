mod bing_maps;
mod scrape;

use clap::Parser;
use std::process::ExitCode;

#[derive(Parser, Clone)]
#[clap(author, version, about, long_about = None)]
enum Cli {
    Scrape {
        #[clap(flatten)]
        args: scrape::ScrapeArgs,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli {
        Cli::Scrape { args } => scrape::scrape(args).await,
    }
}
