use crate::types::InitType;
use crate::types::RepoConfig;
use crate::utils::exists_in_regrate;
use crate::utils::regrate_path;
use crate::utils::regrate_root;
use crate::utils::write_file;
use clap::{Args, ValueHint};
use color_eyre::Help;
use eyre::{eyre, Result, WrapErr};
use rust_embed::RustEmbed;
use std::fs;
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

    let res = {
        if let Some(path) = args.path {
            std::env::set_current_dir(&path)
                .wrap_err_with(|| format!("Failed to change to path {:?}", path))?;
        }

        if args.force {
            fs::remove_dir_all("regrate")?;
        } else if exists_in_regrate("store")? {
            return Err(
                eyre!("Regrate is already set up; aborting init").with_suggestion(|| {
                    "consider --force if you'd like to remove existing migrations"
                }),
            );
        }

        fs::create_dir("regrate").wrap_err("Failed to create regrate directory")?;
        fs::create_dir("regrate/store").wrap_err("failed to create regrate/store")?;
        fs::create_dir("regrate/template").wrap_err("failed to create regrate/template")?;

        let config = RepoConfig { mode: args.which };
        let toml = toml::to_string_pretty(&config).wrap_err("generate repo.toml")?;
        std::fs::write(regrate_root()?.join("repo.toml"), toml).wrap_err("write repo.toml")?;

        if !args.no_template {
            let dest = regrate_path("template")?;
            match args.which {
                InitType::Shell => {
                    for file in ShellTemplate::iter() {
                        let script = ShellTemplate::get(&file)
                            .ok_or_else(|| eyre!("Failed to load template {:?}", file))?;
                        write_file(&script.data, &file, &dest, true)?;
                    }
                }

                InitType::Mysql => {
                    for file in MysqlTemplate::iter() {
                        let script = MysqlTemplate::get(&file)
                            .ok_or_else(|| eyre!("Failed to load template {:?}", file))?;
                        write_file(&script.data, &file, &dest, false)?;
                    }
                }

                InitType::Postgres => {
                    for file in PostgresTemplate::iter() {
                        let script = PostgresTemplate::get(file.as_ref())
                            .ok_or_else(|| eyre!("Failed to load template {:?}", file))?;
                        write_file(&script.data, &file, &dest, false)?;
                    }
                }
            };
        }

        Ok(())
    };

    std::env::set_current_dir(old)?;
    res
}
