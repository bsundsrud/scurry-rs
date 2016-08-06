//! # Scurry
//! Forward-only embeddable SQL migrations for Rust.  Supports Postgres and Sqlite via rust
//! features.
//!
//! Postgres is enabled by default, use `default-features = false` and `features = ["sqlite"]` in
//! the scurry dependency section to use SQLite instead.
//!
//! ## Embedded Migrations
//! This library is meant to be embedded in applications and called at startup to perform automatic
//! migration.  It also supports multiple applications/single database by locking the metadata
//! table before performing the upgrade, ensuring only one process updates the DB while the others
//! wait for the upgrade to complete.  **Note:** Concurrent updates are not available to SQLite.
//!
//! ### Creating the Connection
//! Use `scurry::from_postgres(conn: postgres::Connection, migrations_dir: &str)` method to create
//! a new `ScurryConnection`, or for SQLite use the companion `from_sqlite` function.
//!
//! ### Using the Connection
//! Create a `DesiredVersion` instance for the version you would like
//! (`DesiredVersion::Specific(version)` for a specific version and `DesiredVersion::Latest` for
//! latest) and call the `connection.migrate(DesiredVersion)` method.  The return value will be a
//! `Result<usize, ScurryError>`, with the `usize` being the number of migrations applied.
//!
//! ### Migration Versioning
//! Migrations in the migration directory are identified by a `.sql` extension.  The format of the
//! filename is `<version>__<name>.sql`.  Note the double underscore.  The contents of the file
//! will be executed in a transaction against the database.  Versions are treated as strings and
//! will be run in ascending lexographical order.
//!
//! ### Version Hashing
//! When migrations are applied, the contents are hashed and stored in the metadata table.  Before
//! migrating, the existing migrations are checked against the database's metadata table for
//! mismatched hashes or differing versions.  If discrepencies are found, the migration process
//! will abort.  Version history can be forced by use of the `ScurryConnection.set_schema_level()`
//! method, which will re-write history as though all history matches up to the given version.
//!
#[cfg(feature = "postgres")] extern crate postgres;
#[cfg(feature = "sqlite")] extern crate rusqlite;
extern crate chrono;
extern crate sha1;
#[macro_use]
extern crate log;

pub mod error;
pub mod models;
pub mod versions;
mod util;
pub mod connection;

pub use util::HistoryDifferences;

pub use versions::{Version, DesiredVersion};

use error::ScurryError;

/// Returns a list of versions available in the given directory.  Errors if there is an IOError,
/// or if version information cannot be parsed from the filename.
pub fn get_available_versions(migrations_dir: &str) -> Result<Vec<Version>, ScurryError> {
    util::calculate_available_versions(migrations_dir)
}

/// Creates a new connection for migrating Postgres databases
#[cfg(feature = "postgres")]
pub fn from_postgres(pg_conn: postgres::Connection, migrations_dir: &str) -> connection::postgres::Postgres {
    connection::postgres::establish(pg_conn, migrations_dir)
}

/// Creates a new connection for migrating SQLite databases
#[cfg(feature = "sqlite")]
pub fn from_sqlite(sqlite_conn: rusqlite::Connection, migrations_dir: &str) -> connection::sqlite::Sqlite {
    connection::sqlite::establish(sqlite_conn, migrations_dir)
}
