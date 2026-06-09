use crate::utils::regrate_path;
use eyre::{Report, Result, WrapErr};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::{fs, io};
use walkdir::WalkDir;

/// A committed migration in the chain, in execution order.
#[derive(Debug, Clone)]
pub struct Migration {
    /// zero-based position in the chain
    pub index: usize,
    pub name: String,
    pub path: PathBuf,
    /// name the next migration will receive
    pub next_name: String,
    pub next_path: PathBuf,
}

/// Iterates committed migrations in the store, in order.
///
/// Each migration's name is derived from its predecessor: hash the previous
/// name together with the previous migration's files (sorted by path, with
/// relative file names and sizes mixed in, so the result does not depend on
/// filesystem iteration order).
#[derive(Debug)]
pub struct StoreNameIterator {
    seed: String,
    index: usize,
}

impl fallible_iterator::FallibleIterator for StoreNameIterator {
    type Item = Migration;
    type Error = Report;

    fn next(&mut self) -> Result<Option<Migration>> {
        let current_path = name_to_path(&self.seed)?;

        if !current_path.is_dir() {
            return Ok(None);
        }

        let next_name = hash_dir(&self.seed, &current_path)?;
        let next_path = name_to_path(&next_name)?;

        let migration = Migration {
            index: self.index,
            name: std::mem::replace(&mut self.seed, next_name.clone()),
            path: current_path,
            next_name,
            next_path,
        };
        self.index += 1;

        Ok(Some(migration))
    }
}

impl StoreNameIterator {
    pub fn new() -> StoreNameIterator {
        StoreNameIterator {
            seed: Self::first_hash(),
            index: 0,
        }
    }

    /// The name the next committed migration will receive, and where it will
    /// be stored. Only meaningful once the iterator is exhausted.
    pub fn pending(&self) -> Result<(String, PathBuf)> {
        let path = name_to_path(&self.seed)?;
        Ok((self.seed.clone(), path))
    }

    pub fn first_hash() -> String {
        let mut hasher = Sha256::new();
        hasher.update("#REGRATE v2 \npi: 3.141592653589793238462643383279502884197169399375105820974944592307816406286");
        bs58::encode(hasher.finalize()).into_string()
    }
}

/// Hash every file in a migration directory: relative path, size, contents.
/// Sorting makes the result independent of readdir order, and hashing the
/// path and size makes renames and content boundary shifts detectable.
fn hash_dir(seed: &str, dir: &PathBuf) -> Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(seed);

    for entry in WalkDir::new(dir).sort_by_file_name() {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            let relative = path.strip_prefix(dir)?;
            hasher.update(relative.to_string_lossy().as_bytes());
            hasher.update([0u8]);
            hasher.update(entry.metadata()?.len().to_le_bytes());
            let mut file =
                fs::File::open(path).wrap_err_with(|| format!("hashing {}", path.display()))?;
            io::copy(&mut file, &mut hasher)?;
        }
    }

    Ok(bs58::encode(hasher.finalize()).into_string())
}

/// Store path for a migration name: first two characters become a
/// subdirectory, the rest is the directory name within it.
pub fn name_to_path(name: &str) -> Result<PathBuf> {
    let mut path = regrate_path("store")?;
    path.push(&name[0..2]);
    path.push(&name[2..]);
    Ok(path)
}

/// Walk the whole chain of committed migrations, returning them in order
/// along with the name and path reserved for the next commit.
pub fn chain() -> Result<(Vec<Migration>, String, PathBuf)> {
    use fallible_iterator::FallibleIterator;
    let mut iter = StoreNameIterator::new();
    let mut migrations = Vec::new();
    while let Some(migration) = iter.next()? {
        migrations.push(migration);
    }
    let (pending_name, pending_path) = iter.pending()?;
    Ok((migrations, pending_name, pending_path))
}

/// Store directories that are not reachable by the name chain. A non-empty
/// result means a committed migration was edited (breaking every name after
/// it) or a merge left behind a migration that needs to be re-committed.
pub fn orphans(reachable: &[Migration]) -> Result<Vec<String>> {
    let known: BTreeSet<&str> = reachable.iter().map(|m| m.name.as_str()).collect();
    let store = regrate_path("store")?;
    let mut found = Vec::new();

    if !store.is_dir() {
        return Ok(found);
    }

    for prefix in fs::read_dir(&store)? {
        let prefix = prefix?;
        if !prefix.path().is_dir() {
            continue;
        }
        for rest in fs::read_dir(prefix.path())? {
            let rest = rest?;
            if !rest.path().is_dir() {
                continue;
            }
            let name = format!(
                "{}{}",
                prefix.file_name().to_string_lossy(),
                rest.file_name().to_string_lossy()
            );
            if !known.contains(name.as_str()) {
                found.push(name);
            }
        }
    }

    found.sort();
    Ok(found)
}
