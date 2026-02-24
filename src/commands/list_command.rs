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
            return Err(Box::new(AppError(
                "Flags --only-associated (-a) and --only-unassociated (-u) are mutually exclusive.".to_string()
            )));
        }

        let mut print_newline = false;

        if only_associated || !only_unassociated {
            let associated = self.get_associated_proj();
            if !associated.is_empty() {
                print_newline = true;
                println!("ASSOCIATED PROJECTS:");
                for proj in associated {
                    println!("{proj}");
                }
            }
        }

        if only_unassociated || !only_associated {
            let unassociated = self.get_unassociationed_proj()?;
            if !unassociated.is_empty() {
                if print_newline {
                    println!()
                }

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
        self.projects_retriever.get_unassociated_projects()
    }
}
