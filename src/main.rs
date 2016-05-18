extern crate scurry;
extern crate postgres;
extern crate env_logger;
#[macro_use]
extern crate log;
use postgres::{Connection, SslMode};
use scurry::connection::ScurryConnection;


fn main() {
    env_logger::init().unwrap();
    let pg_conn = Connection::connect("postgres://bsundsrud@localhost/scurry_test", SslMode::None)
                      .unwrap();
    let conn = scurry::connection::postgres::PostgresScurryConnection::new(&pg_conn).unwrap();
    let res = scurry::migrate(&conn, "migrations", scurry::DesiredVersion::Latest);
    match res {
        Ok(migrations) => {
            if migrations > 0 {
                info!("Successful migration, committing changes...");
            }
            match conn.commit() {
                Ok(_) => {
                    info!("Commit successful.");
                },
                Err(e) => {
                    error!("Commit failed! {:?}", e);
                }
            }
        },
        Err(e) => {
            error!("Migration failed! {:?}", e);
            match conn.rollback() {
                Ok(_) => {
                    info!("Successfully rolled back");
                },
                Err(e) => {
                    error!("Rollback failed! {:?}", e);
                }
            }
        }
    }
}
