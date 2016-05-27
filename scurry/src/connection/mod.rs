use error::ScurryError;
use versions::{Version, DesiredVersion};
use models::ScurryMetadata;
pub use util::HistoryDifferences;

#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "sqlite")]
pub mod sqlite;

pub trait ScurryConnection : Sized {
    type DbConnection: Sized;
    fn get_history(&self) -> Result<Vec<ScurryMetadata>, ScurryError>;
    fn override_versions(&self, versions: &[&Version]) -> Result<(), ScurryError>;
    fn get_differences(&self) -> Result<Vec<HistoryDifferences>, ScurryError>;
    fn set_schema_level(&self, desired_version: DesiredVersion) -> Result<(), ScurryError>;
    fn migrate(&mut self, desired_version: DesiredVersion) -> Result<usize, ScurryError>;
    fn take_connection(self) -> Self::DbConnection;
}
