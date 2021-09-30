use std::fmt;

pub type Result<T> = std::result::Result<T, RTError>;

#[derive(Debug)]
pub enum RTError {
    IOError(std::io::Error),
    ParseIntError(std::num::ParseIntError),
    ImgError(image::ImageError),
    RonError(ron::Error),
    GlobError(globset::Error),
    DurError(humantime::DurationError),
    ReqwestError(reqwest::Error),
    JsonError(serde_json::Error),

    InvalidInput(String),
    UnrecognizedCommand(String),
    WrongNumArguments(&'static str, &'static str, usize),
    #[allow(dead_code)]
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
            RonError(e) => write!(f, "RON error: {}", e),
            GlobError(e) => write!(f, "Glob error: {}", e),
            DurError(e) => write!(f, "Parse duration error: {}", e),
            ReqwestError(e) => write!(f, "Reqwest error: {}", e),
            JsonError(e) => write!(f, "Json error: {}", e),
            InvalidInput(r) => write!(f, "Invalid input: {}", r),
            UnrecognizedCommand(c) => write!(f, "Unrecognized command: {}", c),
            WrongNumArguments(name, exp, got) => {
                write!(f, "'{}' expected {} arguments, but got {}", name, exp, got)
            }
            NotYetImplemented(r) => write!(f, "{} is not yet implemented", r),
            Internal(e) => write!(f, "Internal error: {}", e),
        }
    }
}

// Someone's never heard of a macro
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

impl From<ron::Error> for RTError {
    fn from(e: ron::Error) -> Self {
        RTError::RonError(e)
    }
}

impl From<globset::Error> for RTError {
    fn from(e: globset::Error) -> Self {
        RTError::GlobError(e)
    }
}

impl From<humantime::DurationError> for RTError {
    fn from(e: humantime::DurationError) -> Self {
        RTError::DurError(e)
    }
}

impl From<reqwest::Error> for RTError {
    fn from(e: reqwest::Error) -> Self {
        RTError::ReqwestError(e)
    }
}

impl From<serde_json::Error> for RTError {
    fn from(e: serde_json::Error) -> Self {
        RTError::JsonError(e)
    }
}
