mod commit;
mod errors;
mod gen;
mod init;
mod resolve;
mod run;

use clap::Parser;
use color_eyre::eyre::Result;
use eyre::WrapErr;

#[derive(Parser, Debug)]
#[clap(name = "regrate", about, version)]
#[clap(author = "Christopher C Lord")]
#[clap(bin_name = "regrate")]
enum Regrate {
    /// Init a new migration
    Init(init::InitArgs),

    /// Commit change to migration
    Commit(commit::CommitArgs),

    /// Run migrations in order
    Run(run::RunArgs),

    /// Generate completions for your shell
    Generate(gen::GenerateArgs),

    /// Resolve conflict markers into new migration
    Resolve,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    match Regrate::parse() {
        Regrate::Init(args) => init::init_repo(args).wrap_err("initializing new repo"),
        Regrate::Commit(args) => commit::commit_current(args).wrap_err("committing changes"),
        Regrate::Run(args) => run::run_migrations(args).wrap_err("running migration"),
        Regrate::Resolve => resolve::resolve_conflicts().wrap_err("resolving conflicts"),
        Regrate::Generate(args) => {
            gen::generate_completion(args).wrap_err("generating shell autocomplete files")
        }
    }?;

    Ok(())
}
