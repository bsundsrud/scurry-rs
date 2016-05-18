use error::ScurryError;
use versions::Version;
use models::ScurryMetadata;

#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "sqlite")]
pub mod sqlite;

pub trait ScurryConnection {
    fn apply_migration(&self, version: &Version) -> Result<(), ScurryError>;
    fn create_metadata_table(&self) -> Result<(), ScurryError>;
    fn get_history(&self) -> Result<Vec<ScurryMetadata>, ScurryError>;
    fn commit(self) -> Result<(), ScurryError>;
    fn rollback(self) -> Result<(), ScurryError>;
}
