use crate::names;
use crate::types::{InitType, RepoConfig};
use crate::utils::{regrate_path, regrate_root, require_regrate_inited};
use clap::{Args, ValueHint};
use color_eyre::Help;
use eyre::{eyre, Result, WrapErr};
use std::path::{Path, PathBuf};
use std::process;

#[derive(Args, Debug)]
pub struct RunArgs {
    /// after migrating to latest, run the "current" migration too
    #[arg(short, long)]
    current: bool,

    /// What command to execute migrations (replacements: {name}, {path}, {up}, {down}, {next-name}, {next-path})
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, value_hint = ValueHint::CommandWithArguments)]
    command: Vec<String>,
}

struct Step {
    index: usize,
    name: String,
    path: PathBuf,
    next_name: Option<String>,
    next_path: Option<PathBuf>,
}

pub fn run_migrations(args: RunArgs) -> Result<()> {
    require_regrate_inited()?;

    let contents = std::fs::read_to_string(regrate_root()?.join("repo.toml"))?;
    let config: RepoConfig = toml::from_str(&contents)?;

    let (migrations, pending_name, _) = names::chain()?;

    // Refuse to run a store that has unreachable migrations: it means a
    // committed migration was edited or a merge went wrong, and silently
    // skipping the unreachable ones would migrate to the wrong state.
    let orphans = names::orphans(&migrations)?;
    if !orphans.is_empty() {
        return Err(eyre!(
            "store contains migrations not reachable from the name chain:\n  {}",
            orphans.join("\n  ")
        )
        .with_note(|| "a committed migration was probably edited, or a merge left extras behind")
        .with_suggestion(|| {
            "run `regrate valid` for details, or `regrate resolve` after a merge"
        }));
    }

    let mut steps: Vec<Step> = migrations
        .into_iter()
        .map(|m| Step {
            index: m.index,
            name: m.name,
            path: m.path,
            next_name: Some(m.next_name),
            next_path: Some(m.next_path),
        })
        .collect();

    if args.current {
        let current = regrate_path("current")?;
        if !current.is_dir() {
            return Err(
                eyre!("--current requested but there is no current migration")
                    .with_suggestion(|| "use `regrate create` to start one"),
            );
        }
        steps.push(Step {
            index: steps.len(),
            name: pending_name,
            path: current,
            next_name: None,
            next_path: None,
        });
    }

    for step in &steps {
        run_step(step, config.mode, &args.command)?;
    }

    Ok(())
}

fn run_step(step: &Step, mode: InitType, command: &[String]) -> Result<()> {
    let (up, down) = match mode {
        InitType::Shell => ("up.sh", "down.sh"),
        InitType::Mysql => ("up.mysql", "down.mysql"),
        InitType::Postgres => ("up.psql", "down.psql"),
    };
    let up_script = step.path.join(up);
    let down_script = step.path.join(down);

    // Do variable expansion.
    // I purposely do not search inside strings since that gets into escaping madness.
    let args: Vec<&str> = command
        .iter()
        .map(|x| match x.as_ref() {
            "{name}" => Ok(step.name.as_str()),
            "{path}" => path_str(&step.path),
            "{up}" => path_str(&up_script),
            "{down}" => path_str(&down_script),
            "{next-name}" | "{next_name}" => step
                .next_name
                .as_deref()
                .ok_or_else(|| eyre!("{{next-name}} is not known for the current migration")),
            "{next-path}" | "{next_path}" => {
                step.next_path.as_deref().map(path_str).unwrap_or_else(|| {
                    Err(eyre!(
                        "{{next-path}} is not known for the current migration"
                    ))
                })
            }
            x => Ok(x),
        })
        .collect::<Result<_>>()?;

    let (program, rest) = args
        .split_first()
        .ok_or_else(|| eyre!("run command required"))?;

    let status = process::Command::new(program)
        .args(rest)
        .env("REGRATE_INDEX", step.index.to_string())
        .env("REGRATE_NAME", &step.name)
        .env("REGRATE_PATH", &step.path)
        .envs(step.next_name.as_ref().map(|n| ("REGRATE_NEXT_NAME", n)))
        .envs(step.next_path.as_ref().map(|p| ("REGRATE_NEXT_PATH", p)))
        .status()
        .wrap_err("running migration tool")?;

    if status.success() {
        Ok(())
    } else {
        Err(eyre!("{} exited with {} on {}", program, status, step.name))
    }
}

fn path_str(path: &Path) -> Result<&str> {
    path.to_str()
        .ok_or_else(|| eyre!("path {:?} is not valid utf-8", path))
}
