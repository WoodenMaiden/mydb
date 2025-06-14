use anyhow::Context;

use crate::persist::fsync_parent_dir;

pub fn delete(path: std::path::PathBuf) -> anyhow::Result<()> {
    std::fs::remove_file(&path).with_context(|| {
        anyhow::anyhow!(format!("Failed to delete file at path: {:?}", &path))
    })?;

    fsync_parent_dir(path)
}