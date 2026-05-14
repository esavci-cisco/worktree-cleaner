// src/main.rs

mod config;
mod git;
mod ui;
mod worktree;

use anyhow::Result;
use clap::{Parser, Subcommand};

use config::{init_config, load_config};
use git::{discover_git_repos, get_worktrees, remove_worktree};
use ui::{confirm_deletion, select_worktrees, show_selected_worktrees};
use worktree::Worktree;

#[derive(Parser)]
#[command(name = "git-wt")]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Clean {
        #[arg(long)]
        merged: bool,

        #[arg(long)]
        gone: bool,

        #[arg(long)]
        stale: Option<u64>,
    },
    Init,
    Doctor,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => init_config(),
        Commands::Doctor => doctor(),
        Commands::Clean {
            merged,
            gone,
            stale,
        } => clean_worktrees(merged, gone, stale),
    }
}

fn clean_worktrees(
    merged_only: bool,
    gone_only: bool,
    stale_days: Option<u64>,
) -> Result<()> {
    let config = load_config()?;

    let repos = discover_git_repos(&config.roots)?;

    if repos.is_empty() {
        println!("No git repos found");
        return Ok(());
    }

    let mut all_worktrees: Vec<Worktree> = vec![];

    for repo in repos {
        let worktrees = get_worktrees(&repo)?;

        for wt in worktrees {
            if merged_only && !wt.merged {
                continue;
            }

            if gone_only && !wt.remote_gone {
                continue;
            }

            if let Some(days) = stale_days {
                if !wt.stale(days) {
                    continue;
                }
            }

            all_worktrees.push(wt);
        }
    }

    if all_worktrees.is_empty() {
        println!("No removable worktrees found");
        return Ok(());
    }

    let selections = select_worktrees(&all_worktrees)?;

    if selections.is_empty() {
        println!("Nothing selected");
        return Ok(());
    }

    show_selected_worktrees(&all_worktrees, &selections);

    if !confirm_deletion()? {
        println!("Aborted");
        return Ok(());
    }

    for idx in selections {
        let wt = &all_worktrees[idx];

        if wt.dirty {
            println!(
                "Skipping dirty worktree: {}",
                wt.path.display()
            );

            continue;
        }

        remove_worktree(wt)?;
    }

    Ok(())
}

fn doctor() -> Result<()> {
    let config = load_config()?;

    println!("Configured roots:");

    for root in &config.roots {
        println!("  - {}", root);
    }

    let repos = discover_git_repos(&config.roots)?;

    println!();
    println!("Repositories found: {}", repos.len());

    for repo in repos {
        let worktrees = get_worktrees(&repo)?;

        println!(
            "{} ({} worktrees)",
            repo.display(),
            worktrees.len()
        );
    }

    Ok(())
}
