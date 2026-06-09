use crate::types::InitType;
use crate::types::RepoConfig;
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
pub struct InitArgs {
    /// Type of migration
    #[arg(value_enum)]
    which: InitType,

    /// Do not generate a default template for this filetype
    #[arg(short, long)]
    no_template: bool,

    /// Force removal of existing migration (danger!)
    #[arg(short, long)]
    force: bool,

    /// place to insert migration directory (default $PWD)
    #[arg(short, long, value_hint = ValueHint::DirPath)]
    path: Option<PathBuf>,
}

pub fn init_repo(args: InitArgs) -> Result<()> {
    let base = match args.path {
        Some(path) => path,
        None => std::env::current_dir()?,
    };
    let root = base.join("regrate");

    if args.force {
        match fs::remove_dir_all(&root) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                return Err(e).wrap_err("removing existing regrate directory")
            }
            _ => {}
        }
    } else if root.join("store").exists() {
        return Err(eyre!("Regrate is already set up; aborting init")
            .with_suggestion(|| "consider --force if you'd like to remove existing migrations"));
    }

    fs::create_dir(&root).wrap_err("Failed to create regrate directory")?;
    fs::create_dir(root.join("store")).wrap_err("failed to create regrate/store")?;
    fs::create_dir(root.join("template")).wrap_err("failed to create regrate/template")?;

    let config = RepoConfig { mode: args.which };
    let toml = toml::to_string_pretty(&config).wrap_err("generate repo.toml")?;
    fs::write(root.join("repo.toml"), toml).wrap_err("write repo.toml")?;

    if !args.no_template {
        let dest = root.join("template");
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
}
