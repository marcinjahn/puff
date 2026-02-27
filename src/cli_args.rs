use clap::{Args, Parser, Subcommand};
use clap_complete::Shell;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "puff",
    about = "A configuration manager that keeps private configuration files from various projects in a central directory so that they can be easily synced between different dev machines."
)]
pub struct AppArgs {
    /// The path that conamn will treat as a base path for all its data storage (configs, projects)
    #[arg(long, default_value = "default", env = "PUFF_CONFIG_PATH", hide = true)]
    pub config_path: String,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Initializes the project.
    Init,

    /// Adds a new file to be tracked by puff in this project.
    /// If the file does not exist, it will be created.
    Add {
        /// Files to be added
        #[arg(num_args = 1..)]
        files: Vec<PathBuf>,

        /// Adds the new file to .gitignore (.gitignre will be created if it doesn't exist)
        #[arg(short = 'g', long = "git-ignore")]
        git_ignore: bool,
    },

    /// Removes a file from puff. The file will stay in
    /// project's directory unless the --delete flag is added
    Forget {
        /// Files to be removed from puff
        #[arg(num_args = 1..)]
        files: Vec<PathBuf>,

        /// Removes the file from the host
        #[arg(short = 'd', long = "delete")]
        delete_file: bool,
    },

    /// Lists all projects known to puff (both associated and unassociated ones)
    List(ListSubcommand),

    /// Shows the puff status of the current directory
    Status,

    /// Subcommand for managing projects
    Project {
        #[command(subcommand)]
        subcommand: ProjectSubcommand,
    },

    /// Generates shell completions for the given shell and prints them to stdout
    #[command(after_help = "\
Examples:
  bash:       puff completions bash >> ~/.bashrc
  zsh:        puff completions zsh > ~/.zfunc/_puff  (then add ~/.zfunc to $fpath)
  fish:       puff completions fish > ~/.config/fish/completions/puff.fish
  powershell: puff completions powershell >> $PROFILE")]
    Completions {
        /// The shell to generate completions for
        shell: Shell,
    },
}

#[derive(Args)]
pub struct ListSubcommand {
    /// Retrieve only the unassociated projects
    #[arg(short = 'u')]
    pub only_unassociated: bool,

    /// Retrieve only the associated projects
    #[arg(short = 'a')]
    pub only_associated: bool,
}

#[derive(Subcommand)]
pub enum ProjectSubcommand {
    /// Removes a project. By default, all project's files managed by puff will be moved into the associated path (if the project is associated with any path)
    Forget(ProjectForgetSubcommand),
}

#[derive(Args)]
pub struct ProjectForgetSubcommand {
    /// Project to remove
    pub project_name: String, // TODO: Vec<PathBuf>

    /// Deletes the managed files from the filesystem
    #[arg(short = 'd', long = "delete-files")]
    pub delete_files: bool,

    /// Skips the Y/N question
    #[arg(short = 'y')]
    pub skip_confirmation: bool,
}
