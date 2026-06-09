use crate::names;
use crate::utils::{regrate_path, require_regrate_inited};
use color_eyre::Help;
use eyre::{eyre, Result, WrapErr};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Resolve a git merge/rebase conflict where both sides committed a
/// migration claiming the same name. The upstream version is kept in the
/// store (and staged), and the local version is moved back to
/// `regrate/current` so it can be re-committed with a new name on top of
/// the upstream chain.
pub fn resolve_conflicts() -> Result<()> {
    require_regrate_inited()?;

    let toplevel = PathBuf::from(git_stdout(&["rev-parse", "--show-toplevel"])?.trim_end());
    let rebasing = git_dir_exists("rebase-merge")? || git_dir_exists("rebase-apply")?;

    // During a merge, stage 2 is our local branch and stage 3 is what we are
    // merging in. A rebase replays our commits onto upstream, so the sides
    // swap: stage 2 is upstream, stage 3 is our local commit.
    let (local_stage, upstream_stage) = if rebasing { (3, 2) } else { (2, 3) };

    let store_prefix = std::env::current_dir()?
        .strip_prefix(&toplevel)
        .map(Path::to_path_buf)
        .unwrap_or_default()
        .join("regrate")
        .join("store");

    // Unmerged paths, relative to the repository root.
    let unmerged = git_stdout(&["diff", "--name-only", "--diff-filter=U", "-z"])?;
    let mut conflicts: BTreeMap<String, Vec<(PathBuf, PathBuf)>> = BTreeMap::new();
    for path in unmerged.split('\0').filter(|p| !p.is_empty()) {
        let path = PathBuf::from(path);
        let Ok(in_store) = path.strip_prefix(&store_prefix) else {
            continue;
        };
        let mut parts = in_store.components();
        let (Some(prefix), Some(rest)) = (parts.next(), parts.next()) else {
            continue;
        };
        let migration = format!(
            "{}{}",
            prefix.as_os_str().to_string_lossy(),
            rest.as_os_str().to_string_lossy()
        );
        conflicts
            .entry(migration)
            .or_default()
            .push((path.clone(), parts.as_path().to_path_buf()));
    }

    if conflicts.is_empty() {
        println!("no conflicted migrations found; nothing to resolve");
        return Ok(());
    }
    if conflicts.len() > 1 {
        return Err(eyre!(
            "conflicts span {} migrations ({}); resolve cannot untangle that automatically",
            conflicts.len(),
            conflicts.keys().cloned().collect::<Vec<_>>().join(", ")
        )
        .with_suggestion(|| "resolve the store conflicts manually, keeping upstream's versions"));
    }

    let current = regrate_path("current")?;
    if current.exists() {
        return Err(eyre!("a current migration already exists")
            .with_note(|| "resolve needs to move the conflicted local migration to regrate/current")
            .with_suggestion(|| "commit or remove regrate/current first"));
    }
    std::fs::create_dir_all(&current)?;

    let (migration, files) = conflicts.into_iter().next().expect("checked non-empty");
    for (repo_path, rel) in &files {
        let spec = |stage: u32| format!(":{}:{}", stage, repo_path.display());
        let on_disk = toplevel.join(repo_path);

        // Our side of the conflict becomes part of the new current migration.
        if let Some(local) = git_show(&spec(local_stage))? {
            let dest = current.join(rel);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&dest, local).wrap_err_with(|| format!("writing {:?}", dest))?;
        }

        // Upstream's side wins in the store; stage the result.
        match git_show(&spec(upstream_stage))? {
            Some(upstream) => std::fs::write(&on_disk, upstream)
                .wrap_err_with(|| format!("writing {:?}", on_disk))?,
            None => std::fs::remove_file(&on_disk)
                .wrap_err_with(|| format!("removing {:?}", on_disk))?,
        }
        // use the absolute path: git resolves relative paths against the
        // cwd, but repo_path is relative to the repository root
        git_ok(&["add", "--", &on_disk.to_string_lossy()])?;
    }

    println!(
        "kept upstream's migration {} in the store (staged)",
        migration
    );
    println!("moved your version of the conflicted files to regrate/current");
    println!("next steps:");
    println!(
        "  1. finish the {} (git status)",
        if rebasing { "rebase" } else { "merge" }
    );
    println!("  2. review regrate/current and re-test it against the new chain");
    println!("  3. regrate commit -m <message>");

    let (migrations, _, _) = names::chain()?;
    let orphans = names::orphans(&migrations)?;
    if !orphans.is_empty() {
        println!(
            "warning: {} unreachable migration(s) remain in the store ({}); \
             they were committed on top of your old migration and must be \
             re-committed too (see `regrate valid`)",
            orphans.len(),
            orphans.join(", ")
        );
    }

    Ok(())
}

fn git_command(args: &[&str]) -> Result<std::process::Output> {
    Command::new("git")
        .args(args)
        .output()
        .wrap_err("running git")
}

fn git_stdout(args: &[&str]) -> Result<String> {
    let out = git_command(args)?;
    if !out.status.success() {
        return Err(eyre!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8(out.stdout)?)
}

fn git_ok(args: &[&str]) -> Result<()> {
    git_stdout(args).map(|_| ())
}

/// Contents of `git show <spec>`, or None if the object does not exist
/// (e.g. the file has no entry at that conflict stage).
fn git_show(spec: &str) -> Result<Option<Vec<u8>>> {
    let out = git_command(&["show", spec])?;
    if out.status.success() {
        Ok(Some(out.stdout))
    } else {
        Ok(None)
    }
}

fn git_dir_exists(name: &str) -> Result<bool> {
    let path = git_stdout(&["rev-parse", "--git-path", name])?;
    Ok(Path::new(path.trim_end()).exists())
}
