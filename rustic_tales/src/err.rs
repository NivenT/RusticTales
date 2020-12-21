use std::fmt;

pub type Result<T> = std::result::Result<T, RTError>;

#[derive(Debug)]
pub enum RTError {
    IOError(std::io::Error),
    InvalidInput(String),
}

impl fmt::Display for RTError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use RTError::*;
        match self {
            IOError(e) => write!(f, "I/O Error: {}", e),
            InvalidInput(r) => write!(f, "Invalid input: {}", r),
        }
    }
}

impl From<std::io::Error> for RTError {
    fn from(e: std::io::Error) -> Self {
        RTError::IOError(e)
    }
}
