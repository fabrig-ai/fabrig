//! Preflight and pure program plans (client/drive.md, 2.1–2.3).
use crate::EnvironmentEntry;
use std::{ffi::{OsStr, OsString}, fs, io::Write, os::unix::{ffi::OsStrExt, fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt}}, path::{Path, PathBuf}};
#[derive(Clone, Debug)]
pub struct Options { pub bin: PathBuf, pub bundle: PathBuf, pub scratch: PathBuf, pub scripts: Vec<PathBuf>, pub strict: bool }
impl Options {
    pub fn parse(arguments: &[Box<[u8]>]) -> Result<Self, String> {
        let (mut bin, mut bundle, mut scratch) = (None, None, None);
        let mut scripts = vec![]; let mut strict = false; let mut args = arguments.iter();
        while let Some(argument) = args.next() {
            let target = match argument.as_ref() {
                b"--strict" if !strict => { strict = true; continue; }
                b"--bin" => &mut bin, b"--bundle" => &mut bundle, b"--scratch" => &mut scratch,
                b"--scripts" => { scripts.push(path_arg(&mut args, "--scripts FILE")?); continue; }
                other => return Err(format!("unknown or repeated drive option {}", String::from_utf8_lossy(other))),
            };
            if target.is_some() { return Err(format!("repeated {}", String::from_utf8_lossy(argument))); }
            *target = Some(path_arg(&mut args, &format!("{} DIR", String::from_utf8_lossy(argument)))?);
        }
        Ok(Self { bin: bin.ok_or("missing --bin DIR")?, bundle: bundle.ok_or("missing --bundle DIR")?, scratch: scratch.ok_or("missing --scratch DIR")?, scripts, strict })
    }
}
fn path_arg(args: &mut std::slice::Iter<'_, Box<[u8]>>, flag: &str) -> Result<PathBuf, String> {
    let value = args.next().filter(|value| !value.is_empty() && !value.starts_with(b"--")).ok_or_else(|| format!("missing {flag}"))?;
    Ok(PathBuf::from(OsStr::from_bytes(value)))
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Input { Null, Bytes(Box<[u8]>) }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program { pub program: PathBuf, pub args: Vec<OsString>, pub environment: Vec<(OsString, OsString)>, pub directory: PathBuf, pub input: Input }
#[derive(Clone, Debug)]
pub struct Plan { pub options: Options, pub chromium: PathBuf, pub environment: Vec<(OsString, OsString)> }
impl Plan {
    #[must_use]
    pub fn new(options: Options, chromium: PathBuf, path: &OsStr) -> Self {
        let mut environment = vec![("PATH".into(), path.into())];
        for (key, dir) in [("HOME", "home"), ("XDG_STATE_HOME", "state"), ("TMPDIR", "tmp"), ("XDG_RUNTIME_DIR", "runtime"), ("XDG_CONFIG_HOME", "home/config"), ("XDG_CACHE_HOME", "home/cache"), ("XDG_DATA_HOME", "home/data")] {
            environment.push((key.into(), options.scratch.join("session").join(dir).into_os_string()));
        }
        Self { options, chromium, environment }
    }
    #[must_use]
    pub fn box_program(&self) -> Program {
        let mut args = vec!["--fake-backends".into(), "--bundle".into(), self.options.bundle.clone().into_os_string()];
        for (flag, dir) in [("--config", "config"), ("--data", "data"), ("--cache", "cache")] { args.extend([flag.into(), self.options.scratch.join(dir).into_os_string()]); }
        for script in &self.options.scripts { args.extend(["--scripts".into(), script.clone().into_os_string()]); }
        if self.options.strict { args.push("--strict".into()); }
        self.program("temper-box", args, Input::Null)
    }
    #[must_use]
    pub fn ctl(&self, args: &[Box<[u8]>], input: Input) -> Program {
        let mut all = vec!["ctl".into(), self.options.scratch.join("session/ctl").into_os_string()];
        all.extend(args.iter().map(|arg| OsStr::from_bytes(arg).to_os_string()));
        self.program("temper-client", all, input)
    }
    fn program(&self, name: &str, args: Vec<OsString>, input: Input) -> Program {
        Program { program: self.options.bin.join(name), args, environment: self.environment.clone(), directory: self.options.scratch.clone(), input }
    }
    #[must_use]
    pub fn look(&self, line: u64, width: u32) -> PathBuf { self.options.scratch.join(format!("session/looks/{line:04}-{width}.png")) }
}
pub(super) fn prepare(mut options: Options, environment: &[EnvironmentEntry]) -> Result<Plan, String> {
    options.bin = absolute(&options.bin)?;
    for name in ["temper-box", "fake-claude", "temper-client"] { executable(&options.bin.join(name))?; }
    options.bundle = absolute(&options.bundle)?;
    if !options.bundle.join("index.html").is_file() { return Err(format!("{} is not a file", options.bundle.join("index.html").display())); }
    options.scratch = absolute(&options.scratch)?;
    let metadata = fs::metadata(&options.scratch).map_err(|error| format!("{}: {error}", options.scratch.display()))?;
    if !metadata.is_dir() || metadata.uid() != skein_shell::effective_user() || metadata.mode() & 0o077 != 0 { return Err(format!("{} is not an owned private scratch directory", options.scratch.display())); }
    for script in &mut options.scripts { *script = absolute(script)?; }
    let chromium = crate::table::CHROMIUM.value(environment).ok_or("set SKEIN_TEST_CHROMIUM to the Chromium executable")?;
    let chromium = absolute(Path::new(OsStr::from_bytes(chromium)))?; executable(&chromium)?;
    let plan = Plan::new(options, chromium, OsStr::from_bytes(crate::table::PATH.value(environment).unwrap_or_default()));
    fresh_session(&plan.options.scratch).map_err(|error| format!("cannot prepare {}/session: {error}", plan.options.scratch.display()))?;
    Ok(plan)
}
fn absolute(path: &Path) -> Result<PathBuf, String> { fs::canonicalize(path).map_err(|error| format!("{}: {error}", path.display())) }
fn executable(path: &Path) -> Result<(), String> {
    let metadata = fs::metadata(path).map_err(|error| format!("{}: {error}", path.display()))?;
    if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 { return Err(format!("{} is not an executable regular file", path.display())); }
    Ok(())
}
fn fresh_session(scratch: &Path) -> std::io::Result<()> {
    let session = scratch.join("session");
    match fs::symlink_metadata(&session) {
        Ok(metadata) if metadata.is_dir() => fs::remove_dir_all(&session)?, Ok(_) => fs::remove_file(&session)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}, Err(error) => return Err(error),
    }
    for dir in ["", "home", "home/config", "home/cache", "home/data", "state", "tmp", "runtime", "tui-runtime", "browser", "bin", "looks"] { fs::DirBuilder::new().mode(0o700).create(session.join(dir))?; }
    for file in ["lines", "box.log"] { fs::OpenOptions::new().create_new(true).write(true).mode(0o600).open(session.join(file))?.write_all(b"")?; }
    Ok(())
}
