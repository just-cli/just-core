use crate::manifest::Package;
use crate::result::BoxedResult;
use semver::Version;
use semver::VersionReq;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const CONFIG_FILE: &str = "just.toml";
const SHIM_EXE: &str = "shim.exe";

const MANIFEST_DIR: &str = "manifests";
const DOWNLOAD_DIR: &str = "downloads";

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
    pub fn get_packages(&self) -> &HashMap<String, Version> {
        &self.packages
    }

    pub fn remove_package(&mut self, pkg_name: &str) -> Option<Version> {
        self.packages.remove(pkg_name)
    }

    pub fn add_package(&mut self, package: &Package, version: &Version) {
        self.packages
            .insert(package.name.to_owned(), version.to_owned());
    }

    pub fn is_installed(&self, pkg_name: &str, req: Option<VersionReq>) -> bool {
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
    pub fn get_downloads(&self) -> &HashMap<String, HashMap<Version, PathBuf>> {
        &self.downloads
    }

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
    pub root_path: PathBuf,
    pub bin_path: PathBuf,
    pub download_path: PathBuf,
    pub manifest_path: PathBuf,
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

    pub fn save(&self) {
        use crate::toml::write_toml;

        let path = self.path.root_path.join(CONFIG_FILE);

        write_toml(&path, &self)
    }
}

pub trait PackageShims {
    fn create_shims(&self, local: &LocalPackage) -> BoxedResult<()>;
    fn remove_shims(&self, package: &Package) -> BoxedResult<()>;
}

impl PackageShims for Kernel {
    fn create_shims(&self, local: &LocalPackage) -> BoxedResult<()> {
        use log::info;

        info!("Create shims for package {}", local.package.name);

        let pkg_path = self.path.download_path.join(local.path);
        for binary_path in local.package.binaries.iter() {
            let binary_path = pkg_path.join(binary_path);
            let binary_name = binary_path
                .file_name()
                .expect("No Filename for Binary")
                .to_str()
                .unwrap();

            info!(" - Create shim for {}", binary_name);
            let shim = Shim::new(binary_name, &binary_path);
            shim.create(&self.path)?;
        }

        Ok(())
    }

    fn remove_shims(&self, package: &Package) -> BoxedResult<()> {
        use log::info;
        use std::fs::remove_file;

        info!("Remove shims for package {}", package.name);

        for binary_path in package.binaries.iter() {
            let binary_name = binary_path
                .file_name()
                .expect("No Filename for Binary")
                .to_str()
                .unwrap();
            let binary_path = self.path.bin_path.join(binary_name);

            info!(" - Remove shim for {}", binary_name);
            remove_file(binary_path)?;
        }

        Ok(())
    }
}

pub fn init() -> BoxedResult<Folder> {
    use std::env::current_exe;

    let exe_path = current_exe().expect("No running exe detected");
    let bin_path = exe_path.parent().expect("just.exe is not in a bin path?");
    assert!(
        bin_path.exists(),
        "Invalid bin path: {:?} does not exist",
        bin_path
    );
    let root_path = bin_path.parent().expect("bin path is not in another path?");
    let manifest_path = root_path.join(MANIFEST_DIR);
    assert!(
        manifest_path.exists(),
        "Invalid manifest path: {:?} does not exist",
        manifest_path
    );

    create_download_directory_in(&root_path)
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
                root_path: root_path.to_owned(),
                bin_path: bin_path.to_owned(),
                download_path,
                manifest_path,
            };

            Ok(folder)
        })
}

fn create_download_directory_in(root_path: &Path) -> BoxedResult<PathBuf> {
    let download_path = root_path.join(DOWNLOAD_DIR);
    if !download_path.exists() {
        use std::fs::create_dir;

        create_dir(&download_path)?
    }

    Ok(download_path.to_owned())
}

struct Shim<'a> {
    binary_name: &'a str,
    binary_path: &'a Path,
}

impl<'a> Shim<'a> {
    fn new(binary_name: &'a str, binary_path: &'a Path) -> Self {
        Self {
            binary_name,
            binary_path,
        }
    }

    fn create(&self, folder: &Folder) -> BoxedResult<()> {
        use std::fs::File;

        let basename = Path::new(self.binary_name)
            .file_stem()
            .expect("No Basename");

        File::create(folder.bin_path.join(basename))
            .and_then(|mut file| {
                use std::io::Write;

                file.write_all(self.binary_path.to_string_lossy().as_bytes())
                    .and_then(|_| Ok(()))
            })
            .and_then(|_| {
                use std::fs::copy;

                copy(
                    folder.bin_path.join(SHIM_EXE),
                    folder.bin_path.join(self.binary_name),
                )
                .and_then(|_| Ok(()))
            })
            .map_err(|e| e.into())
    }
}
