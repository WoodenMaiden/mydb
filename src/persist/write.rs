use std::io::Write;
use std::path::PathBuf;
use anyhow::{Context, anyhow};
use log::info;
use rand::distr::{Alphanumeric, SampleString};
use std::fs::{OpenOptions, rename};

const RANDOM_NAME_LEN: usize = 16;

/// Writes the given data into a file in a Readers-writer atomic way.
/// It means that if a reader reads the file while a writer edits it it will not get a bad sate, it does that by writing into a temporary file, running dsync on that file and then rename that file. 
/// However this is not the ideal, this is not power-loss atomic, if a crash occcurs between the fsync and the renaming we are screwed
pub fn bwrite1(path: PathBuf, data: Vec<u8>) -> anyhow::Result<()> {
    let mut temp_file_name = PathBuf::from(path.clone());
    temp_file_name.set_extension(format!(
        "tmp.{}",
        Alphanumeric.sample_string(&mut rand::rng(), RANDOM_NAME_LEN)
    ));

    info!("Creating temp file {:?}", temp_file_name.clone());


    let mut target_file = OpenOptions::new()
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
        })?; // 1. we create a temporary file

    target_file.write_all(data.as_slice()).with_context(|| {
        anyhow!(format!(
            "Failed to write data to the file {:#?}",
            temp_file_name
        ))
    })?; // 2. we attempt to write all data 
    target_file
        .sync_all()
        .with_context(|| anyhow!(format!("Failed to fsync the file {:#?}", temp_file_name)))?; // 3. we run fsync in our temp file

    rename(&temp_file_name, path.clone()).with_context(|| {
        anyhow!(format!(
            "Moving the file from {:#?} to {:#?} failed",
            temp_file_name, path
        ))
    })?; // 4. We move the file, doing so we don't end up with a half written file between the write and the fsync

    // Rust closes the file when the destruction (meaning ending bloc of bwrite1 ) happens
    return Ok(());
}
