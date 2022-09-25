use crate::{
    bing_maps::MapItem,
    geo_coord::{GeoCoord, VecGeoCoord},
};
use ndarray::{Array1, Array2};
use rand::{rngs::StdRng, SeedableRng};
use serde_json::Value;
use std::{collections::HashMap, path::PathBuf};

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

    // If we are covering units of radius 10km, then we should need
    // at least 510_000_000/(pi*10^2) = 1.6*10^6 samples to cover
    // most of the planet.
    #[clap(short, long, value_parser, default_value_t = 2_000_000)]
    samples: i64,

    // Default radius is roughly 10km
    #[clap(short, long, value_parser, default_value_t = 0.001569612306)]
    radius: f64,

    #[clap(long, value_parser)]
    single_count: bool,

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
    let sorted_locations = sorted_names
        .iter()
        .map(|x| store_locations[x].clone())
        .collect::<Vec<_>>();

    let (results_tx, mut results_rx) = channel(1000);

    for _ in 0..cli.workers {
        let results_tx_clone = results_tx.clone();
        let locations_clone = sorted_locations.clone();
        let radius = cli.radius;
        let single_count = cli.single_count;
        spawn_blocking(move || {
            let mut rng = StdRng::from_rng(rand::thread_rng()).unwrap();
            loop {
                let query = GeoCoord::random(&mut rng);
                let covec = cooccurrence_at(&locations_clone, query, radius, single_count);
                if !results_tx_clone.blocking_send(covec).is_ok() {
                    return;
                }
            }
        });
    }
    // Make sure we don't block on reading results.
    drop(results_tx);

    println!("computing cooccurrence matrix...");
    let n = sorted_locations.len();
    let mut result = Array2::<f32>::zeros((n, n));
    let mut completed: i64 = 0;
    while completed < cli.samples {
        let sub_result = results_rx.recv().await.unwrap();
        result += &sub_result;
        completed += 1;
        if completed % (cli.samples / 1000).max(1) == 0 {
            println!(
                "completed {}/{} samples ({:.02}%)",
                completed,
                cli.samples,
                100.0 * (completed as f64) / (cli.samples as f64)
            );
        }
    }
    result /= completed as f32;

    // Tell the workers to stop sending results.
    drop(results_rx);

    println!("serializing resulting matrix to {}...", cli.output_path);
    let serialized = serialize_matrix(sorted_names, result)?;
    let mut writer = File::create(cli.output_path).await?;
    writer.write_all(serialized.as_bytes()).await?;
    writer.flush().await?;

    Ok(())
}

pub async fn read_locations(src: PathBuf) -> anyhow::Result<Vec<VecGeoCoord>> {
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

pub fn cooccurrence_at(
    locations: &Vec<Vec<VecGeoCoord>>,
    query_point: GeoCoord,
    radius: f64,
    single_count: bool,
) -> Array2<f32> {
    let vec_query = query_point.into();
    let cos_radius = radius.cos();
    let n = locations.len();
    let mut counts = Array1::<f32>::zeros((n,));
    {
        let slice = counts.as_slice_mut().unwrap();
        for (i, store_locations) in locations.iter().enumerate() {
            for store_location in store_locations {
                if store_location.cos_geo_dist(&vec_query) > cos_radius {
                    if single_count {
                        slice[i] = 1.0;
                    } else {
                        slice[i] = slice[i] + 1.0;
                    }
                }
            }
        }
    }
    let column = counts.clone().into_shape((n, 1)).unwrap();
    let row = counts.into_shape((1, n)).unwrap();
    return column.dot(&row);
}

pub fn serialize_matrix(names: Vec<String>, result: Array2<f32>) -> anyhow::Result<String> {
    let matrix_vec: Vec<Vec<f32>> = result
        .outer_iter()
        .map(|x| x.iter().map(|x| *x).collect())
        .collect();
    let obj = HashMap::from([
        ("matrix", Value::from(matrix_vec)),
        ("names", Value::from(names)),
    ]);
    Ok(serde_json::to_string(&obj)?)
}
