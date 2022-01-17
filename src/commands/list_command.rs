use std::error::Error;

use crate::{config::projects::ProjectsRetriever, error::AppError};

pub struct ListCommand<'a> {
    projects_retriever: &'a ProjectsRetriever<'a>,
}

impl<'a> ListCommand<'a> {
    pub fn new(projects_retriever: &'a ProjectsRetriever) -> Self {
        ListCommand { projects_retriever }
    }

    pub fn list(
        &self,
        only_associated: bool,
        only_unassociated: bool,
    ) -> Result<(), Box<dyn Error>> {
        if only_associated && only_unassociated {
            return Err(Box::new(AppError(format!(
                "The --only-associated (-a) and --only-unassociated (-u) flags cannot be both set to true"
            ))));
        }

        println!("");

        if only_associated || !only_unassociated {
            let associated = self.get_associated_proj();
            if associated.len() != 0 {
                println!("ASSOCIATED PROJECTS:");
                for proj in associated {
                    println!("{proj}");
                }
                println!("");
            }
        }

        if only_unassociated || !only_associated {
            let unassociated = self.get_unassociationed_proj()?;

            if unassociated.len() != 0 {
                println!("UNASSOCIATED PROJECTS:");
                for proj in unassociated {
                    println!("{proj}");
                }
            }
        }

        Ok(())
    }

    fn get_associated_proj(&self) -> Vec<String> {
        self.projects_retriever.get_associated_projects()
    }

    fn get_unassociationed_proj(&self) -> Result<Vec<String>, Box<dyn Error>> {
        Ok(self.projects_retriever.get_unassociated_projects()?)
    }
}
