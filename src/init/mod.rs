use clap::{ArgEnum, Args, ValueHint};
use std::path::PathBuf;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
pub enum InitType {
    Shell,
    Sql,
}

#[derive(Args, Debug)]
#[clap(about, author, version)]
pub struct InitArgs {
    /// Type of migration
    #[clap(arg_enum)]
    which: InitType,

    /// Do not generate a default template for this filetype
    #[clap(short = 'n', long)]
    no_template: bool,

    /// Override the source for template to given directory (will be copied)
    #[clap(short = 't', long, value_hint = ValueHint::DirPath)]
    template: Option<String>,

    /// place to insert migration directory (default $PWD)
    #[clap(short = 'p', long, value_hint = ValueHint::DirPath)]
    path: Option<PathBuf>,
}

pub fn init_repo(args: InitArgs) {
    println!("INIT {:?}", args.which);
    // Folder structure:
    //  - regrate/store/<name>/{up,down}.sh  (but nothing here yet)
    //  - regrate/template/{up,down}.sh  (copy of system template)
    //  - regrate/current/{up,down}.sh (copy of template)
    //  - regrate/config ?
}
