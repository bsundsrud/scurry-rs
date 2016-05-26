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

fn get_history<T>(conn: &T) where T: ScurryConnection {
    match conn.get_history() {
        Ok(history) => {
            println!("{:32} {:10} {:20} {:40}", "DATE", "VERSION", "NAME", "HASH");
            for h in history {
                println!("{:32} {:10} {:20} {:40}",
                    &h.migration_date.to_rfc2822(),
                    &h.script_version,
                    &h.script_name,
                    &h.script_hash);
            }
        },
        Err(e) => {
            error!("Error getting history: {:?}", e);
        }
    }
}

fn override_versions<T>(conn: &T, desired_version: DesiredVersion) where T: ScurryConnection {

    match conn.set_schema_level(desired_version) {
        Ok(_) => {
            info!("Schema level set.");
        },
        Err(e) => {
            error!("Could not set schema level: {:?}", e);
            std::process::exit(1);
        }
    }
}

fn do_migration<T>(connection_type: &mut T, version: DesiredVersion) -> Result<usize, ScurryError>
where T: ScurryConnection {
    match version {
        DesiredVersion::Latest => {
            info!("Target version: Latest");
        },
        DesiredVersion::Specific(ref s) => {
            info!("Target version: {}", s);
        }
    }
    connection_type.migrate(version)
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
        .subcommand(SubCommand::with_name("revisions")
            .about("List available versions"))
        .subcommand(SubCommand::with_name("postgres")
            .about("Migrate Postgres DB")
            .arg(Arg::with_name("connect")
                .short("c")
                .long("connect")
                .value_name("CONNECTION_STRING")
                .required(true)
                .help("Connection string for Postgres DB"))
            .subcommand(SubCommand::with_name("mark")
                .about("Set schema version without running migrations"))
            .subcommand(SubCommand::with_name("migrate")
                .about("Migrate schema"))
            .subcommand(SubCommand::with_name("history")
                .about("List installed versions"))
        )
        .subcommand(SubCommand::with_name("sqlite")
            .about("Migrate Sqlite DB")
            .arg(Arg::with_name("path")
                .short("p")
                .long("path")
                .value_name("PATH")
                .required(true)
                .help("Path to Sqlite DB"))
            .subcommand(SubCommand::with_name("mark")
                .about("Set schema version without running migrations"))
            .subcommand(SubCommand::with_name("migrate")
                .about("Migrate schema"))
            .subcommand(SubCommand::with_name("history")
                .about("List installed versions"))
        ).get_matches();

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
        let pg_conn = match PgConnection::connect(matches.value_of("connect").unwrap(), SslMode::None) {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed connecting to postgres: {:?}", e);
                std::process::exit(1);
            }
        };
        let mut conn = scurry::connection::postgres::establish(pg_conn, migrations_dir);

        if let Some(_) = matches.subcommand_matches("migrate") {
            if let Err(e) = do_migration(&mut conn, version) {
                error!("Failed to migrate: {:?}", e);
                std::process::exit(1);
            }

        } else if let Some(_) = matches.subcommand_matches("history") {
            get_history(&conn);
        } else if let Some(_) = matches.subcommand_matches("mark") {
            override_versions(&conn, version);
        }
    } else if let Some(matches) = matches.subcommand_matches("sqlite") {
        let sqlite_conn = match SqliteConnection::open(matches.value_of("path").unwrap()) {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed opening sqlite db: {:?}", e);
                std::process::exit(1);
            }
        };
        let mut conn = scurry::connection::sqlite::establish(sqlite_conn, migrations_dir);
        if let Some(_) = matches.subcommand_matches("migrate") {
            if let Err(e) = do_migration(&mut conn, version) {
                error!("Failed migration: {:?}", e);
                std::process::exit(1);
            }
        } else if let Some(_) = matches.subcommand_matches("history") {
            get_history(&conn);
        } else if let Some(_) = matches.subcommand_matches("mark") {
            override_versions(&conn, version);
        }
    } else if let Some(_) = matches.subcommand_matches("revisions") {
        unimplemented!();
    } else {
        unreachable!();
    };
}
