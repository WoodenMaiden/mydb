use anyhow::{anyhow, Context, Ok};
use anyhow::Result;
use std::fs::rename;
use std::io::Write;
use std::path::PathBuf;

use crate::persist::{create_temp_file, fsync_parent_dir};


/// Writes the given data into a file in a Readers-writer atomic way.
/// It means that if a reader reads the file while a writer edits it it will not get a bad sate, it does that by writing into a temporary file, running dsync on that file and then rename that file.
/// However this is not the ideal, this is not power-loss atomic, if a crash occcurs between the fsync and the renaming we are screwed
pub fn _bwrite1(path: PathBuf, data: &[u8]) -> Result<()> {
    let (temp_file_name, mut target_file) = create_temp_file(path.clone())?; // 1. we create a temporary file

    target_file.write_all(data).with_context(|| {
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

/// Writes the given data into a file in a readers-writer and power-loss atomic way.
/// Power-loss atomicity means that is a crash occurs during the write, the file will not be left in a inconsistent state.
/// So in addition to what has been done in `bwrite1`, we will also make a fsync on the parent directory of the file. Some filesystem do not support that
pub fn bwrite2(path: PathBuf, data: &[u8]) -> Result<()> {
    let (temp_file_name, mut target_file) = create_temp_file(path.clone())?; // 1. we create a temporary file

    target_file.write_all(data).with_context(|| {
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

    fsync_parent_dir(path) // 5. We run fsync on the parent directory so we update its metadata to tell it the temp file has been renamed 
}
