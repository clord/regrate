use crate::names::StoreNameIterator;
use crate::utils::require_regrate_inited;
use clap::{AppSettings, Args, ValueHint};
use fallible_iterator::FallibleIterator;

use eyre::{eyre, Result, WrapErr};

use std::{env, process};

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
    let mut iter = StoreNameIterator::new();
    while let Some((name, next, path, next_path)) = iter.next()? {
        env::set_var("REGRATE_NAME", &name);
        env::set_var("REGRATE_NEXT_NAME", &next);
        env::set_var("REGRATE_NEXT_PATH", &next_path);
        env::set_var("REGRATE_PATH", &path);
        let up_script = path.join("up.sh");
        let down_script = path.join("down.sh");

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
