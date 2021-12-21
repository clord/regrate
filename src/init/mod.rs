use clap::{ArgEnum, Args, ValueHint};
use rust_embed::RustEmbed;
use std::path::PathBuf;

#[derive(RustEmbed)]
#[folder = "assets/templates"]
struct Template;

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
    #[clap(short, long)]
    no_template: bool,

    /// place to insert migration directory (default $PWD)
    #[clap(short, long, value_hint = ValueHint::DirPath)]
    path: Option<PathBuf>,
}

pub fn init_repo(args: InitArgs) {
    // Folder structure:
    //  - regrate/store/<name>/{up,down}.sh  (but nothing here yet)
    //  - regrate/template/{up,down}.sh  (copy of system template)
    //  - regrate/current/{up,down}.sh (copy of template)
    //  - regrate/config ?

    for file in Template::iter() {
        if let Some(up_script) = Template::get(file.as_ref()) {
            // todo: Write file to location ./regrate/template if it's matching our selected
            // template
            println!("{:?}", std::str::from_utf8(up_script.data.as_ref()));
        } else {
            println!("NONE: {:?}", file);
        }
    }
}
