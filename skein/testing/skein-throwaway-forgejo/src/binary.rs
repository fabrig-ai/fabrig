use crate::{Failure, Instance, Settings};
use skein_forge::forgejo::pin::names_forgejo;
use std::{
    fmt::Write,
    fs,
    path::{Path, PathBuf},
    process::Command,
};
#[derive(Debug, Clone)]
pub struct Forgejo {
    binary: PathBuf,
    version: String,
    sha256: String,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Refusal {
    Unreadable { path: PathBuf, error: String },
    Version { path: PathBuf, printed: String },
}
impl Forgejo {
    /// # Errors
    /// Returns the unreadable path or its unexpected version line.
    pub fn from_path(binary: &Path) -> Result<Self, Refusal> {
        let fail = |e: std::io::Error| Refusal::Unreadable { path: binary.to_owned(), error: e.to_string() };
        let binary = fs::canonicalize(binary).map_err(fail)?;
        let bytes = fs::read(&binary).map_err(fail)?;
        let mut cmd = Command::new(&binary);
        environment(&mut cmd, binary.parent().expect("absolute binary has a parent"));
        let output = cmd.arg("--version").output().map_err(fail)?;
        let version = admit_version(&binary, output.status.success(), &output.stdout)?;
        let mut sha256 = String::new();
        for byte in skein_lib::sha256::digest(&bytes) {
            write!(sha256, "{byte:02x}").expect("string write");
        }
        Ok(Self { binary, version, sha256 })
    }
    #[must_use]
    pub fn binary(&self) -> &Path {
        &self.binary
    }
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }
    #[must_use]
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
    /// Starts a server that dies with this calling thread on Linux.
    /// Keep that thread alive for the instance's use (testing/fakes/throwaway.md, 1).
    /// # Errors
    /// Returns the provisioning failure, cleaning up anything started.
    pub fn start(&self, scratch: &Path, settings: &Settings) -> Result<Instance, Failure> {
        Instance::start(self, scratch, settings)
    }
}
fn admit_version(binary: &Path, succeeded: bool, stdout: &[u8]) -> Result<String, Refusal> {
    let printed = String::from_utf8_lossy(stdout).lines().next().unwrap_or_default().to_owned();
    if !succeeded || !names_forgejo(&printed) {
        return Err(Refusal::Version { path: binary.to_owned(), printed });
    }
    Ok(printed)
}

pub(crate) fn environment(cmd: &mut Command, scratch: &Path) {
    cmd.env_clear()
        .env("HOME", scratch.join("home"))
        .env("TMPDIR", scratch.join("tmp"))
        .env("PATH", "/usr/bin:/bin")
        .env("TZ", "Asia/Kolkata")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", scratch.join("gitconfig"));
}

#[cfg(test)]
mod tests {
    use super::{Path, Refusal, admit_version};

    #[test]
    fn a_binary_printing_another_version_is_refused_naming_it() {
        let binary = Path::new("/scratch/forgejo");
        assert_eq!(
            admit_version(binary, true, b"forgejo version 15.0.0+gitea-1.22.0\n").expect_err("refused"),
            Refusal::Version { path: binary.into(), printed: "forgejo version 15.0.0+gitea-1.22.0".into() }
        );
    }
}
