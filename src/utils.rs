use eyre::{eyre, Result};
use std::path::{PathBuf,Path};

/// Checks if a given path is a valid regrate path (i.e., )
pub fn exists_in_regrate(file_name: &str) -> Result<bool> {
    let path = regrate_path(file_name)?;
    let metadata = std::fs::metadata(path)?;
    Ok(metadata.is_dir())
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

pub fn regrate_root() -> Result<std::path::PathBuf> {
    let mut path = std::env::current_dir()?;
    path.push("regrate");
    Ok(path)
}

/// Verify regreate is set up then pass regrate sub_path (which is not checked)
pub fn regrate_path(sub_path: &str) -> Result<std::path::PathBuf> {
    let mut path = regrate_root()?;
    path.push(sub_path);
    Ok(path)
}

/// Write file from source to destination
pub fn write_file(source: &[u8], dest_path: &str, dest_folder: &Path) -> Result<()> {
    let mut dest_file = PathBuf::from(dest_folder);

    let contents = std::str::from_utf8(source)?;

    let filename = Path::new(dest_path)
        .file_name()
        .ok_or_else(|| eyre!("Could not get filename"))?;

    dest_file.push(filename);

    std::fs::write(dest_file.as_path(), contents)?;
    Ok(())
}
