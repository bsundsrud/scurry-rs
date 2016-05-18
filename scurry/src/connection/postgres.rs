use postgres::Connection;
use postgres::rows::Row;
use postgres::error::Error as PgError;
use error::ScurryError;
use models::ScurryMetadata;
use versions::Version;
use util;
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

const INSERT_HISTORY_LINE: &'static str = "INSERT INTO _scurry(script_hash, script_name, script_version) values($1, $2, $3);";

const GET_ALL_REVISIONS: &'static str = "SELECT id, migration_date, script_hash, script_name, script_version FROM _scurry ORDER BY script_version ASC;";

const ACQUIRE_LOCK: &'static str = "LOCK TABLE _scurry IN ACCESS EXCLUSIVE MODE;";

pub struct PostgresScurryConnection<'a> {
    xact: Transaction<'a>
}

impl<'a> PostgresScurryConnection<'a> {

    fn history_table_exists(&self) -> Result<bool, ScurryError> {
        let exists = try!(self.xact.query(METADATA_EXISTS, &[]));
        for row in &exists {
            let res: bool = row.get(0);
            return Ok(res);
        }
        Ok(false)
    }

    fn write_history_line(&self, version: &Version) -> Result<(), ScurryError> {
        try!(self.xact.execute(INSERT_HISTORY_LINE,
                               &[&version.hash, &version.name, &version.version]));
        Ok(())
    }

    fn lock_table(&self) -> Result<(), ScurryError> {
        let exists = try!(self.history_table_exists());
        if !exists {
            try!(self.create_metadata_table());
        }
        try!(self.xact.execute(ACQUIRE_LOCK, &[]));
        info!("Locked table for updating");
        Ok(())
    }

    pub fn new(conn: &'a Connection) -> Result<PostgresScurryConnection<'a>, ScurryError> {
        let xact = try!(conn.transaction());
        let conn = PostgresScurryConnection { xact: xact };
        try!(conn.lock_table());
        Ok(conn)
    }
}

impl<'a> ScurryConnection for PostgresScurryConnection<'a> {
    fn create_metadata_table(&self) -> Result<(), ScurryError> {
        try!(self.xact.batch_execute(CREATE_METADATA_TABLE));
        info!("Metadata table created");
        Ok(())
    }

    fn apply_migration(&self, version: &Version) -> Result<(), ScurryError> {
        let xact = try!(self.xact.transaction());
        let contents = try!(util::get_file_contents(&version.path));
        try!(self.xact.batch_execute(&contents));
        try!(self.write_history_line(&version));
        try!(xact.commit());
        Ok(())
    }

    fn get_history(&self) -> Result<Vec<ScurryMetadata>, ScurryError> {
        let revisions_query = try!(self.xact.query(GET_ALL_REVISIONS, &[]));
        Ok(revisions_query.iter().map(ScurryMetadata::from).collect::<Vec<_>>())
    }

    fn commit(self) -> Result<(), ScurryError> {
        try!(self.xact.commit());
        Ok(())
    }

    fn rollback(self) -> Result<(), ScurryError> {
        try!(self.xact.finish());
        Ok(())
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
