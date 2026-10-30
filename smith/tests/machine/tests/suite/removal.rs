//! Removal through the machine and skein's file driver on the real filesystem.

use std::fs;
use std::path::Path;

use skein_io::{
    file,
    file_layer::{self, FileIo},
};
use skein_lib::{Duration, Env, Queue, Time, Token};
use skein_shell::{Clock, Config, Kernel, Wait, open_root};
use smith_domain::{run, tools};
use smith_protocol_machine::{Below, BelowEvent, Component, FromDomain, ToDomain};

const ROOT: Token = Token::new(1);
const OWNER: Token = Token::new(2);
const CLOSE: Token = Token::new(3);

fn place(path: &[u8]) -> tools::Place {
    tools::Place { root: ROOT, path: path.into() }
}

fn remove(path: &[u8], expect: tools::Expect) -> tools::Op {
    tools::Op::Remove { at: place(path), expect }
}

fn perform(directory: &Path, op: tools::Op) -> tools::Done {
    let limits = super::listing::limits();
    let clock = Clock::new();
    let now = clock.now();
    let env = Env { now: now.now, wall: now.wall, limits };
    let mut kernel = Kernel::open(Config { operations: 16 }).expect("real file kernel");
    let mut files = FileIo::with_whole_limit(8, 16, 8, 16, Duration::from_secs(1));
    let root = files.adopt_root(open_root(directory).expect("scratch root")).expect("root slot");
    assert_eq!(root, ROOT);
    let mut component =
        Component::new(&limits, smith_protocol_machine::Environment::encode(&[], &limits).expect("empty environment"));
    component.workspace(&run::Workspace {
        directories: Box::new([run::Directory {
            host: b"/host".as_slice().into(),
            name: b"repo".as_slice().into(),
            root,
            writable: true,
            git: true,
            conflicts: Box::new([]),
        }]),
    });
    let mut up = Queue::with_capacity(4);
    let mut below = Queue::with_capacity(4);
    let mut events = Queue::with_capacity(16);
    let mut submissions = Queue::with_capacity(16);
    let mut completions = Queue::with_capacity(16);
    let mut result = None;
    component.from_domain(
        &env,
        FromDomain::Op { owner: OWNER, op, deadline: Time::from_nanos(u64::MAX) },
        &mut up,
        &mut below,
    );
    // Each iteration advances one finite file operation or its parent close.
    for _ in 0..128 {
        while let Some(request) = below.pop() {
            let (request, deadline) = file_request(&files, request);
            file_layer::down_until(&mut files, deadline, request, &mut events, &mut submissions);
        }
        while let Some(event) = events.pop() {
            if matches!(event, file::Event::Closed { owner } if owner == CLOSE) {
                assert_eq!(files.open_files(), 0, "operation and root released every descriptor");
                return result.expect("operation answered before closing root");
            }
            component.from_below(&env, BelowEvent::File(event), &mut up, &mut below);
        }
        if let Some(terminal) = up.pop() {
            assert!(result.is_none(), "one terminal per operation");
            result = Some(match terminal {
                ToDomain::Done { owner: OWNER, done } => done,
                other @ (ToDomain::Done { .. }
                | ToDomain::Read { .. }
                | ToDomain::Probed { .. }
                | ToDomain::Checked { .. }
                | ToDomain::Aborted { .. }) => panic!("file terminal: {other:?}"),
            });
            file_layer::down_until(
                &mut files,
                Time::from_nanos(u64::MAX),
                file::Request::Close { owner: CLOSE, file: root },
                &mut events,
                &mut submissions,
            );
        }
        if below.is_empty() && events.is_empty() {
            kernel.submit(&mut submissions, Wait::Forever);
            kernel.reap(&mut completions);
            while let Some(complete) = completions.pop() {
                file_layer::up(&mut files, complete, &mut events, &mut submissions);
            }
        }
    }
    panic!("finite removal did not settle");
}

fn file_request(files: &FileIo, below: Below) -> (file::Request, Time) {
    match below {
        Below::File { request, deadline } => (request, deadline),
        Below::OpenDirectory { owner, root, path, no_follow, deadline } => (
            file::Request::OpenDirectory {
                owner,
                root: files.descriptor(root).expect("installed root"),
                name: path,
                no_follow,
            },
            deadline,
        ),
        Below::Remove { owner, parent, name, directory, deadline } => (
            file::Request::Remove { owner, root: files.descriptor(parent).expect("owned parent"), name, directory },
            deadline,
        ),
        other @ (Below::OpenRead { .. } | Below::CancelFile { .. } | Below::Spawn { .. } | Below::Process(_)) => {
            panic!("file-only scenario: {other:?}")
        }
    }
}

#[test]
fn real_removal_checks_current_bytes_and_preserves_refused_targets() {
    let scratch = std::env::temp_dir().join(format!("smith-real-remove-{}", std::process::id()));
    fs::create_dir(&scratch).expect("unique scratch");
    fs::create_dir(scratch.join("src")).expect("source directory");
    fs::create_dir(scratch.join("empty")).expect("empty directory");
    fs::create_dir(scratch.join(".GiT")).expect("protected directory");
    fs::write(scratch.join("src/a"), b"old").expect("fixture file");
    fs::write(scratch.join(".GiT/config"), b"protected").expect("git fixture");
    std::os::unix::fs::symlink("src", scratch.join("alias")).expect("parent link");
    std::os::unix::fs::symlink("src/a", scratch.join("link")).expect("file link");
    let loaded = perform(&scratch, tools::Op::Load { at: place(b"src/a"), max: 16 });
    let tools::Done::Loaded { version, .. } = loaded else {
        panic!("read before removal: {loaded:?}");
    };
    for (path, expect, expected) in [
        (b"src/a".as_slice(), tools::Expect::Absent, tools::Done::Conflict { now: Some(version) }),
        (b"src", tools::Expect::Absent, tools::Done::NotEmpty),
        (b"alias/a", tools::Expect::Is { version }, tools::Done::Linked),
        (b"link", tools::Expect::Is { version }, tools::Done::Linked),
        (b".GiT/config", tools::Expect::Absent, tools::Done::Failed { fault: tools::Fault::Denied }),
        (b"", tools::Expect::Absent, tools::Done::Failed { fault: tools::Fault::Denied }),
        (b"../outside", tools::Expect::Absent, tools::Done::Escapes),
    ] {
        assert_eq!(perform(&scratch, remove(path, expect)), expected);
        assert_eq!(fs::read(scratch.join("src/a")).expect("refusal preserves file"), b"old");
        assert_eq!(fs::read(scratch.join(".GiT/config")).expect("protected file"), b"protected");
        assert!(scratch.join("empty").is_dir());
    }
    fs::write(scratch.join("src/a"), b"new").expect("external change");
    assert!(matches!(perform(&scratch, remove(b"src/a", tools::Expect::Is { version })), tools::Done::Conflict { .. }));
    assert_eq!(fs::read(scratch.join("src/a")).expect("stale removal preserves file"), b"new");
    let current = tools::Version::new(skein_io::digest::digest(b"new").0);
    assert_eq!(perform(&scratch, remove(b"src/a", tools::Expect::Is { version: current })), tools::Done::Removed);
    assert!(!scratch.join("src/a").exists());
    assert_eq!(perform(&scratch, remove(b"empty", tools::Expect::Absent)), tools::Done::Removed);
    assert!(!scratch.join("empty").exists());
    fs::remove_dir_all(scratch).expect("scratch reclaimed");
}
