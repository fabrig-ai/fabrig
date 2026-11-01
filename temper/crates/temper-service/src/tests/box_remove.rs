//! Kept box removals through serve's pipe.
//! temper/docs/plan/playground-reset-r4/notes/contracts.md, 3; temper/docs/design/protocol/setup.md, 7.1.
use super::{Ask, Link, Out, Reply, decoded, remove_line};
use crate::tests::fixtures;
use alloc::boxed::Box;
use skein_io::kernel::{Complete, Done, Fd, Kind, Op, Stat};
use skein_lib::{Queue, Time, Token, Wall};
use temper_engine_domain as root;
use temper_engine_domain_forge as forge;

const FORGE: u16 = 1;
const KEY: [u8; 16] = [7; 16];
const LINE_BYTES: u32 = 128;
const OUTPUTS: u32 = 3;
// Allow the root journal, output credit, and pipe completions to settle.
const TURNS: u16 = 128;
const FILE_FD: i32 = 9_000;

fn kept() -> forge::Kept {
    forge::Kept { key: KEY, act: forge::KeptAct::BoxRemove { forge: FORGE } }
}

fn piped() -> (crate::Service, fixtures::Kernel) {
    let mut previous = fixtures::ready();
    let mut store = previous.engine.take().unwrap().store;
    fixtures::commit(
        &mut store,
        1,
        Box::new([root::Write::Save(root::Record::Forge { row: Box::new(forge::Stored::Kept(kept())) })]),
    );
    store.above.clear();
    let mut service = fixtures::start_with_store(&[(1, 100)], Some(store));
    service.adopt_box(Fd::new(3), Fd::new(4), Fd::new(90), 0, LINE_BYTES).unwrap();
    let mut kernel = fixtures::Kernel::new();
    turns(&mut service, &mut kernel);
    assert!(service.engine.as_ref().unwrap().domain.ready());
    assert_eq!(kernel.box_written.as_slice(), b"remove 1\n");
    (service, kernel)
}

// Removal needs only the adopted data root, without group-file writes. The
// system fixture's kernel handles the pipe while this script adopts that root.
#[expect(clippy::wildcard_enum_match_arm, reason = "the fixture intercepts only the box data-root adoption")]
fn turns(service: &mut crate::Service, kernel: &mut fixtures::Kernel) {
    for _ in 0..TURNS {
        crate::iterate(service, Time::from_nanos(1), Wall::from_nanos(1));
        for _ in 0..service.submissions.len() {
            let submit = service.submissions.pop().unwrap();
            let result = match &submit.kind {
                Op::Open { root, .. } if *root == Fd::new(90) => Some(Done::Fd(Fd::new(FILE_FD))),
                Op::Stat { fd } if *fd == Fd::new(FILE_FD) => {
                    Some(Done::Stat(Stat { kind: Kind::Directory, size: 0, mode: 0o700, owner: 0, links: 1 }))
                }
                _ => None,
            };
            if let Some(done) = result {
                service.completions.push(Complete { op: submit.op, kind: submit.kind, result: Ok(done) });
            } else {
                service.submissions.push(submit);
            }
        }
        kernel.answer(service);
    }
}

fn answer(service: &mut crate::Service, kernel: &mut fixtures::Kernel, line: &[u8]) {
    for byte in line {
        kernel.box_answers.push(*byte);
    }
    turns(service, kernel);
}

fn stored(service: &mut crate::Service) -> bool {
    for row in fixtures::load(&mut service.engine.as_mut().unwrap().store, root::Range::Forge) {
        if row == (root::Record::Forge { row: Box::new(forge::Stored::Kept(kept())) }) {
            return true;
        }
    }
    false
}

fn send_again(service: &mut crate::Service, kernel: &mut fixtures::Kernel) {
    service.engine.as_mut().unwrap().events.push(root::Event::Forge(Box::new(forge::Event::SendKept)));
    turns(service, kernel);
}

#[test]
fn a_kept_box_remove_writes_remove_and_removed_erases() {
    let (mut service, mut kernel) = piped();
    assert!(stored(&mut service));
    // A different forge's answer cannot release the kept act.
    answer(&mut service, &mut kernel, b"removed 2\n");
    assert!(stored(&mut service));
    answer(&mut service, &mut kernel, b"removed 1\n");
    assert!(!stored(&mut service));
    send_again(&mut service, &mut kernel);
    assert_eq!(kernel.box_written.as_slice(), b"remove 1\n");
}

#[test]
fn a_failed_remove_keeps_the_act() {
    for why in [b"busy".as_slice(), b"stopping", b"files not removed"] {
        let (mut service, mut kernel) = piped();
        let engine = service.engine.as_mut().unwrap();
        engine.notes.clear();
        for part in [b"failed 1 ".as_slice(), why, b"\n"] {
            for byte in part {
                kernel.box_answers.push(*byte);
            }
        }
        turns(&mut service, &mut kernel);
        assert!(stored(&mut service));
        let mut named = false;
        for note in &service.engine.as_ref().unwrap().notes {
            if let crate::Note::BoxLink { line } = note
                && line.starts_with(b"box link: remove failed for forge 1: ")
                && line.ends_with(why)
            {
                named = true;
            }
        }
        assert!(named, "the failed removal names its reason");
        send_again(&mut service, &mut kernel);
        assert_eq!(kernel.box_written.as_slice(), b"remove 1\nremove 1\n");
        answer(&mut service, &mut kernel, b"removed 1\n");
        assert!(!stored(&mut service));
    }
}

#[test]
fn a_remove_waits_for_the_ask_in_flight() {
    for reply in [b"started 1\n".as_slice(), b"failed 1 busy\n"] {
        let mut link = Link::new(LINE_BYTES, true);
        link.limit_held(4);
        let mut out = Queue::with_capacity(OUTPUTS);
        link.ask(FORGE, Token::new(1), &mut out);
        out.clear();
        link.remove(FORGE, &mut out);
        link.ask(2, Token::new(2), &mut out);
        out.clear();
        link.remove(3, &mut out);
        assert!(out.is_empty());
        assert!(link.asked == Some(Ask::Start(FORGE)));
        link.read(reply, &mut out);
        assert_eq!(
            out.pop(),
            Some(if reply.starts_with(b"started") {
                Out::Started { forge: FORGE }
            } else {
                Out::Failed { forge: FORGE, why: b"busy".as_slice().into() }
            })
        );
        assert_eq!(out.pop(), Some(Out::Write(remove_line(FORGE))));
        link.read(b"removed 1\n", &mut out);
        assert_eq!(out.pop(), Some(Out::Removed { forge: FORGE, removed: true }));
        assert_eq!(out.pop(), Some(Out::Write(b"start 2\n".as_slice().into())));
        link.read(b"started 2\n", &mut out);
        assert_eq!(out.pop(), Some(Out::Started { forge: 2 }));
        assert_eq!(out.pop(), Some(Out::Write(remove_line(3))));
        assert!(out.is_empty());
    }
}

fn removed_event(engine: &mut crate::engine::Engine, forge: u16, removed: bool) {
    let Some(root::Event::Forge(event)) = engine.events.pop() else { panic!("forge event") };
    let forge::Event::BoxRemoved { forge: number, removed: answer } = *event else { panic!("box removal event") };
    assert_eq!((number, answer), (forge, removed));
}

#[test]
fn the_pipes_end_answers_not_removed() {
    let mut service = fixtures::ready();
    let engine = service.engine.as_mut().unwrap();
    engine.events.clear();
    engine.box_link = Link::new(LINE_BYTES, true);
    crate::forge::table::box_remove(engine, FORGE);
    assert_eq!(engine.box_link.write.take(), Some(remove_line(FORGE)));
    let mut out = Queue::with_capacity(OUTPUTS);
    engine.box_link.closed(&mut out);
    super::drain(engine, &mut out);
    removed_event(engine, FORGE, false);
    let Some(root::Event::Forge(event)) = engine.events.pop() else { panic!("box closed event") };
    let forge::Event::BoxClosed = *event else { panic!("box closed event") };
    engine.box_link.closed(&mut out);
    assert!(out.is_empty());
    crate::forge::table::box_remove(engine, 2);
    removed_event(engine, 2, false);
}

#[test]
fn without_the_box_a_remove_is_removed_at_once() {
    let mut service = fixtures::ready();
    let engine = service.engine.as_mut().unwrap();
    engine.events.clear();
    crate::forge::table::box_remove(engine, FORGE);
    removed_event(engine, FORGE, true);
    assert!(engine.box_link.write.is_none());
}

#[test]
fn removal_lines_use_canonical_decimal() {
    for (forge, line) in [(0, b"remove 0\n".as_slice()), (u16::MAX, b"remove 65535\n")] {
        assert_eq!(remove_line(forge).as_ref(), line);
    }
    for (forge, line) in [(0, b"removed 0\n".as_slice()), (u16::MAX, b"removed 65535\n")] {
        let Some(Reply::Removed(number)) = decoded(line) else { panic!("removed reply") };
        assert_eq!(number, forge);
    }
    for line in [
        b"removed 01\n".as_slice(),
        b"removed +1\n",
        b"removed 65536\n",
        b"removed 1 \n",
        b"removed 1\r\n",
        b"removed 1",
        b"removed \n",
        b"removed -1\n",
        b"removed  1\n",
        b"failed 01 busy\n",
        b"failed +1 busy\n",
        b"failed 65536 busy\n",
        b"failed 1 \n",
        b"started 01\n",
    ] {
        assert!(decoded(line).is_none(), "noncanonical line {line:?}");
    }
}

#[test]
fn held_removals_are_bounded_by_forge_forges() {
    let mut link = Link::new(LINE_BYTES, true);
    link.limit_held(2);
    let mut out = Queue::with_capacity(OUTPUTS);
    link.remove(1, &mut out);
    out.clear();
    link.remove(2, &mut out);
    link.remove(3, &mut out);
    assert!(out.is_empty());
    link.remove(4, &mut out);
    let Some(Out::Named(_)) = out.pop() else { panic!("named diagnostic") };
    assert_eq!(out.pop(), Some(Out::Removed { forge: 4, removed: false }));
    // Sending the same kept act twice does not take another queue slot.
    link.remove(2, &mut out);
    assert!(out.is_empty());
    link.read(b"failed 1 busy\n", &mut out);
    let Some(Out::Named(_)) = out.pop() else { panic!("named diagnostic") };
    assert_eq!(out.pop(), Some(Out::Removed { forge: 1, removed: false }));
    assert_eq!(out.pop(), Some(Out::Write(remove_line(2))));
    link.read(b"removed 2\n", &mut out);
    assert_eq!(out.pop(), Some(Out::Removed { forge: 2, removed: true }));
    assert_eq!(out.pop(), Some(Out::Write(remove_line(3))));
}

#[test]
fn a_remove_waits_for_the_group_file_and_its_start_answer() {
    let mut link = Link::new(LINE_BYTES, true);
    link.files = Some(crate::route::group_file::Files::new(Fd::new(90), 0, skein_lib::Duration::from_secs(1), 7));
    let mut out = Queue::with_capacity(OUTPUTS);
    link.ask(FORGE, Token::new(1), &mut out);
    out.clear();
    link.remove(FORGE, &mut out);
    link.read(b"removed 1\n", &mut out);
    let Some(Out::Named(_)) = out.pop() else { panic!("named diagnostic") };
    link.file_written(true, &mut out);
    assert_eq!(out.pop(), Some(Out::Write(b"start 1\n".as_slice().into())));
    link.read(b"started 1\n", &mut out);
    assert_eq!(out.pop(), Some(Out::Started { forge: FORGE }));
    assert_eq!(out.pop(), Some(Out::Write(remove_line(FORGE))));
}

#[test]
fn a_pipe_end_keeps_the_act_for_the_next_start() {
    let (mut service, mut kernel) = piped();
    let mut ended = false;
    for _ in 0..service.submissions.len() {
        let submit = service.submissions.pop().unwrap();
        let pipe_read = if let Op::PipeRead { fd, .. } = &submit.kind { *fd == Fd::new(4) } else { false };
        if pipe_read {
            service.completions.push(Complete { op: submit.op, kind: submit.kind, result: Ok(Done::Count(0)) });
            ended = true;
        } else {
            service.submissions.push(submit);
        }
    }
    assert!(ended, "the pipe read is in flight");
    turns(&mut service, &mut kernel);
    assert!(stored(&mut service));
    let store = service.engine.take().unwrap().store;
    let mut restarted = fixtures::start_with_store(&[(1, 100)], Some(store));
    restarted.adopt_box(Fd::new(3), Fd::new(4), Fd::new(90), 0, LINE_BYTES).unwrap();
    let mut kernel = fixtures::Kernel::new();
    turns(&mut restarted, &mut kernel);
    assert_eq!(kernel.box_written.as_slice(), b"remove 1\n");
    answer(&mut restarted, &mut kernel, b"removed 1\n");
    assert!(!stored(&mut restarted));
}

#[test]
fn a_failed_group_file_write_releases_the_held_remove() {
    let mut link = Link::new(LINE_BYTES, true);
    link.files = Some(crate::route::group_file::Files::new(Fd::new(90), 0, skein_lib::Duration::from_secs(1), 7));
    let mut out = Queue::with_capacity(OUTPUTS);
    link.ask(FORGE, Token::new(1), &mut out);
    out.clear();
    link.remove(FORGE, &mut out);
    link.file_written(false, &mut out);
    assert_eq!(out.pop(), Some(Out::Failed { forge: FORGE, why: b"group file not written".as_slice().into() }));
    assert_eq!(out.pop(), Some(Out::Write(remove_line(FORGE))));
}
