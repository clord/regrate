use crate::names::StoreNameIterator;
use crate::utils::regrate_path;
use crate::utils::require_regrate_inited;
use clap::Args;
use eyre::{eyre, Context, Result};
use fallible_iterator::FallibleIterator;
use serde::Serialize;

#[derive(Args, Debug)]
#[clap(about, author, version)]
pub struct CommitArgs {
    /// Message to pass (used to describe in comment?)
    #[clap(short, long)]
    message: String,
}

#[derive(Serialize)]
struct Info {
    name: String,
    message: String,
}

pub fn commit_current(args: CommitArgs) -> Result<()> {
    require_regrate_inited()?;
    if let Some((name, path, _)) = StoreNameIterator::new().last()? {
        let parent = path.parent().ok_or(eyre!("Could not get parent path"))?;
        std::fs::create_dir_all(parent)?;
        let current = regrate_path("current")?;

        let info = Info {
            message: args.message,
            name,
        };

        let toml = toml::to_string(&info).wrap_err("generating info.toml")?;
        std::fs::write(current.join("info.toml"), toml).wrap_err("writing info.toml")?;

        // move current to the new name
        println!("moving {:?} -> {:?}", current, path);
        std::fs::rename(current, path).wrap_err("renaming current to path")?;
    }

    Ok(())
}
