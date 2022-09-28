use crate::{bing_maps::MapItem, geo_coord::VecGeoCoord};
use ndarray::{Array2, LinalgScalar};
use serde_json::{Map, Value};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{collections::HashMap, path::PathBuf, sync::Arc};

use clap::Parser;
use tokio::{
    fs::{read_dir, File},
    io::{AsyncReadExt, AsyncWriteExt},
    sync::mpsc::channel,
    task::spawn_blocking,
};

#[derive(Clone, Parser)]
pub struct CoocurrenceArgs {
    #[clap(value_parser)]
    input_dir: String,

    #[clap(value_parser)]
    output_path: String,

    // Default radius is roughly 1 mile (in radians).
    #[clap(short, long, value_parser, default_value_t = 0.00025260179852480549)]
    radius: f64,

    #[clap(short, long, value_parser, default_value_t = 8)]
    workers: i32,
}

pub async fn cooccurrence(cli: CoocurrenceArgs) -> anyhow::Result<()> {
    let input_dir = PathBuf::from(cli.input_dir);

    let mut store_locations = HashMap::new();
    let mut reader = read_dir(&input_dir).await?;
    while let Some(entry) = reader.next_entry().await? {
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| anyhow::Error::msg("failed to convert strings"))?;
        if name.starts_with(".") {
            continue;
        }
        if let Some(store_name) = name.strip_suffix(".json") {
            store_locations.insert(store_name.to_owned(), read_locations(entry.path()).await?);
        }
    }

    // Get a canonical ordering of stores for the matrix.
    let mut sorted_names = store_locations
        .keys()
        .map(|x| x.clone())
        .collect::<Vec<_>>();
    sorted_names.sort();

    // Create a flat list of (store_index, location) pairs.
    let mut pairs = Vec::new();
    for (i, name) in sorted_names.iter().enumerate() {
        for location in &store_locations[name] {
            pairs.push((i, location.clone()));
        }
    }

    println!("computing cooccurrence matrix...");

    let cur_index = Arc::new(AtomicUsize::new(0));
    let (results_tx, mut results_rx) = channel(cli.workers as usize);
    for _ in 0..cli.workers {
        let results_tx_clone = results_tx.clone();
        let pairs_clone = pairs.clone();
        let num_stores = store_locations.len();
        let cos_radius = cli.radius.cos();
        let cur_index_clone = cur_index.clone();
        spawn_blocking(move || {
            let mut result = Array2::<f64>::zeros((num_stores, num_stores));
            loop {
                let src = cur_index_clone.fetch_add(1, Ordering::SeqCst);
                if src % (pairs_clone.len() / 100) == 0 {
                    eprintln!(
                        "done {}/{} ({:.2}%)",
                        src,
                        pairs_clone.len(),
                        100.0 * (src as f64) / (pairs_clone.len() as f64)
                    );
                }
                if src >= pairs_clone.len() {
                    break;
                }
                let (src_store, src_loc) = &pairs_clone[src];
                for (dst, (dst_store, dst_loc)) in pairs_clone.iter().enumerate().skip(src) {
                    if src != dst && src_loc.cos_geo_dist(dst_loc) > cos_radius {
                        result[(*src_store, *dst_store)] += 1.0;
                        result[(*dst_store, *src_store)] += 1.0;
                    }
                }
            }
            results_tx_clone.blocking_send(result).unwrap();
        });
    }
    // Make sure we don't block on reading results.
    drop(results_tx);

    let mut result = Array2::<f64>::zeros((store_locations.len(), store_locations.len()));
    while let Some(x) = results_rx.recv().await {
        result = result + x;
    }

    println!("serializing resulting matrix to {}...", cli.output_path);
    let result_dict = Value::Object(Map::from_iter([
        ("radius".to_owned(), Value::from(cli.radius)),
        ("names".to_owned(), Value::from(sorted_names)),
        ("matrix".to_owned(), Value::from(matrix_vec(result))),
    ]));
    let serialized = serde_json::to_string(&result_dict)?;
    let mut writer = File::create(cli.output_path).await?;
    writer.write_all(serialized.as_bytes()).await?;
    writer.flush().await?;

    Ok(())
}

async fn read_locations(src: PathBuf) -> anyhow::Result<Vec<VecGeoCoord>> {
    let mut reader = File::open(src).await?;
    let mut data = String::new();
    reader.read_to_string(&mut data).await?;
    let result = data
        .split("\n")
        .into_iter()
        .filter(|x| x.len() > 0)
        .map(|x| serde_json::from_str::<MapItem>(&x).map(|x| x.location.into()))
        .collect::<Result<Vec<_>, _>>();
    Ok(result?)
}

fn matrix_vec<T: LinalgScalar>(matrix: Array2<T>) -> Vec<Vec<T>> {
    return matrix
        .outer_iter()
        .map(|x| x.iter().map(|x| *x).collect())
        .collect();
}
