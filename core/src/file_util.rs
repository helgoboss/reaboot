use anyhow::{bail, ensure, Context};
use std::fs::OpenOptions;
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

/// Doesn't overwrite
pub fn move_file_or_dir_all(src: impl AsRef<Path>, dest: impl AsRef<Path>) -> anyhow::Result<()> {
    let src = src.as_ref();
    if src.is_file() {
        move_file(src, &dest, false)?;
    } else {
        move_dir_all(src, dest)?;
    }
    Ok(())
}

/// Doesn't overwrite
pub fn move_dir_all(src: impl AsRef<Path>, dest: impl AsRef<Path>) -> anyhow::Result<()> {
    let dest = dest.as_ref();
    if dest.exists() {
        bail!("Destination {dest:?} already exists");
    }
    if fs::rename(&src, &dest).is_err() {
        copy_dir_all(src, dest)?;
    }
    Ok(())
}

/// Overwrites!
pub fn copy_dir_all(src: impl AsRef<Path>, dest: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dest.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dest.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn move_file_overwriting_with_backup(
    src_file: impl AsRef<Path>,
    dest_file: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let dest_file = dest_file.as_ref();
    if dest_file.exists() {
        let dest_file_name = dest_file
            .file_name()
            .context("destination path has no file name")?
            .to_str()
            .context("destination file name is not valid UTF-8")?;
        let backup_file = dest_file.with_file_name(format!("{dest_file_name}.bak"));
        fs::rename(&dest_file, &backup_file)?;
    }
    move_file(src_file, dest_file, true)
}

pub fn move_file(
    src_file: impl AsRef<Path>,
    dest_file: impl AsRef<Path>,
    overwrite: bool,
) -> anyhow::Result<()> {
    let dest_file = dest_file.as_ref();
    if !overwrite && dest_file.exists() {
        bail!("Destination file {dest_file:?} already exists");
    } else {
        create_parent_dirs(&dest_file)?;
    }
    if fs::rename(&src_file, &dest_file).is_err() {
        fs::copy(src_file, dest_file).context("Copying file to destination failed")?;
    }
    Ok(())
}

pub fn create_parent_dirs(path: impl AsRef<Path>) -> anyhow::Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

/// Returns whether the file or directory path is writable or at least creatable.
///
/// This is for early return purposes. Of course, it can happen that a path that was reported
/// as writable now is not writable later.
pub fn file_or_dir_is_writable_or_creatable(path: &Path) -> bool {
    match path.try_exists() {
        Ok(true) => existing_file_or_dir_is_writable(path),
        Ok(false) => match get_first_existing_parent_dir(path.to_path_buf()) {
            Ok(d) => existing_file_or_dir_is_writable(&d),
            Err(_) => false,
        },
        Err(_) => false,
    }
}

/// Returns whether the file or directory path is writable or at least creatable.
///
/// This is for early return purposes. Of course, it can happen that a path that was reported
/// as writable now is not writable later.
pub fn existing_file_or_dir_is_writable(path: &Path) -> bool {
    match fs::metadata(path) {
        Ok(md) => {
            // If the path is marked as read-only, we can know early.
            if md.permissions().readonly() {
                return false;
            }
            // Not read-only, but we can still have a permission issues.
            if md.is_dir() {
                // It's a directory. Attempt to write a temporary dir.
                tempdir::TempDir::new_in(path, "reaboot-check-").is_ok()
            } else {
                // It's a file. Attempt to open the file in write mode
                OpenOptions::new().write(true).open(path).is_ok()
            }
        }
        Err(_) => {
            // Permission issues
            false
        }
    }
}
