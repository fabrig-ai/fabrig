//! Staged builds are checked before publication (temper/docs/design/testing/qa/builds.md, 3 and 4).
use sha2::{Digest, Sha256};
use std::{
    fmt::Write,
    io,
    path::{Path, PathBuf},
};
use temper_qa_tool::{
    BuildState, Exit, Home, Refusal,
    build::keep_with,
    faces::{
        DirectoryMode, FileEntry, FilesFace, LockMode, PathFacts, PathKind, ProcessOutput, RemoveMode, WriteOptions,
    },
    testing::{Agents, Clock, Connection, FileHandle, Listener, Lock, Tree, World},
};

const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const SHORT: &str = "0123456789ab";
const REQUIRED: [&str; 8] = [
    "bin/temper-box",
    "bin/fake-claude",
    "bin/temper",
    "bin/temper-client-journeys",
    "bin/temper-engine",
    "bin/temper-worker",
    "bin/temper-agent",
    "bundle/index.html",
];

fn stage() -> (Tree, Home, PathBuf, Agents) {
    let world = World::new();
    let mut tree = world.files();
    let mut processes = world.agents();
    let stage = world.home.directory.join("stage");
    for directory in ["bin", "bundle"] {
        tree.directory(&stage.join(directory), DirectoryMode::Parents).expect("stage directory");
    }
    for binary in ["temper-box", "temper", "temper-client-journeys", "temper-engine", "temper-worker", "temper-agent"] {
        executable(&mut tree, &mut processes, &stage.join("bin").join(binary), &format!("{binary} 0.1.0 ({SHORT})"));
    }
    // This executable's version would fail the commit check applied to the temper binaries.
    executable(&mut tree, &mut processes, &stage.join("bin/fake-claude"), "2.1.312 (Claude Code)");
    tree.write(&stage.join("bundle/index.html"), b"web\n", WriteOptions::REPLACE, false).expect("bundle");
    (tree, world.home, stage, processes)
}

fn executable(tree: &mut Tree, processes: &mut Agents, path: &Path, line: &str) {
    let stdout = format!("{line}\n").into_bytes();
    tree.write(path, &stdout, WriteOptions::REPLACE, false).expect("executable bytes");
    tree.permissions(path, 0o700).expect("executable");
    processes.register(path.as_os_str(), move |command, _| {
        assert_eq!(command.arguments, ["--version"]);
        Ok(ProcessOutput {
            exit: Exit::Code(0),
            status: "exit status: 0".into(),
            stdout: stdout.clone(),
            stderr: Vec::new(),
        })
    });
}

fn refusal(tree: &mut Tree, processes: &mut Agents, home: &Home, stage: &Path, words: &str) {
    let before = tree.read(&home.ledger).ok();
    assert_eq!(
        keep_with(home, COMMIT, stage, &[], tree, processes, &Clock::default()).expect_err("stage refused").to_string(),
        words
    );
    assert_eq!(tree.kind(stage), Some(PathKind::Directory), "staging was moved");
    assert!(!tree.exists(home.builds.join(COMMIT)), "build was published");
    assert!(!tree.exists(home.builds.join(format!(".{COMMIT}.partial"))), "staging was moved to partial");
    assert_eq!(tree.read(&home.ledger).ok(), before, "ledger was changed");
}

#[test]
fn missing_deployment_binaries_are_named_before_anything_moves() {
    for missing in [vec!["bin/temper-worker"], vec!["bin/temper-engine", "bin/temper-agent"]] {
        let (mut tree, home, stage, mut processes) = stage();
        for path in &missing {
            tree.remove(&stage.join(path), RemoveMode::File).expect("remove required file");
        }
        let words =
            missing.iter().map(|path| format!("{} lacks {path}", stage.display())).collect::<Vec<_>>().join("; ");
        refusal(&mut tree, &mut processes, &home, &stage, &words);
    }
}

#[test]
fn a_deployment_stage_keeps_the_coding_agent_version_and_checksums_all_eight_files() {
    let (mut tree, home, stage, mut processes) = stage();
    let staged = tree.facts(&stage, false).expect("staged directory");
    let mut paths = REQUIRED;
    paths.sort_unstable();
    let mut expected = format!("commit {COMMIT}\n");
    for path in paths {
        let bytes = tree.read(&stage.join(path)).expect("staged bytes");
        writeln!(expected, "{:x}  {path}", Sha256::digest(&bytes)).expect("manifest row");
    }
    assert_eq!(
        keep_with(&home, COMMIT, &stage, &[], &mut tree, &mut processes, &Clock::default()).expect("kept"),
        format!("build {SHORT} kept: 8 files\n")
    );
    assert!(!tree.exists(&stage));
    assert!(!tree.exists(home.builds.join(format!(".{COMMIT}.partial"))));
    let kept = home.builds.join(COMMIT);
    let published = tree.facts(&kept, false).expect("published directory");
    assert_eq!((published.device, published.inode), (staged.device, staged.inode));
    for path in paths {
        assert_eq!(tree.kind(kept.join(path)), Some(PathKind::File), "{path}");
    }
    assert_eq!(tree.read_to_string(&kept.join("manifest")).expect("manifest"), expected);
    assert!(!tree.exists(kept.join("bundle-record")));
    home.read(&mut tree, |ledger| {
        assert_eq!(ledger.builds.len(), 1);
        assert_eq!(ledger.builds[0].commit, COMMIT);
        assert_eq!(ledger.builds[0].state, BuildState::Kept);
        assert_eq!(ledger.builds[0].holds, None);
        Ok(())
    })
    .expect("ledger");
}

#[test]
fn an_older_recording_bundle_is_kept_in_the_manifest() {
    let (mut tree, home, stage, mut processes) = stage();
    tree.directory(&stage.join("bundle-record"), DirectoryMode::One).expect("recording directory");
    tree.write(&stage.join("bundle-record/index.html"), b"record\n", WriteOptions::REPLACE, false)
        .expect("recording bundle");
    assert_eq!(
        keep_with(&home, COMMIT, &stage, &[], &mut tree, &mut processes, &Clock::default()).expect("kept"),
        format!("build {SHORT} kept: 9 files\n")
    );
    let kept = home.builds.join(COMMIT);
    assert_eq!(tree.read(&kept.join("bundle-record/index.html")).expect("kept recording"), b"record\n");
    let manifest = tree.read_to_string(&kept.join("manifest")).expect("manifest");
    assert!(
        manifest.lines().any(|line| line == format!("{:x}  bundle-record/index.html", Sha256::digest(b"record\n")))
    );
}

#[test]
fn the_engine_must_print_the_kept_commit() {
    let (mut tree, home, stage, mut processes) = stage();
    executable(&mut tree, &mut processes, &stage.join("bin/temper-engine"), "temper-engine 0.1.0 (abcdef012345)");
    refusal(
        &mut tree,
        &mut processes,
        &home,
        &stage,
        &format!("bin/temper-engine prints temper-engine 0.1.0 (abcdef012345), not {SHORT}"),
    );
}

#[test]
fn even_the_coding_agent_and_extra_bin_files_must_be_executable() {
    let (mut tree, home, stage, mut processes) = stage();
    for path in ["bin/fake-claude", "bin/temper-client-journeys", "bin/extra"] {
        let full = stage.join(path);
        if !tree.exists(&full) {
            executable(&mut tree, &mut processes, &full, &format!("extra ({SHORT})"));
        }
        tree.permissions(&full, 0o600).expect("not executable");
        refusal(&mut tree, &mut processes, &home, &stage, &format!("{path} is not executable"));
        tree.permissions(&full, 0o700).expect("restore executable");
    }
}

#[test]
fn the_drive_tool_and_every_file_but_fake_claude_must_print_the_kept_commit() {
    let (mut tree, home, stage, mut processes) = stage();
    let path = stage.join("bin/temper-client-journeys");
    executable(&mut tree, &mut processes, &path, "temper-client-journeys 0.1.0 (abcdef012345)");
    refusal(
        &mut tree,
        &mut processes,
        &home,
        &stage,
        &format!("bin/temper-client-journeys prints temper-client-journeys 0.1.0 (abcdef012345), not {SHORT}"),
    );
    executable(&mut tree, &mut processes, &path, &format!("temper-client-journeys 0.1.0 ({SHORT})"));
    executable(&mut tree, &mut processes, &stage.join("bin/fake-claude-extra"), "2.1.312 (Claude Code)");
    refusal(
        &mut tree,
        &mut processes,
        &home,
        &stage,
        &format!("bin/fake-claude-extra prints 2.1.312 (Claude Code), not {SHORT}"),
    );
}

#[test]
fn a_manifest_uses_one_sha256sum_faces_answer_for_every_sorted_byte_path() {
    manifest_keeps_path(Path::new("bundle/weird\\\n\r"), b"bundle/weird\\\\\\n\\r");
}

#[test]
fn a_manifest_keeps_a_non_utf8_path_as_its_bytes() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};
    manifest_keeps_path(
        &PathBuf::from(OsString::from_vec(b"bundle/weird\\\n\r\xff".to_vec())),
        b"bundle/weird\\\\\\n\\r\xff",
    );
}

fn manifest_keeps_path(name: &Path, escaped: &[u8]) {
    let (mut tree, home, stage, mut processes) = stage();
    tree.write(&stage.join(name), b"asset", WriteOptions::REPLACE, false).expect("byte path");
    let mut paths: Vec<_> = REQUIRED.iter().map(PathBuf::from).collect();
    paths.push(name.into());
    paths.sort();
    let mut answer = Vec::new();
    for path in &paths {
        // The face's hashes deliberately differ from the files' bytes.
        let line = if path == name {
            let mut line = format!("\\{}  ", "a".repeat(64)).into_bytes();
            line.extend_from_slice(escaped);
            line.push(b'\n');
            line
        } else {
            format!("{}  {}\n", "b".repeat(64), path.display()).into_bytes()
        };
        answer.extend_from_slice(&line);
    }
    let mut face = ScriptedHash { tree: tree.clone(), answer: answer.clone(), hashes: Vec::new() };
    keep_with(&home, COMMIT, &stage, &[], &mut face, &mut processes, &Clock::default()).expect("kept");
    assert_eq!(face.hashes, [(stage, paths)]);
    let mut expected = format!("commit {COMMIT}\n").into_bytes();
    expected.extend_from_slice(&answer);
    assert_eq!(tree.read(&home.builds.join(COMMIT).join("manifest")).expect("manifest"), expected);
}

#[test]
fn a_sha256sum_answer_missing_a_file_or_not_parsing_never_moves_or_records() {
    for (answer, expected) in [
        (format!("{}  bin/fake-claude\n", "0".repeat(64)).into_bytes(), "absent from sha256sum answer"),
        (b"not a checksum\n".to_vec(), "invalid checksum line"),
    ] {
        let (tree, home, stage, mut processes) = stage();
        let mut face = ScriptedHash { tree: tree.clone(), answer, hashes: Vec::new() };
        let why =
            keep_with(&home, COMMIT, &stage, &[], &mut face, &mut processes, &Clock::default()).expect_err("refused");
        assert!(why.0.contains(expected), "{why}");
        assert_eq!(tree.kind(&stage), Some(PathKind::Directory));
        assert!(!tree.exists(home.builds.join(COMMIT)));
        assert!(!tree.exists(home.builds.join(format!(".{COMMIT}.partial"))));
        assert!(!tree.exists(&home.ledger));
    }
}

// The stage and transactions still go through the tree; only the checksum answer is scripted.
struct ScriptedHash {
    tree: Tree,
    answer: Vec<u8>,
    hashes: Vec<(PathBuf, Vec<PathBuf>)>,
}

impl FilesFace for ScriptedHash {
    type Lock = Lock;
    type Listener = Listener;
    type Connection = Connection;
    type Reader = FileHandle;
    type Writer = FileHandle;

    fn read(&mut self, path: &Path) -> io::Result<Vec<u8>> {
        self.tree.read(path)
    }
    fn write(&mut self, path: &Path, bytes: &[u8], options: WriteOptions, sync: bool) -> io::Result<()> {
        self.tree.write(path, bytes, options, sync)
    }
    fn directory(&mut self, path: &Path, mode: DirectoryMode) -> io::Result<()> {
        self.tree.directory(path, mode)
    }
    fn facts(&mut self, path: &Path, follow: bool) -> io::Result<PathFacts> {
        self.tree.facts(path, follow)
    }
    fn list(&mut self, path: &Path) -> io::Result<Vec<FileEntry>> {
        self.tree.list(path)
    }
    fn remove(&mut self, path: &Path, mode: RemoveMode) -> io::Result<()> {
        self.tree.remove(path, mode)
    }
    fn rename(&mut self, source: &Path, destination: &Path) -> io::Result<()> {
        self.tree.rename(source, destination)
    }
    fn lock(&mut self, path: &Path, mode: LockMode, exact: bool) -> io::Result<Option<Lock>> {
        self.tree.lock(path, mode, exact)
    }
    fn connect(&mut self, path: &Path) -> io::Result<Option<Connection>> {
        self.tree.connect(path)
    }
    fn hash(&mut self, directory: &Path, files: &[PathBuf]) -> Result<Vec<u8>, Refusal> {
        self.hashes.push((directory.into(), files.into()));
        Ok(self.answer.clone())
    }
}
