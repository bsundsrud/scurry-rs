use error::ScurryError;
use std::path::Path;
use std::io::Error as IoError;
use std::fs::{self, File};
use std::io::prelude::*;
use sha1;
use versions::{Version, DesiredVersion};
use models::{self, ScurryMetadata};
use postgres::Connection;

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

pub fn calculate_available_versions(migrations_dir: &str) -> Result<Vec<Version>, ScurryError>  {
    let all_paths = try!(fs::read_dir(migrations_dir));
    let sql_files = all_paths
        .filter_map(|dirent| dirent.ok())
        .map(|dirent| dirent.path())
        .filter(|path| {
            match path.extension() {
                None => false,
                Some(s) => s == "sql"
            }
        });
    let mut res = vec![];
    for file in sql_files {
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
    res.sort();
    Ok(res)
}

pub fn choose_upgrade_path<'a>(available: &'a Vec<Version>, installed: &Option<&ScurryMetadata>, desired: &DesiredVersion) -> Vec<&'a Version> {
    available.iter()
        .filter(|v| {
            if let &Some(ref installed_vers) = installed {
                &v.version > &installed_vers.script_version
            } else {
                true
            }
        })
        .filter(|v| if let &DesiredVersion::Specific(ref s) = desired { &v.version <= s } else { true }).collect::<Vec<_>>()

}

pub fn verify_common_history(available: &Vec<Version>, installed: &Vec<ScurryMetadata>) -> Result<(), ScurryError> {
    let mut avail = available.iter();
    for i in installed {
        if let Some(v) = avail.next() {
            if &v.version != &i.script_version {
                return Err(ScurryError::Consistency(format!("Version mismatch: {} != {}", &i.script_version, &v.version)));
            }
            if &i.script_hash != &v.hash {
                return Err(ScurryError::Consistency(format!("Version hash mismatch for version {}: {} != {}",
                    &i.script_version, &i.script_hash, &v.hash)));
            }
        } else {
            // Schema is ahead of migrations
            return Err(ScurryError::Consistency(format!("Schema contains unknown version {}", &i.script_version)));
        }
    }
    Ok(())
}

pub fn apply_version(conn: &Connection, version: &Version) -> Result<(), ScurryError> {
    let mut f = try!(File::open(&version.path));
    let mut contents = String::new();
    try!(f.read_to_string(&mut contents));
    let xact = try!(conn.transaction());
    try!(conn.execute(&contents, &[]));
    try!(models::write_history_line(&conn, &version));
    try!(xact.commit());
    Ok(())
}
