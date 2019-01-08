pub use self::instruction::Package;
use self::instruction::{Download, Install, Uninstall, Versions};
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const BLUEPRINT_PATH: &str = "blueprints";

pub mod instruction;

#[derive(Debug, Default, Deserialize)]
pub struct Blueprint {
    pub package: Package,
    pub download: Download,
    pub install: Option<Install>,
    pub uninstall: Option<Uninstall>,
    pub versions: Option<Versions>,
}

pub struct Blueprints {
    blueprints: HashMap<String, PathBuf>,
}

impl Blueprints {
    pub fn scan() -> Self {
        use log::debug;
        use walkdir::WalkDir;

        let path_buf = Path::new("..").join(BLUEPRINT_PATH);
        let blueprints: HashMap<String, PathBuf> = WalkDir::new(path_buf)
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

        debug!("Found {} blueprints", blueprints.len());

        Self { blueprints }
    }

    pub fn load(&self, name: &str) -> Option<Blueprint> {
        if let Some(path) = self.blueprints.get(name) {
            let mut blueprint: Blueprint = crate::toml::read_toml(path, &mut String::new());
            blueprint.package.name = name.to_owned();

            Some(blueprint)
        } else {
            None
        }
    }
}
