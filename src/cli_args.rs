use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct AppArgs {
    /// The path that conamn will treat as a base path for all its data storage (configs, projects)
    #[structopt(default_value = "default", env, hidden = true)]
    pub config_path: String,

    #[structopt(subcommand)]
    pub command: Command,
}

#[derive(StructOpt)]
#[structopt(
    name = "conman",
    about = "A configuration manager that keeps private configuration files from various projects in a central directory so that they can be easily synced between different dev machines."
)]
pub enum Command {
    /// Initializes the project.
    Init,

    /// Adds a new file to be tracked by conman in this project.
    /// If the file does not exist, it will be created.
    Add {
        /// File to be added
        file: PathBuf, // TODO: Vec<PathBuf>

        /// Adds the new file to .gitignore (.gitignre will be created if it doesn't exist)
        #[structopt(short = "g", long = "git-ignore")]
        git_ignore: bool
    },

    /// Removes a file from conman. The file will stay in
    /// project's directory unless the --delete flag is added
    Rm {
        /// File to be removed from conman
        file: PathBuf,

        /// Removes the file from the host
        #[structopt(short = "d", long = "delete")]
        delete_file: bool
    },

    /// Lists all projects known to conman (both associated and unassociated ones)
    List(ListSubcommand),

    /// Subcommand for managing projects
    Project(ProjectSubcommand)
}

#[derive(StructOpt)]
pub struct ListSubcommand {
    /// Retrieve only the unassociated projects
    #[structopt(short = "u")]
    pub only_unassociated: bool,

    /// Retrieve only the associated projects
    #[structopt(short = "a")]
    pub only_associated: bool
}

#[derive(StructOpt)]
pub enum ProjectSubcommand {

    /// Removes a project. By default, all project's files managed by conman will be moved into the associated path (if the project is associated with any path)
    Rm(ProjectRmSubcommand)
}


#[derive(StructOpt)]
pub struct ProjectRmSubcommand {
    /// Project to remove
    #[structopt()]
    pub project_name: String, // TODO: Vec<PathBuf>

    /// Deletes the managed files from the filesystem
    #[structopt(short = "d", long = "delete-files")]
    pub delete_files: bool,

    /// Skips the Y/N question
    #[structopt(short = "y")]
    pub skip_confirmation: bool
}