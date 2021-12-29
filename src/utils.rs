use eyre::{eyre, Result};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::{env, fs::OpenOptions, io::Write, path};

/// Checks if a given path is a valid regrate directory
pub fn exists_in_regrate(file_name: &str) -> Result<bool> {
    let path = regrate_path(file_name)?;
    Ok(path.exists())
}

/// error unless regrate is initialized
pub fn require_regrate_inited() -> Result<()> {
    if !exists_in_regrate("store")? {
        return Err(eyre!(
            "regrate is not initialized correctly in current directory"
        ));
    }
    Ok(())
}

pub fn regrate_root() -> Result<path::PathBuf> {
    let mut path = env::current_dir()?;
    path.push("regrate");
    Ok(path)
}

/// Verify regreate is set up then pass regrate sub_path (which is not checked)
pub fn regrate_path(sub_path: &str) -> Result<path::PathBuf> {
    let mut path = regrate_root()?;
    path.push(sub_path);
    Ok(path)
}

// Write bytes to destination file in destination folder, with optional executable flag
pub fn write_file(
    contents: &[u8],
    dest_path: &str,
    dest_folder: &Path,
    is_exe: bool,
) -> Result<()> {
    let mut dest_file = PathBuf::from(dest_folder);

    let filename = Path::new(dest_path)
        .file_name()
        .ok_or_else(|| eyre!("Could not get filename of {}", dest_path))?;

    dest_file.push(filename);

    let mut opts = OpenOptions::new();
    let mut opts = opts.create(true).write(true);
    if is_exe {
        opts = opts.mode(0o755);
    }

    let mut f = opts.open(dest_file.as_path())?;
    f.write_all(contents)?;

    Ok(())
}
