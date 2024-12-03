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

/// Moves the contents of `src_dir` into `dest_dir`, creating latter if it doesn't exist yet.
///
/// - Attempts to do it via cheap renaming and falls back to recursive copying.
/// - Overwrites contents within `dest_dir`!
pub fn move_dir_contents(
    src_dir: impl AsRef<Path>,
    dest_dir: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let src_dir = src_dir.as_ref();
    let dest_dir = dest_dir.as_ref();
    fs::create_dir_all(dest_dir)?;
    for src_entry in fs::read_dir(src_dir)? {
        let src_entry = src_entry?;
        let src_entry_path = src_entry.path();
        let dest_entry_path = dest_dir.join(src_entry.file_name());
        if fs::rename(&src_entry_path, &dest_entry_path).is_err() {
            // Renaming didn't work. Fall back to copying.
            let ty = src_entry.file_type()?;
            if ty.is_dir() {
                copy_dir_recursively(&src_entry_path, &dest_entry_path)?;
            } else {
                fs::copy(&src_entry_path, &dest_entry_path)?;
            }
        }
    }
    Ok(())
}

/// Copies the contents of directory `src_dir` into directory `dest_dir`, creating latter if it doesn't exist yet.
///
/// Overwrites!
pub fn copy_dir_recursively(
    src_dir: impl AsRef<Path>,
    dest_dir: impl AsRef<Path>,
) -> io::Result<()> {
    let src_dir = src_dir.as_ref();
    let dest_dir = dest_dir.as_ref();
    fs::create_dir_all(dest_dir)?;
    for src_entry in fs::read_dir(src_dir)? {
        let src_entry = src_entry?;
        let src_entry_path = src_entry.path();
        let dest_entry_path = dest_dir.join(src_entry.file_name());
        let ty = src_entry.file_type()?;
        if ty.is_dir() {
            copy_dir_recursively(src_entry_path, dest_entry_path)?;
        } else {
            fs::copy(src_entry_path, dest_entry_path)?;
        }
    }
    Ok(())
}

/// Moves `src_file` to `dest_file`, creating a backup file of `dest_file` if it already exists,
/// before overwriting it.
///
/// It's okay if the provided backup directory doesn't exist yet.
pub fn move_file_overwriting_with_backup(
    src_file: impl AsRef<Path>,
    dest_file: impl AsRef<Path>,
    backup_dir: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let dest_file = dest_file.as_ref();
    let backup_dir = backup_dir.as_ref();
    if dest_file.exists() {
        let dest_file_name = dest_file
            .file_name()
            .context("destination path has no file name")?;
        let backup_file = backup_dir.join(dest_file_name);
        fs::create_dir_all(backup_dir).context("couldn't create backup file directory")?;
        fs::rename(dest_file, backup_file)?;
    }
    move_file(src_file, dest_file, true)
}

/// Moves `src_file` to `dest_file`.
///
/// Tries to use cheap renaming and falls back to copying.
pub fn move_file(
    src_file: impl AsRef<Path>,
    dest_file: impl AsRef<Path>,
    overwrite: bool,
) -> anyhow::Result<()> {
    let dest_file = dest_file.as_ref();
    if !overwrite && dest_file.exists() {
        bail!("Destination file {dest_file:?} already exists");
    } else {
        create_parent_dirs(dest_file)?;
    }
    if fs::rename(&src_file, dest_file).is_err() {
        fs::copy(src_file, dest_file).context("Copying file to destination failed")?;
    }
    Ok(())
}

/// Creates all parent directories of `path` if they don't exist already.
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
