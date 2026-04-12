# 🐡 Puff

[![CI](https://github.com/marcinjahn/puff/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/marcinjahn/puff/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/puff)](https://crates.io/crates/puff)

Puff is a CLI tool that keeps your projects' private configuration files (`.env`,
`appsettings.json`, credentials, etc.) in a central directory and replaces them
with symlinks. Your applications work exactly as before (they don't know the
files are symlinks), and all your private configs live in one place that you
can back up, version-control in a private repo, or copy to a new machine in
seconds.

![Puff demo](./demo/puff-demo.gif)

## Why Puff

Most projects have files that shouldn't be committed to version control:
environment files with API keys, local database credentials, editor configs with
personal preferences. These files are gitignored, which means:

- **They don't transfer between machines.** Set up a new laptop, and you're
  recreating every `.env` file from memory or old backups.
- **They don't survive git worktrees.** Create a worktree and you're missing
  every gitignored file the project needs to run.
- **They're scattered everywhere.** Each project keeps its own private files in
  its own directory, with no central view or backup strategy.

Puff solves all three problems. It moves your private files into a single managed
directory, creates symlinks so your projects still find them where they expect,
and gives you commands to re-link everything on a new machine or in a new
worktree.

Existing tools solve adjacent problems — dotfile managers (chezmoi, GNU Stow)
target personal configs in `$HOME`, secret managers (Doppler, Vault) require
infrastructure, and in-repo encryption (git-crypt, SOPS) keeps secrets in version
control. Puff is different: it's project-scoped, works with any file or
directory, requires zero infrastructure, and has first-class git worktree
support.

## How It Works

**Your project directory:**

```
my-app/
  src/
  .env          -> symlink
  secrets.json  -> symlink
```

**Puff's central storage:**

```
~/.local/share/puff/projects/my-app/
  .env            (actual file)
  secrets.json    (actual file)
```

1. You tell puff which files to manage (`puff add`).
2. Puff moves them to its central storage and creates symlinks in their place.
3. Your application reads the symlink transparently, no code changes needed.
4. On a new machine (or in a new worktree), `puff init` or `puff link` recreates
   the symlinks.

Puff also supports managing entire directories, not just individual files.

## Getting Started

### 1. Initialize a project

```sh
cd /path/to/my-app
puff init -n my-app
```

This registers the project with puff. If you omit `-n`, puff will prompt you for
a name interactively.

### 2. Add files to puff

```sh
puff add .env -g
puff add config/secrets.json
```

The `-g` flag also adds the path to `.gitignore`. After this, `.env` is a
symlink pointing to puff's central storage. The original file contents are
preserved.

If the file doesn't exist yet, puff creates an empty one in its storage and
symlinks to it.

To add a directory:

```sh
puff add config/local/
```

Puff detects existing directories automatically. For paths that don't exist yet,
use `--dir` to indicate you want a directory, not a file.

### 3. Check what puff manages

```sh
puff status
```

This shows the project name and all managed files and directories for the current
project.

### 4. Set up on a new machine

Copy puff's data directory (see [Storage Locations](#storage-locations)) to the
same location on the new machine, install puff, then initialize your projects.
You can also [keep the data directory in a private Git
repo](#syncing-puff-configs-via-a-private-git-repository) to make syncing easier.

```sh
cd /path/to/my-app
puff init --associate my-app
```

Puff recognizes the project configs you copied over and creates all the symlinks.
If you run `puff init` without `--associate`, puff will interactively ask whether
you want to create a fresh project or associate with one of the existing
unassociated configs.

## Installation

### Homebrew (Linux and macOS, recommended)

```sh
brew install marcinjahn/tap/puff
```

This builds puff from source and installs shell completions automatically.

### Cargo

```sh
cargo install puff
```

This builds puff from source and places the binary in `~/.cargo/bin/`.

### GitHub Releases

Pre-built binaries are available on the
[Releases](https://github.com/marcinjahn/puff/releases) page for Linux, macOS,
and Windows.

Download the archive for your platform, extract it, and place the `puff` binary
somewhere in your `$PATH` (e.g. `~/.local/bin` on Linux).

**macOS note:** The first time you run a downloaded binary, macOS may block it
with a "developer cannot be verified" warning. To allow it: open Finder at the
binary's location, right-click the binary, select _Open_, and confirm.

### Building from Source

```sh
git clone https://github.com/marcinjahn/puff
cd puff
cargo install --path .
```

## Command Reference

| Command                         | Description                                                                                                                             |
| ------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| `puff init`                     | Initialize a project in the current directory. Use `-n <name>` to skip the prompt, or `--associate <name>` to link to existing configs. |
| `puff add <paths...>`           | Add files or directories to puff. Use `-g` to also add to `.gitignore`, `--dir` for non-existing directories.                           |
| `puff forget <paths...>`        | Stop managing files. The files are restored to the project directory (use `-d` to delete them instead).                                 |
| `puff status`                   | Show the puff status of the current directory.                                                                                          |
| `puff list`                     | List all projects. Use `-a` for associated only, `-u` for unassociated only.                                                            |
| `puff link <project>`           | Create symlinks for a project's files in the current directory. Designed for worktrees and secondary working copies.                    |
| `puff project forget <project>` | Remove a project from puff. Files are restored by default (use `-d` to delete).                                                         |
| `puff cd`                       | Open a shell in puff's data directory. Use `-p` to print the path instead.                                                              |
| `puff completions <shell>`      | Generate shell completions (bash, zsh, fish, powershell, elvish).                                                                       |

## Storage Locations

Puff stores managed files and its configuration in OS-standard directories:

| OS      | Data (managed files)                                          | Configuration                                                   |
| ------- | ------------------------------------------------------------- | --------------------------------------------------------------- |
| Linux   | `~/.local/share/puff/projects/`                               | `~/.config/puff/config.json`                                    |
| macOS   | `~/Library/Application Support/com.marcinjahn.puff/projects/` | `~/Library/Application Support/com.marcinjahn.puff/config.json` |
| Windows | `C:\Users\<User>\AppData\Roaming\marcinjahn\puff\projects\`   | `C:\Users\<User>\AppData\Roaming\marcinjahn\puff\config.json`   |

Each project gets its own subdirectory under `projects/`. The `config.json` file
tracks which projects exist and where they're located on disk. When transferring
to a new machine, copy the `projects/` directory but **not** `config.json` (it
contains machine-specific paths), unless your projects will live under the same
paths as on the old machine. Puff will rebuild `config.json` as you run
`puff init` in each project.

## Shell Completions

Puff supports dynamic shell completions (including project name completion). Add
one of the following to your shell configuration:

```sh
# Bash (~/.bashrc)
source <(puff completions bash)

# Zsh (~/.zshrc)
source <(puff completions zsh)

# Fish (~/.config/fish/completions/puff.fish)
puff completions fish | source

# PowerShell ($PROFILE)
puff completions powershell | Invoke-Expression
```

## Recipes

### Syncing Puff Configs via a Private Git Repository

Instead of manually copying the data directory between machines, you can keep it
in a private Git repository (e.g. on GitHub). This gives you version history and
easy syncing.

**Initial setup (first machine):**

```sh
puff cd
# You're now in puff's data directory
cd projects
git init
git remote add origin git@github.com:youruser/puff-configs.git
git add -A
git commit -m "Initial puff configs"
git push -u origin main
```

**On a new machine:**

```sh
# Clone into puff's data directory
puff cd
git clone git@github.com:youruser/puff-configs.git projects
exit

# Then initialize each project
cd /path/to/my-app
puff init --associate my-app
```

**Keeping things in sync:**

After adding or changing managed files, commit and push from the `projects/`
directory. On other machines, pull to get the latest configs. You could automate
this with a cron job or a Git hook, but even doing it manually is straightforward
since everything is in one directory.

Note: make sure the repository is **private**. These files likely contain
secrets.

### Using Puff with Git Worktrees

Git worktrees share the same `.git` directory but get a fresh working copy,
which means gitignored files are missing. Puff's `link` command exists
specifically for this situation.

**Manual workflow:**

```sh
git worktree add ../my-app-feature feature-branch
cd ../my-app-feature
puff link my-app
```

That's it. Puff creates symlinks for all of `my-app`'s managed files in the
worktree directory.

**Automated with a shell function:**

Add this to your shell configuration to create worktrees with puff linking in
one step:

```sh
# Bash/Zsh
worktree-new() {
    local project_name
    project_name=$(basename "$(pwd)")
    git worktree add "$1" "$2" && cd "$1" && puff link "$project_name"
}

# Usage: worktree-new ../my-app-feature feature-branch
```

```fish
# Fish
function worktree-new
    set project_name (basename (pwd))
    git worktree add $argv[1] $argv[2]; and cd $argv[1]; and puff link $project_name
end
```

### Automatic Puff Linking with Claude Code Worktree Hooks

[Claude Code](https://docs.anthropic.com/en/docs/claude-code) can create git
worktrees for subagent isolation. You can configure a hook so that puff
automatically links your project's managed files into every new worktree.

Add the following to your `.claude/settings.json` (or
`.claude/settings.local.json`):

```json
{
  "hooks": {
    "WorktreeCreate": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "bash -c 'INPUT=$(cat); CWD=$(echo \"$INPUT\" | jq -r .cwd); NAME=$(echo \"$INPUT\" | jq -r .name); DIR=\"$HOME/worktrees/$NAME\"; mkdir -p \"$(dirname \"$DIR\")\" && git -C \"$CWD\" worktree add \"$DIR\" HEAD >&2 && PROJECT=$(basename \"$CWD\") && (cd \"$DIR\" && puff link \"$PROJECT\" >&2 || true) && echo \"$DIR\"'"
          }
        ]
      }
    ]
  }
}
```

How this works:

- **`WorktreeCreate`** fires when Claude Code needs an isolated worktree for a
  subagent. It receives JSON on stdin with `cwd` (the repo root) and `name` (a
  unique identifier). The script creates a git worktree at
  `~/worktrees/<name>`, runs `puff link` inside it, and prints the worktree
  path to stdout. Claude Code handles worktree cleanup automatically.
- The `|| true` ensures that if puff linking fails (e.g. the project isn't
  registered with puff), worktree creation still succeeds.

You can adjust the `$HOME/worktrees` path to wherever you prefer worktrees to
live.

## Cross-Platform Support

Puff runs on Linux, macOS, and Windows. Symlink behavior is consistent across
platforms. On Windows, creating symlinks may require Developer Mode to be enabled
or running as administrator.

## License

Puff is licensed under the [Apache License 2.0](./LICENSE).
