use crate::{
    bing_maps::MapItem,
    geo_coord::{GeoCoord, VecGeoCoord},
};
use ndarray::{Array1, Array2, LinalgScalar};
use rand::{rngs::StdRng, SeedableRng};
use serde_json::{Map, Value};
use std::{collections::HashMap, f64::consts::PI, ops::AddAssign, path::PathBuf};

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

    #[clap(short, long, value_parser, default_value_t = 8)]
    workers: i32,
}

pub async fn cooccurrence(cli: CoocurrenceArgs) -> anyhow::Result<()> {
    println!("Taking {} samples with radius {}", cli.samples, cli.radius);
    println!(
        "For radius {}, samples should be at least {}",
        cli.radius,
        (4.0 * PI / (PI * cli.radius.powi(2))).ceil() as i64
    );
    println!(
        "For {} samples, radius should be at least {:.08}",
        cli.samples,
        ((4.0 * PI / (cli.samples as f64)) / PI).sqrt()
    );

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
        spawn_blocking(move || {
            let mut rng = StdRng::from_rng(rand::thread_rng()).unwrap();
            loop {
                let query = GeoCoord::random(&mut rng);
                let covec = matrices_in_region(&locations_clone, query, radius);
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
    let mut matrices = Matrices::zeros(n);
    let mut completed: i64 = 0;
    while completed < cli.samples {
        let sub_result = results_rx.recv().await.unwrap();
        matrices.add(&sub_result);
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

    // Tell the workers to stop sending results.
    drop(results_rx);

    println!("serializing resulting matrix to {}...", cli.output_path);
    let metadata = Value::Object(Map::from_iter([
        ("samples".to_owned(), Value::from(cli.samples)),
        ("radius".to_owned(), Value::from(cli.radius)),
    ]));
    let serialized = serde_json::to_string(&matrices.into_json(sorted_names, metadata))?;
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

fn matrices_in_region(
    locations: &Vec<Vec<VecGeoCoord>>,
    query_point: GeoCoord,
    radius: f64,
) -> Matrices {
    let n = locations.len();

    // Cache some computation for a speedup.
    let cos_radius = radius.cos();
    let vec_query = query_point.into();

    let mut counts = Array1::<f64>::zeros((n,));
    let mut one_hot = Array1::<f64>::zeros((n,));
    {
        for (i, store_locations) in locations.iter().enumerate() {
            for store_location in store_locations {
                if store_location.cos_geo_dist(&vec_query) > cos_radius {
                    one_hot[i] = 1.0;
                    counts[i] = counts[i] + 1.0;
                }
            }
        }
    }
    let counts_outer = outer_product(counts);
    Matrices {
        cooccurrence: counts_outer.clone(),
        correlation: outer_product(one_hot),
        second_moment: counts_outer,
        count: 1,
    }
}

fn outer_product<T: LinalgScalar>(vec: Array1<T>) -> Array2<T> {
    let n = vec.shape()[0];
    let column = vec.clone().into_shape((n, 1)).unwrap();
    let row = vec.into_shape((1, n)).unwrap();
    return column.dot(&row);
}

struct Matrices {
    cooccurrence: Array2<f64>,
    correlation: Array2<f64>,
    second_moment: Array2<f64>,
    count: usize,
}

impl Matrices {
    fn zeros(n: usize) -> Matrices {
        Matrices {
            cooccurrence: Array2::zeros((n, n)),
            correlation: Array2::zeros((n, n)),
            second_moment: Array2::zeros((n, n)),
            count: 0,
        }
    }

    fn add(&mut self, other: &Matrices) {
        self.cooccurrence.add_assign(&other.cooccurrence);
        self.correlation.add_assign(&other.correlation);
        self.second_moment.add_assign(&other.second_moment);
        self.count += other.count;
    }

    fn into_json(mut self, names: Vec<String>, metadata: Value) -> Value {
        let correlation_denom = outer_product(self.correlation.clone().into_diag().to_owned())
            .map(|x| x.sqrt().max(1e-8));
        self.correlation = self.correlation / correlation_denom;
        self.second_moment /= self.count as f64;
        Map::from_iter([
            ("metadata".to_owned(), metadata),
            (
                "cooccurrence".to_owned(),
                Value::from(matrix_vec(self.cooccurrence)),
            ),
            (
                "correlation".to_owned(),
                Value::from(matrix_vec(self.correlation)),
            ),
            (
                "second_moment".to_owned(),
                Value::from(matrix_vec(self.second_moment)),
            ),
            ("names".to_owned(), names.into()),
        ])
        .into()
    }
}

fn matrix_vec<T: LinalgScalar>(matrix: Array2<T>) -> Vec<Vec<T>> {
    return matrix
        .outer_iter()
        .map(|x| x.iter().map(|x| *x).collect())
        .collect();
}
