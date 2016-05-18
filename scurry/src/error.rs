use std::io::Error as IoError;
use std::error::Error;

#[derive(Debug)]
pub enum ScurryError {
    Io(IoError),
    Parse(String),
    Sql(Box<Error>),
    Consistency(String),
}

impl From<IoError> for ScurryError {
    fn from(e: IoError) -> ScurryError {
        ScurryError::Io(e)
    }
}
