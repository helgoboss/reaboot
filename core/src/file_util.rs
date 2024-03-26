use anyhow::{ensure, Context};
use std::path::{Path, PathBuf};
use std::{fs, io};

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

pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
