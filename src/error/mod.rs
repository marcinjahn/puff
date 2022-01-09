use std::{error::Error, fmt};

#[derive(Debug)]
pub struct InitError(pub(crate) String);

impl Error for InitError {}

impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "conman initialization failed")
    }
}
