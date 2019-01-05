use self::instruction::{Download, Install, Uninstall, Versions};
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub use self::instruction::Package;

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
    pub fn new() -> Self {
        Self {
            blueprints: HashMap::new(),
        }
    }

    pub fn read_from_path(&mut self, path: &Path) {
        use log::{debug, warn};
        use std::fs::read_dir;

        debug!("Read blueprints from {:?}", path);

        let blueprints: HashMap<String, PathBuf> = read_dir(path)
            .and_then(|read_dir| {
                let blueprints = read_dir
                    .filter_map(|entry| entry.ok())
                    .map(|dir| {
                        let path = dir.path();
                        let pkg = path
                            .file_stem()
                            .expect("Could not determine basename")
                            .to_string_lossy();
                        (pkg.to_string(), path)
                    })
                    .collect();

                Ok(blueprints)
            })
            .expect("Could not read packages");

        debug!("Read {} blueprints from {:?}", blueprints.len(), path);

        for (name, path) in blueprints {
            if self.blueprints.contains_key(&name) {
                warn!("Overriding package {}", name);
            }

            self.blueprints.insert(name, path);
        }
    }

    pub fn load_blueprint(&self, name: &str) -> Option<Blueprint> {
        if let Some(path) = self.blueprints.get(name) {
            let blueprint = crate::toml::read_toml(path, &mut String::new());

            Some(blueprint)
        } else {
            None
        }
    }
}
