use std::fs::{metadata, File};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Result;
use log::LevelFilter;

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

pub fn parse_options(config_path: PathBuf) -> Result<Config> {
    let conf = read_config_file(config_path)?;

    let data_path = metadata(conf.data.path.clone())
        .map_err(|e| anyhow::anyhow!("Failed to read metadata for data path: {}", e))?;
   
    if !data_path.is_dir() {
        return Err(anyhow::anyhow!("Data path must be a directory"));
    }

    if data_path.permissions().readonly() {
        return Err(anyhow::anyhow!("Directory must have written permissions"));
    }

    Ok(conf)    
}

fn read_config_file(config_path: PathBuf) -> Result<Config> {
    Ok(serde_yaml::from_reader(
        File::open(config_path)?
    ).map_err(|e| {
        anyhow::anyhow!("Failed to parse config file: {}", e)
    })?)
}