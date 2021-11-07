use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Debug, Clone)]
pub enum ConfigError {
    KeyMissingError(String),
    TypeError(String, String),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            ConfigError::KeyMissingError(ref field_name) => write!(f, "Field {} missing", field_name),
            ConfigError::TypeError(ref field_name, ref field_value) => write!(f, "Value {} for field {} has wrong type", field_name, field_value),
        }
    }
}

impl Error for ConfigError {
    fn description(&self) -> &str {
        "NO DESCRIPTION"
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}