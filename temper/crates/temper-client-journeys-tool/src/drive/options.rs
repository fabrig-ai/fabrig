//! Preflight and pure program plans (client/drive.md, 2.1–2.3).
use crate::EnvironmentEntry;
use std::{
    ffi::{OsStr, OsString},
    fs,
    io::Write,
    os::unix::{
        ffi::OsStrExt,
        fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt},
    },
    path::{Path, PathBuf},
};
#[derive(Clone, Debug)]
pub struct Options {
    pub bin: PathBuf,
    pub bundle: PathBuf,
    pub scratch: PathBuf,
    pub scripts: Vec<PathBuf>,
    pub strict: bool,
    pub ports: Option<String>,
    pub files: Option<PathBuf>,
}
impl Options {
    pub fn parse(arguments: &[Box<[u8]>]) -> Result<Self, String> {
        let (mut bin, mut bundle, mut scratch) = (None, None, None);
        let mut scripts = vec![];
        let mut strict = false;
        let mut ports = None;
        let mut files = None;
        let mut args = arguments.iter();
        while let Some(argument) = args.next() {
            let target = match argument.as_ref() {
                b"--strict" if !strict => {
                    strict = true;
                    continue;
                }
                b"--ports" if ports.is_none() => {
                    let value = args.next().ok_or("missing --ports LIST")?;
                    let list = String::from_utf8_lossy(value);
                    let parsed = list.split(',').map(str::parse::<u16>).collect::<Result<Vec<_>, _>>();
                    if !parsed.as_ref().is_ok_and(|values| {
                        values.len() == 9
                            && !values.contains(&0)
                            && values.iter().collect::<std::collections::BTreeSet<_>>().len() == 9
                    }) {
                        return Err(format!("--ports {list}: expected nine distinct nonzero ports"));
                    }
                    ports = Some(list.into_owned());
                    continue;
                }
                b"--files" => &mut files,
                b"--bin" => &mut bin,
                b"--bundle" => &mut bundle,
                b"--scratch" => &mut scratch,
                b"--scripts" => {
                    scripts.push(path_arg(&mut args, "--scripts FILE")?);
                    continue;
                }
                other => return Err(format!("unknown or repeated drive option {}", String::from_utf8_lossy(other))),
            };
            if target.is_some() {
                return Err(format!("repeated {}", String::from_utf8_lossy(argument)));
            }
            *target = Some(path_arg(&mut args, &format!("{} DIR", String::from_utf8_lossy(argument)))?);
        }
        Ok(Self {
            bin: bin.ok_or("missing --bin DIR")?,
            bundle: bundle.ok_or("missing --bundle DIR")?,
            scratch: scratch.ok_or("missing --scratch DIR")?,
            scripts,
            strict,
            ports,
            files,
        })
    }
}
fn path_arg(args: &mut std::slice::Iter<'_, Box<[u8]>>, flag: &str) -> Result<PathBuf, String> {
    let value = args
        .next()
        .filter(|value| !value.is_empty() && !value.starts_with(b"--"))
        .ok_or_else(|| format!("missing {flag}"))?;
    Ok(PathBuf::from(OsStr::from_bytes(value)))
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Input {
    Null,
    Bytes(Box<[u8]>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    pub program: PathBuf,
    pub args: Vec<OsString>,
    pub environment: Vec<(OsString, OsString)>,
    pub directory: PathBuf,
    pub input: Input,
}
#[derive(Clone, Debug)]
pub struct Plan {
    pub options: Options,
    pub chromium: PathBuf,
    pub environment: Vec<(OsString, OsString)>,
    pub(super) tui_display: Option<OsString>,
}
impl Plan {
    #[must_use]
    pub fn new(options: Options, chromium: PathBuf, path: &OsStr) -> Self {
        let mut environment = vec![("PATH".into(), path.into())];
        for (key, dir) in [
            ("HOME", "home"),
            ("XDG_STATE_HOME", "state"),
            ("TMPDIR", "tmp"),
            ("XDG_RUNTIME_DIR", "runtime"),
            ("XDG_CONFIG_HOME", "home/config"),
            ("XDG_CACHE_HOME", "home/cache"),
            ("XDG_DATA_HOME", "home/data"),
        ] {
            environment.push((key.into(), options.scratch.join("session").join(dir).into_os_string()));
        }
        Self { options, chromium, environment, tui_display: None }
    }
    #[must_use]
    pub fn box_program(&self) -> Program {
        let mut args = vec!["--fake-backends".into(), "--bundle".into(), self.options.bundle.clone().into_os_string()];
        for (flag, dir) in [("--config", "config"), ("--data", "data"), ("--cache", "cache")] {
            args.extend([flag.into(), self.options.scratch.join(dir).into_os_string()]);
        }
        for script in &self.options.scripts {
            args.extend(["--scripts".into(), script.clone().into_os_string()]);
        }
        if self.options.strict {
            args.push("--strict".into());
        }
        let mut program = self.program("temper-box", args, Input::Null);
        if let Some(ports) = &self.options.ports {
            program.environment.push(("TEMPER_INTERNAL_FAKE_BOX_PORTS".into(), ports.into()));
        }
        program
    }
    #[must_use]
    pub fn ctl(&self, args: &[Box<[u8]>], input: Input) -> Program {
        let mut all = vec!["ctl".into(), self.options.scratch.join("session/ctl").into_os_string()];
        all.extend(args.iter().map(|arg| OsStr::from_bytes(arg).to_os_string()));
        self.program("temper", all, input)
    }
    pub(super) fn tui(&self, origin: &[u8]) -> Result<Program, String> {
        let mut given = self
            .environment
            .iter()
            .map(|(name, value)| (name.as_bytes().into(), value.as_bytes().into()))
            .collect::<Vec<_>>();
        if let Some(display) = &self.tui_display {
            given.push((b"DISPLAY".as_slice().into(), display.as_bytes().into()));
        }
        let mut environment = crate::tui_driver::environment(&given, &self.options.scratch.join("session"))
            .map_err(|why| format!("cannot build TUI environment: {why}"))?;
        for (name, value) in &mut environment {
            if ["XDG_CONFIG_HOME", "XDG_CACHE_HOME", "XDG_DATA_HOME"].iter().any(|key| name == key) {
                value.clone_from(
                    &self.environment.iter().find(|(key, _)| key == name).expect("session XDG directory").1,
                );
            }
        }
        let mut program = self.program("temper", vec![OsStr::from_bytes(origin).into()], Input::Null);
        program.environment = environment;
        Ok(program)
    }
    fn program(&self, name: &str, args: Vec<OsString>, input: Input) -> Program {
        Program {
            program: self.options.bin.join(name),
            args,
            environment: self.environment.clone(),
            directory: self.options.scratch.clone(),
            input,
        }
    }
    /// A ctl file destination is explicit and remains in the session's scratch.
    pub fn check_ctl_writes(&self, args: &[Box<[u8]>]) -> Result<(), String> {
        for path in destinations(args, &self.options.scratch) {
            if !path.starts_with(&self.options.scratch) {
                return Err(format!("ctl --out {} is outside scratch", path.display()));
            }
        }
        Ok(())
    }
    #[must_use]
    pub fn look(&self, line: u64, width: u32) -> PathBuf {
        self.options.scratch.join(format!("session/looks/{line:04}-{width}.png"))
    }
}
pub(super) fn destinations(args: &[Box<[u8]>], scratch: &Path) -> Vec<PathBuf> {
    args.windows(2)
        .filter(|pair| pair[0].as_ref() == b"--out")
        .map(|pair| {
            let path = scratch.join(OsStr::from_bytes(&pair[1]));
            let mut normalized = PathBuf::new();
            for component in path.components() {
                match component {
                    std::path::Component::ParentDir => {
                        normalized.pop();
                    }
                    std::path::Component::CurDir => {}
                    other => normalized.push(other.as_os_str()),
                }
            }
            normalized
        })
        .collect()
}
pub(super) fn check_destinations(program: &Program, scratch: &Path) -> Result<(), String> {
    let args: Vec<Box<[u8]>> = program.args.iter().map(|arg| arg.as_bytes().into()).collect();
    for path in destinations(&args, scratch) {
        let resolved = if fs::symlink_metadata(&path).is_ok() {
            fs::canonicalize(&path)
        } else {
            path.parent()
                .ok_or_else(|| format!("ctl --out {} has no parent", path.display()))?
                .canonicalize()
                .map(|parent| parent.join(path.file_name().unwrap_or_default()))
        }
        .map_err(|error| format!("ctl --out {}: {error}", path.display()))?;
        if !resolved.starts_with(scratch) {
            return Err(format!("ctl --out {} follows a path outside scratch", path.display()));
        }
    }
    Ok(())
}
pub(super) fn prepare(mut options: Options, environment: &[EnvironmentEntry]) -> Result<Plan, String> {
    if let Some(files) = &mut options.files {
        *files = absolute(files)?;
        if !files.is_dir() {
            return Err(format!("{}: --files needs a directory", files.display()));
        }
    }
    options.bin = absolute(&options.bin)?;
    for name in ["temper-box", "fake-claude", "temper"] {
        executable(&options.bin.join(name))?;
    }
    options.bundle = absolute(&options.bundle)?;
    if !options.bundle.join("index.html").is_file() {
        return Err(format!("{} is not a file", options.bundle.join("index.html").display()));
    }
    options.scratch = absolute(&options.scratch)?;
    let metadata = fs::metadata(&options.scratch).map_err(|error| format!("{}: {error}", options.scratch.display()))?;
    if !metadata.is_dir() || metadata.uid() != skein_shell::effective_user() || metadata.mode() & 0o077 != 0 {
        return Err(format!("{} is not an owned private scratch directory", options.scratch.display()));
    }
    for script in &mut options.scripts {
        *script = absolute(script)?;
    }
    let chromium =
        crate::table::CHROMIUM.value(environment).ok_or("set SKEIN_TEST_CHROMIUM to the Chromium executable")?;
    let chromium = absolute(Path::new(OsStr::from_bytes(chromium)))?;
    executable(&chromium)?;
    let mut plan =
        Plan::new(options, chromium, OsStr::from_bytes(crate::table::PATH.value(environment).unwrap_or_default()));
    plan.tui_display = environment
        .iter()
        .find(|(name, _)| name.as_ref() == b"DISPLAY")
        .map(|(_, value)| OsStr::from_bytes(value).to_os_string());
    fresh_session(&plan.options.scratch)
        .map_err(|error| format!("cannot prepare {}/session: {error}", plan.options.scratch.display()))?;
    Ok(plan)
}
fn absolute(path: &Path) -> Result<PathBuf, String> {
    fs::canonicalize(path).map_err(|error| format!("{}: {error}", path.display()))
}
fn executable(path: &Path) -> Result<(), String> {
    let metadata = fs::metadata(path).map_err(|error| format!("{}: {error}", path.display()))?;
    if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
        return Err(format!("{} is not an executable regular file", path.display()));
    }
    Ok(())
}
fn fresh_session(scratch: &Path) -> std::io::Result<()> {
    let session = scratch.join("session");
    match fs::symlink_metadata(&session) {
        Ok(metadata) if metadata.is_dir() => fs::remove_dir_all(&session)?,
        Ok(_) => fs::remove_file(&session)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    for dir in [
        "",
        "home",
        "home/config",
        "home/cache",
        "home/data",
        "state",
        "tmp",
        "runtime",
        "tui-runtime",
        "browser",
        "bin",
        "looks",
    ] {
        fs::DirBuilder::new().mode(0o700).create(session.join(dir))?;
    }
    for name in ["xdg-open", "open"] {
        std::os::unix::fs::symlink("/usr/bin/true", session.join("bin").join(name))?;
    }
    for file in ["lines", "box.log"] {
        fs::OpenOptions::new().create_new(true).write(true).mode(0o600).open(session.join(file))?.write_all(b"")?;
    }
    Ok(())
}
