use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Debug, Clone)]
pub enum DbError {
    ConnectionError(String),
}

impl Display for DbError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            DbError::ConnectionError(ref v) => write!(f, "{}", v),
        }
    }
}

impl Error for DbError {
    fn description(&self) -> &str {
        "invalid first item to double"
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}