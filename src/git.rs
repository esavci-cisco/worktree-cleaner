// src/git.rs

use anyhow::{Context, Result};
use chrono::Utc;
use std::{
    path::{Path, PathBuf},
    process::Command,
};
use walkdir::WalkDir;

use crate::worktree::Worktree;

pub fn discover_git_repos(
    roots: &[String],
) -> Result<Vec<PathBuf>> {
    let mut repos = vec![];

    for root in roots {
        for entry in WalkDir::new(root)
            .max_depth(3)
            .follow_links(false)
        {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            if !entry.file_type().is_dir() {
                continue;
            }

            let git_dir = entry.path().join(".git");

            if git_dir.is_dir() {
                repos.push(entry.path().to_path_buf());
            }
        }
    }

    Ok(repos)
}

pub fn get_worktrees(
    repo: &Path,
) -> Result<Vec<Worktree>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["worktree", "list", "--porcelain"])
        .output()
        .with_context(|| {
            format!("Failed to inspect {}", repo.display())
        })?;

    if !output.status.success() {
        return Ok(vec![]);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut worktrees = parse_worktrees(
        repo.file_name()
            .unwrap()
            .to_string_lossy()
            .to_string(),
        repo.to_path_buf(),
        &stdout,
    )?;

    if !worktrees.is_empty() {
        worktrees.remove(0);
    }

    Ok(worktrees)
}

fn parse_worktrees(
    repo_name: String,
    repo_path: PathBuf,
    input: &str,
) -> Result<Vec<Worktree>> {
    let mut result = vec![];

    let blocks = input.split("\n\n");

    for block in blocks {
        let mut path = None;
        let mut branch = None;

        for line in block.lines() {
            if let Some(v) = line.strip_prefix("worktree ")
            {
                path = Some(PathBuf::from(v));
            }

            if let Some(v) =
                line.strip_prefix("branch refs/heads/")
            {
                branch = Some(v.to_string());
            }
        }

        let path = match path {
            Some(p) => p,
            None => continue,
        };

        let dirty = is_worktree_dirty(&path)?;

        let merged = branch
            .as_ref()
            .map(|b| is_branch_merged(&repo_path, b))
            .transpose()?
            .unwrap_or(false);

        let remote_gone = branch
            .as_ref()
            .map(|b| is_remote_gone(&repo_path, b))
            .transpose()?
            .unwrap_or(false);

        let age_days = branch
            .as_ref()
            .and_then(|b| get_branch_age_days(&repo_path, b).ok());

        result.push(Worktree {
            repo: repo_name.clone(),
            repo_path: repo_path.clone(),
            path,
            branch,
            dirty,
            merged,
            remote_gone,
            age_days,
        });
    }

    Ok(result)
}

fn is_worktree_dirty(path: &Path) -> Result<bool> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["status", "--porcelain"])
        .output()?;

    Ok(!output.stdout.is_empty())
}

fn get_branch_age_days(
    repo: &Path,
    branch: &str,
) -> Result<u64> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args([
            "log",
            "-1",
            "--format=%ct",
            branch,
        ])
        .output()?;

    let stdout =
        String::from_utf8_lossy(&output.stdout);

    let timestamp: i64 = stdout.trim().parse()?;

    let now = Utc::now().timestamp();

    Ok(((now - timestamp) / 86400) as u64)
}

fn is_branch_merged(
    repo: &Path,
    branch: &str,
) -> Result<bool> {
    let branch_head = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["rev-parse", branch])
        .output()?;

    let main_head = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["rev-parse", "main"])
        .output()?;

    let branch_sha =
        String::from_utf8_lossy(&branch_head.stdout);

    let main_sha =
        String::from_utf8_lossy(&main_head.stdout);

    // untouched branch
    if branch_sha.trim() == main_sha.trim() {
        return Ok(false);
    }

    let status = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args([
            "merge-base",
            "--is-ancestor",
            branch,
            "main",
        ])
        .status()?;

    Ok(status.success())
}

fn is_remote_gone(
    repo: &Path,
    branch: &str,
) -> Result<bool> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["branch", "-vv"])
        .output()?;

    let stdout =
        String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        if line.contains(branch)
            && line.contains(": gone]")
        {
            return Ok(true);
        }
    }

    Ok(false)
}

pub fn remove_worktree(
    wt: &Worktree,
) -> Result<()> {
    println!("Removing {}", wt.path.display());

    let status = Command::new("git")
        .arg("-C")
        .arg(&wt.repo_path)
        .args(["worktree", "remove", "--force"])
        .arg(&wt.path)
        .status()?;

    if status.success() {
        println!("Deleted {}", wt.path.display());
    } else {
        println!("Failed {}", wt.path.display());
    }

    Ok(())
}
