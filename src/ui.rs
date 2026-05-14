// src/ui.rs

use anyhow::Result;
use dialoguer::{
    theme::ColorfulTheme,
    Confirm,
    MultiSelect,
};

use crate::worktree::Worktree;

pub fn select_worktrees(
    worktrees: &[Worktree],
) -> Result<Vec<usize>> {
    let items: Vec<String> = worktrees
        .iter()
        .map(|wt| wt.display_name())
        .collect();

    Ok(
        MultiSelect::with_theme(
            &ColorfulTheme::default(),
        )
        .with_prompt("Select worktrees to delete")
        .items(&items)
        .interact()?,
    )
}

pub fn show_selected_worktrees(
    worktrees: &[Worktree],
    selections: &[usize],
) {
    println!();
    println!("Selected worktrees:");
    println!();

    for idx in selections {
        let wt = &worktrees[*idx];

        println!("{}", wt.display_name());
        println!("  {}", wt.path.display());
        println!();
    }
}

pub fn confirm_deletion() -> Result<bool> {
    Ok(
        Confirm::with_theme(
            &ColorfulTheme::default(),
        )
        .with_prompt("Delete selected worktrees?")
        .default(false)
        .interact()?,
    )
}
