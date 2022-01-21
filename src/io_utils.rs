use std::error::Error;

pub fn confirm(question: String) -> Result<bool, Box<dyn Error>> {
    print!("{question}");
    println!(" (y/N)");
    
    let mut choice = String::new();
    std::io::stdin().read_line(&mut choice)?;

    if choice == "y\n" || choice == "Y\n" {
        return Ok(true)
    }

    Ok(false)
}