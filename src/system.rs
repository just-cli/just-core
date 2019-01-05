use crate::result::BoxedResult;
use duct::{cmd, ToExecutable};
use serde_derive::{Deserialize, Serialize};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub struct WinPath {
    paths: Vec<PathBuf>,
}

impl WinPath {
    pub fn inherit() -> Self {
        use std::env;

        Self {
            paths: if let Ok(path) = env::var("PATH") {
                env::split_paths(&path).collect()
            } else {
                Vec::new()
            },
        }
    }

    pub fn exists(&self, path: &Path) -> bool {
        self.paths.iter().any(|p| p == path)
    }

    pub fn append(&mut self, path: &Path) {
        self.paths.push(path.to_owned())
    }

    pub fn remove(&mut self, path: &Path) {
        if let Some(index) = self.paths.iter().position(|p| p == path) {
            self.paths.remove(index);
        }
    }

    pub fn save(&self) -> BoxedResult<()> {
        use log::debug;

        let vec: Vec<String> = self
            .paths
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        let win_path = vec.join(";");
        debug!("Path is: {}", win_path);
        self::cmd_run_silent("setx", &["PATH", &win_path])
    }
}

pub fn cmd_run<T, U, V>(exe: T, args: U) -> BoxedResult<()>
where
    T: ToExecutable,
    U: IntoIterator<Item = V>,
    V: Into<OsString>,
{
    cmd(exe, args)
        .run()
        .and_then(|_| Ok(()))
        .map_err(|e| e.into())
}

pub fn cmd_run_silent<T, U, V>(exe: T, args: U) -> BoxedResult<()>
where
    T: ToExecutable,
    U: IntoIterator<Item = V>,
    V: Into<OsString>,
{
    cmd(exe, args)
        .stdout_capture()
        .stderr_capture()
        .run()
        .and_then(|_| Ok(()))
        .map_err(|e| e.into())
}

pub fn cmd_read_silent<T, U, V>(exe: T, args: U) -> BoxedResult<String>
where
    T: ToExecutable,
    U: IntoIterator<Item = V>,
    V: Into<OsString>,
{
    cmd(exe, args)
        .stdout_capture()
        .stderr_capture()
        .read()
        .map_err(|e| e.into())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Cmd {
    pub exe: String,
    #[serde(default)]
    pub args: Vec<String>,
}

impl Cmd {
    pub fn parse(cmd: &str) -> Option<Self> {
        let args: Vec<&str> = cmd.split_whitespace().collect();

        if args.is_empty() {
            None
        } else {
            Some(Self {
                exe: args.first().unwrap().to_string(),
                args: args.iter().skip(1).map(|arg| arg.to_string()).collect(),
            })
        }
    }

    pub fn run(&self) -> BoxedResult<()> {
        cmd_run_silent(&self.exe, &self.args)
    }

    pub fn read(&self) -> BoxedResult<String> {
        cmd_read_silent(&self.exe, &self.args)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn cmd_parse_empty() {
        use super::Cmd;

        let cmd = Cmd::parse("");
        assert!(cmd.is_none());
    }

    #[test]
    fn cmd_parse_with_argument() {
        use super::Cmd;

        let cmd = Cmd::parse("curl foo");
        assert!(cmd.is_some());
        let cmd = cmd.unwrap();
        assert_eq!("curl", &cmd.exe);
        assert_eq!(&["foo"], &cmd.args[..]);
    }

    #[test]
    fn cmd_parse_without_argument() {
        use super::Cmd;

        let cmd = Cmd::parse("curl");
        assert!(cmd.is_some());
        let cmd = cmd.unwrap();
        assert_eq!("curl", &cmd.exe);
        assert!(cmd.args.is_empty());
    }
}
