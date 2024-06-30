use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct EOFError;

impl fmt::Display for EOFError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "End of file has reached!")
    }
}

impl Error for EOFError {
    fn description(&self) -> &str {
        "End of file has reached!"
    }
}
