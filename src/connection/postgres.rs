use postgres::Connection;
use postgres::rows::Row;
use postgres::error::Error as PgError;
use error::ScurryError;
use models::ScurryMetadata;
use versions::{Version, DesiredVersion};
use util::{self, HistoryDifferences};
use connection::ScurryConnection;
use postgres::transaction::Transaction;

const METADATA_EXISTS: &'static str = "
SELECT EXISTS (
    SELECT 1 FROM pg_catalog.pg_class c
    where c.relname = '_scurry'
    and c.relkind = 'r');";

const CREATE_METADATA_TABLE: &'static str = "
CREATE TABLE _scurry (
    id serial,
    migration_date TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    script_hash TEXT NOT NULL,
    script_name TEXT NOT NULL,
    script_version TEXT NOT NULL
);";

const INSERT_HISTORY_LINE: &'static str = "INSERT INTO _scurry(script_hash, script_name, \
                                           script_version) values($1, $2, $3);";

const GET_ALL_REVISIONS: &'static str = "SELECT id, migration_date, script_hash, script_name, \
                                         script_version FROM _scurry ORDER BY script_version ASC;";

const ACQUIRE_LOCK: &'static str = "LOCK TABLE _scurry IN ACCESS EXCLUSIVE MODE;";

const DELETE_HISTORY: &'static str = "DELETE FROM _scurry;";

pub struct Postgres {
    conn: Connection,
    migrations_dir: String,
}
pub fn establish(conn: Connection, migrations_dir: &str) -> Postgres {
    Postgres {
        conn: conn,
        migrations_dir: migrations_dir.into(),
    }
}

fn history_table_exists(xact: &Transaction) -> Result<bool, ScurryError> {
    let exists = try!(xact.query(METADATA_EXISTS, &[]));
    for row in &exists {
        let res: bool = row.get(0);
        return Ok(res);
    }
    Ok(false)
}

fn write_history_line(xact: &Transaction, version: &Version) -> Result<(), ScurryError> {
    try!(xact.execute(INSERT_HISTORY_LINE,
                      &[&version.hash, &version.name, &version.version]));
    Ok(())
}

fn lock_table(xact: &Transaction) -> Result<(), ScurryError> {
    try!(xact.execute(ACQUIRE_LOCK, &[]));
    info!("Locked table for updating");
    Ok(())
}

fn create_metadata_table(xact: &Transaction) -> Result<(), ScurryError> {
    let exists = try!(history_table_exists(xact));
    if !exists {
        try!(xact.batch_execute(CREATE_METADATA_TABLE));
        info!("Metadata table created");
    }
    Ok(())
}

fn get_history(xact: &Transaction) -> Result<Vec<ScurryMetadata>, ScurryError> {
    let revisions_query = try!(xact.query(GET_ALL_REVISIONS, &[]));
    Ok(revisions_query.iter().map(ScurryMetadata::from).collect::<Vec<_>>())
}

fn clear_history_table(xact: &Transaction) -> Result<(), ScurryError> {
    try!(xact.execute(DELETE_HISTORY, &[]));
    Ok(())
}

fn apply_migration(xact: &Transaction, version: &Version) -> Result<(), ScurryError> {
    let sub_xact = try!(xact.transaction());
    let contents = try!(util::get_file_contents(&version.path));
    try!(sub_xact.batch_execute(&contents));
    try!(write_history_line(&sub_xact, &version));
    try!(sub_xact.commit());
    Ok(())
}

impl ScurryConnection for Postgres {
    type DbConnection = Connection;

    fn migrate(&mut self, desired_version: DesiredVersion) -> Result<usize, ScurryError> {
        let versions = try!(util::calculate_available_versions(&self.migrations_dir));
        info!("Found {} migrations", versions.len());
        let xact = try!(self.conn.transaction());
        try!(create_metadata_table(&xact));
        try!(lock_table(&xact));
        let history = try!(get_history(&xact));
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
            try!(apply_migration(&xact, &v));
        }
        try!(xact.commit());
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
        let revisions_query = try!(self.conn.query(GET_ALL_REVISIONS, &[]));
        Ok(revisions_query.iter().map(ScurryMetadata::from).collect::<Vec<_>>())
    }

    fn override_versions(&self, versions: &[&Version]) -> Result<(), ScurryError> {
        let xact = try!(self.conn.transaction());
        try!(clear_history_table(&xact));
        for v in versions {
            try!(write_history_line(&xact, v));
        }
        try!(xact.commit());
        Ok(())
    }

    fn take_connection(self) -> Connection {
        self.conn
    }
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

impl From<PgError> for ScurryError {
    fn from(e: PgError) -> ScurryError {
        ScurryError::Sql(Box::new(e))
    }
}
