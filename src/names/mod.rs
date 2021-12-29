use crate::utils::regrate_path;
use eyre::{Report, Result};
use walkdir::WalkDir;

use sha2::{Digest, Sha256};
use std::{fs, io};

// Iterates the store
#[derive(Debug)]
pub struct StoreNameIterator {
    pub seed: String,
    pub seed_exists: bool,
}

impl<'a> fallible_iterator::FallibleIterator for StoreNameIterator {
    type Item = (String, std::path::PathBuf, bool);
    type Error = Report;

    fn next(&mut self) -> Result<Option<Self::Item>> {
        let seed = &self.seed;
        let mut root = std::path::PathBuf::from(regrate_path("store")?);

        // encode name as two path components: first two characters and rest.
        root.push(&seed[0..2]);
        root.push(&seed[2..]);

        if !self.seed_exists {
            return Ok(None);
        }

        if root.exists() && root.is_dir() {
            self.seed_exists = true;
            let mut hasher = Sha256::new();
            hasher.update(seed);
            // iterate all files in the directory, updating hasher.
            for entry in WalkDir::new(&root) {
                let entry = entry?;
                let path = entry.path();
                if !path.is_dir() {
                    let mut file = fs::File::open(entry.path())?;
                    io::copy(&mut file, &mut hasher)?;
                }
            }
            let hash_bytes = hasher.finalize();
            let name = bs58::encode(hash_bytes).into_string();
            self.seed = name;

            Ok(Some((self.seed.clone(), root, true)))
        } else {
            self.seed_exists = false;
            Ok(Some((self.seed.clone(), root, false)))
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
        StoreNameIterator {
            seed,
            seed_exists: true,
        }
    }
}
