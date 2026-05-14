// src/worktree.rs

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Worktree {
    pub repo: String,
    pub repo_path: PathBuf,
    pub path: PathBuf,
    pub branch: Option<String>,
    pub dirty: bool,
    pub merged: bool,
    pub remote_gone: bool,
    pub age_days: Option<u64>,
}

impl Worktree {
    pub fn stale(&self, days: u64) -> bool {
        match self.age_days {
            Some(age) => age >= days,
            None => false,
        }
    }

    pub fn display_name(&self) -> String {
        let mut parts = vec![];

        parts.push(format!(
            "{} :: {}",
            self.repo,
            self.branch
                .clone()
                .unwrap_or_else(|| "detached".into())
        ));

        if let Some(age) = self.age_days {
            parts.push(format!("[{}d]", age));
        }

        if self.merged {
            parts.push("[merged]".into());
        }

        if self.remote_gone {
            parts.push("[gone]".into());
        }

        if self.dirty {
            parts.push("[DIRTY]".into());
        }

        parts.join(" ")
    }
}
