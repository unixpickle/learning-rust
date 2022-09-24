use crate::bing_maps;
use crate::bing_maps::{Client, MapItem};
use crate::geo_coord::GeoBounds;
use clap::Parser;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;
use tokio::{
    fs::{remove_file, File},
    io::AsyncWriteExt,
    spawn,
    sync::mpsc::channel,
};

#[derive(Clone, Parser)]
pub struct ScrapeArgs {
    #[clap(short, long, value_parser, default_value_t = 2)]
    max_subdivisions: i32,

    #[clap(short, long, value_parser, default_value_t = 2.0)]
    step_size: f64,

    #[clap(short, long, value_parser, default_value_t = 4)]
    parallelism: i32,

    #[clap(short, long, value_parser, default_value_t = 5)]
    retries: i32,

    #[clap(value_parser)]
    store_name: String,

    #[clap(value_parser)]
    output_path: String,
}

pub async fn scrape(cli: ScrapeArgs) -> anyhow::Result<()> {
    let mut output = File::create(&cli.output_path).await?;

    let regions = world_regions(cli.step_size);
    let region_count = regions.lock().await.len();
    let (response_tx, response_rx) = channel((cli.parallelism as usize) * 10);
    for _ in 0..cli.parallelism {
        spawn(fetch_regions(
            cli.store_name.clone(),
            cli.max_subdivisions,
            cli.retries,
            regions.clone(),
            response_tx.clone(),
        ));
    }
    // Make sure the channel is ended once all the workers finish.
    drop(response_tx);

    if let result @ Err(_) = write_outputs(&cli, &mut output, response_rx, region_count).await {
        eprintln!("deleting output due to error...");
        drop(output);
        remove_file(cli.output_path).await?;
        result
    } else {
        output.flush().await?;
        Ok(())
    }
}

async fn write_outputs(
    cli: &ScrapeArgs,
    output: &mut File,
    mut response_rx: Receiver<bing_maps::Result<Vec<MapItem>>>,
    region_count: usize,
) -> anyhow::Result<()> {
    let mut found = HashSet::new();
    let mut completed_regions: usize = 0;
    while let Some(response) = response_rx.recv().await {
        let listing = response?;
        for x in listing {
            if found.insert(x.id.clone()) {
                output
                    .write_all((serde_json::to_string(&x)? + "\n").as_bytes())
                    .await?;
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
    Ok(())
}

fn world_regions(step_size: f64) -> Arc<Mutex<Vec<GeoBounds>>> {
    let all_regions = GeoBounds::globe(step_size);
    Arc::new(Mutex::new(all_regions))
}

async fn fetch_regions(
    store_name: String,
    max_subdivisions: i32,
    max_retries: i32,
    tasks: Arc<Mutex<Vec<GeoBounds>>>,
    results: Sender<bing_maps::Result<Vec<MapItem>>>,
) {
    let client = Client::new();
    while let Some(bounds) = pop_task(&tasks).await {
        let response =
            fetch_bounds_subdivided(&client, &store_name, bounds, max_retries, max_subdivisions)
                .await;
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

async fn pop_task(tasks: &Arc<Mutex<Vec<GeoBounds>>>) -> Option<GeoBounds> {
    tasks.lock().await.pop()
}

async fn fetch_bounds_subdivided(
    client: &Client,
    query: &str,
    bounds: GeoBounds,
    max_retries: i32,
    max_subdivisions: i32,
) -> bing_maps::Result<Vec<MapItem>> {
    // This would be easier with recursion than a depth-first search,
    // but recursion with futures is super annoying and wouldn't allow
    // us to use finite lifetimes for the arguments.
    let initial_results = client.map_search(query, &bounds, max_retries).await?;
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
                let new_results = client.map_search(query, &subdivided, max_retries).await?;
                queue.push_back((subdivided, new_results, depth + 1));
            }
        }
    }
    Ok(results.into_values().into_iter().collect())
}
