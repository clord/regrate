use clap::{ArgEnum, Args, IntoApp, Parser, ValueHint};
use clap_generate::{generate, Shell};

#[derive(Parser, Debug)]
#[clap(name = "regrate", about, version)]
#[clap(author = "Christopher C Lord")]
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

    /// Migrate to the 'current' migration but don't revert (for dev)
    #[clap(short, long)]
    current: bool,

    /// What command to execute migrations
    #[clap(short = 'x', long, value_hint = ValueHint::CommandString)]
    command: Option<String>,

    /// where to put the version in command invocation
    #[clap(short, long, value_hint = ValueHint::Other, default_value = "{}")]
    replace: String
}

#[derive(Args, Debug)]
#[clap(about, author, version)]
struct InitArgs {
    /// Type of migration
    #[clap(arg_enum)]
    which: InitType,

    /// Do not generate a default template for this filetype
    #[clap(short = 'n', long)]
    no_template: bool,

    /// Override the source for template to given directory (will be copied)
    #[clap(short = 't', long, value_hint = ValueHint::DirPath)]
    template: Option<String>
}

fn main() {
    match Regrate::parse() {
        Regrate::Init(args) => {
            println!("INIT {:?}", args.which);
            // Folder structure:
            //  - regrates/store/<name>/{up,down}.sh
            //  - regrates/initial -> <name>
            //  - regrates/template/{up,down}.sh
            //  - regrates/current/{up,down}.sh
        }

        Regrate::Commit(args) => {
            //  - Compute new name.
            //  - move current to the new name (regrates/store/<name>/{up,down}.sh)
            //  - create a new 'current' from template (or empty)
            println!("COMMIT {:?}", args.message);
        }

        Regrate::Run(args) => {
            //  - Compute the sequence of names
            //  - execute command on each name in sequence, passing name in place of first `{}` and
            //    setting REGRATE_VERSION environment variable.
            //  - Does not execute 'current' by default (use --current options for that)
            println!("RUN {:?} {:?} {:?}", args.command, args.replace, args.current);
        }

        Regrate::Resolve => {
            //  - in a temp directory,
            //  - create a new migration populated with my old 'current' migration
            //  - revert merge conflict on new current
            //  - commit and make "current"
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
