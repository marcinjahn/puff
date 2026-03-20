use anyhow::Result;
use inquire::{
    validator::{StringValidator, Validation},
    Confirm, Select, Text,
};

pub fn confirm(question: String) -> Result<bool> {
    Ok(Confirm::new(&question).with_default(false).prompt()?)
}

#[derive(Clone)]
struct NonEmptyValidator;

impl StringValidator for NonEmptyValidator {
    fn validate(&self, input: &str) -> std::result::Result<Validation, Box<dyn std::error::Error + Send + Sync>> {
        if input.trim().is_empty() {
            Ok(Validation::Invalid("Input cannot be empty".into()))
        } else {
            Ok(Validation::Valid)
        }
    }
}

pub fn prompt_input(prompt: &str, default: Option<String>) -> Result<String> {
    let mut input = Text::new(prompt).with_validator(NonEmptyValidator);
    if let Some(ref def) = default {
        input = input.with_default(def);
    }
    Ok(input.prompt()?)
}

pub fn prompt_select(prompt: &str, items: &[String]) -> Result<usize> {
    let items: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
    let choice = Select::new(prompt, items.clone()).prompt()?;
    Ok(items.iter().position(|&s| s == choice).unwrap_or(0))
}
