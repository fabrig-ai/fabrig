//! The QA world's answers to the runner (temper/docs/design/testing/qa/contracts.md, 4).
use std::{
    ffi::{OsStr, OsString},
    io::{ErrorKind, Read, Write},
    os::unix::ffi::OsStrExt,
    path::Path,
    time::Duration,
};
use temper_qa_tool::{
    Exit,
    events::Events,
    faces::{
        ClockFace, DirectoryMode, FileEntry, FilesFace, GitFace, LockMode, Mode, OutputCommand, PathKind,
        ProcessCommand, ProcessOutput, Processes, RemoveMode, WriteMode, WriteOptions,
    },
    testing::{Agents, Child, Clock, Connection, DriveStreams, FileOp, Git, Tree},
};

fn write(tree: &mut Tree, path: &str, bytes: &[u8]) {
    tree.write(Path::new(path), bytes, WriteOptions::private(WriteMode::Replace), false).expect("seed file");
}

#[test]
fn private_modes_are_exact() {
    let mut tree = Tree::default();
    tree.default_modes(0o666, 0o777);
    let directory = Path::new("/nested/home");
    let file = directory.join("ledger");
    tree.directory(directory, DirectoryMode::Private).expect("make private directory");
    assert_eq!(tree.mode("/nested"), Some(0o700));
    assert_eq!(tree.mode(directory), Some(0o700));
    tree.permissions(directory, 0o777).expect("broader mode");
    tree.directory(directory, DirectoryMode::Private).expect("tighten existing directory");
    assert_eq!(tree.mode(directory), Some(0o700));
    tree.write(&file, b"first", WriteOptions::private(WriteMode::New), true).expect("private file");
    assert_eq!(tree.mode(&file), Some(0o600));
    tree.permissions(&file, 0o666).expect("broader mode");
    tree.write(&file, b"second", WriteOptions::private(WriteMode::Replace), false).expect("tighten existing file");
    assert_eq!(tree.mode(file), Some(0o600));
}

#[test]
fn replace_new_and_append_have_distinct_effects() {
    let mut tree = Tree::default();
    let path = Path::new("/file");
    let options = |way| WriteOptions { way, mode: Mode::Default, read: false };
    tree.write(path, b"first", options(WriteMode::New), false).expect("new");
    assert_eq!(tree.write(path, b"lost", options(WriteMode::New), false).unwrap_err().kind(), ErrorKind::AlreadyExists);
    assert_eq!(tree.read(path).expect("unchanged"), b"first");
    tree.write(path, b"second", options(WriteMode::Append), false).expect("append");
    assert_eq!(tree.read(path).expect("both"), b"firstsecond");
    tree.write(path, b"last", options(WriteMode::Replace), false).expect("replace");
    assert_eq!(tree.read(path).expect("only last"), b"last");
}

#[test]
fn rename_replaces_the_destination() {
    let mut tree = Tree::default();
    write(&mut tree, "/temp", b"new");
    write(&mut tree, "/document", b"old");
    tree.rename(Path::new("/temp"), Path::new("/document")).expect("rename");
    assert!(!tree.exists("/temp"));
    assert_eq!(tree.read(Path::new("/document")).expect("new document"), b"new");
}

#[test]
fn remove_unlinks_a_symlink_without_visiting_its_target() {
    let mut tree = Tree::default();
    write(&mut tree, "/target", b"keep");
    tree.symlink("target", "/link").expect("symlink");
    assert_eq!(tree.read(Path::new("/link")).expect("follow"), b"keep");
    tree.remove(Path::new("/link"), RemoveMode::Any).expect("unlink");
    assert!(!tree.exists("/link"));
    assert_eq!(tree.read(Path::new("/target")).expect("target"), b"keep");
}

#[test]
fn list_answers_kinds_and_byte_names() {
    let mut tree = Tree::default();
    tree.directory(Path::new("/dir"), DirectoryMode::One).expect("directory");
    let name = b"file\xff";
    tree.write(&Path::new("/").join(OsStr::from_bytes(name)), b"bytes", WriteOptions::private(WriteMode::New), false)
        .expect("byte path");
    tree.symlink("dir", "/link").expect("link");
    let _socket = tree.socket("/sock").expect("socket");
    assert_eq!(
        tree.list(Path::new("/")).expect("entries"),
        [
            FileEntry { name: b"dir".as_slice().into(), kind: PathKind::Directory },
            FileEntry { name: name.as_slice().into(), kind: PathKind::File },
            FileEntry { name: b"link".as_slice().into(), kind: PathKind::Link },
            FileEntry { name: b"sock".as_slice().into(), kind: PathKind::Socket },
        ]
    );
}

#[test]
fn hard_links_share_identity_but_cannot_cross_devices() {
    let mut tree = Tree::default();
    tree.directory(Path::new("/volume"), DirectoryMode::One).expect("volume");
    tree.set_device("/volume", 22).expect("second device");
    write(&mut tree, "/file", b"bytes");
    let file = Path::new("/file");
    let alias = Path::new("/alias");
    tree.link(file, alias).expect("same device");
    let original = tree.facts(file, false).expect("file facts");
    assert_eq!(tree.facts(alias, false).expect("alias facts"), original);
    assert_eq!((original.device, original.links, original.size), (1, 2, 5));
    tree.permissions(alias, 0o640).expect("shared mode");
    assert_eq!(tree.mode(file), Some(0o640));
    tree.remove(alias, RemoveMode::File).expect("unlink alias");
    assert_eq!(tree.facts(file, false).expect("remaining link").links, 1);
    assert_eq!(tree.facts(Path::new("/volume"), false).expect("device").device, 22);
    assert_eq!(tree.link(file, Path::new("/volume/copy")).unwrap_err().kind(), ErrorKind::CrossesDevices);
    assert!(!tree.exists("/volume/copy"));
}

#[test]
fn locks_exclude_other_handles_until_drop() {
    let mut tree = Tree::default();
    let mut other = tree.clone();
    let path = Path::new("/drive.lock");
    let lock = tree.lock(path, LockMode::Wait, false).expect("lock call").expect("held");
    assert_eq!(tree.mode(path), Some(0o600));
    assert!(other.lock(path, LockMode::Try, false).expect("try local lock").is_none());
    drop(lock);
    let external = tree.hold_lock(path).expect("other process holds the lock");
    assert!(other.lock(path, LockMode::Try, false).expect("try external lock").is_none());
    drop(external);
    assert!(other.lock(path, LockMode::Try, false).expect("try free lock").is_some());
}

#[test]
fn bind_replaces_stale_socket_entries() {
    let mut tree = Tree::default();
    let path = Path::new("/run.sock");
    write(&mut tree, "/run.sock", b"stale");
    let listener = tree.bind(path).expect("replace stale path");
    assert_eq!(tree.kind(path), Some(PathKind::Socket));
    assert_eq!(tree.mode(path), Some(0o600));
    assert!(tree.accept(&listener).expect("no waiting").is_none());
    let _client = tree.connect(path).expect("connect").expect("bound");
    assert!(tree.accept(&listener).expect("accept").is_some());
}

#[test]
fn missing_or_unbound_sockets_are_absent() {
    let mut tree = Tree::default();
    let path = Path::new("/run.sock");
    assert!(tree.connect(path).expect("missing").is_none());
    let listener = tree.socket(path).expect("bind");
    drop(listener);
    assert_eq!(tree.kind(path), Some(PathKind::Socket));
    assert!(tree.connect(path).expect("unbound").is_none());
}

#[test]
fn connected_bytes_reach_the_accepted_end() {
    let mut tree = Tree::default();
    let path = Path::new("/drive.sock");
    let listener = tree.socket(path).expect("bind");
    let mut client = tree.clone().connect(path).expect("connect").expect("client");
    let mut server = tree.accept(&listener).expect("accept").expect("server");
    let mut bytes = [0; 4];
    assert_eq!(server.read(&mut bytes).unwrap_err().kind(), ErrorKind::WouldBlock);
    client.write_all(b"wake").expect("send");
    drop(client);
    server.read_exact(&mut bytes).expect("receive queued bytes");
    assert_eq!(&bytes, b"wake");
    assert_eq!(server.read(&mut bytes).expect("EOF"), 0);
}

#[test]
fn injected_failures_match_the_path_and_operation_once() {
    let mut tree = Tree::default();
    write(&mut tree, "/file", b"before");
    let path = Path::new("/file");
    tree.inject(path, FileOp::Write, std::io::Error::new(ErrorKind::PermissionDenied, "scripted refusal"));
    write(&mut tree, "/other", b"not the path");
    assert_eq!(tree.read(path).expect("not the operation"), b"before");
    let error = tree.clone().write(path, b"lost", WriteOptions::private(WriteMode::Replace), false).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::PermissionDenied);
    assert_eq!(error.to_string(), "scripted refusal");
    assert_eq!(tree.read(path).expect("unchanged"), b"before");
    write(&mut tree, "/file", b"after");
}

#[test]
fn a_program_sees_the_command_and_the_live_tree() {
    let mut tree = Tree::default();
    let mut agents = Agents::new(tree.clone());
    let directory = Path::new("/work");
    let arguments = [OsString::from("report")];
    let environment = [(Box::from(b"MODE".as_slice()), Box::from(b"draft".as_slice()))];
    agents.register("tool", move |command, files| {
        assert_eq!(command.program, "/bin/tool");
        assert_eq!(command.arguments, ["report"]);
        assert_eq!(command.directory, Some(directory));
        assert_eq!(command.environment, [(Box::from(b"MODE".as_slice()), Box::from(b"draft".as_slice()))]);
        let bytes = files.read(&directory.join("input"))?;
        files.write(&directory.join("out"), &bytes, WriteOptions::private(WriteMode::New), false)?;
        Ok(ProcessOutput { exit: Exit::Code(3), stdout: b"report".into(), stderr: b"diagnostic".into() })
    });
    tree.directory(directory, DirectoryMode::One).expect("work directory");
    write(&mut tree, "/work/input", b"written after registration");
    let answer = agents
        .output(&OutputCommand {
            program: OsStr::new("/bin/tool"),
            arguments: &arguments,
            directory: Some(directory),
            environment: &environment,
            log: None,
        })
        .expect("scripted output");
    assert_eq!(answer.exit, Exit::Code(3));
    assert_eq!(answer.stdout, b"report");
    assert_eq!(answer.stderr, b"diagnostic");
    assert_eq!(tree.read(&directory.join("out")).expect("handler's file"), b"written after registration");
}

#[test]
fn bounded_programs_answer_from_stdout_and_keep_both_streams() {
    let mut tree = Tree::default();
    let mut agents = Agents::new(tree.clone());
    agents.register("file", |command, _| {
        assert_eq!(command.arguments, ["goal"]);
        Ok(ProcessOutput { exit: Exit::Code(0), stdout: b"progress\ngoal 41\n \n".into(), stderr: b"goal 99\n".into() })
    });
    let command = ProcessCommand {
        arguments: vec!["file".into(), "goal".into()],
        directory: "/".into(),
        stdin: "/unused".into(),
        output: "/output".into(),
        environment: vec![],
    };
    let answer = agents.run(&command, 20).expect("scripted request command");
    assert_eq!(answer.last_stdout.as_deref(), Some("goal 41"));
    assert_eq!(tree.read(&command.output).expect("kept streams"), b"goal 99\nprogress\ngoal 41\n \n");
}

#[test]
fn version_reads_the_first_line_of_scripted_stdout() {
    let mut agents = Agents::new(Tree::default());
    agents.register("/stage/bin/tool", |command, _| {
        assert_eq!(command.arguments, ["--version"]);
        Ok(ProcessOutput {
            exit: Exit::Code(8),
            stdout: b"tool v1\r\nextra\n".into(),
            stderr: b"not a version\n".into(),
        })
    });
    let capture = Path::new("/qa/version.tmp");
    assert_eq!(agents.version(Path::new("/stage/bin/tool"), capture, &[]).expect("scripted version"), "tool v1");
}

#[test]
fn sleeping_advances_both_clock_readings() {
    let mut clock = Clock::default();
    clock.time = 10;
    clock.sleep(Duration::from_millis(750));
    assert_eq!(clock.now().wall, 10);
    assert_eq!(clock.monotonic(), Duration::from_millis(10_750));
    clock.sleep(Duration::from_millis(500));
    assert_eq!(clock.now().wall, 11);
    assert_eq!(clock.now().monotonic, 11);
    assert_eq!(clock.monotonic(), Duration::from_millis(11_250));
}

#[test]
fn git_worktrees_live_in_the_files_face() {
    let mut tree = Tree::default();
    let mut git = Git::new(tree.clone());
    let worktree = Path::new("/trees/triage");
    git.add_tree(worktree, "first").expect("add worktree");
    write(&mut tree, "/trees/triage/old", b"left by agent");
    git.move_tree(worktree, "second").expect("checkout and clean");
    assert_eq!(tree.kind(worktree), Some(PathKind::Directory));
    assert!(!tree.exists(worktree.join("old")));
    git.remove_tree(worktree).expect("remove worktree");
    assert!(!tree.exists(worktree));
}

#[test]
fn a_scripted_holder_releases_the_probe_before_launch() {
    let tree = Tree::default();
    let mut agents = Agents::new(tree.clone());
    let lock_path = Path::new("/drive.lock");
    let probe = tree.hold_lock(lock_path).expect("probe");
    let log = Path::new("/drive.log");
    let child = Child::default();
    let launched = child.clone();
    agents.script_holder(move |command, files| {
        assert!(command.ctl);
        assert_eq!(command.log, log);
        assert_eq!(files.mode(log), Some(0o600));
        let _lock = files.hold_lock(lock_path).expect("probe released");
        Ok(launched)
    });
    let pid = agents.holder(log, true, &[], || drop(probe)).expect("holder");
    assert_eq!(agents.poll(pid).expect("running"), None);
    child.exit(Exit::Code(7));
    assert_eq!(agents.poll(pid).expect("exit"), Some(Exit::Code(7)));
}

#[test]
fn a_scripted_drive_has_separate_input_and_output() {
    let mut agents = Agents::new(Tree::default());
    let (input, mut stdin) = Connection::pair();
    let (output, mut stdout) = Connection::pair();
    agents.script_drive(move |command, _| {
        assert_eq!(command.program, "drive");
        Ok(DriveStreams { child: Child::default(), input, output })
    });
    let mut drive = agents.drive(OsStr::new("drive"), &[], &[]).expect("drive session");
    drive.input.write_all(b"line").expect("send line");
    let mut bytes = [0; 4];
    stdin.read_exact(&mut bytes).expect("stdin");
    assert_eq!(&bytes, b"line");
    stdout.write_all(b"okay").expect("send answer");
    drive.output.read_exact(&mut bytes).expect("stdout");
    assert_eq!(&bytes, b"okay");
}

#[test]
fn a_scripted_end_reaches_the_holder_releases_and_wakes() {
    let mut tree = Tree::default();
    let mut agents = Agents::new(tree.clone());
    let place = Path::new("/");
    let events = Events::default();
    let lock_path = place.join("drive.lock");
    let lock = tree.hold_lock(&lock_path).expect("holder lock");
    let listener = tree.socket(place.join("drive.sock")).expect("holder socket");
    let mut peer = listener.script_connection();
    agents.script_end_holder(place, move |_| {
        let mut request = [0; b"end\n".len()];
        peer.read_exact(&mut request).expect("end request");
        assert_eq!(&request, b"end\n");
        peer.write_all(b"0\nended\n").expect("end answer");
        drop(lock);
    });
    agents.end_holder(place.into(), events.waker());
    assert_eq!(agents.ended_holders, [place]);
    assert!(tree.lock(&lock_path, LockMode::Try, false).expect("drive lock").is_some());
    assert!(events.take());
}
