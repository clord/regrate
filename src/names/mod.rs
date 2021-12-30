use crate::utils::regrate_path;
use eyre::{Report, Result};
use walkdir::WalkDir;

use sha2::{Digest, Sha256};
use std::{fs, io, mem};

// Iterates the store
#[derive(Debug)]
pub struct StoreNameIterator {
    pub seed: String,
}

impl<'a> fallible_iterator::FallibleIterator for StoreNameIterator {
    type Item = (String, String, std::path::PathBuf, std::path::PathBuf);
    type Error = Report;

    fn next(&mut self) -> Result<Option<Self::Item>> {
        let mut current_path = regrate_path("store")?;
        let mut next_path = current_path.clone();

        // encode name as two path components: first two characters and rest.
        current_path.push(&self.seed[0..2]);
        current_path.push(&self.seed[2..]);

        if current_path.exists() && current_path.is_dir() {
            let mut hasher = Sha256::new();
            hasher.update(&self.seed);

            // iterate all files in the directory, updating hasher.
            for entry in WalkDir::new(&current_path) {
                let entry = entry?;
                let path = entry.path();
                if !path.is_dir() {
                    let mut file = fs::File::open(entry.path())?;
                    io::copy(&mut file, &mut hasher)?;
                }
            }
            let hash_bytes = hasher.finalize();
            let mut name = bs58::encode(hash_bytes).into_string();

            next_path.push(&name[0..2]);
            next_path.push(&name[2..]);

            mem::swap(&mut name, &mut self.seed);

            Ok(Some((name, self.seed.clone(), current_path, next_path)))
        } else {
            Ok(None)
        }
    }
}

impl StoreNameIterator {
    pub fn new() -> StoreNameIterator {
        let mut v1hasher = Sha256::new();
        v1hasher.update("#REGRATE v1 \npi: 3.141592653589793238462643383279502884197169399375105820974944592307816406286");
        let seed = bs58::encode(v1hasher.finalize()).into_string();
        // Seed exists defaults to true, as if the "first" element is based on some unknown
        // previous that exists.
        StoreNameIterator { seed }
    }
}
