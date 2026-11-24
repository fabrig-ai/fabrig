use skein_io::kernel::{Op, Submit};
use skein_lib::{Queue, Token};
use skein_sim::{Answer, Config, Handle, Sim};

use super::machine;
use crate::{How, Item, Machine, Opened, Refusal};

const FILE_MODE: u32 = 0o644;
const CONTENTS: &[u8] = b"data";

fn file_tree() -> (Machine, Opened) {
    machine(&[Item::file(b"src", CONTENTS).mode(FILE_MODE)])
}

#[test]
fn opens_by_name_and_hard_link_share_a_node() {
    let (mut machine, root) = machine(&[Item::file(b"src", CONTENTS), Item::hard_link(b"alias", b"src")]);
    let file = machine.open(root, b"src", How::Read).unwrap();
    let alias = machine.open(root, b"alias", How::Read).unwrap();
    assert_eq!(machine.node(file), machine.node(alias));
}

#[test]
fn a_renamed_file_keeps_its_node_at_the_new_name() {
    let (mut machine, root) = file_tree();
    let file = machine.open(root, b"src", How::Read).unwrap();
    let before = machine.node(file);
    machine.rename(root, b"src", root, b"moved", false).unwrap();
    let moved = machine.open(root, b"moved", How::Read).unwrap();
    assert_eq!(machine.node(moved), before);
}

#[test]
fn recreating_the_same_file_gives_it_a_new_node() {
    let (mut machine, root) = file_tree();
    let file = machine.open(root, b"src", How::Read).unwrap();
    let before = machine.node(file);
    machine.close(file);
    machine.remove(root, b"src", false).unwrap();
    let replaced = machine.open(root, b"src", How::Create { mode: FILE_MODE }).unwrap();
    machine.write(replaced, 0, CONTENTS).unwrap();
    assert_ne!(machine.node(replaced), before);
}

#[test]
fn a_directory_has_a_different_node_from_its_file() {
    let (mut machine, root) = file_tree();
    let file = machine.open(root, b"src", How::Read).unwrap();
    assert_ne!(machine.node(root), machine.node(file));
}

#[test]
fn a_file_that_survives_a_crash_keeps_its_node() {
    const CRASH_SEED: u64 = 0; // No pending work; the laid file is durable.
    const ROOT_INDEX: u32 = 0; // The first and only laid root.
    let (mut machine, root) = file_tree();
    let file = machine.open(root, b"src", How::Read).unwrap();
    let before = machine.node(file);
    machine.crash(CRASH_SEED);
    let root = machine.reopen_root(ROOT_INDEX);
    let file = machine.open(root, b"src", How::Read).unwrap();
    assert_eq!(machine.node(file), before);
}

#[test]
fn a_laid_link_gives_back_its_relative_target_bytes() {
    let (mut machine, root) = machine(&[Item::link(b"alias", b"src")]);
    let link = machine.open(root, b"alias", How::PathNoFollow).unwrap();
    assert_eq!(machine.link_target(link).as_deref(), Some(b"src".as_slice()));
}

#[test]
fn an_absolute_link_gives_back_its_target_without_escaping() {
    let (mut machine, root) = machine(&[Item::link(b"alias", b"/outside/x")]);
    let link = machine.open(root, b"alias", How::PathNoFollow).unwrap();
    let followed = machine.open(root, b"alias", How::Read);
    assert_eq!(
        (machine.link_target(link).as_deref(), followed),
        (Some(b"/outside/x".as_slice()), Err(Refusal::Escape))
    );
}

#[test]
fn a_new_hard_link_to_a_symlink_preserves_its_target() {
    let (mut machine, root) = machine(&[Item::link(b"alias", b"src")]);
    machine.hard_link(root, b"alias", root, b"second").unwrap();
    let link = machine.open(root, b"second", How::PathNoFollow).unwrap();
    assert_eq!(machine.link_target(link).as_deref(), Some(b"src".as_slice()));
}

#[test]
fn files_and_directories_do_not_have_link_targets() {
    let (mut machine, root) = file_tree();
    for (path, how) in [(b"src".as_slice(), How::Read), (b".".as_slice(), How::Directory)] {
        let opened = machine.open(root, path, how).unwrap();
        assert_eq!(machine.link_target(opened), None, "{path:?}");
    }
}

fn face_stat(machine: &mut Machine, root: Opened) -> Answer {
    const SIM_SEED: u64 = 0; // Calm configuration, one stat request.
    const ONE_STAT: u32 = 1;
    let mut sim = Sim::new(SIM_SEED, Config::calm());
    let pid = sim.spawn_process();
    let fd = sim.root(pid, Handle::new(root.raw()));
    let mut submissions = Queue::with_capacity(ONE_STAT);
    submissions.push(Submit { op: Token::new(1), kind: Op::Stat { fd } });
    sim.submit(pid, &mut submissions);
    let mut calls = Queue::with_capacity(ONE_STAT);
    sim.calls(&mut calls);
    let mut answers = Queue::with_capacity(ONE_STAT);
    crate::step(machine, calls.pop().expect("one stat call"), &mut answers);
    answers.pop().expect("one stat answer")
}

#[test]
fn reading_back_leaves_handles_and_later_answers_unchanged() {
    const PAGE: u32 = 1; // Enough for one of the two laid names.
    const NAME_ROOM: u32 = 255; // A page must fit at least one NAME_MAX name.
    let (mut machine, root) = machine(&[Item::file(b"src", CONTENTS), Item::link(b"alias", b"src")]);
    let file = machine.open(root, b"src", How::Read).unwrap();
    let link = machine.open(root, b"alias", How::PathNoFollow).unwrap();
    let control = machine.open(root, b".", How::Directory).unwrap();
    for dir in [root, control] {
        machine.list(dir, PAGE, NAME_ROOM).unwrap();
    }
    let rest = machine.list(control, PAGE, NAME_ROOM).unwrap();
    let handles = machine.open_handles();
    let facts = machine.stat(link);
    let length = u32::try_from(CONTENTS.len()).unwrap();
    let bytes = machine.read(file, 0, length).unwrap();
    let face = face_stat(&mut machine, root);
    let state = machine.state_digest();

    let _node = machine.node(root);
    let _target = machine.link_target(link);

    assert_eq!(machine.open_handles(), handles);
    assert_eq!(machine.state_digest(), state);
    assert_eq!(machine.stat(link), facts);
    assert_eq!(machine.read(file, 0, length).unwrap(), bytes);
    assert_eq!(machine.list(root, PAGE, NAME_ROOM).unwrap(), rest);
    assert_eq!(face_stat(&mut machine, root), face);
}
