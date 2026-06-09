use crate::names;
use crate::utils::{exists_in_regrate, regrate_path, require_regrate_inited};
use clap::Args;
use color_eyre::Help;
use eyre::{eyre, Context, Result};
use serde::Serialize;

#[derive(Args, Debug)]
pub struct CommitArgs {
    /// Message describing the migration (stored in its info.toml)
    #[arg(short, long)]
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

    let (migrations, next_name, next_path) = names::chain()?;

    let orphans = names::orphans(&migrations)?;
    if !orphans.is_empty() {
        return Err(eyre!(
            "store contains migrations not reachable from the name chain:\n  {}",
            orphans.join("\n  ")
        )
        .with_note(|| "committing on a broken chain would orphan this migration too")
        .with_suggestion(|| "run `regrate valid` for details"));
    }

    let parent = next_path
        .parent()
        .ok_or_else(|| eyre!("Could not get parent path"))?;
    std::fs::create_dir_all(parent)?;
    let current = regrate_path("current")?;

    let info = Info {
        message: args.message,
        name: next_name,
    };

    let toml = toml::to_string_pretty(&info).wrap_err("generating info.toml")?;
    std::fs::write(current.join("info.toml"), toml).wrap_err("writing info.toml")?;

    // move current to the new name
    println!("moving {:?} -> {:?}", current, next_path);
    println!("use `regrate create` to start a new migration");
    std::fs::rename(current, next_path).wrap_err("renaming current to path")?;

    Ok(())
}
