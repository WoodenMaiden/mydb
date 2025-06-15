use std::fs::{File, create_dir};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Ok, Result, anyhow};

use log::debug;
use serde::{Deserialize, Serialize};

const HASH_LENGTH: usize = 64;
const METADATA_LENGTH: usize = HASH_LENGTH + 1;

const LINE_FEED: u8 = 10u8;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum WriteEvent {
    Write(String, Vec<u8>),
    Delete(String),
}

#[derive(Debug)]
pub struct WOLEngine {
    // directory: PathBuf,
    current_log_file: File,
}

impl WOLEngine {
    pub fn new(directory: PathBuf) -> Result<Self> {
        if !directory.exists() {
            create_dir(directory.clone())?;
        } else {
            // Do the WOL reading here since it eans that there might be previous data left
        }

        let current_log_file = File::create_new(directory.join(format!(
            "{}",
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
        )))?;

        Ok(WOLEngine {
            // directory,
            current_log_file,
        })
    }

    // fn read_events(file: File) -> Result<Option<WriteEvent>> {

    // }

    pub fn _parse_entry(&self, buf: &[u8]) -> Result<WriteEvent> {
        let meta = &buf[..METADATA_LENGTH];
        let hash = &meta[..HASH_LENGTH];
        let data_length = &meta[HASH_LENGTH-1..METADATA_LENGTH][0];

        let data = &buf[METADATA_LENGTH-1..buf.len()-1];

        if sha256::digest(data).as_bytes() != hash && *data_length == (data.len() as u8) {
            return Err(anyhow!("Entry is corrupted"));
        };

        Ok(bson::from_reader(data)?)
    }

    pub fn write_event(&mut self, event: WriteEvent) -> Result<()> {
        let data = bson::to_vec(&event)?;
        let length = data.clone().len();
        let binding = sha256::digest(data.clone());
        let digest = binding.as_bytes();

        debug!("hash is {} bytes long", digest.len());

        self.current_log_file.write_all(
            &[
                digest,
                &[length as u8],
                &data,
                &[LINE_FEED],
            ]
            .concat(),
        )?;

        let _ = self.current_log_file.sync_all();

        Ok(())
    }
}
