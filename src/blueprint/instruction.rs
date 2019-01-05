use semver::Version;
use serde_derive::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize)]
pub struct Versions {
    pub cmd: String,
    #[serde(rename = "match")]
    pub pattern: String,
    pub format: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Install {
    pub before: Option<String>,
    pub after: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Uninstall {
    pub before: Option<String>,
    pub after: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Package {
    pub alias: Vec<String>,
    #[serde(rename = "bin")]
    pub binaries: Vec<PathBuf>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Download {
    pub url: String,
    pub version: Option<Version>,
}

impl Package {
    pub fn get_first_alias(&self) -> &str {
        self.alias
            .first()
            .expect("There is no alias for this package")
    }
}
