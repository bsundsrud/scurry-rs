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

use connection::ScurryConnection;
use error::ScurryError;
use models::ScurryMetadata;

pub use versions::{Version, DesiredVersion};

pub fn migrate(conn: &ScurryConnection,
               migrations_dir: &str,
               desired_version: DesiredVersion)
               -> Result<usize, ScurryError> {
    let versions = try!(util::calculate_available_versions(migrations_dir));
    info!("Found {} migrations", versions.len());
    let history = try!(conn.get_history());
    try!(util::verify_common_history(&versions, &history));
    let latest_version = history.iter().last();
    match latest_version {
        None => {
            info!("Schema at version 0");
        }
        Some(rev) => {
            info!("Schema at version {}", rev.script_version);
        }
    }

    let upgrade_path = util::choose_upgrade_path(&versions, &latest_version, &desired_version);
    let upgrade_len = upgrade_path.len();
    info!("Applying {} migrations", upgrade_len);
    for v in upgrade_path {
        info!("Applying version {}...", &v.version);
        try!(conn.apply_migration(&v));
    }

    Ok(upgrade_len)
}

pub fn get_differences(conn: &ScurryConnection, migrations_dir: &str) -> Result<Vec<HistoryDifferences>, ScurryError> {
    let available = try!(util::calculate_available_versions(migrations_dir));
    let installed = try!(conn.get_history());
    Ok(util::get_history_differences(&available, &installed))
}

pub fn get_available_versions(migrations_dir: &str) -> Result<Vec<Version>, ScurryError> {
    util::calculate_available_versions(migrations_dir)
}

pub fn get_schema_history(conn: &ScurryConnection) -> Result<Vec<ScurryMetadata>, ScurryError> {
    conn.get_history()
}

pub fn set_schema_level(conn: &ScurryConnection, migrations_dir: &str, desired_version: DesiredVersion) -> Result<(), ScurryError> {
    let versions = try!(util::calculate_available_versions(migrations_dir));
    let upgrade_path = util::choose_upgrade_path(&versions, &None, &desired_version);
    try!(conn.override_versions(&upgrade_path));
    Ok(())
}
