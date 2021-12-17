use clap::{Parser, Args, ArgEnum};

#[derive(Parser)]
#[clap(name="regrate")]
#[clap(bin_name="regrate")]
enum Regrate {
    /// Init a new migration
    Init(InitArgs),
    /// Run migrations in order
    Run(RunArgs),
    /// Commit change to migration
    Commit(CommitArgs),
    /// Resolve conflict markers into new migration
    Resolve,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum InitType {
    Shell,
    SQL,
}

#[derive(Args)]
#[clap(about, author, version)]
struct CommitArgs {
    /// Message to pass
    #[clap(short, long)]
    message: String
}

#[derive(Args)]
#[clap(about, author, version)]
struct RunArgs {
    /// What command to execute ({} for script)
    command: String
}

#[derive(Args)]
#[clap(about, author, version)]
struct InitArgs {
    /// Type of migration
    #[clap(arg_enum)]
    which: InitType,
}

fn main() {
    match Regrate::parse() {
        Regrate::Init(args) => {
            println!("init: {:?}", args.which);
        }
        Regrate::Commit(args) => {
            println!("commit: {:?}", args.message);
        }
        Regrate::Run(args) => {
            println!("RUN {:?}", args.command);
        }
        Regrate::Resolve => {
            println!("Resolve");
        }
    }
}
