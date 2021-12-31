use crate::names::StoreNameIterator;
use crate::utils::exists_in_regrate;
use crate::utils::regrate_path;
use crate::utils::require_regrate_inited;
use clap::Args;
use color_eyre::Help;
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
    if !exists_in_regrate("current")? {
        return Err(eyre!("No current migration, can not commit.")
            .with_suggestion(|| "use `regrate create` to start a migration"));
    }
    if let Some((_name, next, _path, next_path)) = StoreNameIterator::new().last()? {
        let parent = next_path
            .parent()
            .ok_or_else(|| eyre!("Could not get parent path"))?;
        std::fs::create_dir_all(parent)?;
        let current = regrate_path("current")?;

        let info = Info {
            message: args.message,
            name: next,
        };

        let toml = toml::to_string_pretty(&info).wrap_err("generating info.toml")?;
        std::fs::write(current.join("info.toml"), toml).wrap_err("writing info.toml")?;

        // move current to the new name
        println!("moving {:?} -> {:?}", current, next_path);
        println!("use `regrate create` to start a new migration");
        std::fs::rename(current, next_path).wrap_err("renaming current to path")?;
    }

    Ok(())
}
