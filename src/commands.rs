use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::persist::{del::delete, read::read, write::bwrite2};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandController {
    directory: PathBuf,
    write_fn: fn(&PathBuf, &[u8]) -> Result<()>,
}

impl CommandController {
    pub fn new(directory: PathBuf) -> Self {
        Self {
            directory,
            write_fn: |p, b| bwrite2(p.to_path_buf(), b),
        }
    }

    pub fn set(&self, key: &str, value: &[u8]) -> Result<()> {
        let path = self.directory.join(key);
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
