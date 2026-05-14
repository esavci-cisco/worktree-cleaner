// src/main.rs

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "wt")]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Clean,
    Init,
    Doctor,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    roots: Vec<String>,
}

#[derive(Debug, Clone)]
struct Worktree {
    repo: String,
    repo_path: PathBuf,
    path: PathBuf,
    branch: Option<String>,
    dirty: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => init_config(),
        Commands::Clean => clean_worktrees(),
        Commands::Doctor => doctor(),
    }
}

fn init_config() -> Result<()> {
    let path = config_path();

    if path.exists() {
        println!("{}", "Config already exists".yellow());
        return Ok(());
    }

    let parent = path.parent().unwrap();
    fs::create_dir_all(parent)?;

    let config = Config {
        roots: vec![],
    };

    fs::write(&path, toml::to_string_pretty(&config)?)?;

    println!(
        "{} {}",
        "Created config:".green(),
        path.display()
    );

    Ok(())
}

fn clean_worktrees() -> Result<()> {
    let config = load_config()?;

    let repos = discover_git_repos(&config.roots)?;

    if repos.is_empty() {
        println!("{}", "No git repos found".yellow());
        return Ok(());
    }

    let mut all_worktrees = vec![];

    for repo in repos {
        let worktrees = get_worktrees(&repo)?;

        for wt in worktrees {
            all_worktrees.push(wt);
        }
    }

    if all_worktrees.is_empty() {
        println!("{}", "No removable worktrees found".yellow());
        return Ok(());
    }

    let items: Vec<String> = all_worktrees
        .iter()
        .map(|wt| {
            let branch = wt
                .branch
                .clone()
                .unwrap_or_else(|| "detached".into());

            let dirty = if wt.dirty {
                format!(" {}", "[DIRTY]".red())
            } else {
                "".to_string()
            };

            format!(
                "{} :: {}{}",
                wt.repo.cyan(),
                branch.yellow(),
                dirty
            )
        })
        .collect();

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select worktrees to delete")
        .items(&items)
        .interact()?;

    if selections.is_empty() {
        println!("{}", "Nothing selected".yellow());
        return Ok(());
    }

    println!();
    println!("{}", "Selected worktrees:".bold());

    for idx in &selections {
        let wt = &all_worktrees[*idx];

        println!(
            "  {} :: {}",
            wt.repo.cyan(),
            wt.branch
                .clone()
                .unwrap_or_else(|| "detached".into())
                .yellow()
        );

        println!("      {}", wt.path.display());

        if wt.dirty {
            println!("      {}", "DIRTY WORKTREE".red());
        }

        println!();
    }

    let confirmed = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Delete selected worktrees?")
        .default(false)
        .interact()?;

    if !confirmed {
        println!("{}", "Aborted".yellow());
        return Ok(());
    }

    for idx in selections {
        let wt = &all_worktrees[idx];

        if wt.dirty {
            println!(
                "{} {}",
                "Skipping dirty worktree (commit/stash changes first):".yellow(),
                wt.path.display()
            );

            continue;
        }

        println!(
            "{} {}",
            "Removing".red(),
            wt.path.display()
        );

        remove_worktree(wt)?;
    }

    Ok(())
}

fn load_config() -> Result<Config> {
    let path = config_path();

    if !path.exists() {
        return Err(anyhow!(
            "Config not found. Run: wt init"
        ));
    }

    let content = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;

    Ok(config)
}

fn config_path() -> PathBuf {
    config_dir()
        .unwrap()
        .join("git-wt")
        .join("config.toml")
}

fn discover_git_repos(roots: &[String]) -> Result<Vec<PathBuf>> {
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

fn get_worktrees(repo: &Path) -> Result<Vec<Worktree>> {
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

    // first entry is always the main checkout
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
        if block.trim().is_empty() {
            continue;
        }

        let mut path = None;
        let mut branch = None;
        for line in block.lines() {
            if let Some(v) = line.strip_prefix("worktree ") {
                path = Some(PathBuf::from(v));
            }

            if let Some(v) = line.strip_prefix("branch refs/heads/") {
                branch = Some(v.to_string());
            }
        }

        let path = match path {
            Some(p) => p,
            None => continue,
        };

        let dirty = is_worktree_dirty(&path)?;

        result.push(Worktree {
            repo: repo_name.clone(),
            repo_path: repo_path.clone(),
            path,
            branch,
            dirty,
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

    if !output.status.success() {
        return Ok(false);
    }

    Ok(!output.stdout.is_empty())
}

fn doctor() -> Result<()> {
    println!("{}", "git-wt diagnostics".bold().cyan());
    println!();

    let config_path = config_path();

    println!(
        "{} {}",
        "Config:".green(),
        config_path.display()
    );

    if !config_path.exists() {
        println!("{}", "Config file does not exist".red());
        println!();
        println!("Run:");
        println!("  git wt init");
        return Ok(());
    }

    let config = load_config()?;

    println!();
    println!("{}", "Configured roots:".green());

    for root in &config.roots {
        let path = Path::new(root);

        if path.exists() {
            println!("  {} {}", "✓".green(), root);
        } else {
            println!("  {} {}", "✗".red(), root);
        }
    }

    println!();

    let repos = discover_git_repos(&config.roots)?;

    println!(
        "{} {}",
        "Repositories found:".green(),
        repos.len()
    );

    if repos.is_empty() {
        println!();
        println!("{}", "No repositories discovered.".yellow());
        println!();
        println!("Suggestions:");
        println!("  - verify configured roots");
        println!("  - ensure repositories exist");
        println!("  - ensure repos contain .git");
        return Ok(());
    }

    let mut total_worktrees = 0usize;

    println!();
    println!("{}", "Repository summary:".green());

    for repo in &repos {
        let worktrees = get_worktrees(repo)?;

        total_worktrees += worktrees.len();

        println!(
            "  {} {} ({} worktrees)",
            "✓".green(),
            repo.display(),
            worktrees.len()
        );
    }

    println!();

    println!(
        "{} {}",
        "Total worktrees:".green(),
        total_worktrees
    );

    if total_worktrees <= repos.len() {
        println!();
        println!("{}", "No additional worktrees detected.".yellow());
        println!();
        println!("Create one with:");
        println!(
            "  git worktree add ../my-feature feature/my-branch"
        );
    }

    Ok(())
}

fn remove_worktree(wt: &Worktree) -> Result<()> {
    let status = Command::new("git")
        .arg("-C")
        .arg(&wt.repo_path)
        .args(["worktree", "remove", "--force"])
        .arg(&wt.path)
        .status()?;

    if !status.success() {
        println!(
            "{} {}",
            "Failed:".red(),
            wt.path.display()
        );
    } else {
        println!(
            "{} {}",
            "Deleted:".green(),
            wt.path.display()
        );
    }

    Ok(())
}
