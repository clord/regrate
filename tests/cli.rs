use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn regrate(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_regrate"))
        .args(args)
        .current_dir(dir)
        .output()
        .expect("failed to run regrate")
}

fn ok(dir: &Path, args: &[&str]) -> String {
    let out = regrate(dir, args);
    assert!(
        out.status.success(),
        "regrate {:?} failed:\nstdout: {}\nstderr: {}",
        args,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn fails(dir: &Path, args: &[&str]) -> String {
    let out = regrate(dir, args);
    assert!(
        !out.status.success(),
        "regrate {:?} unexpectedly succeeded:\nstdout: {}",
        args,
        String::from_utf8_lossy(&out.stdout),
    );
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn git(dir: &Path, args: &[&str]) -> Output {
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("failed to run git")
}

fn git_ok(dir: &Path, args: &[&str]) {
    let out = git(dir, args);
    assert!(
        out.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Names of committed migrations in store, in directory order.
fn store_names(dir: &Path) -> Vec<String> {
    let mut names = Vec::new();
    let store = dir.join("regrate/store");
    for prefix in fs::read_dir(store).unwrap() {
        let prefix = prefix.unwrap();
        for rest in fs::read_dir(prefix.path()).unwrap() {
            let rest = rest.unwrap();
            names.push(format!(
                "{}{}",
                prefix.file_name().to_string_lossy(),
                rest.file_name().to_string_lossy()
            ));
        }
    }
    names.sort();
    names
}

fn commit_migration(dir: &Path, script: &str, message: &str) {
    ok(dir, &["create"]);
    fs::write(dir.join("regrate/current/up.sh"), script).unwrap();
    ok(dir, &["commit", "-m", message]);
}

#[test]
fn init_creates_expected_layout() {
    let tmp = tempfile::tempdir().unwrap();
    ok(tmp.path(), &["init", "shell"]);

    assert!(tmp.path().join("regrate/store").is_dir());
    assert!(tmp.path().join("regrate/repo.toml").is_file());
    assert!(tmp.path().join("regrate/template/up.sh").is_file());
    assert!(tmp.path().join("regrate/template/down.sh").is_file());
}

#[test]
fn run_executes_migrations_in_commit_order() {
    let tmp = tempfile::tempdir().unwrap();
    ok(tmp.path(), &["init", "shell"]);
    commit_migration(tmp.path(), "#!/bin/sh\necho first\n", "first");
    commit_migration(tmp.path(), "#!/bin/sh\necho second\n", "second");

    let out = ok(tmp.path(), &["run", "sh", "{up}"]);
    let first = out.find("first").expect("first migration ran");
    let second = out.find("second").expect("second migration ran");
    assert!(first < second, "migrations ran out of order: {}", out);
}

#[test]
fn names_are_deterministic_across_repos() {
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    for dir in [a.path(), b.path()] {
        ok(dir, &["init", "shell"]);
        commit_migration(dir, "#!/bin/sh\necho same content\n", "same message");
    }
    assert_eq!(store_names(a.path()), store_names(b.path()));
}

#[test]
fn file_names_affect_migration_names() {
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    for dir in [a.path(), b.path()] {
        ok(dir, &["init", "--no-template", "shell"]);
        ok(dir, &["create"]);
    }
    fs::write(a.path().join("regrate/current/up.sh"), "echo hi\n").unwrap();
    fs::write(b.path().join("regrate/current/renamed.sh"), "echo hi\n").unwrap();
    ok(a.path(), &["commit", "-m", "m"]);
    ok(b.path(), &["commit", "-m", "m"]);

    // committed dirs land at the same first name...
    let first_a = store_names(a.path());
    let first_b = store_names(b.path());
    assert_eq!(first_a, first_b);

    // ...but the names derived from their contents must differ.
    for dir in [a.path(), b.path()] {
        ok(dir, &["create"]);
        ok(dir, &["commit", "-m", "m2"]);
    }
    assert_ne!(store_names(a.path()), store_names(b.path()));
}

#[test]
fn editing_committed_migration_is_detected() {
    let tmp = tempfile::tempdir().unwrap();
    ok(tmp.path(), &["init", "shell"]);
    commit_migration(tmp.path(), "#!/bin/sh\necho one\n", "one");
    commit_migration(tmp.path(), "#!/bin/sh\necho two\n", "two");
    ok(tmp.path(), &["valid"]);

    // tamper with the first committed migration
    let first = store_names(tmp.path())
        .into_iter()
        .map(|n| {
            tmp.path()
                .join("regrate/store")
                .join(&n[0..2])
                .join(&n[2..])
        })
        .find(|p| fs::read_to_string(p.join("up.sh")).unwrap().contains("one"))
        .expect("first migration dir");
    fs::write(first.join("up.sh"), "#!/bin/sh\necho tampered\n").unwrap();

    let err = fails(tmp.path(), &["valid"]);
    assert!(err.contains("not reachable"), "unexpected error: {}", err);

    // run must refuse rather than silently skipping the orphaned migration
    let err = fails(tmp.path(), &["run", "sh", "{up}"]);
    assert!(err.contains("not reachable"), "unexpected error: {}", err);
    assert!(!err.contains("echo two"), "must not run on broken chain");
}

#[test]
fn run_current_includes_uncommitted_migration() {
    let tmp = tempfile::tempdir().unwrap();
    ok(tmp.path(), &["init", "shell"]);
    commit_migration(tmp.path(), "#!/bin/sh\necho committed\n", "committed");
    ok(tmp.path(), &["create"]);
    fs::write(
        tmp.path().join("regrate/current/up.sh"),
        "#!/bin/sh\necho work-in-progress\n",
    )
    .unwrap();

    let out = ok(tmp.path(), &["run", "sh", "{up}"]);
    assert!(!out.contains("work-in-progress"));

    let out = ok(tmp.path(), &["run", "--current", "sh", "{up}"]);
    assert!(out.contains("committed"));
    assert!(out.contains("work-in-progress"));

    // --current without a current migration is an error
    fs::remove_dir_all(tmp.path().join("regrate/current")).unwrap();
    fails(tmp.path(), &["run", "--current", "sh", "{up}"]);
}

#[test]
fn run_sets_regrate_index_and_name() {
    let tmp = tempfile::tempdir().unwrap();
    ok(tmp.path(), &["init", "shell"]);
    commit_migration(tmp.path(), "", "a");
    commit_migration(tmp.path(), "", "b");

    let out = ok(
        tmp.path(),
        &[
            "run",
            "sh",
            "-c",
            "echo index=$REGRATE_INDEX name=$REGRATE_NAME",
        ],
    );
    assert!(out.contains("index=0"), "missing index 0: {}", out);
    assert!(out.contains("index=1"), "missing index 1: {}", out);
}

#[test]
fn resolve_moves_local_conflict_to_current() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    git_ok(dir, &["init", "-b", "main"]);
    git_ok(dir, &["config", "user.email", "test@example.com"]);
    git_ok(dir, &["config", "user.name", "Test"]);
    git_ok(dir, &["config", "commit.gpgsign", "false"]);

    ok(dir, &["init", "--no-template", "shell"]);
    ok(dir, &["create"]);
    fs::write(dir.join("regrate/current/up.sh"), "echo base\n").unwrap();
    ok(dir, &["commit", "-m", "base"]);
    git_ok(dir, &["add", "-A"]);
    git_ok(dir, &["commit", "-m", "base migration"]);

    // upstream commits the next migration...
    git_ok(dir, &["checkout", "-b", "upstream"]);
    ok(dir, &["create"]);
    fs::write(dir.join("regrate/current/up.sh"), "echo upstream\n").unwrap();
    ok(dir, &["commit", "-m", "upstream"]);
    git_ok(dir, &["add", "-A"]);
    git_ok(dir, &["commit", "-m", "upstream migration"]);

    // ...while we commit a different one claiming the same name
    git_ok(dir, &["checkout", "main"]);
    ok(dir, &["create"]);
    fs::write(dir.join("regrate/current/up.sh"), "echo local\n").unwrap();
    ok(dir, &["commit", "-m", "local"]);
    git_ok(dir, &["add", "-A"]);
    git_ok(dir, &["commit", "-m", "local migration"]);

    let merge = git(dir, &["merge", "upstream"]);
    assert!(!merge.status.success(), "merge should conflict");

    ok(dir, &["resolve"]);

    // upstream's version stays in the store, ours moved to current
    let names = store_names(dir);
    assert_eq!(names.len(), 2);
    let contents: Vec<String> = names
        .iter()
        .map(|n| {
            fs::read_to_string(
                dir.join("regrate/store")
                    .join(&n[0..2])
                    .join(&n[2..])
                    .join("up.sh"),
            )
            .unwrap()
        })
        .collect();
    assert!(contents.iter().any(|c| c.contains("upstream")));
    assert!(!contents.iter().any(|c| c.contains("local")));
    assert_eq!(
        fs::read_to_string(dir.join("regrate/current/up.sh")).unwrap(),
        "echo local\n"
    );

    // chain is intact, and committing our migration again completes the merge
    git_ok(dir, &["commit", "--no-edit"]);
    ok(dir, &["valid"]);
    ok(dir, &["commit", "-m", "local again"]);
    ok(dir, &["valid"]);
    assert_eq!(store_names(dir).len(), 3);
}
