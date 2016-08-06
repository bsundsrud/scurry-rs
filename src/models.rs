use chrono::{UTC, DateTime};

#[derive(Debug)]
pub struct ScurryMetadata {
    pub id: i32,
    pub migration_date: DateTime<UTC>,
    pub script_hash: String,
    pub script_name: String,
    pub script_version: String,
}
