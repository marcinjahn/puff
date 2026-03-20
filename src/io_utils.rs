use anyhow::Result;
use dialoguer::Confirm;

pub fn confirm(question: String) -> Result<bool> {
    Ok(Confirm::new()
        .with_prompt(question)
        .default(false)
        .interact()?)
}
