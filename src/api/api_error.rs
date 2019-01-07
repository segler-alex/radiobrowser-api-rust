use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Debug, Clone)]
pub enum ApiError {
    ConnectionError(String),
    UnknownApiVersion(u32),
}

impl Display for ApiError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            ApiError::ConnectionError(ref v) => write!(f, "{}", v),
            ApiError::UnknownApiVersion(ref v) => write!(f, "UnknownApiVersion {}", v),
        }
    }
}

impl Error for ApiError {
    fn description(&self) -> &str {
        "invalid first item to double"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}