use rusqlite::{Connection, Error as SqliteError};
use error::ScurryError;
use models::ScurryMetadata;
use versions::{Version, DesiredVersion};
use chrono::UTC;
use connection::ScurryConnection;
use util::{self, HistoryDifferences};

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

pub struct Sqlite {
    conn: Connection,
    migrations_dir: String,
}

pub fn establish(conn: Connection, migrations_dir: &str) -> Sqlite {
    Sqlite {
        conn: conn,
        migrations_dir: migrations_dir.into(),
    }
}

fn history_table_exists(xact: &Connection) -> Result<bool, ScurryError> {
    let mut stmt = try!(xact.prepare(METADATA_EXISTS));
        let exists = try!(stmt.query_map(&[], |_| { true }));

        Ok(exists.count() > 0)
}

fn write_history_line(xact: &Connection, version: &Version) -> Result<(), ScurryError> {
    try!(xact.execute(INSERT_HISTORY_LINE,
                           &[&version.hash, &version.name, &version.version, &UTC::now()]));
    Ok(())
}

fn create_metadata_table(xact: &Connection) -> Result<(), ScurryError> {
    try!(xact.execute_batch(CREATE_METADATA_TABLE));
    info!("Metadata table created");
    Ok(())
}

fn clear_history_table(xact: &Connection) -> Result<(), ScurryError> {
    try!(xact.execute(DELETE_HISTORY, &[]));
    Ok(())
}

fn apply_migration(xact: &mut Connection, version: &Version) -> Result<(), ScurryError> {
    let sub_xact = try!(xact.transaction());
    let contents = try!(util::get_file_contents(&version.path));
    try!(sub_xact.execute_batch(&contents));
    try!(write_history_line(&sub_xact, &version));
    try!(sub_xact.commit());
    Ok(())
}

impl ScurryConnection for Sqlite {
    type DbConnection = Connection;

    fn migrate(&mut self, desired_version: DesiredVersion) -> Result<usize, ScurryError> {
        let versions = try!(util::calculate_available_versions(&self.migrations_dir));
        info!("Found {} migrations", versions.len());
        let history = try!(self.get_history());
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
        let metadata_exists = try!(history_table_exists(&self.conn));
        if !metadata_exists {
            try!(create_metadata_table(&self.conn));
        }
        let upgrade_path = util::choose_upgrade_path(&versions, &latest_version, &desired_version);
        let upgrade_len = upgrade_path.len();
        info!("Applying {} migrations", upgrade_len);
        for v in upgrade_path {
            info!("Applying version {}...", &v.version);
            try!(apply_migration(&mut self.conn, &v));
        }
        Ok(upgrade_len)
    }

    fn get_differences(&self) -> Result<Vec<HistoryDifferences>, ScurryError> {
        let available = try!(util::calculate_available_versions(&self.migrations_dir));
        let installed = try!(self.get_history());
        Ok(util::get_history_differences(&available, &installed))
    }

    fn set_schema_level(&self, desired_version: DesiredVersion) -> Result<(), ScurryError> {
        let versions = try!(util::calculate_available_versions(&self.migrations_dir));
        let upgrade_path = util::choose_upgrade_path(&versions, &None, &desired_version);
        try!(self.override_versions(&upgrade_path));
        Ok(())
    }

    fn get_history(&self) -> Result<Vec<ScurryMetadata>, ScurryError> {

        let exists = try!(history_table_exists(&self.conn));
        if !exists {
            try!(create_metadata_table(&self.conn));
        }
        let mut stmt = try!(self.conn.prepare(GET_ALL_REVISIONS));
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

    fn override_versions(&self, versions: &[&Version]) -> Result<(), ScurryError> {
        try!(clear_history_table(&self.conn));
        for v in versions {
            try!(write_history_line(&self.conn, v));
        }
        Ok(())
    }

    fn take_connection(self) -> Connection {
        self.conn
    }
}

impl From<SqliteError> for ScurryError {
    fn from(e: SqliteError) -> ScurryError {
        ScurryError::Sql(Box::new(e))
    }
}
