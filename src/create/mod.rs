use crate::utils::{exists_in_regrate, require_regrate_inited};
use clap::{Args, ValueHint};
use eyre::{eyre, Result};

#[derive(Args, Debug)]
#[clap(about, author, version)]
pub struct CreateArgs {
    /// Migrate to the 'current' migration but don't revert (for dev)
    #[clap(short, long)]
    current: bool,

    /// What command to execute migrations
    #[clap(short = 'x', long, value_hint = ValueHint::CommandString)]
    command: Option<String>,

    /// where to put the version in command invocation
    #[clap(short, long, value_hint = ValueHint::Other, default_value = "{}")]
    replace: String,
}

pub fn do_create(args: CreateArgs) -> Result<()> {
    require_regrate_inited()?;
    if exists_in_regrate("current")? {
        return Err(eyre!(
            "Migration already exists, commit or abandon and try again"
        ));
    }
    println!(
        "CREATE {:?} {:?} {:?}",
        args.command, args.replace, args.current
    );
    Ok(())
}
