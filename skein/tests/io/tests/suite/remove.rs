//! Single-name removal through the file driver (runtime/io/files.md).

use skein_fake_machine::{How, Item, Machine, Opened, Refusal, kernel_error};
use skein_io::file::{Event, Request};
use skein_io::file_layer::{FileIo, Limits, step};
use skein_io::kernel::{Complete, Done, Error, Fd, Op};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};

const OWNER: Token = Token::new(7);
const ROOT: Fd = Fd::new(41);

fn remove(machine: &mut Machine, root: Opened, name: &[u8], directory: bool) -> Event {
    let limits = Limits { files: 1, read: 16, entries: 4, whole: 16, timeout: Duration::from_secs(1) };
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut io = FileIo::with_limits(&env.limits);
    let mut events = Queue::with_capacity(2);
    let mut subs = Queue::with_capacity(2);
    step::down(
        &mut io,
        &env,
        Request::Remove { owner: OWNER, root: ROOT, name: Box::from(name), directory },
        &mut events,
        &mut subs,
    );
    assert!(events.is_empty());
    let submit = subs.pop().expect("one removal");
    assert!(subs.is_empty());
    let Op::Remove { dir, name: leaf, directory: remove_directory } = &submit.kind else {
        panic!("a single removal without traversal");
    };
    assert_eq!(*dir, ROOT);
    assert_eq!(leaf.as_ref(), name);
    assert_eq!(*remove_directory, directory);
    let result = machine.remove(root, leaf, *remove_directory).map(|()| Done::Nothing).map_err(kernel_error);
    step::up(&mut io, &env, Complete { op: submit.op, kind: submit.kind, result }, &mut events, &mut subs);
    let event = events.pop().expect("one terminal");
    assert!(events.is_empty() && subs.is_empty() && io.takes());
    assert_eq!(io.next_deadline(), None);
    event
}

#[test]
fn an_empty_directory_is_removed_by_name() {
    let mut machine = Machine::new();
    let root = machine.lay(&[Item::directory(b"empty")]);
    assert_eq!(remove(&mut machine, root, b"empty", true), Event::Removed { owner: OWNER });
    assert_eq!(machine.open(root, b"empty", How::Directory), Err(Refusal::NotFound));
    machine.close(root);
    assert_eq!(machine.open_handles(), 0);
}

#[test]
fn a_nonempty_directory_is_refused_and_its_files_are_kept() {
    let mut machine = Machine::new();
    let root =
        machine.lay(&[Item::directory(b"kept"), Item::file(b"kept/one", b"first"), Item::file(b"kept/two", b"second")]);
    assert_eq!(
        remove(&mut machine, root, b"kept", true),
        Event::Failed { owner: OWNER, error: Error::NotEmpty, committed: false, residue: None }
    );
    let directory = machine.open(root, b"kept", How::Directory).expect("directory kept");
    for (name, bytes) in [(b"one", b"first".as_slice()), (b"two", b"second".as_slice())] {
        let file = machine.open(directory, name, How::Read).expect("file kept");
        assert_eq!(machine.read(file, 0, 16).expect("read kept file"), bytes);
        machine.close(file);
    }
    machine.close(directory);
    machine.close(root);
    assert_eq!(machine.open_handles(), 0);
}

#[test]
fn a_file_is_removed_with_directory_false() {
    let mut machine = Machine::new();
    let root = machine.lay(&[Item::file(b"file", b"bytes")]);
    assert_eq!(remove(&mut machine, root, b"file", false), Event::Removed { owner: OWNER });
    assert_eq!(machine.open(root, b"file", How::Read), Err(Refusal::NotFound));
    machine.close(root);
    assert_eq!(machine.open_handles(), 0);
}
