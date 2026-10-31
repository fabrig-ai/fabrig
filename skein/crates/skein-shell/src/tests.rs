//! The shared listing face's buffer boundary, startup identity, and the
//! blocking flags on a regular file after Open.

use std::fs;

use skein_io::kernel::{Complete, Done, Entry, Exit, Op, OpenHow, Submit};
use skein_lib::{Queue, Time, Token, Wall};
use skein_scratch::Scratch;

use crate::Kernel;
use crate::system::{open_root, status_flags};
fn close(fd: i32) {
    crate::system::close(skein_io::kernel::Fd::new(fd)).expect("close test descriptor");
}
use crate::{Config, Delegate, Host, Wait};

/// A policy-only host, using the loop's provided deadlines (shell.md, 12).
struct PolicyHost {
    policy: Option<Time>,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
}

impl PolicyHost {
    fn new(policy: Option<Time>) -> Self {
        Self { policy, completions: Queue::uncapped(), submissions: Queue::uncapped() }
    }
}

impl Host for PolicyHost {
    fn iterate(&mut self, _now: Time, _wall: Wall) {}

    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }

    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }

    fn work_pending(&self, _now: Time) -> bool {
        false
    }

    fn next_policy_deadline(&self) -> Option<Time> {
        self.policy
    }

    fn is_empty(&self) -> bool {
        self.completions.is_empty() && self.submissions.is_empty()
    }

    fn operations(&self) -> u32 {
        3
    }
}

/// A host with both layers' deadlines and observable hooks (shell.md, 12).
struct MechanismHost {
    inner: PolicyHost,
    mechanism: Option<Time>,
    iterated: Option<(Time, Wall)>,
    drained: bool,
}

impl MechanismHost {
    fn new(mechanism: Option<Time>, policy: Option<Time>) -> Self {
        Self { inner: PolicyHost::new(policy), mechanism, iterated: None, drained: false }
    }
}

impl Host for MechanismHost {
    fn state_digest(&self) -> Option<[u8; 32]> {
        Some([7; 32])
    }

    fn drain(&mut self) {
        assert!(!self.drained, "diagnostics drain once");
        self.drained = true;
    }

    fn iterate(&mut self, now: Time, wall: Wall) {
        self.iterated = Some((now, wall));
    }

    fn completions(&mut self) -> &mut Queue<Complete> {
        self.inner.completions()
    }

    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.inner.submissions()
    }

    fn work_pending(&self, now: Time) -> bool {
        self.iterated.is_some_and(|(iterated, _wall)| iterated == now)
    }

    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.mechanism
    }

    fn next_policy_deadline(&self) -> Option<Time> {
        self.inner.next_policy_deadline()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn exit(&self) -> Option<Exit> {
        Some(Exit::Code(23))
    }

    fn operations(&self) -> u32 {
        self.inner.operations()
    }
}

/// Selects either concrete host without forwarding its methods (shell.md, 12).
enum Process {
    Policy(PolicyHost),
    Mechanism(MechanismHost),
}

impl Delegate for Process {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Policy(host) => host,
            Self::Mechanism(host) => host,
        }
    }

    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Policy(host) => host,
            Self::Mechanism(host) => host,
        }
    }
}

#[test]
fn host_deadline_defaults_to_its_policy_deadline() {
    for policy in [None, Some(Time::from_nanos(20))] {
        let host = PolicyHost::new(policy);
        assert_eq!(host.next_mechanism_deadline(), None, "no mechanism deadline stated");
        assert_eq!(host.next_deadline(), policy, "the policy deadline wakes the loop");
    }
}

#[test]
fn host_deadline_is_the_earlier_of_mechanism_and_policy() {
    let early = Some(Time::from_nanos(10));
    let late = Some(Time::from_nanos(20));
    for (mechanism, policy, expected) in [
        (early, late, early),
        (late, early, early),
        (early, early, early),
        (early, None, early),
        (None, late, late),
        (None, None, None),
    ] {
        let host = MechanismHost::new(mechanism, policy);
        assert_eq!(host.next_deadline(), expected, "mechanism {mechanism:?}, policy {policy:?}");
    }
}

#[test]
fn delegate_forwards_to_the_selected_host() {
    let now = Time::from_nanos(10);
    let wall = Wall::from_nanos(30);
    let mechanism = Some(Time::from_nanos(20));
    let policy = Some(Time::from_nanos(40));
    let mut process = Process::Mechanism(MechanismHost::new(mechanism, policy));
    assert!(!process.work_pending(now), "no iteration yet");
    process.iterate(now, wall);
    assert!(process.work_pending(now), "the selected host iterated at now");
    assert!(!process.work_pending(Time::ZERO), "work_pending receives its time");
    process.drain();
    assert_eq!(process.state_digest(), Some([7; 32]), "the selected host's digest");
    assert_eq!(process.exit(), Some(Exit::Code(23)), "the selected host's failure status");
    assert_eq!(process.next_mechanism_deadline(), mechanism, "the selected mechanism deadline");
    assert_eq!(process.next_policy_deadline(), policy, "the selected policy deadline");
    assert_eq!(process.next_deadline(), mechanism, "the selected host's earliest deadline");
    assert_eq!(process.operations(), 3, "the selected host's ring size");
    assert!(process.is_empty(), "queues start empty");
    process.completions().push(Complete {
        op: Token::new(5),
        kind: Op::Close { fd: skein_io::kernel::Fd::new(8) },
        result: Ok(Done::Nothing),
    });
    process.submissions().push(Submit { op: Token::new(6), kind: Op::Close { fd: skein_io::kernel::Fd::new(9) } });
    assert!(!process.is_empty(), "the selected host holds the queued records");
    let Process::Mechanism(host) = &mut process else { panic!("mechanism host selected") };
    assert_eq!(host.iterated, Some((now, wall)), "both iteration times reach the host");
    assert!(host.drained, "the selected host drained");
    assert_eq!(host.inner.completions.pop().expect("forwarded completion").op, Token::new(5));
    assert_eq!(host.inner.submissions.pop().expect("forwarded submission").op, Token::new(6));

    let mut process = Process::Policy(PolicyHost::new(policy));
    process.iterate(now, wall);
    process.drain();
    assert_eq!(process.state_digest(), None, "the other host's default digest");
    assert_eq!(process.exit(), Some(Exit::Code(0)), "the other host's default exit");
    assert_eq!(process.next_mechanism_deadline(), None, "the other host has no mechanism deadline");
    assert_eq!(process.next_deadline(), policy, "the other host's policy deadline");
}

#[test]
fn startup_process_id_is_this_process() {
    assert_eq!(crate::process_id(), std::process::id());
}

#[test]
fn a_name_that_fits_in_no_names_is_too_long_and_stays_next() {
    let scratch = Scratch::new("shell");
    fs::write(scratch.path().join("a-name-of-twenty-bytes"), b"").expect("a file laid");
    let dir = open_root(scratch.path()).expect("a scratch directory opens");
    let mut listing = crate::system::Listings::new();
    let mut entries = [Entry::BLANK; 2];
    let mut small = [0_u8; 8];
    assert_eq!(listing.list(dir, &mut entries, &mut small), Err(libc::ENAMETOOLONG));
    let mut names = [0_u8; 255];
    assert_eq!(listing.list(dir, &mut entries, &mut names), Ok(1), "the name is still next");
    let first = entries.first().expect("an entry").name(&names);
    assert_eq!(first, Some(&b"a-name-of-twenty-bytes"[..]));
    assert_eq!(listing.list(dir, &mut entries, &mut names), Ok(0), "then the end");
    listing.forget(dir);
    close(dir.raw());
}

/// Every `Open` goes down without blocking, so that a FIFO opens to be
/// refused; a file it opened is blocking again, so that io_uring sends a
/// read it cannot do at once to its worker (kernel.md, 6.1).
#[test]
fn a_file_opened_blocks_again() {
    let scratch = Scratch::new("shell");
    fs::write(scratch.path().join("f"), b"f").expect("a file laid");
    let dir = open_root(scratch.path()).expect("a scratch directory opens");
    let mut kernel = Kernel::open(Config { operations: 4 }).expect("the kernel is usable here");
    let mut submissions = Queue::with_capacity(1);
    let path = Box::from(&b"f"[..]);
    submissions.push(Submit { op: Token::new(1), kind: Op::Open { root: dir, path, how: OpenHow::Read } });
    kernel.submit(&mut submissions, Wait::Forever);
    let mut completions = Queue::with_capacity(1);
    kernel.reap(&mut completions);
    let Some(Complete { result: Ok(Done::Fd(file)), .. }) = completions.pop() else { panic!("the file opened") };
    let flags = status_flags(file.raw()).expect("an open descriptor's flags");
    assert_eq!(flags & libc::O_NONBLOCK, 0, "the file blocks");
    close(file.raw());
    close(dir.raw());
}

#[test]
fn make_root_makes_and_opens_a_nested_absent_path() {
    let scratch = Scratch::new("make-root");
    let path = scratch.path().join("state/private");
    let fd = crate::make_root(&path, 0o700).expect("nested root made and opened");
    assert!(path.is_dir(), "the last directory exists");
    assert_eq!(open_root(&path).map(|opened| close(opened.raw())), Ok(()), "opens as a root");
    close(fd.raw());
}

#[test]
fn make_root_refuses_a_file_in_its_path() {
    let scratch = Scratch::new("blocked-root");
    let file = scratch.path().join("file");
    fs::write(&file, b"contents").expect("blocking file");
    for (label, path) in [("final file", file.clone()), ("parent file", file.join("private"))] {
        assert_eq!(crate::make_root(&path, 0o700), Err(libc::ENOTDIR), "{label}");
    }
    assert_eq!(fs::read(file).expect("blocking file remains"), b"contents", "file contents retained");
}

#[test]
fn make_root_preserves_an_existing_directorys_mode() {
    use std::os::unix::fs::PermissionsExt;
    let scratch = Scratch::new("existing-root");
    fs::set_permissions(scratch.path(), fs::Permissions::from_mode(0o711)).expect("existing mode");
    let fd = crate::make_root(scratch.path(), 0o700).expect("existing root opened");
    close(fd.raw());
    assert_eq!(
        fs::metadata(scratch.path()).expect("existing directory").permissions().mode() & 0o777,
        0o711,
        "existing mode retained"
    );
}
