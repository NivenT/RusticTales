use std::fmt;

pub type Result<T> = std::result::Result<T, RTError>;

#[derive(Debug)]
pub enum RTError {
    IOError(std::io::Error),
    ParseIntError(std::num::ParseIntError),
    ImgError(image::ImageError),
    InvalidInput(String),
    UnrecognizedCommand(String),
    NotYetImplemented(String),
    Internal(&'static str),
}

impl fmt::Display for RTError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use RTError::*;
        match self {
            IOError(e) => write!(f, "I/O error: {}", e),
            ParseIntError(e) => write!(f, "Parse error: {}", e),
            ImgError(e) => write!(f, "Image error: {}", e),
            InvalidInput(r) => write!(f, "Invalid input: {}", r),
            UnrecognizedCommand(c) => write!(f, "Unrecognized command: {}", c),
            NotYetImplemented(r) => write!(f, "{} is not yet implemented", r),
            Internal(e) => write!(f, "Internal error: {}", e),
        }
    }
}

impl From<std::io::Error> for RTError {
    fn from(e: std::io::Error) -> Self {
        RTError::IOError(e)
    }
}

impl From<std::num::ParseIntError> for RTError {
    fn from(e: std::num::ParseIntError) -> Self {
        RTError::ParseIntError(e)
    }
}

impl From<image::ImageError> for RTError {
    fn from(e: image::ImageError) -> Self {
        RTError::ImgError(e)
    }
}
