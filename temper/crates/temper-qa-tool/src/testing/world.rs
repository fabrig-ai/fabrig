//! The runner's home and its faces (temper/docs/design/testing/qa/contracts.md, 4).
use super::{Agent, Agents, Clock, Git, Tree, id};
use crate::{
    Home, Ledger, Refusal,
    faces::{DirectoryMode, Environment, FilesFace, Mode, RealFiles, WriteMode, WriteOptions},
    refused,
    runner::Run,
};
use sha2::{Digest, Sha256};
use std::{
    fmt::Write,
    io,
    path::{Path, PathBuf},
};

const CONFIGURATION: &[u8] = br#"{"owner":"qa","git":{"directory":"repo"},"agent":{"command":["agent","--effort={effort}"],"triage":"cheap","pass":"high"},"request":{"command":["file"]},"bounds":{"poll":60,"triage":30,"build_wait":90},"notify":["notify"]}"#;
const REPLACE: WriteOptions = WriteOptions { way: WriteMode::Replace, mode: Mode::Default, read: false };

pub type Loop<F = Tree> = Run<Git<F>, Agents<F>, Clock, F>;

pub struct World<F = Tree> {
    pub home: Home,
    files: Box<dyn Fn() -> F>,
    agents: fn(F) -> Agents<F>,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        let mut tree = Tree::default();
        let directory = Path::new("/qa");
        tree.directory(directory, DirectoryMode::One).expect("virtual home");
        Self::over(directory, move || tree.clone())
    }
}

impl World<RealFiles> {
    /// The real-loop suite supplies and keeps a scratch directory, and permits real program fallback.
    pub fn on_disk(directory: &Path) -> Self {
        Self { agents: Agents::on_disk, ..Self::over(directory, || RealFiles) }
    }
}

impl<F: FilesFace> World<F> {
    pub fn start(&self, git: Git<F>, agents: &[Agent]) -> Loop<F> {
        self.start_with_environment(git, agents, vec![(b"CAPTURED".as_slice().into(), b"kept".as_slice().into())])
    }

    pub fn start_with_environment(&self, mut git: Git<F>, agents: &[Agent], environment: Environment) -> Loop<F> {
        git.files = self.files();
        let mut processes = self.agents();
        processes.next.extend(agents.iter().copied());
        Run::start(self.home.clone(), git, processes, Clock::default(), self.files(), environment).expect("run")
    }

    /// Put the loop fixture's configuration in an existing home. Each call to `files` supplies
    /// a handle on the same paths, including the handles held by Git and Agents.
    pub fn over(directory: &Path, files: impl Fn() -> F + 'static) -> Self {
        let mut handle = files();
        handle.write(&directory.join("qa.json"), CONFIGURATION, REPLACE, false).expect("config");
        let home = Home::open(directory, &mut handle).expect("home");
        Self { home, files: Box::new(files), agents: Agents::new }
    }

    pub fn files(&self) -> F {
        (self.files)()
    }

    pub fn agents(&self) -> Agents<F> {
        (self.agents)(self.files())
    }

    pub fn ledger(&self) -> Ledger {
        self.home.read(&mut self.files(), |ledger| Ok(ledger.clone())).expect("read")
    }

    pub fn installable_build(&self, commit: u64) -> PathBuf {
        let mut files = self.files();
        let directory = self.build(commit);
        let mut manifest = format!("commit {}\n", id(commit));
        for (path, bytes) in [
            ("bin/temper-box", b"box".as_slice()),
            ("bin/fake-claude", b"fake".as_slice()),
            ("bin/temper", b"client".as_slice()),
            ("bin/temper-client-journeys", b"journeys".as_slice()),
            ("bundle/index.html", b"index".as_slice()),
            ("bundle/assets/app.js", b"javascript".as_slice()),
        ] {
            let full = directory.join(path);
            files.directory(full.parent().expect("parent"), DirectoryMode::Parents).expect("build directory");
            files.write(&full, bytes, REPLACE, false).expect("build file");
            if path.starts_with("bin/") {
                files.permissions(&full, 0o751).expect("executable");
            }
            writeln!(manifest, "{:x}  {path}", Sha256::digest(bytes)).expect("string");
        }
        files.write(&directory.join("manifest"), manifest.as_bytes(), REPLACE, false).expect("manifest");
        directory
    }

    pub fn build(&self, commit: u64) -> PathBuf {
        let mut files = self.files();
        let directory = self.home.builds.join(id(commit));
        files.directory(&directory, DirectoryMode::Parents).expect("build directory");
        self.home.transact(&mut files, |ledger, _| ledger.keep_build(&id(commit), 0)).expect("kept");
        directory
    }
}

pub fn land<F: FilesFace>(run: &mut Loop<F>, commit: u64, goal: u64) {
    let parent = u64::from_str_radix(&run.git.head, 16).expect("short id");
    run.git.commit(commit - 1, &[parent], "goal tip");
    run.git.merge(commit, &[parent, commit - 1], goal);
}

pub(super) fn path_io<T>(path: &Path, result: io::Result<T>) -> Result<T, Refusal> {
    result.map_err(|why| refused(format!("{}: {why}", path.display())))
}
