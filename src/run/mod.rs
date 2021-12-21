use clap::{Args, ValueHint};
use eyre::Result;

#[derive(Args, Debug)]
#[clap(about, author, version)]
pub struct RunArgs {
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

pub fn run_migrations(args: RunArgs) -> Result<()> {
    //  - Compute the sequence of names
    //  - execute command on each name in sequence, passing name in place of first `{}` and
    //    setting REGRATE_VERSION environment variable.
    //  - Does not execute 'current' by default (use --current options for that)
    println!(
        "RUN {:?} {:?} {:?}",
        args.command, args.replace, args.current
    );
    Ok(())
}
