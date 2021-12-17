use clap::{ArgEnum, Args, IntoApp, Parser};
use clap_generate::{generate, Shell};

#[derive(Parser, Debug)]
#[clap(name = "regrate")]
#[clap(bin_name = "regrate")]
enum Regrate {
    /// Init a new migration
    Init(InitArgs),

    /// Run migrations in order
    Run(RunArgs),

    /// Commit change to migration
    Commit(CommitArgs),

    /// Generate completions for your shell
    Generate(GenerateArgs),

    /// Resolve conflict markers into new migration
    Resolve,
}

#[derive(Args, Debug)]
#[clap(about, author, version)]
struct GenerateArgs {
    /// output completion script
    #[clap(arg_enum)]
    shell: Shell,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum InitType {
    Shell,
    Sql,
}

#[derive(Args, Debug)]
#[clap(about, author, version)]
struct CommitArgs {
    /// Message to pass
    #[clap(short, long)]
    message: String,
}

#[derive(Args, Debug)]
#[clap(about, author, version)]
struct RunArgs {
    /// What command to execute ({} for script)
    command: String,
}

#[derive(Args, Debug)]
#[clap(about, author, version)]
struct InitArgs {
    /// Type of migration
    #[clap(arg_enum)]
    which: InitType,
}

fn main() {
    match Regrate::parse() {
        Regrate::Init(args) => {
            println!("INIT {:?}", args.which);
        }
        Regrate::Commit(args) => {
            println!("COMMIT {:?}", args.message);
        }
        Regrate::Run(args) => {
            println!("RUN {:?}", args.command);
        }
        Regrate::Resolve => {
            println!("RESOLVE");
        }
        Regrate::Generate(args) => {
            let generator = args.shell;
            let mut app = Regrate::into_app();
            eprintln!("Generating completion file for {:?}...", generator);
            let name = app.get_name().to_string();
            generate(generator, &mut app, name, &mut std::io::stdout());
        }
    }
}
