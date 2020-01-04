use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Debug, Clone)]
pub enum DbError {
    ConnectionError(String),
    VoteError(String),
}

impl Display for DbError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            DbError::ConnectionError(ref v) => write!(f, "{}", v),
            DbError::VoteError(ref v) => write!(f, "{}", v),
        }
    }
}

impl Error for DbError {
    fn description(&self) -> &str {
        "NO DESCRIPTION"
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}