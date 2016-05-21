use rusqlite::{Connection, Error as SqliteError, Transaction};
use error::ScurryError;
use models::ScurryMetadata;
use versions::Version;
use chrono::UTC;
use connection::ScurryConnection;
use util;

const METADATA_EXISTS: &'static str = "
SELECT name FROM sqlite_master WHERE type = 'table' and name = '_scurry';";

const CREATE_METADATA_TABLE: &'static str = "
CREATE TABLE _scurry (
    id INTEGER PRIMARY KEY,
    migration_date TEXT NOT NULL,
    script_hash TEXT NOT NULL,
    script_name TEXT NOT NULL,
    script_version TEXT NOT NULL
);";

const INSERT_HISTORY_LINE: &'static str = "INSERT INTO _scurry(script_hash, script_name, script_version, migration_date) values($1, $2, $3, $4);";

const GET_ALL_REVISIONS: &'static str = "SELECT id, migration_date, script_hash, script_name, script_version FROM _scurry ORDER BY script_version ASC;";

const DELETE_HISTORY: &'static str = "DELETE FROM _scurry;";

pub struct SqliteScurryConnection<'a> {
    xact: Transaction<'a>,
}

impl<'a> SqliteScurryConnection<'a> {
    fn history_table_exists(&self) -> Result<bool, ScurryError> {
        let mut stmt = try!(self.xact.prepare(METADATA_EXISTS));
        let exists = try!(stmt.query_map(&[], |_| { true }));

        Ok(exists.count() > 0)
    }

    fn write_history_line(&self, version: &Version) -> Result<(), ScurryError> {
        try!(self.xact.execute(INSERT_HISTORY_LINE,
                               &[&version.hash, &version.name, &version.version, &UTC::now()]));
        Ok(())
    }

    fn clear_history_table(&self) -> Result<(), ScurryError> {
        try!(self.xact.execute(DELETE_HISTORY, &[]));
        Ok(())
    }

    pub fn new(conn: &'a mut Connection) -> Result<SqliteScurryConnection<'a>, ScurryError> {
        let xact = try!(conn.transaction());

        Ok(SqliteScurryConnection { xact: xact })
    }
}

impl<'a> ScurryConnection for SqliteScurryConnection<'a> {
    fn create_metadata_table(&self) -> Result<(), ScurryError> {
        try!(self.xact.execute_batch(CREATE_METADATA_TABLE));
        info!("Metadata table created");
        Ok(())
    }

    fn apply_migration(&self, version: &Version) -> Result<(), ScurryError> {
        let contents = try!(util::get_file_contents(&version.path));
        try!(self.xact.execute_batch(&contents));
        try!(self.write_history_line(&version));
        Ok(())
    }

    fn get_history(&self) -> Result<Vec<ScurryMetadata>, ScurryError> {
        let exists = try!(self.history_table_exists());
        if !exists {
            try!(self.create_metadata_table());
        }
        let mut stmt = try!(self.xact.prepare(GET_ALL_REVISIONS));
        let revisions = try!(stmt.query_map(&[], |row| {
            ScurryMetadata {
                id: row.get(0),
                migration_date: row.get(1),
                script_hash: row.get(2),
                script_name: row.get(3),
                script_version: row.get(4),
            }
        }));
        let mut result = vec![];
        for revision in revisions {
            let item = try!(revision);
            result.push(item);
        }
        Ok(result)
    }

    fn commit(self) -> Result<(), ScurryError> {
        try!(self.xact.commit());
        Ok(())
    }

    fn rollback(mut self) -> Result<(), ScurryError> {
        self.xact.set_rollback();
        try!(self.xact.finish());
        Ok(())
    }

    fn record_version(&self, version: &Version) -> Result<(), ScurryError> {
        self.write_history_line(version)
    }

    fn override_versions(&self, versions: &[&Version]) -> Result<(), ScurryError> {
        try!(self.clear_history_table());
        for v in versions {
            try!(self.write_history_line(v));
        }
        Ok(())
    }
}

impl From<SqliteError> for ScurryError {
    fn from(e: SqliteError) -> ScurryError {
        ScurryError::Sql(Box::new(e))
    }
}
