use std::path::Path;
use clap::{ArgEnum, Args, ValueHint};
use eyre::{eyre, Result, WrapErr};
use rust_embed::RustEmbed;
use std::fs;
use std::path::PathBuf;

#[derive(RustEmbed)]
#[folder = "assets/templates/shell"]
struct ShellTemplate;

#[derive(RustEmbed)]
#[folder = "assets/templates/mysql"]
struct MysqlTemplate;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
pub enum InitType {
    Shell,
    Postgres,
    Mysql,
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

fn write_file(file: &str, contents:&str) -> Result<()> {
    println!("{:?}: {:?}", file, contents);
    Ok(())
}

pub fn init_repo(args: InitArgs) -> Result<()> {
    // Folder structure:
    //  - regrate/template/{up,down}.sh  (copy of system template)
    //  - regrate/current/{up,down}.sh (copy of template)
    //  - regrate/config ?

    let old = std::env::current_dir()?;

    let res = {
        if let Some(path) = args.path {
            std::env::set_current_dir(&path)
                .wrap_err_with(|| format!("Failed to change to path {:?}", path))?;
        }

        fs::create_dir("regrate").wrap_err("Failed to create regrate directory")?;
        fs::create_dir("regrate/store").wrap_err("failed to create regrate/store")?;
        fs::create_dir("regrate/template").wrap_err("failed to create regrate/template")?;
        fs::create_dir("regrate/current").wrap_err("failed to create regreate/current")?;

        if !args.no_template {
            match args.which {
                InitType::Shell => {
                    for file in ShellTemplate::iter() {
                        let script = ShellTemplate::get(file.as_ref())
                            .ok_or_else(|| eyre!("Failed to load shell template {:?}", file))?;
                        let contents = std::str::from_utf8(script.data.as_ref())?;
                        write_file(file.as_ref(), contents)?;
                    }
                },
                InitType::Mysql => {
                    for file in MysqlTemplate::iter() {
                        let script = MysqlTemplate::get(file.as_ref())
                            .ok_or_else(|| eyre!("Failed to load mysql template {:?}", file))?;
                        let contents = std::str::from_utf8(script.data.as_ref())?;
                        write_file(file.as_ref(), contents)?;
                    }
                },
                InitType::Postgres => {},
            };
        }
        Ok(())
    };

    std::env::set_current_dir(old)?;
    return res;
}
