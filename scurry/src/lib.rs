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
