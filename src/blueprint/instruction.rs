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
    #[serde(default)]
    pub name: String,
    #[serde(rename = "bin")]
    pub binaries: Vec<PathBuf>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Download {
    pub url: String,
    pub version: Option<Version>,
}
