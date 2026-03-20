use anyhow::Result;
use dialoguer::{Confirm, Input, Select};

pub fn confirm(question: String) -> Result<bool> {
    Ok(Confirm::new()
        .with_prompt(question)
        .default(false)
        .interact()?)
}

pub fn prompt_input(prompt: &str, default: Option<String>) -> Result<String> {
    let mut input = Input::<String>::new().with_prompt(prompt);
    if let Some(def) = default {
        input = input.default(def);
    }
    Ok(input
        .validate_with(|s: &String| {
            if s.trim().is_empty() {
                Err("Input cannot be empty")
            } else {
                Ok(())
            }
        })
        .interact_text()?)
}

pub fn prompt_select(prompt: &str, items: &[String]) -> Result<usize> {
    Ok(Select::new()
        .with_prompt(prompt)
        .items(items)
        .default(0)
        .interact()?)
}
