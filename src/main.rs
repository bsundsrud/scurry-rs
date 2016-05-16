extern crate scurry;
extern crate postgres;
extern crate env_logger;
#[macro_use]
extern crate log;
use postgres::{Connection, SslMode};


fn main() {
    env_logger::init().unwrap();
    let conn = Connection::connect("postgres://bsundsrud@localhost/scurry_test", SslMode::None).unwrap();
    let res = scurry::migrate(&conn, "migrations", scurry::DesiredVersion::Latest);
    info!("{:?}", res);
}
