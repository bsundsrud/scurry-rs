use chrono::NaiveDateTime;
use postgres::Connection;
use postgres::rows::Row;
use postgres::error::Error;
use std::convert::From;

const METADATA_EXISTS: &'static str = "SELECT EXISTS (
    SELECT 1 FROM pg_catalog.pg_class c
    where c.relname = '_scurry'
    and c.relkind = 'r');";

const CREATE_METADATA_TABLE: &'static str = "CREATE TABLE _scurry (
    id serial,
    migration_date TIMESTAMP NOT NULL,
    script_hash TEXT NOT NULL,
    script_name TEXT NOT NULL,
    script_version TEXT NOT NULL
);";

const GET_ALL_REVISIONS: &'static str = "SELECT id, migration_date, script_hash, script_name, script_version FROM _scurry ORDER BY script_version ASC;";

#[derive(Debug)]
pub struct ScurryMetadata {
    pub id: i32,
    pub migration_date: NaiveDateTime,
    pub script_hash: String,
    pub script_name: String,
    pub script_version: String,
}

impl<'a> From<Row<'a>> for ScurryMetadata {
    fn from(row: Row<'a>) -> ScurryMetadata {
        ScurryMetadata {
            id: row.get(0),
            migration_date: row.get(1),
            script_hash: row.get(2),
            script_name: row.get(3),
            script_version: row.get(4),
        }
    }
}

impl ScurryMetadata {
    pub fn get_all(conn: &Connection) -> Result<Option<DbHistory>, Error> {
        let exists = try!(conn.query(METADATA_EXISTS, &[]));
        for row in &exists {
            let res: bool = row.get(0);
            if !res {
                return Ok(None);
            }
        }
        info!("Found metadata table");
        let revisions_query = try!(conn.query(GET_ALL_REVISIONS, &[]));
        Ok(Some(DbHistory {
            version_history: revisions_query.iter().map(ScurryMetadata::from).collect::<Vec<_>>()
        }))
    }

    pub fn create_metadata_table(conn: &Connection) -> Result<(), Error> {
        try!(conn.execute(CREATE_METADATA_TABLE, &[]));
        info!("Metadata table created");
        Ok(())
    }
}

#[derive(Debug)]
pub struct DbHistory {
    pub version_history: Vec<ScurryMetadata>
}

impl DbHistory {
    pub fn latest_version(&self) -> Option<&ScurryMetadata> {
        self.version_history.iter().last()
    }
}
