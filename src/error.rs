use std::{error::Error, fmt};

#[derive(Debug)]
pub struct AppError(pub(crate) String);

impl Error for AppError {}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
