use anyhow::Result;
use std::path::Path;

use crate::config::{locations::LocationsProvider, projects::ProjectsRetriever};

pub struct StatusCommand<'a> {
    locations_provider: &'a LocationsProvider,
    projects_retriever: &'a ProjectsRetriever<'a>,
}

impl<'a> StatusCommand<'a> {
    pub fn new(
        locations_provider: &'a LocationsProvider,
        projects_retriever: &'a ProjectsRetriever<'a>,
    ) -> Self {
        StatusCommand {
            locations_provider,
            projects_retriever,
        }
    }

    pub fn status(&self, cwd: &Path) -> Result<()> {
        let project = self.locations_provider.find_project_for_path(cwd);

        match project {
            Err(_) => {
                println!("Current directory is not managed by any puff project.");
            }
            Ok((project_name, _)) => {
                let Some(details) = self.projects_retriever.get_details(&project_name)? else {
                    println!("Current directory is not managed by any puff project.");
                    return Ok(());
                };

                println!("Project: {}", details.name);
                println!("Managed files:");
                if details.files.is_empty() {
                    println!("  (none)");
                } else {
                    for file in &details.files {
                        println!("  {}", file.display());
                    }
                }
            }
        }

        Ok(())
    }
}
