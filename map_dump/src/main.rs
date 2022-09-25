mod bing_maps;
mod clean;
mod cooccurrence;
mod geo_coord;
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
    Clean {
        #[clap(flatten)]
        args: clean::CleanArgs,
    },
    Cooccurrence {
        #[clap(flatten)]
        args: cooccurrence::CoocurrenceArgs,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    if let Err(e) = match cli {
        Cli::Scrape { args } => scrape::scrape(args).await,
        Cli::Clean { args } => clean::clean(args).await,
        Cli::Cooccurrence { args } => cooccurrence::cooccurrence(args).await,
    } {
        eprintln!("{}", e);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
