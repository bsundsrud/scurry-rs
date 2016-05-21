extern crate scurry;
extern crate postgres;
extern crate rusqlite;
#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

mod termlog;

use clap::{Arg, App, SubCommand};

use postgres::{Connection as PgConnection, SslMode};
use rusqlite::Connection as SqliteConnection;
use scurry::connection::ScurryConnection;
use scurry::DesiredVersion;
use scurry::error::ScurryError;

fn do_postgres_migration(conn_str: &str, migrations_dir: &str, version: DesiredVersion) {
    let pg_conn = match PgConnection::connect(conn_str, SslMode::None) {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed connecting to postgres: {:?}", e);
            std::process::exit(1);
        }
    };
    let conn = match scurry::connection::postgres::PostgresScurryConnection::new(&pg_conn) {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed creating transaction: {:?}", e);
            std::process::exit(1);
        }
    };
    match do_migration(&conn, migrations_dir, version) {
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
                    std::process::exit(1);
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
            };
            std::process::exit(1);
        }
    }

}

fn do_sqlite_migration(db_path: &str, migrations_dir: &str, version: DesiredVersion) {
    let mut sqlite_conn = match SqliteConnection::open(db_path) {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed opening sqlite db: {:?}", e);
            std::process::exit(1);
        }
    };
    let conn = match scurry::connection::sqlite::SqliteScurryConnection::new(&mut sqlite_conn) {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed creating transaction: {:?}", e);
            std::process::exit(1);
        }
    };
    match do_migration(&conn, migrations_dir, version) {
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
                    std::process::exit(1);
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
            };
            std::process::exit(1);
        }
    }
}

fn do_migration<T>(connection_type: &T, migrations_path: &str, version: DesiredVersion) -> Result<usize, ScurryError>
where T: ScurryConnection {
    match version {
        DesiredVersion::Latest => {
            info!("Target version: Latest");
        },
        DesiredVersion::Specific(ref s) => {
            info!("Target version: {}", s);
        }
    }
    scurry::migrate(connection_type, migrations_path, version)
}


fn main() {
    termlog::init().unwrap();
    let matches = App::new("Scurry CLI")
        .about("Forward-only migrations for Postgres and Sqlite")
        .arg(Arg::with_name("migrations")
            .short("d")
            .long("dir")
            .value_name("MIGRATIONS")
            .help("Path to migrations.  Default: ./migrations")
            .takes_value(true))
        .arg(Arg::with_name("version")
            .short("r")
            .long("revision")
            .value_name("VERSION")
            .help("Version to migrate to.  Defaults to latest")
            .takes_value(true))
        .subcommand(SubCommand::with_name("postgres")
            .about("Migrate Postgres DB")
            .arg(Arg::with_name("connect")
                .help("Connection string for Postgres DB")))
        .subcommand(SubCommand::with_name("sqlite")
            .about("Migrate Sqlite DB")
            .arg(Arg::with_name("path")
                .help("Path to Sqlite DB"))).get_matches();

    let migrations_dir = matches.value_of("migrations").unwrap_or("./migrations");
    let version = match matches.value_of("version") {
        Some(v) => {
            if v == "latest" {
                DesiredVersion::Latest
            } else {
                DesiredVersion::Specific(v.into())
            }
        },
        None => DesiredVersion::Latest,
    };
    if let Some(matches) = matches.subcommand_matches("postgres") {
        do_postgres_migration(matches.value_of("connect").unwrap(), migrations_dir, version);
    } else if let Some(matches) = matches.subcommand_matches("sqlite") {
        do_sqlite_migration(matches.value_of("path").unwrap(), migrations_dir, version);
    } else {
        unreachable!();
    };
}
