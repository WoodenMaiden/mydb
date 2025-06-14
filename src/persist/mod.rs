use std::{fs::{File, OpenOptions}, path::PathBuf};
use anyhow::{anyhow, Context, Result};
use log::warn;
use rand::distr::{Alphanumeric, SampleString};

pub mod write;
pub mod read;
pub mod del;

const RANDOM_NAME_LEN: usize = 16;

fn create_temp_file(og_path: PathBuf) -> Result<( PathBuf, File )> {
    let ext = Alphanumeric.sample_string(&mut rand::rng(), RANDOM_NAME_LEN);
    let mut temp_file_name = PathBuf::from(og_path);
    temp_file_name.set_extension(format!("tmp.{}", ext));
    
    let file = OpenOptions::new()
        // We truncate the file as if we are not doing so we might end up whith a half written file
        .truncate(true)
        .read(true)
        .write(true)
        .create(true)
        .open(temp_file_name.clone())
        .with_context(|| {
            anyhow!(format!(
                "Failed to create a temp file {:#?}",
                temp_file_name
            ))
        })?;

    Ok((temp_file_name, file))
}

fn fsync_parent_dir(path: PathBuf) -> Result<()> {
    match File::open(
        path.parent()
            .ok_or_else(|| anyhow!("Failed to get parent directory of {:?}", path))?,
    )
    .with_context(|| anyhow!(format!("Failed to open parent directory of {:#?}", path)))?
    .sync_all() { 
        Ok(_) => Ok(()),
        Err(_) => {
            warn!("fsync() on directories is not supported on this filesystem!");
            Ok(())
        }
    }

}