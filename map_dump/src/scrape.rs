use crate::bing_maps;
use crate::bing_maps::{Client, GeoBounds, MapItem};
use async_channel::{bounded, unbounded, Receiver, Sender};
use clap::Parser;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    process::ExitCode,
};
use tokio::{fs::File, io::AsyncWriteExt, spawn};

#[derive(Clone, Parser)]
pub struct ScrapeArgs {
    #[clap(short, long, value_parser, default_value_t = 2)]
    max_subdivisions: i32,

    #[clap(short, long, value_parser, default_value_t = 2.0)]
    step_size: f64,

    #[clap(short, long, value_parser, default_value_t = 4)]
    parallelism: i32,

    #[clap(value_parser)]
    store_name: String,

    #[clap(value_parser)]
    output_path: String,
}

pub async fn scrape(cli: ScrapeArgs) -> ExitCode {
    let mut output = match File::create(cli.output_path).await {
        Ok(x) => x,
        Err(e) => {
            eprintln!("error opening output: {}", e);
            return ExitCode::FAILURE;
        }
    };

    let (regions, region_count) = world_regions(cli.step_size);
    let (response_tx, response_rx) = bounded((cli.parallelism as usize) * 10);
    for _ in 0..cli.parallelism {
        spawn(fetch_regions(
            cli.store_name.clone(),
            cli.max_subdivisions,
            regions.clone(),
            response_tx.clone(),
        ));
    }
    // Make sure the channel is ended once all the workers finish.
    drop(response_tx);

    let mut found = HashSet::new();
    let mut completed_regions: usize = 0;
    while let Ok(response) = response_rx.recv().await {
        match response {
            Ok(listing) => {
                for x in listing {
                    if found.insert(x.id.clone()) {
                        if let Err(e) = output
                            .write_all((serde_json::to_string(&x).unwrap() + "\n").as_bytes())
                            .await
                        {
                            eprintln!("error writing output: {}", e);
                            return ExitCode::FAILURE;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                return ExitCode::FAILURE;
            }
        }
        completed_regions += 1;
        println!(
            "store \"{}\": completed {}/{} queries ({:.3}%, found {})",
            cli.store_name,
            completed_regions,
            region_count,
            100.0 * (completed_regions as f64) / (region_count as f64),
            found.len()
        );
    }

    if let Err(e) = output.flush().await {
        eprintln!("error writing output: {}", e);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn world_regions(step_size: f64) -> (Receiver<GeoBounds>, usize) {
    let all_regions = GeoBounds::globe(step_size);
    let count = all_regions.len();

    let (tx, rx) = unbounded();
    spawn(async move {
        for region in all_regions {
            if tx.send(region).await.is_err() {
                break;
            }
        }
        tx.close();
    });

    (rx, count)
}

async fn fetch_regions(
    store_name: String,
    max_subdivisions: i32,
    tasks: Receiver<GeoBounds>,
    results: Sender<bing_maps::Result<Vec<MapItem>>>,
) {
    let client = Client::new();
    while let Ok(bounds) = tasks.recv().await {
        let response =
            fetch_bounds_subdivided(&client, &store_name, bounds, max_subdivisions).await;
        let was_ok = response.is_ok();
        if results.send(response).await.is_err() || !was_ok {
            // If we cannot send, it means the main coroutine died
            // due to some error. If we sent an error, there is no
            // point continuing to do work, since the main coroutine
            // will die.
            break;
        }
    }
}

async fn fetch_bounds_subdivided(
    client: &Client,
    query: &str,
    bounds: GeoBounds,
    max_subdivisions: i32,
) -> bing_maps::Result<Vec<MapItem>> {
    // This would be easier with recursion than a depth-first search,
    // but recursion with futures is super annoying and wouldn't allow
    // us to use finite lifetimes for the arguments.
    let initial_results = client.map_search(query, &bounds).await?;
    let mut queue = VecDeque::from([(bounds, initial_results, 0)]);
    let mut results = HashMap::new();
    while let Some((bounds, sub_results, depth)) = queue.pop_front() {
        let old_count = results.len();
        for result in sub_results {
            results.insert(result.id.clone(), result);
        }
        let new_count = results.len();

        // Only expand a region if expanding it is still giving new
        // results, indicating that this area is dense with stores.
        if new_count > old_count && depth < max_subdivisions {
            for subdivided in bounds.split() {
                let new_results = client.map_search(query, &subdivided).await?;
                queue.push_back((subdivided, new_results, depth + 1));
            }
        }
    }
    Ok(results.into_values().into_iter().collect())
}
