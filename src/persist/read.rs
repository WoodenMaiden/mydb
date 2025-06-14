use std::{fs::File, io::Read, path::PathBuf};
use anyhow::{anyhow, Context, Result};

pub fn read(path: PathBuf) -> Result<Vec<u8>> {
    let mut file = File::open(&path)
        .with_context(|| anyhow!(format!("Failed to open file {:#?}", path)))?;
    
    let mut data = Vec::new();
    file.read_to_end(&mut data)
        .with_context(|| anyhow!(format!("Failed to read data from file {:#?}", path)))?;
    
    Ok(data)
}