use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use kondo_lib::{scan, ScanOptions};

fn git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

fn git(dir: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args([
            "-c",
            "user.name=kondo",
            "-c",
            "user.email=kondo@example.com",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=",
        ])
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn scan_paths(root: &Path) -> Vec<PathBuf> {
    let options = ScanOptions {
        follow_symlinks: false,
        same_file_system: false,
        worktrees: true,
    };
    let mut paths: Vec<PathBuf> = scan(&root, &options)
        .map(|r| r.ok().expect("scan error").path)
        .collect();
    paths.sort();
    paths
}

/// A committed Cargo repo at `<tmp>/repo`, with a `target/` artifact dir.
fn cargo_repo() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::Builder::new().prefix("kondo").tempdir().unwrap();
    let repo = tmp.path().join("repo");
    fs::create_dir_all(repo.join("target")).unwrap();
    fs::write(repo.join("Cargo.toml"), "[package]\n").unwrap();
    fs::write(repo.join(".gitignore"), "target/\n.claude/\n").unwrap();
    git(&repo, &["init", "--quiet"]);
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "--quiet", "-m", "init"]);
    (tmp, repo)
}

fn add_worktree(repo: &Path, path: &str, extra: &[&str]) -> PathBuf {
    let mut args = extra.to_vec();
    args.extend(["worktree", "add", "--quiet", "--detach", path]);
    git(repo, &args);
    let worktree = repo.join(path);
    fs::create_dir_all(worktree.join("target")).unwrap();
    worktree
}

#[test]
fn finds_worktrees_created_by_git() {
    if !git_available() {
        eprintln!("skipping: git not found on PATH");
        return;
    }
    let (tmp, repo) = cargo_repo();
    let x = add_worktree(&repo, ".claude/worktrees/x", &[]);

    assert_eq!(scan_paths(tmp.path()), [repo, x]);
}

#[test]
fn finds_worktrees_with_relative_paths() {
    if !git_available() {
        eprintln!("skipping: git not found on PATH");
        return;
    }
    let (tmp, repo) = cargo_repo();
    let x = add_worktree(
        &repo,
        ".claude/worktrees/x",
        &["-c", "worktree.useRelativePaths=true"],
    );
    let gitdir = fs::read_to_string(repo.join(".git/worktrees/x/gitdir")).unwrap();
    if Path::new(gitdir.trim()).is_absolute() {
        eprintln!("skipping: this git does not support worktree.useRelativePaths");
        return;
    }

    assert_eq!(scan_paths(tmp.path()), [repo, x]);
}

#[test]
fn skips_locked_worktrees() {
    if !git_available() {
        eprintln!("skipping: git not found on PATH");
        return;
    }
    let (tmp, repo) = cargo_repo();
    let x = add_worktree(&repo, ".claude/worktrees/x", &[]);
    add_worktree(&repo, ".claude/worktrees/y", &[]);
    add_worktree(&repo, "../sibling", &[]);
    git(
        &repo,
        &[
            "worktree",
            "lock",
            "--reason",
            "agent",
            ".claude/worktrees/y",
        ],
    );
    git(&repo, &["worktree", "lock", "../sibling"]);

    assert_eq!(scan_paths(tmp.path()), [repo, x]);
}
