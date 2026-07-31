use std::{fs, path::Path};

use anyhow::{Context, Result};

pub(super) fn write_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed creating {}", parent.display()))?;
    }
    fs::write(path, bytes).with_context(|| format!("failed writing {}", path.display()))?;
    Ok(())
}
