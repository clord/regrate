use clap::{ArgEnum, Args, ValueHint};
use eyre::{eyre, Result, WrapErr};
use rust_embed::RustEmbed;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[derive(RustEmbed)]
#[folder = "assets/templates/shell"]
struct ShellTemplate;

#[derive(RustEmbed)]
#[folder = "assets/templates/postgres"]
struct PostgresTemplate;

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

    /// Force removal of existing migration (danger!)
    #[clap(short, long)]
    force: bool,

    /// place to insert migration directory (default $PWD)
    #[clap(short, long, value_hint = ValueHint::DirPath)]
    path: Option<PathBuf>,
}

pub fn init_repo(args: InitArgs) -> Result<()> {
    let old = std::env::current_dir()?;

    let mut dest = PathBuf::new();
    dest.push("regrate");
    dest.push("template");

    let res = {
        if let Some(path) = args.path {
            std::env::set_current_dir(&path)
                .wrap_err_with(|| format!("Failed to change to path {:?}", path))?;
        }

        if args.force {
            fs::remove_dir_all("regrate")?;
        }

        fs::create_dir("regrate").wrap_err("Failed to create regrate directory")?;
        fs::create_dir("regrate/store").wrap_err("failed to create regrate/store")?;
        fs::create_dir("regrate/template").wrap_err("failed to create regrate/template")?;
        fs::create_dir("regrate/current").wrap_err("failed to create regreate/current")?;

        if !args.no_template {
            match args.which {
                InitType::Shell => {
                    for file in ShellTemplate::iter() {
                        let script = ShellTemplate::get(&file)
                            .ok_or_else(|| eyre!("Failed to load template {:?}", file))?;
                        write_file(&script.data, &file, dest.clone())?;
                    }
                }

                InitType::Mysql => {
                    for file in MysqlTemplate::iter() {
                        let script = MysqlTemplate::get(&file)
                            .ok_or_else(|| eyre!("Failed to load template {:?}", file))?;
                        write_file(&script.data, &file, dest.clone())?;
                    }
                }

                InitType::Postgres => {
                    for file in PostgresTemplate::iter() {
                        let script = PostgresTemplate::get(file.as_ref())
                            .ok_or_else(|| eyre!("Failed to load template {:?}", file))?;
                        write_file(&script.data, &file, dest.clone())?;
                    }
                }
            };
        }

        Ok(())
    };

    std::env::set_current_dir(old)?;
    res
}

/// Write file from source to destination
fn write_file(source: &[u8], dest_path: &str, mut dest_folder: PathBuf) -> Result<()> {
    let contents = std::str::from_utf8(source)?;

    let filename = Path::new(dest_path)
        .file_name()
        .ok_or_else(|| eyre!("Could not get filename"))?;
    dest_folder.push(filename);

    fs::write(dest_folder.as_path(), contents)?;
    Ok(())
}
