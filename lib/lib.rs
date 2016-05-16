extern crate postgres;
extern crate chrono;
extern crate sha1;
#[macro_use]
extern crate log;

mod models;

use std::fs::{self, File};
use std::path::Path;
use std::io::prelude::*;
use std::io::Error as IoError;
use std::convert::From;
use std::ffi::OsStr;

use postgres::Connection;
use postgres::error::Error;

#[derive(Debug)]
pub enum DesiredVersion {
    Latest,
    Specific(String)
}

#[derive(Debug)]
pub enum ScurryError {
    Io(IoError),
    Parse(String),
    Sql(Error)
}

impl From<IoError> for ScurryError {
    fn from(e: IoError) -> ScurryError {
        ScurryError::Io(e)
    }
}

impl From<Error> for ScurryError {
    fn from(e: Error) -> ScurryError {
        ScurryError::Sql(e)
    }
}

#[derive(Debug)]
struct Version {
    path: String,
    name: String,
    hash: String,
    version: String,
}

fn get_name_and_version(path: &Path) -> Result<(String, String), ScurryError> {
    let file_name = match path.file_stem() {
        Some(s) => s,
        None => { return Err(ScurryError::Parse("Could not determine file name".into())) }
    };
    match file_name.to_str() {
        None => Err(ScurryError::Parse("Could not get string path".into())),
        Some(s) => {
            let mut parts = s.split("__");
            let version = match parts.next() {
                Some(s) => s.into(),
                None => {
                    return Err(ScurryError::Parse("No string parts".into()))
                }
            };
            let remaining = parts.collect::<Vec<_>>();
            if remaining.len() == 0 {
                return Err(ScurryError::Parse("Invalid version and name; separate version and name with '__'".into()));
            }
            let name = remaining.join("__");
            Ok((version, name))
        }
    }
}

fn hash_file_contents(path: &Path) -> Result<String, IoError> {
    let mut f = try!(File::open(path));
    let mut buffer = vec![];
    try!(f.read_to_end(&mut buffer));
    let mut m = sha1::Sha1::new();
    m.update(&buffer);
    Ok(m.hexdigest())
}

fn calculate_available_versions(migrations_dir: &str) -> Result<Vec<Version>, ScurryError>  {
    let all_paths = try!(fs::read_dir(migrations_dir));
    let sql_files = all_paths
        .filter_map(|dirent| dirent.ok())
        .map(|dirent| dirent.path())
        .filter(|path| {
            info!("{:?}", path);
            match path.extension() {
                None => false,
                Some(s) => s == "sql"
            }
        });
    let mut res = vec![];
    for file in sql_files {
        info!("Processing file: {:?}", file);
        let hash = try!(hash_file_contents(&file));
        let (version, name) = try!(get_name_and_version(&file));
        let path = match file.to_str() {
            Some(p) => p.into(),
            None => {
                return Err(ScurryError::Parse("couldn't get file path".into()))
            }
        };
        res.push(Version {
            path: path,
            name: name,
            hash: hash,
            version: version
        });
    }
    Ok(res)
}

pub fn migrate(conn: &Connection, migrations_dir: &str, desired_version: DesiredVersion) -> Result<DesiredVersion, ScurryError> {
    let versions = try!(calculate_available_versions(migrations_dir));
    info!("{:?}", versions);
    let res = try!(models::ScurryMetadata::get_all(conn));
    if let Some(history) = res {
        match history.latest_version() {
            None => {
                info!("No revisions found.");
            },
            Some(rev) => {
                info!("{:?}", rev);
            }
        }
    } else {
        try!(models::ScurryMetadata::create_metadata_table(&conn));
    }
    Ok(DesiredVersion::Latest)
}
