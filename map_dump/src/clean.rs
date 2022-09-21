use crate::bing_maps::MapItem;
use std::path::PathBuf;

use clap::Parser;
use tokio::{
    fs::{read_dir, File},
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
    // TODO: filter the lines here to use the most popular chain_id.
    write_map_items(dst, items).await
}

pub async fn read_map_items(src: PathBuf) -> anyhow::Result<Vec<MapItem>> {
    let mut reader = File::open(src).await?;
    let mut data = String::new();
    reader.read_to_string(&mut data).await?;
    let result = data
        .split("\n")
        .into_iter()
        .map(|x| serde_json::from_str(&x))
        .collect::<Result<Vec<_>, _>>();
    Ok(result?)
}

pub async fn write_map_items(path: PathBuf, items: Vec<MapItem>) -> anyhow::Result<()> {
    let mut writer = File::create(path).await?;
    for item in items {
        writer
            .write_all(serde_json::to_string(&item)?.as_bytes())
            .await?;
    }
    writer.flush().await?;
    Ok(())
}
