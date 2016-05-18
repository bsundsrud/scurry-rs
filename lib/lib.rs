extern crate postgres;
extern crate chrono;
extern crate sha1;
#[macro_use]
extern crate log;

mod error;
mod models;
mod versions;
mod util;
pub mod connection;

use connection::ScurryConnection;
use error::ScurryError;

pub use versions::{Version, DesiredVersion};

pub fn migrate(conn: &ScurryConnection,
               migrations_dir: &str,
               desired_version: DesiredVersion)
               -> Result<usize, ScurryError> {
    let versions = try!(util::calculate_available_versions(migrations_dir));
    info!("Found {} migrations.", versions.len());
    let history = try!(conn.get_history());
    try!(util::verify_common_history(&versions, &history));
    let latest_version = history.iter().last();
    match latest_version {
        None => {
            info!("No existing versions found.");
        }
        Some(rev) => {
            info!("Schema at version {}", rev.script_version);
        }
    }

    let upgrade_path = util::choose_upgrade_path(&versions, &latest_version, &desired_version);
    let upgrade_len = upgrade_path.len();
    info!("Applying {} migrations.", upgrade_len);
    for v in upgrade_path {
        info!("Applying version {}...", &v.version);
        try!(conn.apply_migration(&v));
        info!("Applied version {}.", &v.version);
    }

    Ok(upgrade_len)
}
