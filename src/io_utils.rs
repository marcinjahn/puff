use anyhow::Result;

pub fn confirm(question: String) -> Result<bool> {
    print!("{question}");
    println!(" (y/N)");

    let mut choice = String::new();
    std::io::stdin().read_line(&mut choice)?;

    if choice == "y\n" || choice == "Y\n" {
        return Ok(true);
    }

    Ok(false)
}
