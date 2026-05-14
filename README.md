# git-wt

Interactive Git worktree manager written in Rust.

`git-wt` discovers Git repositories inside configured directories, lists their worktrees, and lets you interactively remove them using proper Git commands.

It does **not** manually delete directories.

Internally it uses:

```bash
git worktree remove
```

so Git metadata stays consistent.

---

# Features

- Interactive worktree cleanup
- Multi-select deletion UI
- Recursive repository discovery
- Proper Git worktree removal
- Configurable root directories
- Fast single-binary CLI
- Native Git subcommand support

---

# Prerequisites

You must have:

- Git
- Rust toolchain

---

# Install Git

## Ubuntu

```bash
sudo apt install git
```

## macOS

```bash
brew install git
```

---

# Install Rust

Install Rust using rustup:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Restart your shell afterward.

Verify installation:

```bash
rustc --version
cargo --version
```

---

# Installation

## Clone Repository

```bash
git clone https://github.com/YOUR_USERNAME/git-wt.git
cd git-wt
```

## Build

```bash
cargo build --release
```

Binary:

```bash
target/release/git-wt
```

---

# Global Installation

Install globally:

```bash
cargo install --path .
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
  "/dev",
  "/work"
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
[api-server] feature/auth-redesign /dev/api-server-auth
[frontend] fix/navbar-overflow /dev/frontend-fix
```

Controls:

- `SPACE` → select
- `ENTER` → confirm deletion

---

# Creating Worktrees

If you do not already use Git worktrees, create one manually:

```bash
git worktree add ../my-feature-worktree feature/my-branch
```

Verify:

```bash
git worktree list
```

Example output:

```text
/dev/myrepo                  abc123 [main]
/dev/my-feature-worktree    def456 [feature/my-branch]
```

`git-wt clean` becomes useful once repositories contain additional worktrees.

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

Format:

```bash
cargo fmt
```

Lint:

```bash
cargo clippy
```

---

# Future Ideas

- fuzzy search
- branch age display
- merged branch cleanup
- stale worktree detection
- TUI mode
- GitHub PR integration
- `git wt new`
- `git wt open`

---

# License

MIT
