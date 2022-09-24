use crate::bing_maps::MapItem;
use std::{collections::HashMap, path::PathBuf};

use clap::Parser;
use tokio::{
    fs::{create_dir, read_dir, File},
    io::{AsyncReadExt, AsyncWriteExt},
};

#[derive(Clone, Parser)]
pub struct CleanArgs {
    #[clap(value_parser)]
    input_dir: String,

    #[clap(value_parser)]
    output_dir: String,
}

pub async fn clean(cli: CleanArgs) -> anyhow::Result<()> {
    let input_dir = PathBuf::from(cli.input_dir);
    let output_dir = PathBuf::from(cli.output_dir);

    create_dir(&output_dir).await?;

    let mut reader = read_dir(&input_dir).await?;
    while let Some(entry) = reader.next_entry().await? {
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| anyhow::Error::msg("failed to convert strings"))?;
        if name.starts_with(".") || !name.to_lowercase().ends_with(".json") {
            continue;
        }
        clean_file(entry.path(), output_dir.join(entry.file_name())).await?;
    }
    Ok(())
}

pub async fn clean_file(src: PathBuf, dst: PathBuf) -> anyhow::Result<()> {
    let items = read_map_items(src).await?;

    // Count using the entry() API, as presented in the docs:
    // https://doc.rust-lang.org/std/collections/index.html#counting-the-number-of-times-each-character-in-a-string-occurs
    let mut counter = HashMap::new();
    for item in &items {
        if let Some(id) = &item.chain_id {
            *counter.entry(id).or_insert(0) += 1;
        }
    }

    // Only filter by chain ID if some chain was present.
    if let Some((chain_id, _)) =
        counter
            .into_iter()
            .reduce(|(chain_id, count), (new_chain_id, new_count)| {
                if new_count > count {
                    (new_chain_id, new_count)
                } else {
                    (chain_id, count)
                }
            })
    {
        write_map_items(
            dst,
            items
                .iter()
                .filter(|x| x.chain_id.as_deref() == Some(chain_id)),
        )
        .await
    } else {
        write_map_items(dst, items.iter()).await
    }
}

pub async fn read_map_items(src: PathBuf) -> anyhow::Result<Vec<MapItem>> {
    let mut reader = File::open(src).await?;
    let mut data = String::new();
    reader.read_to_string(&mut data).await?;
    let result = data
        .split("\n")
        .into_iter()
        .filter(|x| x.len() > 0)
        .map(|x| serde_json::from_str(&x))
        .collect::<Result<Vec<_>, _>>();
    Ok(result?)
}

pub async fn write_map_items(
    path: PathBuf,
    items: impl Iterator<Item = &MapItem>,
) -> anyhow::Result<()> {
    let mut writer = File::create(path).await?;
    for item in items {
        writer
            .write_all(format!("{}\n", serde_json::to_string(item)?).as_bytes())
            .await?;
    }
    writer.flush().await?;
    Ok(())
}
