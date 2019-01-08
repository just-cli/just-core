use crate::blueprint::Package;
use crate::result::BoxedResult;
use semver::Version;
use semver::VersionReq;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const CONFIG_FILE: &str = "just.toml";
const SHIM_EXE: &str = "shim.exe";
const PACKAGE_DIR: &str = "packages";

#[derive(Debug, Serialize, Deserialize)]
pub struct Kernel {
    pub path: Folder,
    #[serde(flatten)]
    pub packages: InstalledPackages,
    #[serde(flatten)]
    pub downloads: AvailableDownloads,
    #[serde(flatten)]
    pub workspaces: AvailableWorkspaces,
    #[serde(flatten)]
    pub versions: AvailableVersions,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct InstalledPackages {
    packages: HashMap<String, Version>,
}

impl InstalledPackages {
    pub fn add_package(&mut self, package: &Package, version: &Version) {
        self.packages
            .insert(package.name.to_owned(), version.to_owned());
    }

    pub fn is_installed(&self, package: &Package, req: Option<VersionReq>) -> bool {
        let pkg_name = &package.name;
        if let Some((_, version)) = self.packages.iter().find(|(name, _)| *name == pkg_name) {
            match req {
                Some(req) => req.matches(version),
                None => true,
            }
        } else {
            false
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AvailableDownloads {
    downloads: HashMap<String, HashMap<Version, PathBuf>>,
}

impl AvailableDownloads {
    pub fn add_download(&mut self, local: &LocalPackage, download_path: &Path) {
        let name = local.package.name.as_str();

        for binary in local.package.binaries.iter() {
            let full_path = download_path.join(local.path).join(binary);

            self.downloads
                .entry(name.to_owned())
                .and_modify(|items| {
                    items
                        .entry(local.version.to_owned())
                        .and_modify(|path| *path = full_path.to_owned())
                        .or_insert_with(|| full_path.to_owned());
                })
                .or_insert_with(|| {
                    vec![(local.version.clone(), full_path.to_owned())]
                        .into_iter()
                        .collect()
                });
        }
    }

    pub fn get_download(&self, name: &str, req: &VersionReq) -> Option<(&Version, &PathBuf)> {
        if let Some(ref downloads) = self.downloads.get(name) {
            downloads.iter().find(|(version, _)| req.matches(version))
        } else {
            None
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AvailableVersions {
    versions: HashMap<String, Vec<Version>>,
}

impl AvailableVersions {
    pub fn add_version(&mut self, package: &Package, version: &Version) {
        self.versions
            .entry(package.name.to_owned())
            .and_modify(|set| {
                if !set.contains(version) {
                    set.push(version.to_owned());
                    set.sort_by(|a, b| b.cmp(a));
                }
            })
            .or_insert_with(|| vec![version.to_owned()]);
    }

    pub fn get_all_versions_of(&self, package: &Package) -> Option<&Vec<Version>> {
        self.versions.get(package.name.as_str())
    }

    pub fn get_latest_versions_of(&self, package: &Package) -> Option<&Version> {
        match self.get_all_versions_of(package) {
            Some(versions) => versions.first(),
            None => None,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AvailableWorkspaces {
    workspaces: HashMap<String, Workspace>,
}

impl AvailableWorkspaces {
    pub fn add_workspace(&mut self, name: &str, path: &Path) {
        let mut ws = Workspace::default();
        ws.path = path.to_owned();

        self.workspaces.insert(name.to_owned(), ws);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Folder {
    pub home: PathBuf,
    pub bin: PathBuf,
    pub downloads: PathBuf,
    pub packages: PathBuf,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Workspace {
    pub path: PathBuf,
    pub active: bool,
}

pub struct LocalPackage<'a> {
    pub path: &'a Path,
    pub version: &'a Version,
    pub package: &'a Package,
}

impl Default for Kernel {
    fn default() -> Self {
        let folder = init().expect("Could not init just");

        Self {
            path: folder,
            packages: InstalledPackages::default(),
            downloads: AvailableDownloads::default(),
            workspaces: AvailableWorkspaces::default(),
            versions: AvailableVersions::default(),
        }
    }
}

impl Drop for Kernel {
    fn drop(&mut self) {
        self.save();
    }
}

impl Kernel {
    pub fn load() -> Self {
        use crate::toml::read_toml;
        use log::debug;

        //  We have to use '/..' because we are in the bin folder
        let path = Path::new("../").join(CONFIG_FILE);
        if path.exists() {
            debug!("Load existing configuration");

            read_toml(&path, &mut String::new())
        } else {
            debug!("Create new default configuration");

            Self::default()
        }
    }

//    pub fn add_package(&mut self, local: &LocalPackage) {
//        self.packages.add_package(local.package, local.version);
//        self.versions.add_version(local.package, local.version);
//    }
//
//    pub fn add_download(&mut self, local: &LocalPackage) {
//        self.downloads.add_download(local, &self.path.downloads);
//        self.add_package(local);
//    }

    pub fn create_shims(&self, local: &LocalPackage) -> BoxedResult<()> {
        use log::info;

        let pkg_path = self.path.downloads.join(local.path);
        for binary in local.package.binaries.iter() {
            let binary_path = pkg_path.join(binary);
            let binary_name = binary.file_name().expect("No Filename").to_str().unwrap();

            info!("Create shim for {}", binary_name);
            let shim = Shim::new(binary_name, binary_path);
            shim.create(&self)?;
        }

        Ok(())
    }

    pub fn save(&self) {
        use crate::toml::write_toml;

        let path = self.path.home.join(CONFIG_FILE);

        write_toml(&path, &self)
    }
}

pub fn init() -> BoxedResult<Folder> {
    use std::env::current_exe;

    let current_path = current_exe().expect("No running exe detected");
    let bin_path = current_path
        .parent()
        .expect("just.exe is not in a bin path?");
    assert!(
        bin_path.exists(),
        "Invalid bin path: bin path {:?} does not exist",
        bin_path
    );
    let home_path = bin_path.parent().expect("bin path is not in another path?");
    let package_path = home_path.join(PACKAGE_DIR);
    assert!(
        package_path.exists(),
        "Invalid package path: package path {:?} does not exist",
        package_path
    );

    create_download_directory_in(&home_path)
        .and_then(|download_path| {
            use log::info;

            let mut win_path = crate::system::WinPath::inherit();
            if win_path.exists(&bin_path) {
                Ok(download_path)
            } else {
                info!("Add {:?} to PATH", bin_path);
                win_path.append(&bin_path);
                win_path.save().and_then(|_| Ok(download_path))
            }
        })
        .and_then(|download_path| {
            let folder = Folder {
                home: home_path.to_owned(),
                bin: bin_path.to_owned(),
                downloads: download_path,
                packages: package_path,
            };

            Ok(folder)
        })
}

fn create_download_directory_in(home_path: &Path) -> BoxedResult<PathBuf> {
    let download_path = home_path.join("downloads");
    if !download_path.exists() {
        use std::fs::create_dir;

        create_dir(&download_path)?
    }

    Ok(download_path.to_owned())
}

struct Shim<'a> {
    binary_name: &'a str,
    binary_path: PathBuf,
}

impl<'a> Shim<'a> {
    fn new(binary_name: &'a str, binary_path: PathBuf) -> Self {
        Self {
            binary_name,
            binary_path,
        }
    }

    fn create(&self, config: &Kernel) -> BoxedResult<()> {
        use std::fs::File;

        let basename = Path::new(self.binary_name)
            .file_stem()
            .expect("No Basename");

        File::create(config.path.bin.join(basename))
            .and_then(|mut file| {
                use std::io::Write;

                file.write_all(self.binary_path.to_string_lossy().as_bytes())
                    .and_then(|_| Ok(()))
            })
            .and_then(|_| {
                use std::fs::copy;

                copy(
                    config.path.bin.join(SHIM_EXE),
                    config.path.bin.join(self.binary_name),
                )
                .and_then(|_| Ok(()))
            })
            .map_err(|e| e.into())
    }
}
