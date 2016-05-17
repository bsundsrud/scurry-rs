use std::io::Error as IoError;
use postgres::error::Error as PgError;

#[derive(Debug)]
pub enum ScurryError {
    Io(IoError),
    Parse(String),
    Sql(PgError),
    Consistency(String),
}

impl From<IoError> for ScurryError {
    fn from(e: IoError) -> ScurryError {
        ScurryError::Io(e)
    }
}

impl From<PgError> for ScurryError {
    fn from(e: PgError) -> ScurryError {
        ScurryError::Sql(e)
    }
}
