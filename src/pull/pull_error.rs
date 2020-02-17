use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Debug, Clone)]
pub enum PullError {
    UnknownApiVersion(u32),
}

impl Display for PullError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            PullError::UnknownApiVersion(ref v) => write!(f, "UnknownApiVersion {}", v),
        }
    }
}

impl Error for PullError {
    fn description(&self) -> &str {
        "invalid first item to double"
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}