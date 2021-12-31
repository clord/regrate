use crate::names::StoreNameIterator;
use crate::types::{InitType, RepoConfig};
use crate::utils::regrate_root;
use crate::utils::require_regrate_inited;
use clap::{AppSettings, Args, ValueHint};
use eyre::{eyre, Result, WrapErr};
use fallible_iterator::FallibleIterator;

use std::{env, path, process};

#[derive(Args, Debug)]
#[clap(about, author, version, setting = AppSettings::TrailingVarArg)]
pub struct RunArgs {
    /// after migrating to latest, run the "current" migration too
    #[clap(short, long)]
    current: bool,

    /// What command to execute migrations (replacements: {path}, {name})
    #[clap(multiple_values(true), value_hint = ValueHint::CommandWithArguments)]
    command: Vec<String>,
}

pub fn run_migrations(args: RunArgs) -> Result<()> {
    require_regrate_inited()?;

    let contents = std::fs::read_to_string(regrate_root()?.join("repo.toml"))?;
    let config: RepoConfig = toml::from_str(&contents)?;

    let mut iter = StoreNameIterator::new();
    while let Some((Some(name), next, Some(path), next_path)) = iter.next()? {
        env::set_var("REGRATE_NAME", &name);
        env::set_var("REGRATE_NEXT_NAME", &next);
        env::set_var("REGRATE_NEXT_PATH", &next_path);
        env::set_var("REGRATE_PATH", &path);
        let up_script: path::PathBuf;
        let down_script: path::PathBuf;

        match config.mode {
            InitType::Shell => {
                up_script = path.join("up.sh");
                down_script = path.join("down.sh");
            }
            InitType::Mysql => {
                up_script = path.join("up.mysql");
                down_script = path.join("down.mysql");
            }
            InitType::Postgres => {
                up_script = path.join("up.psql");
                down_script = path.join("down.psql");
            }
        }

        // Do variable expansion.
        // I purposely do not search inside strings since that gets into escaping madeness.
        let args: Vec<&str> = args
            .command
            .iter()
            .map(|x| match x.as_ref() {
                "{name}" => &name,
                "{path}" => path.to_str().unwrap_or("{path_invalidutf8}"),
                "{up}" => up_script.to_str().unwrap_or("{up_invalidutf8}"),
                "{down}" => down_script.to_str().unwrap_or("{down_invalidutf8}"),
                "{next-path}" | "{next_path}" => {
                    next_path.to_str().unwrap_or("{next_path_invalidutf8}")
                }
                "{next-name}" | "{next_name}" => &next,
                x => x,
            })
            .collect();

        run_migration_command(&args)?;
    }

    env::remove_var("REGRATE_NAME");
    env::remove_var("REGRATE_NEXT_NAME");
    env::remove_var("REGRATE_NEXT_PATH");
    env::remove_var("REGRATE_PATH");

    //  TODO: - Does not execute 'current' by default (use --current options for that)
    Ok(())
}

fn run_migration_command(command: &[&str]) -> Result<()> {
    if let Some((command, args)) = command.split_first() {
        let status = process::Command::new(command)
            .args(args)
            .status()
            .wrap_err("running migration tool")?;

        if status.success() {
            Ok(())
        } else {
            Err(eyre!("{} exited with {}", command, status))
        }
    } else {
        Err(eyre!("run command required"))
    }
}
