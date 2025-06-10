use anyhow::{Context, Result};
use log::{info, LevelFilter};
use serde::{Deserialize, Serialize};
use std::fs::{File, metadata};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub data: DataConfig,
    pub log: LogConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataConfig {
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    pub level: LevelFilter,
}

pub fn parse_config(config_path: PathBuf) -> Result<Config> {
    let conf = read_config_file(config_path.clone())
        .with_context(|| format!("Failed to read config file at {:#?}", config_path))?;

    let data_path = metadata(conf.data.path.clone()).with_context(|| {
        anyhow::anyhow!(
            "Failed to read metadata for data path: {:?}",
            conf.data.path
        )
    })?;

    if !data_path.is_dir() {
        return Err(anyhow::anyhow!("Data path must be a directory"));
    }

    if data_path.permissions().readonly() {
        return Err(anyhow::anyhow!("Directory must have written permissions"));
    }

    info!("{:?}", conf);

    Ok(conf)
}

fn read_config_file(config_path: PathBuf) -> Result<Config> {
    Ok(serde_yaml::from_reader(File::open(config_path)?)
        .map_err(|e| anyhow::anyhow!("Failed to serielize config file: {}", e))?)
}
