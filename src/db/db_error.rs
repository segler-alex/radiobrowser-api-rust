use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Debug, Clone)]
pub enum DbError {
    //ConnectionError(String),
    VoteError(String),
    AddStationError(String),
    IllegalOrderError(String),
}

impl Display for DbError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            //DbError::ConnectionError(ref v) => write!(f, "ConnectionError '{}'", v),
            DbError::VoteError(ref v) => write!(f, "VoteError '{}'", v),
            DbError::AddStationError(ref v) => write!(f, "AddStationError '{}'", v),
            DbError::IllegalOrderError(ref v) => write!(f, "IllegalOrderError '{}'", v),
        }
    }
}

impl Error for DbError {}
