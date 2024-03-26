use anyhow::{ensure, Context};
use std::path::PathBuf;

/// Passed path must be absolute.
pub fn get_first_existing_parent_dir(path: PathBuf) -> anyhow::Result<PathBuf> {
    let first_existing_parent = find_first_existing_parent(path.clone()).with_context(|| {
        format!("No parent of destination file {path:?} exists. Shouldn't happen.")
    })?;
    ensure!(
        first_existing_parent.is_dir(),
        "Parent of destination file is not a directory"
    );
    Ok(first_existing_parent)
}

/// Passed path must be absolute.
pub fn find_first_existing_parent(mut path: PathBuf) -> Option<PathBuf> {
    assert!(path.is_absolute(), "passed path must be absolute");
    loop {
        if !path.pop() {
            return None;
        }
        if path.exists() {
            return Some(path);
        }
    }
}
