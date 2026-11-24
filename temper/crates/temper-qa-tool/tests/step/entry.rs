//! The QA entry answers admission before touching its faces (temper/docs/design/testing/qa/contracts.md, 3).
use std::{
    ffi::{OsStr, OsString},
    io::{self, Read},
    path::{Path, PathBuf},
    time::Duration,
};
use temper_qa_tool::{
    CommandFaces, Exit, Refusal, TABLE, answer,
    events::{Events, Waker},
    faces::{
        ArchiveFace, Changed, ClockFace, CommandAnswer, Commit, DirectoryMode, DirectoryScan, DriveFace, DriveProcess,
        Entry, EnvironmentEntry, FileEntry, FilesFace, GitFace, LockMode, Moment, OutputCommand, PathFacts,
        ProcessCommand, ProcessOutput, Processes, RecipeWalk, RemoveMode, ResolveMode, ScannedEntry, Signal,
        WalkOutput, WriteOptions,
    },
    place::PlacePaths,
};

struct Untouched;

// Implement even the face defaults: an unexpected call must fail, not return a silent no-op or an I/O refusal.
macro_rules! deny_effects {
    ($(fn $method:ident($($args:tt)*) $(-> $answer:ty)?;)*) => {$(
        fn $method($($args)*) $(-> $answer)? {
            unreachable!("admission touched {}", stringify!($method))
        }
    )*};
}

impl FilesFace for Untouched {
    type Reader = io::Empty;
    type Writer = io::Sink;
    type Connection = io::Cursor<Vec<u8>>;
    type Listener = ();
    type Lock = ();

    deny_effects! {
        fn read(&mut self, _path: &Path) -> io::Result<Vec<u8>>;
        fn read_to_string(&mut self, _path: &Path) -> io::Result<String>;
        fn open_read(&mut self, _path: &Path) -> io::Result<Self::Reader>;
        fn write(&mut self, _path: &Path, _bytes: &[u8], _options: WriteOptions, _sync: bool) -> io::Result<()>;
        fn open_write(&mut self, _path: &Path, _options: WriteOptions) -> io::Result<Self::Writer>;
        fn diagnostic(&mut self, _line: &str);
        fn reader_facts(&mut self, _file: &Self::Reader) -> io::Result<PathFacts>;
        fn writer_permissions(&mut self, _file: &Self::Writer, _mode: u32) -> io::Result<()>;
        fn sync_writer(&mut self, _file: &Self::Writer) -> io::Result<()>;
        fn open_log_tail(&mut self, _path: &Path) -> Result<Self::Reader, Refusal>;
        fn rename(&mut self, _from: &Path, _to: &Path) -> io::Result<()>;
        fn link(&mut self, _from: &Path, _to: &Path) -> io::Result<()>;
        fn directory(&mut self, _path: &Path, _mode: DirectoryMode) -> io::Result<()>;
        fn remove(&mut self, _path: &Path, _mode: RemoveMode) -> io::Result<()>;
        fn list(&mut self, _path: &Path) -> io::Result<Vec<FileEntry>>;
        fn names(&mut self, _path: &Path) -> io::Result<DirectoryScan<Box<[u8]>>>;
        fn scan(&mut self, _path: &Path) -> io::Result<DirectoryScan<ScannedEntry>>;
        fn facts(&mut self, _path: &Path, _follow: bool) -> io::Result<PathFacts>;
        fn permissions(&mut self, _path: &Path, _mode: u32) -> io::Result<()>;
        fn copy(&mut self, _from: &Path, _to: &Path) -> io::Result<u64>;
        fn resolve(&mut self, _path: &Path, _mode: ResolveMode) -> io::Result<PathBuf>;
        fn lock(&mut self, _path: &Path, _mode: LockMode, _exact: bool) -> io::Result<Option<Self::Lock>>;
        fn open_lock(&mut self, _path: &Path, _exact: bool) -> io::Result<Self::Lock>;
        fn try_lock(&mut self, _lock: &Self::Lock) -> io::Result<bool>;
        fn bind(&mut self, _path: &Path) -> io::Result<Self::Listener>;
        fn bind_wake(&mut self, _path: &Path) -> io::Result<Self::Listener>;
        fn accept_wake(&mut self, _listener: &Self::Listener) -> io::Result<bool>;
        fn serve_wakes(&mut self, _listener: Self::Listener, _waker: Waker) -> Result<(), Refusal>;
        fn accept(&mut self, _listener: &Self::Listener) -> io::Result<Option<Self::Connection>>;
        fn connect(&mut self, _path: &Path) -> io::Result<Option<Self::Connection>>;
        fn hash(&mut self, _directory: &Path, _paths: &[PathBuf]) -> Result<Vec<u8>, Refusal>;
        fn check(&mut self, _directory: &Path, _manifest: &Path) -> Result<(), Refusal>;
    }
}

impl GitFace for Untouched {
    deny_effects! {
        fn set_directory(&mut self, _directory: &Path);
        fn fetch(&mut self, _remote: &str, _branch: &str) -> Result<(), Refusal>;
        fn head(&mut self, _reference: &str) -> Result<String, Refusal>;
        fn commits(&mut self, _old: &str, _new: &str) -> Result<Vec<Commit>, Refusal>;
        fn parents(&mut self, _commit: &str) -> Result<Vec<String>, Refusal>;
        fn message(&mut self, _commit: &str) -> Result<String, Refusal>;
        fn contains(&mut self, _ancestor: &str, _descendant: &str) -> Result<bool, Refusal>;
        fn count(&mut self, _commit: &str) -> Result<u64, Refusal>;
        fn entries(&mut self, _commit: &str, _directory: &str) -> Result<Vec<Entry>, Refusal>;
        fn file(&mut self, _commit: &str, _path: &str) -> Result<Option<Vec<u8>>, Refusal>;
        fn diff(&mut self, _parent: &str, _commit: &str) -> Result<Vec<u8>, Refusal>;
        fn stat(&mut self, _parent: &str, _commit: &str) -> Result<Vec<Changed>, Refusal>;
        fn changed_paths(&mut self, _parent: &str, _commit: &str) -> Result<Vec<String>, Refusal>;
        fn qa_trailers(&mut self, _parent: &str, _commit: &str) -> Result<Vec<String>, Refusal>;
        fn gap_trailers(&mut self, _parent: &str, _commit: &str) -> Result<Vec<String>, Refusal>;
        fn add_tree(&mut self, _path: &Path, _commit: &str) -> Result<(), Refusal>;
        fn move_tree(&mut self, _path: &Path, _commit: &str) -> Result<(), Refusal>;
        fn remove_tree(&mut self, _path: &Path) -> Result<(), Refusal>;
        fn prune(&mut self) -> Result<(), Refusal>;
    }
}

impl Processes for Untouched {
    type Connection = io::Cursor<Vec<u8>>;

    deny_effects! {
        fn wake_on_exit(&mut self, _waker: Waker);
        fn run(&mut self, _command: &ProcessCommand, _seconds: u64) -> Result<CommandAnswer, Refusal>;
        fn start(&mut self, _command: &ProcessCommand) -> Result<u64, Refusal>;
        fn poll(&mut self, _process: u64) -> Result<Option<Exit>, Refusal>;
        fn signal(&mut self, _process: u64, _signal: Signal) -> Result<(), Refusal>;
        fn output(&mut self, _command: &OutputCommand<'_>) -> io::Result<ProcessOutput>;
        fn version(&mut self, _file: &Path, _capture: &Path, _environment: &[EnvironmentEntry]) -> Result<String, Refusal>;
        fn holder(&mut self, _log: &Path, _ctl: bool, _environment: &[EnvironmentEntry], _release: impl FnOnce()) -> Result<u64, Refusal>;
        fn drive(&mut self, _program: &OsStr, _arguments: &[OsString], _environment: &[EnvironmentEntry]) -> Result<DriveProcess<Self::Connection>, Refusal>;
        fn end_holder(&mut self, _place: PathBuf, _waker: Waker);
    }
}

impl ClockFace for Untouched {
    deny_effects! {
        fn now(&self) -> Moment;
        fn monotonic(&self) -> Duration;
        fn sleep(&mut self, _duration: Duration);
        fn wait(&mut self, _events: &Events, _timeout: Option<u64>);
    }
}

impl DriveFace for Untouched {
    deny_effects! {
        fn list(&mut self, _place: &PlacePaths, _setups: &Path, _processes: &mut impl Processes) -> Result<String, Refusal>;
        fn hash(&mut self, _place: &PlacePaths, _setups: &Path, _name: &str, _processes: &mut impl Processes) -> Result<String, Refusal>;
        fn walk(&mut self, _walk: &RecipeWalk<'_>, _files: &mut impl FilesFace, _processes: &mut impl Processes) -> Result<WalkOutput, Refusal>;
    }
}

impl ArchiveFace for Untouched {
    deny_effects! {
        fn create(&mut self, _scratch: &Path, _archive: &Path, _left_out: &[&str], _files: &mut impl FilesFace, _processes: &mut impl Processes) -> Result<(), Refusal>;
        fn extract(&mut self, _archive: &Path, _scratch: &Path, _left_out: &[&str], _files: &mut impl FilesFace, _processes: &mut impl Processes) -> Result<(), Refusal>;
    }
}

impl Read for Untouched {
    deny_effects! {
        fn read(&mut self, _bytes: &mut [u8]) -> io::Result<usize>;
    }
}

#[test]
fn the_entry_answers_admission_without_touching_any_face() {
    const FACES: CommandFaces<Untouched, Untouched, Untouched, Untouched, Untouched, Untouched> = CommandFaces {
        git: Untouched,
        processes: Untouched,
        clock: Untouched,
        files: Untouched,
        drive: Untouched,
        archive: Untouched,
    };
    const BAD_ARGUMENTS: &str = "temper-qa triage case: FILE is not given\nusage:\n  temper-qa triage case FILE\n";
    const UNKNOWN: &str = "temper-qa: no command nope; temper-qa --help says more\n";
    let help = TABLE.help();
    let rows: &[(&[&str], u8, &str, &str)] =
        &[(&["--help"], 0, &help, ""), (&["triage", "case"], 2, "", BAD_ARGUMENTS), (&["nope"], 2, "", UNKNOWN)];
    for &(words, code, out, err) in rows {
        let arguments: Vec<_> = words.iter().map(|word| word.as_bytes().into()).collect();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let exit = answer(FACES, "1.2.3", "0123456789ab", &arguments, &[], &mut Untouched, &mut stdout, &mut stderr);
        assert_eq!((exit, stdout.as_slice(), stderr.as_slice()), (code, out.as_bytes(), err.as_bytes()), "{words:?}");
    }
}
