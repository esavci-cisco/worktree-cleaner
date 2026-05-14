# git-wt

Interactive Git worktree manager written in Rust.

`git-wt` discovers Git repositories inside configured directories, lists their worktrees, and lets you interactively clean them up safely using native Git commands.

It does **not** manually delete directories.

Internally it uses:

```bash
git worktree remove
```

so Git metadata stays fully consistent.

---

# Features

- Interactive worktree cleanup
- Multi-select deletion UI
- Safe deletion confirmation
- Dirty worktree protection
- Branch age display
- Merged branch filtering
- Stale worktree filtering
- Remote-gone branch detection
- Recursive repository discovery
- Configurable repository roots
- Native Git subcommand support
- Diagnostic command (`git wt doctor`)
- Fast single-binary CLI

---

# Installation

## Prerequisites

You must have:

- Git
- Rust toolchain

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Verify:

```bash
rustc --version
cargo --version
```

---

## Clone

```bash
git clone https://github.com/esavci-cisco/worktree-cleaner.git
cd worktree-cleaner
```

---

## Install

Install globally:

```bash
cargo install --path . --force
```

Verify:

```bash
git wt
```

Because the binary is named `git-wt`, Git automatically exposes it as:

```bash
git wt
```

This is the same mechanism used by tools like:

- git-lfs
- git-flow

---

# Configuration

Initialize config:

```bash
git wt init
```

This creates:

```bash
~/.config/git-wt/config.toml
```

Example:

```toml
roots = [
  "/home/eggy/Desktop/dev"
]
```

`git-wt` recursively scans these directories for Git repositories.

---

# Usage

## Clean Worktrees

```bash
git wt clean
```

Example:

```text
worktree-cleaner :: feature/auth [12d]
worktree-cleaner :: feature/old-branch [120d] [merged]
worktree-cleaner :: feature/deleted-pr [gone]
worktree-cleaner :: feature/wip [DIRTY]
```

Controls:

- `SPACE` → select
- `ENTER` → continue
- confirmation prompt before deletion

Dirty worktrees are skipped automatically.

---

# Filtering

## Show Only Merged Worktrees

```bash
git wt clean --merged
```

---

## Show Only Stale Worktrees

```bash
git wt clean --stale 30
```

Shows worktrees whose latest commit is older than 30 days.

---

## Show Only Remote-Gone Branches

```bash
git wt clean --gone
```

Useful after branches are deleted on GitHub/GitLab.

---

## Combine Filters

```bash
git wt clean --merged --stale 30
```

Example workflow:
- already merged
- older than 30 days
- safe to remove

---

# Doctor

Run diagnostics:

```bash
git wt doctor
```

Example output:

```text
Configured roots:
  - /home/eggy/Desktop/dev

Repositories found: 12

/home/eggy/Desktop/dev/api (3 worktrees)
/home/eggy/Desktop/dev/frontend (2 worktrees)
```

Useful for:
- validating config
- debugging repository discovery
- verifying worktree detection

---

# Creating Worktrees

Create new worktree + branch:

```bash
git worktree add -b feature/my-feature \
/home/eggy/Desktop/dev/worktrees/my-feature
```

Verify:

```bash
git worktree list
```

Example:

```text
/home/eggy/Desktop/dev/myrepo                      abc123 [main]
/home/eggy/Desktop/dev/worktrees/my-feature       def456 [feature/my-feature]
```

---

# How Removal Works

`git-wt` uses:

```bash
git worktree remove --force <path>
```

This safely removes:

- worktree directory
- `.git/worktrees/*` metadata
- refs
- internal Git bookkeeping

---

# Development

Run locally:

```bash
cargo run -- clean
```

Build:

```bash
cargo build
```

Release build:

```bash
cargo build --release
```

Install updated binary globally:

```bash
cargo install --path . --force
```

Format:

```bash
cargo fmt
```

Lint:

```bash
cargo clippy
```

---

# Project Structure

```text
src/
├── main.rs
├── git.rs
├── worktree.rs
├── ui.rs
└── config.rs
```

---

# License

[MIT](LICENSE.md)
