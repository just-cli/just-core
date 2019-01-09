use crate::kernel::Kernel;
use semver::Version;
use serde_derive::Deserialize;
use std::collections::HashMap;
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

#[derive(Debug, Default, Deserialize)]
pub struct Manifest {
    pub package: Package,
    pub download: Download,
    pub install: Option<Install>,
    pub uninstall: Option<Uninstall>,
    pub versions: Option<Versions>,
}

pub struct ManifestFiles {
    files: HashMap<String, PathBuf>,
}

impl ManifestFiles {
    pub fn scan(kernel: &Kernel) -> Self {
        use log::debug;
        use walkdir::WalkDir;

        let files: HashMap<String, PathBuf> = WalkDir::new(kernel.path.manifest_path.as_path())
            .into_iter()
            .filter_map(|entry| match entry {
                Ok(dir) => {
                    let path = dir.path();
                    match path.extension() {
                        Some(extension) if extension == "toml" => {
                            let pkg = path
                                .file_stem()
                                .expect("Could not determine basename")
                                .to_string_lossy();
                            Some((pkg.to_string(), path.to_owned()))
                        }
                        _ => None,
                    }
                }
                _ => None,
            })
            .collect();

        debug!("Found {} manifests", files.len());

        Self { files }
    }

    pub fn load_manifest(&self, name: &str) -> Option<Manifest> {
        if let Some(path) = self.files.get(name) {
            let mut manifest: Manifest = crate::toml::read_toml(path, &mut String::new());
            manifest.package.name = name.to_owned();

            Some(manifest)
        } else {
            None
        }
    }
}
