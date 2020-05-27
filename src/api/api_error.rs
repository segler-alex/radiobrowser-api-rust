use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Debug, Clone)]
pub enum ApiError {
    InternalError(String),
}

impl Display for ApiError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            ApiError::InternalError(ref v) => write!(f, "InternalError '{}'", v),
        }
    }
}

impl Error for ApiError {
    fn description(&self) -> &str {
        "NO DESCRIPTION"
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}