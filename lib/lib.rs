extern crate postgres;
extern crate chrono;
extern crate sha1;
#[macro_use]
extern crate log;

mod error;
mod models;
mod versions;
mod migrations;

use postgres::Connection;
use error::ScurryError;

pub use versions::{Version, DesiredVersion};


pub fn migrate(conn: &Connection, migrations_dir: &str, desired_version: DesiredVersion) -> Result<(), ScurryError> {
    let versions = try!(migrations::calculate_available_versions(migrations_dir));
    info!("Found {} migrations.", versions.len());
    let mut res = try!(models::ScurryMetadata::get_all(conn));
    if let None = res {
        try!(models::ScurryMetadata::create_metadata_table(&conn));
        res = try!(models::ScurryMetadata::get_all(conn));
    }
    if let Some(history) = res {
        try!(migrations::verify_common_history(&versions, &history.version_history));
        let latest_version = history.latest_version();
        match latest_version {
            None => {
                info!("No existing versions found.");
            },
            Some(rev) => {
                info!("Schema at version {}", rev.script_version);
            }
        }
        let upgrade_path = migrations::choose_upgrade_path(&versions, &latest_version, &desired_version);
        info!("Applying {} migrations.", upgrade_path.len());
        for v in upgrade_path {
            info!("Applying version {}...", &v.version);
            try!(migrations::apply_version(&conn, &v));
            info!("Applied version {}.", &v.version);
        }
    } else {
        return Err(ScurryError::Consistency("Could not create metadata table".into()));
    }

    Ok(())
}
