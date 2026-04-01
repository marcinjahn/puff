use anyhow::{Result, bail};
use std::{fs, path::Path};

use crate::{
    config::{
        locations::LocationsProvider,
        projects::{ProjectDetails, ProjectsRetriever},
    },
    project_init::existing::create_symlinks_for_managed_files,
};

pub struct LinkCommand<'a> {
    projects_retriever: &'a ProjectsRetriever<'a>,
    locations_provider: &'a LocationsProvider,
}

impl<'a> LinkCommand<'a> {
    pub fn new(
        projects_retriever: &'a ProjectsRetriever<'a>,
        locations_provider: &'a LocationsProvider,
    ) -> Self {
        LinkCommand {
            projects_retriever,
            locations_provider,
        }
    }

    pub fn link(&self, project_name: &str, cwd: &Path) -> Result<()> {
        let details = self.projects_retriever.get_details(project_name)?;

        let associated = match details {
            None => bail!(
                "Project '{}' was not found. Check 'puff list' for available projects.",
                project_name
            ),
            Some(ProjectDetails::Unassociated(_)) => bail!(
                "Project '{}' is not associated with any directory on this machine. Run 'puff init' first.",
                project_name
            ),
            Some(ProjectDetails::Associated(a)) => a,
        };

        if cwd == associated.user_dir
            || fs::canonicalize(cwd).ok() == fs::canonicalize(&associated.user_dir).ok()
        {
            bail!("You're already in the project's main directory. Nothing to link.");
        }

        if associated.info.items.is_empty() {
            println!("Project '{}' has no managed files.", project_name);
            return Ok(());
        }

        let managed_dir = self.locations_provider.get_managed_dir(project_name);
        create_symlinks_for_managed_files(cwd, &managed_dir)?;

        let count = associated.info.items.len();
        println!(
            "Linked {} item{} from project '{}' into current directory.",
            count,
            if count == 1 { "" } else { "s" },
            project_name
        );

        Ok(())
    }
}
