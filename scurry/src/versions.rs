use std::cmp::Ordering;

#[derive(Debug)]
pub struct Version {
    pub path: String,
    pub name: String,
    pub hash: String,
    pub version: String,
}

impl PartialEq for Version {
    fn eq(&self, other: &Version) -> bool {
        self.hash == other.hash && self.version == other.version
    }
}

impl Eq for Version {}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Version) -> Option<Ordering> {
        self.version.partial_cmp(&other.version)
    }

}

impl Ord for Version {
    fn cmp(&self, other: &Version) -> Ordering {
        self.version.cmp(&other.version)
    }
}

#[derive(Debug)]
pub enum DesiredVersion {
    Latest,
    Specific(String),
}
