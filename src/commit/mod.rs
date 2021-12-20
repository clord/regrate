use clap::Args;

#[derive(Args, Debug)]
#[clap(about, author, version)]
pub struct CommitArgs {
    /// Message to pass
    #[clap(short, long)]
    message: String,
}

pub fn commit_current(args: CommitArgs) {
    //  - Compute new name.
    //  - move current to the new name (regrates/store/<name>/{up,down}.sh)
    //  - create a new 'current' from template (or empty)
    println!("COMMIT {:?}", args.message);
}
