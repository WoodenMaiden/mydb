use anyhow::{Context, Result};
use std::{cell::RefCell, path::PathBuf};

use crate::{
    persist::{del::delete, read::read, write::bwrite2},
    wol::{self, WOLEngine},
};

#[derive(Debug)]
pub struct CommandController {
    directory: PathBuf,
    write_fn: fn(&PathBuf, &[u8]) -> Result<()>,
    wol: RefCell<WOLEngine>,
}

impl CommandController {
    pub fn new(directory: PathBuf) -> Result<Self> {
        let wol = RefCell::new(
            WOLEngine::new(directory.clone().join("wol"))
                .with_context(|| "Error when setting up WOL (Write Ahead Log)")?,
        );

        Ok(Self {
            directory,
            write_fn: |p, b| bwrite2(p.to_path_buf(), b),
            wol,
        })
    }

    pub fn set(&self, key: &str, value: &[u8]) -> Result<()> {
        let path = self.directory.join(key);
        self.wol.borrow_mut()
            .write_event(wol::WriteEvent::Write(key.to_owned(), value.to_vec()))
            .with_context(|| "Could not write to WOL")?;

        (self.write_fn)(&path, value)
    }

    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let path = self.directory.join(key);

        match read(path).with_context(|| format!("Failed to read key: {}", key)) {
            Ok(data) => Ok(Some(data)),
            Err(e) => {
                if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                    if io_err.kind() == std::io::ErrorKind::NotFound {
                        Ok(None) // If the error is NotFound, we return None
                    } else {
                        Err(e)
                    }
                } else {
                    Err(e)
                }
            }
        }
    }

    pub fn delete(&self, key: &str) -> Result<()> {
        let path = self.directory.join(key);
        delete(path).with_context(|| format!("Failed to delete key: {}", key))
    }
}
