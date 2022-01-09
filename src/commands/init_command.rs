use crate::{
    config::{
        locations,
        projects::{self, get_unassociated_projects},
    },
    error::AppError,
    project_init::{existing, fresh},
};
use std::{env, error::Error, path::Path};

enum UserChoice<'a> {
    Fresh,
    Existing(&'a str),
}

pub fn init() -> Result<(), Box<dyn Error>> {
    let cwd = env::current_dir()?;
    if projects::is_associated(&cwd)? {
        return Err(Box::new(AppError(
            "This project is already configured in conman".into(),
        )));
    }

    let unassociated = get_unassociated_projects()?;
    if !unassociated.is_empty() {
        handle_with_unassociated(unassociated, &cwd)?;
    } else {
        let name = get_fresh_project_name(&cwd)?;
        handle_fresh_project(&name)?;
    }

    println!("Project has been set up with conman");

    Ok(())
}

fn handle_with_unassociated(unassociated: Vec<String>, cwd: &Path) -> Result<(), Box<dyn Error>> {
    println!("conman has a few projects that are still not associated with any path on your machine. Do you want");
    println!("to associate one of them with the current path, or do you want to set up a fresh project?");
    let choice = ask_about_unassociated(&unassociated)?;
    match choice {
        UserChoice::Fresh => {
            let name = get_fresh_project_name(cwd)?;
            handle_fresh_project(&name)?;
        }
        UserChoice::Existing(name) => existing::init_project(name, cwd)?,
    }

    Ok(())
}

fn get_fresh_project_name(cwd: &Path) -> Result<String, Box<dyn Error>> {
    let mut proposed_name = String::new();
    if let Some(osstr) = cwd.file_name() {
        if let Some(osstr) = osstr.to_str() {
            proposed_name = osstr.to_owned();
        }
    }

    if !proposed_name.is_empty() {
        println!("Provide a name for this new project ({}): ", proposed_name);
    } else {
        println!("Provide a name for this new project: ");
    }
    
    let mut user_name = String::new();
    std::io::stdin().read_line(&mut user_name)?;
    user_name = user_name.trim().to_owned();

    if !user_name.is_empty() {
        Ok(user_name)
    } else if !proposed_name.is_empty() {
        Ok(proposed_name)
    } else {
        println!("The provided name cannot be empty.");
        get_fresh_project_name(cwd)
    }
}

fn ask_about_unassociated(unassociated: &[String]) -> Result<UserChoice, Box<dyn Error>> {
    println!("0) Set up a fresh project");
    for (i, project) in unassociated.iter().enumerate() {
        println!("{}) Associate with the project '{}'", i + 1, project);
    }

    println!("Which option do you select? (awaiting input...)");
    print!("> ");

    let mut choice = String::new();
    std::io::stdin().read_line(&mut choice)?;

    if choice == "0" {
        return Ok(UserChoice::Fresh);
    }

    for (i, project) in unassociated.iter().enumerate() {
        if choice == (i + 1).to_string() {
            return Ok(UserChoice::Existing(project));
        }
    }

    println!("Provided option {} is uncrecognized. Please choose one of the below or press CTRL+C to cancel.", choice);

    ask_about_unassociated(unassociated)
}

fn handle_fresh_project(name: &str) -> Result<(), Box<dyn Error>> {
    let location = locations::get_project_config_path(name)?;
    fresh::init_project(name, &location)?;
    Ok(())
}
