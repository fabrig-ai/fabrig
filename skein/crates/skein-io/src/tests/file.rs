//! Root admission and file metadata through the file driver (io.md, section 5).

#![expect(clippy::wildcard_enum_match_arm, reason = "unexpected test events are reported with their values")]

use alloc::boxed::Box;

use skein_lib::{Duration, Env, Queue, Time, Token, Wall};

use crate::file::{Event, Request};
use crate::file_layer::{self, FileIo, step};
use crate::kernel::{Complete, Done, Fd, Kind, Op, OpenHow, Stat, Submit};

const OWNER: Token = Token::new(7);
const ROOT: Fd = Fd::new(41);
const CHILD: Fd = Fd::new(42);

fn io() -> FileIo {
    FileIo::with_limits(&file_env().limits)
}

fn socket_and_file_operations() -> (Token, FileIo, Submit) {
    let mut rig = super::Rig::new(super::limits());
    let first = rig.down(crate::Request::Usage { owner: OWNER }).take(super::Kind::Usage);
    rig.complete(first, Ok(Done::Usage(crate::kernel::Usage::ZERO)));
    rig.next();
    let reused = rig.down(crate::Request::Usage { owner: OWNER }).take(super::Kind::Usage);

    let mut io = io();
    let env = file_env();
    let mut events = Queue::uncapped();
    let mut subs = Queue::uncapped();
    step::down(
        &mut io,
        &env,
        Request::OpenRead { owner: OWNER, root: ROOT, name: Box::from(&b"file"[..]) },
        &mut events,
        &mut subs,
    );
    (reused.op, io, subs.pop().expect("first file submission"))
}

#[test]
fn a_failure_constructor_never_commits_or_leaves_residue() {
    let Event::Failed { owner, error, committed, residue } = Event::failed(OWNER, crate::kernel::Error::Cancelled)
    else {
        panic!("a failed event")
    };
    assert_eq!(owner, OWNER);
    assert_eq!(error, crate::kernel::Error::Cancelled);
    assert!(!committed);
    assert_eq!(residue, None);
}

#[test]
fn an_io_and_a_file_operation_are_never_the_same_token() {
    let (socket, _, file) = socket_and_file_operations();
    assert_eq!(socket, Token::new(1), "slot zero, generation one after reclaim");
    assert_ne!(socket, file.op);
}

#[test]
fn owns_tells_a_file_operation_from_an_io_one() {
    let (socket, mut io, file) = socket_and_file_operations();
    let mut subs = Queue::uncapped();
    file_layer::cancel(&mut io, OWNER, &mut subs);
    let cancel = subs.pop().expect("file cancellation");
    assert_eq!(cancel.kind, Op::Cancel { target: file.op });
    for (op, expected) in [(socket, false), (file.op, true), (cancel.op, true)] {
        assert_eq!(file_layer::owns(op), expected, "{op:?}");
    }
}

#[test]
fn flight_limits_keep_io_tokens_below_the_file_lane() {
    // io.md, 5: 2^27 flight slots, sized as eight per socket plus four
    // independent slots. The largest representable table is four below it.
    const MOST_SOCKETS: u32 = (1 << 24) - 1;
    const MOST_FLIGHTS: u32 = (1 << 27) - 4;
    for (sockets, expected) in [(MOST_SOCKETS, Some(MOST_FLIGHTS)), (MOST_SOCKETS + 1, None)] {
        let limits = crate::Limits { sockets, ..super::limits() };
        assert_eq!(crate::limits::flights(&limits), expected, "{sockets} sockets");
        assert_eq!(crate::valid(&limits), expected.is_some(), "{sockets} sockets");
    }
}

pub(super) fn complete(
    io: &mut FileIo,
    env: &Env<file_layer::Limits>,
    submit: Submit,
    result: Result<Done, crate::kernel::Error>,
    events: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) {
    step::up(io, env, Complete { op: submit.op, kind: submit.kind, result }, events, subs);
}

fn read(
    io: &mut FileIo,
    env: &Env<file_layer::Limits>,
    submit: Submit,
    bytes: &[u8],
    events: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) {
    let Op::Read { fd, mut buf, at } = submit.kind else { panic!("a read") };
    for (slot, byte) in buf.get_mut(..bytes.len()).expect("read fits the buffer").iter_mut().zip(bytes) {
        *slot = *byte;
    }
    let kind = Op::Read { fd, buf, at };
    step::up(
        io,
        env,
        Complete {
            op: submit.op,
            kind,
            result: Ok(Done::Count(u32::try_from(bytes.len()).expect("short test input"))),
        },
        events,
        subs,
    );
}

pub(super) fn listed(
    io: &mut FileIo,
    env: &Env<file_layer::Limits>,
    submit: Submit,
    names_to_list: &[&[u8]],
    events: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) {
    let Op::List { fd, mut entries, mut names } = submit.kind else { panic!("a list") };
    let mut offset = 0_usize;
    for (index, name) in names_to_list.iter().enumerate() {
        let end = offset.checked_add(name.len()).expect("test names fit");
        for (position, byte) in name.iter().enumerate() {
            let at = offset.checked_add(position).expect("test name position fits");
            names[at] = *byte;
        }
        entries[index] = crate::kernel::Entry {
            kind: Kind::File,
            start: u32::try_from(offset).expect("small test offset"),
            len: u32::try_from(name.len()).expect("small test name"),
        };
        offset = end;
    }
    step::up(
        io,
        env,
        Complete {
            op: submit.op,
            kind: Op::List { fd, entries, names },
            result: Ok(Done::Count(u32::try_from(names_to_list.len()).expect("small test batch"))),
        },
        events,
        subs,
    );
}

#[test]
fn adopted_root_and_open_directory_keep_tokens_inside_io() {
    let mut io = io();
    let env = file_env();
    let root = io.adopt_root(ROOT).expect("room for a root");
    assert_eq!(io.descriptor(root), Some(ROOT));

    let mut events = Queue::with_capacity(2);
    let mut subs = Queue::with_capacity(2);
    step::down(
        &mut io,
        &env,
        Request::OpenDirectory { owner: OWNER, root: ROOT, name: Box::from(&b"child"[..]), no_follow: true },
        &mut events,
        &mut subs,
    );
    let submit = subs.pop().expect("one open");
    let Op::Open { how, .. } = &submit.kind else { panic!("an open") };
    assert_eq!(*how, OpenHow::DirectoryNoFollow);
    step::up(
        &mut io,
        &env,
        Complete { op: submit.op, kind: submit.kind, result: Ok(Done::Fd(CHILD)) },
        &mut events,
        &mut subs,
    );
    let child = match events.pop().expect("one terminal") {
        Event::Opened { owner: OWNER, file, len: 0 } => file,
        other => panic!("unexpected event: {other:?}"),
    };
    assert_eq!(io.descriptor(child), Some(CHILD));

    step::down(&mut io, &env, Request::Stat { owner: OWNER, file: child }, &mut events, &mut subs);
    let submit = subs.pop().expect("one stat");
    let Op::Stat { fd } = &submit.kind else { panic!("a stat") };
    assert_eq!(*fd, CHILD);
    let stat = Stat { kind: Kind::Directory, size: 0, mode: 0o755, owner: 17, links: 2 };
    step::up(
        &mut io,
        &env,
        Complete { op: submit.op, kind: submit.kind, result: Ok(Done::Stat(stat)) },
        &mut events,
        &mut subs,
    );
    assert_eq!(events.pop(), Some(Event::Stated { owner: OWNER, stat }));
}

#[test]
fn read_and_create_no_follow_use_the_safe_kernel_modes() {
    let mut io = io();
    let env = file_env();
    let mut events = Queue::with_capacity(2);
    let mut subs = Queue::with_capacity(2);
    step::down(
        &mut io,
        &env,
        Request::OpenReadNoFollow { owner: OWNER, root: ROOT, name: Box::from(&b"file"[..]) },
        &mut events,
        &mut subs,
    );
    let submit = subs.pop().expect("one open");
    let Op::Open { how, .. } = &submit.kind else { panic!("an open") };
    assert_eq!(*how, OpenHow::ReadNoFollow);
    step::up(
        &mut io,
        &env,
        Complete { op: submit.op, kind: submit.kind, result: Err(crate::kernel::Error::NotFound) },
        &mut events,
        &mut subs,
    );
    assert_eq!(events.pop(), Some(Event::failed(OWNER, crate::kernel::Error::NotFound)));

    step::down(
        &mut io,
        &env,
        Request::CreateNoFollow { owner: OWNER, root: ROOT, name: Box::from(&b"file"[..]), mode: 0o600 },
        &mut events,
        &mut subs,
    );
    let submit: Submit = subs.pop().expect("one create");
    let Op::Open { how, .. } = submit.kind else { panic!("an open") };
    assert_eq!(how, OpenHow::CreateNoFollow { mode: Some(0o600) });
}

#[test]
fn a_whole_load_continues_short_reads_and_closes_before_its_terminal() {
    let env = Env { limits: file_layer::Limits { read: 2, whole: 8, ..file_env().limits }, ..file_env() };
    let mut io = FileIo::with_limits(&env.limits);
    let root = io.adopt_root(ROOT).expect("room for the root");
    let mut events = Queue::with_capacity(2);
    let mut subs = Queue::with_capacity(2);
    step::down(
        &mut io,
        &env,
        Request::Load { owner: OWNER, root, path: Box::from(&b"f"[..]), max: 4, no_follow: true },
        &mut events,
        &mut subs,
    );
    let open = subs.pop().expect("open");
    let Op::Open { how, .. } = &open.kind else { panic!("an open") };
    assert_eq!(*how, OpenHow::ReadNoFollow);
    complete(&mut io, &env, open, Ok(Done::Fd(CHILD)), &mut events, &mut subs);
    let stat = subs.pop().expect("stat");
    complete(
        &mut io,
        &env,
        stat,
        Ok(Done::Stat(Stat { kind: Kind::File, size: 3, mode: 0o644, owner: 1000, links: 1 })),
        &mut events,
        &mut subs,
    );
    read(&mut io, &env, subs.pop().expect("first read"), b"ab", &mut events, &mut subs);
    read(&mut io, &env, subs.pop().expect("second read"), b"c", &mut events, &mut subs);
    read(&mut io, &env, subs.pop().expect("end read"), b"", &mut events, &mut subs);
    assert!(events.is_empty(), "the close must finish first");
    let close = subs.pop().expect("close");
    complete(&mut io, &env, close, Ok(Done::Nothing), &mut events, &mut subs);
    assert_eq!(events.pop(), Some(Event::Loaded { owner: OWNER, bytes: Box::from(&b"abc"[..]) }));
    assert!(io.takes());
    assert_eq!(io.open_files(), 1, "only the root stays open");
}

#[test]
fn a_whole_load_refuses_a_file_over_its_bound_after_closing() {
    let env = Env { limits: file_layer::Limits { read: 2, whole: 8, ..file_env().limits }, ..file_env() };
    let mut io = FileIo::with_limits(&env.limits);
    let root = io.adopt_root(ROOT).expect("room for the root");
    let mut events = Queue::with_capacity(2);
    let mut subs = Queue::with_capacity(2);
    step::down(
        &mut io,
        &env,
        Request::Load { owner: OWNER, root, path: Box::from(&b"f"[..]), max: 3, no_follow: false },
        &mut events,
        &mut subs,
    );
    complete(&mut io, &env, subs.pop().expect("open"), Ok(Done::Fd(CHILD)), &mut events, &mut subs);
    complete(
        &mut io,
        &env,
        subs.pop().expect("stat"),
        Ok(Done::Stat(Stat { kind: Kind::File, size: 4, mode: 0o644, owner: 1000, links: 1 })),
        &mut events,
        &mut subs,
    );
    assert!(events.is_empty());
    complete(&mut io, &env, subs.pop().expect("close"), Ok(Done::Nothing), &mut events, &mut subs);
    assert_eq!(events.pop(), Some(Event::TooLarge { owner: OWNER, size: 4 }));
}

#[test]
fn a_whole_scan_returns_entries_or_a_bound_refusal_after_close() {
    for max in [0, 1] {
        let mut io = io();
        let env = file_env();
        let root = io.adopt_root(ROOT).expect("room for the root");
        let mut events = Queue::with_capacity(2);
        let mut subs = Queue::with_capacity(2);
        step::down(
            &mut io,
            &env,
            Request::Scan { owner: OWNER, root, path: Box::from(&b"dir"[..]), max, max_bytes: 100, no_follow: true },
            &mut events,
            &mut subs,
        );
        let open = subs.pop().expect("open");
        let Op::Open { how, .. } = &open.kind else { panic!("an open") };
        assert_eq!(*how, OpenHow::DirectoryNoFollow);
        complete(&mut io, &env, open, Ok(Done::Fd(CHILD)), &mut events, &mut subs);
        let list = subs.pop().expect("list");
        let Op::List { fd, mut entries, mut names } = list.kind else { panic!("a list") };
        entries[0] = crate::kernel::Entry { kind: Kind::File, start: 0, len: 1 };
        names[0] = b'f';
        step::up(
            &mut io,
            &env,
            Complete { op: list.op, kind: Op::List { fd, entries, names }, result: Ok(Done::Count(1)) },
            &mut events,
            &mut subs,
        );
        complete(&mut io, &env, subs.pop().expect("end listing"), Ok(Done::Count(0)), &mut events, &mut subs);
        assert!(events.is_empty());
        complete(&mut io, &env, subs.pop().expect("close"), Ok(Done::Nothing), &mut events, &mut subs);
        let entries = if max == 0 {
            Box::from([])
        } else {
            Box::from([crate::file::Entry { name: Box::from(&b"f"[..]), kind: Kind::File }])
        };
        assert_eq!(events.pop(), Some(Event::Scanned { owner: OWNER, entries, more: u64::from(max == 0) }));
    }
}

#[test]
fn a_scan_keeps_the_name_order_prefix_across_batches_and_counts_omissions() {
    let mut io = io();
    let env = file_env();
    let root = io.adopt_root(ROOT).expect("room for root");
    let mut events = Queue::with_capacity(2);
    let mut subs = Queue::with_capacity(2);
    let entry_bytes = u64::try_from(size_of::<crate::file::Entry>()).expect("entry cell fits u64");
    step::down(
        &mut io,
        &env,
        Request::Scan {
            owner: OWNER,
            root,
            path: Box::from(&b"dir"[..]),
            max: 3,
            max_bytes: entry_bytes.checked_mul(2).expect("two cells fit").checked_add(2).expect("two names fit"),
            no_follow: true,
        },
        &mut events,
        &mut subs,
    );
    complete(&mut io, &env, subs.pop().expect("open"), Ok(Done::Fd(CHILD)), &mut events, &mut subs);
    listed(&mut io, &env, subs.pop().expect("first batch"), &[b"z", b"c"], &mut events, &mut subs);
    listed(&mut io, &env, subs.pop().expect("second batch"), &[b"b", b"a"], &mut events, &mut subs);
    listed(&mut io, &env, subs.pop().expect("end"), &[], &mut events, &mut subs);
    assert!(events.is_empty());
    complete(&mut io, &env, subs.pop().expect("close"), Ok(Done::Nothing), &mut events, &mut subs);
    assert_eq!(
        events.pop(),
        Some(Event::Scanned {
            owner: OWNER,
            entries: Box::from([
                crate::file::Entry { name: Box::from(&b"a"[..]), kind: Kind::File },
                crate::file::Entry { name: Box::from(&b"b"[..]), kind: Kind::File },
            ]),
            more: 2,
        })
    );
}

#[test]
fn an_unfit_first_scan_entry_makes_the_prefix_empty() {
    let mut io = io();
    let env = file_env();
    let root = io.adopt_root(ROOT).expect("room for root");
    let mut events = Queue::with_capacity(2);
    let mut subs = Queue::with_capacity(2);
    let entry_bytes = u64::try_from(size_of::<crate::file::Entry>()).expect("entry cell fits u64");
    step::down(
        &mut io,
        &env,
        Request::Scan {
            owner: OWNER,
            root,
            path: Box::from(&b"dir"[..]),
            max: 2,
            max_bytes: entry_bytes + 1,
            no_follow: true,
        },
        &mut events,
        &mut subs,
    );
    complete(&mut io, &env, subs.pop().expect("open"), Ok(Done::Fd(CHILD)), &mut events, &mut subs);
    listed(&mut io, &env, subs.pop().expect("batch"), &[b"b", b"aa"], &mut events, &mut subs);
    listed(&mut io, &env, subs.pop().expect("end"), &[], &mut events, &mut subs);
    complete(&mut io, &env, subs.pop().expect("close"), Ok(Done::Nothing), &mut events, &mut subs);
    assert_eq!(events.pop(), Some(Event::Scanned { owner: OWNER, entries: Box::from([]), more: 2 }));
}

#[test]
fn a_stat_answered_after_its_deadline_keeps_its_own_answer() {
    let stat = Stat { kind: Kind::File, size: 10, mode: 0o644, owner: 1000, links: 1 };
    for (result, expected) in [
        (Ok(Done::Stat(stat)), Event::Stated { owner: OWNER, stat }),
        (Err(crate::kernel::Error::Permission), Event::failed(OWNER, crate::kernel::Error::Permission)),
    ] {
        let (mut io, file, _) = write_opened();
        let env = file_env();
        let mut events = Queue::with_capacity(2);
        let mut subs = Queue::with_capacity(2);
        step::down(&mut io, &env, Request::Stat { owner: OWNER, file }, &mut events, &mut subs);
        let submit = subs.pop().expect("stat in flight");
        assert_eq!(submit.kind, Op::Stat { fd: CHILD });
        step::expire(&mut io, &Env { now: Time::ZERO.saturating_add(Duration::from_secs(2)), ..env }, &mut subs);
        assert!(subs.is_empty(), "stat is not cancellable");
        assert!(events.is_empty() && !io.takes(), "the deadline waits for the stat's answer");
        complete(&mut io, &env, submit, result, &mut events, &mut subs);
        assert_eq!(events.pop(), Some(expected));
        assert!(events.is_empty(), "exactly one terminal");
        assert!(subs.is_empty() && io.takes());
        assert_eq!(io.descriptor(file), Some(CHILD));
        assert_eq!(io.next_deadline(), None);
    }
}

#[test]
fn a_deadline_waits_for_uncancellable_stat_then_closes_the_file() {
    let env = Env { limits: file_layer::Limits { read: 2, whole: 8, ..file_env().limits }, ..file_env() };
    let mut io = FileIo::with_limits(&env.limits);
    let root = io.adopt_root(ROOT).expect("room for the root");
    let mut events = Queue::with_capacity(2);
    let mut subs = Queue::with_capacity(2);
    step::down(
        &mut io,
        &env,
        Request::Load { owner: OWNER, root, path: Box::from(&b"f"[..]), max: 4, no_follow: false },
        &mut events,
        &mut subs,
    );
    complete(&mut io, &env, subs.pop().expect("open"), Ok(Done::Fd(CHILD)), &mut events, &mut subs);
    let stat = subs.pop().expect("stat");
    step::expire(&mut io, &Env { now: Time::ZERO.saturating_add(Duration::from_secs(1)), ..env }, &mut subs);
    assert!(subs.is_empty(), "stat is not cancellable");
    complete(
        &mut io,
        &env,
        stat,
        Ok(Done::Stat(Stat { kind: Kind::File, size: 1, mode: 0o644, owner: 1000, links: 1 })),
        &mut events,
        &mut subs,
    );
    assert!(events.is_empty());
    complete(&mut io, &env, subs.pop().expect("cleanup close"), Ok(Done::Nothing), &mut events, &mut subs);
    assert_eq!(events.pop(), Some(Event::failed(OWNER, crate::kernel::Error::TimedOut)));
}

#[test]
fn each_file_request_uses_its_own_absolute_deadline() {
    let mut io = io();
    let env = file_env();
    let mut events = Queue::with_capacity(2);
    let mut subs = Queue::with_capacity(2);
    let deadline = Time::from_nanos(7_000_000_000);
    step::down_until(
        &mut io,
        &env,
        deadline,
        Request::OpenRead { owner: OWNER, root: ROOT, name: Box::from(&b"file"[..]) },
        &mut events,
        &mut subs,
    );
    assert_eq!(io.next_deadline(), Some(deadline));
    let open = subs.pop().expect("open in flight");
    step::expire(&mut io, &Env { now: Time::from_nanos(1_000_000_000), ..env }, &mut subs);
    assert!(subs.is_empty());
    step::expire(&mut io, &Env { now: deadline, ..env }, &mut subs);
    let cancel = subs.pop().expect("deadline cancels open");
    assert_eq!(cancel.kind, Op::Cancel { target: open.op });
    complete(&mut io, &env, cancel, Ok(Done::Nothing), &mut events, &mut subs);
    complete(&mut io, &env, open, Err(crate::kernel::Error::Cancelled), &mut events, &mut subs);
    assert_eq!(events.pop(), Some(Event::failed(OWNER, crate::kernel::Error::TimedOut)));
    assert!(io.takes());
}

#[test]
fn an_owner_cancel_waits_for_the_target_and_reports_when_it_stopped() {
    let mut io = io();
    let env = file_env();
    let mut events = Queue::with_capacity(2);
    let mut subs = Queue::with_capacity(2);
    step::down_until(
        &mut io,
        &env,
        Time::from_nanos(9_000_000_000),
        Request::OpenRead { owner: OWNER, root: ROOT, name: Box::from(&b"file"[..]) },
        &mut events,
        &mut subs,
    );
    let open = subs.pop().expect("open in flight");
    file_layer::cancel(&mut io, Token::new(99), &mut subs);
    assert!(subs.is_empty(), "another owner cannot cancel the request");
    file_layer::cancel(&mut io, OWNER, &mut subs);
    file_layer::cancel(&mut io, OWNER, &mut subs);
    let cancel = subs.pop().expect("one cancel");
    assert_eq!(cancel.kind, Op::Cancel { target: open.op });
    complete(&mut io, &env, open, Err(crate::kernel::Error::Cancelled), &mut events, &mut subs);
    assert_eq!(events.pop(), Some(Event::Cancelled { owner: OWNER }));
    assert!(!io.takes(), "the cancel completion still holds its slot");
    complete(&mut io, &env, cancel, Ok(Done::Nothing), &mut events, &mut subs);
    assert!(io.takes());
    assert!(events.is_empty());
}

#[test]
fn a_successful_direct_write_reports_written_when_cancel_was_too_late() {
    let mut io = io();
    let env = file_env();
    let file = io.adopt_root(ROOT).expect("room for file");
    let mut events = Queue::with_capacity(2);
    let mut subs = Queue::with_capacity(2);
    step::down_until(
        &mut io,
        &env,
        Time::from_nanos(9_000_000_000),
        Request::WriteAt { owner: OWNER, file, offset: 0, bytes: Box::from(&b"hi"[..]) },
        &mut events,
        &mut subs,
    );
    let write = subs.pop().expect("write in flight");
    file_layer::cancel(&mut io, OWNER, &mut subs);
    let cancel = subs.pop().expect("cancel submitted");
    complete(&mut io, &env, cancel, Err(crate::kernel::Error::TooLate), &mut events, &mut subs);
    complete(&mut io, &env, write, Ok(Done::Count(2)), &mut events, &mut subs);
    assert_eq!(events.pop(), Some(Event::Written { owner: OWNER }));
    assert!(io.takes() && events.is_empty());
}

#[test]
fn a_short_write_can_be_cancelled_before_its_remaining_bytes() {
    let mut io = io();
    let env = file_env();
    let file = io.adopt_root(ROOT).expect("room for file");
    let mut events = Queue::with_capacity(2);
    let mut subs = Queue::with_capacity(2);
    step::down_until(
        &mut io,
        &env,
        Time::from_nanos(9_000_000_000),
        Request::WriteAt { owner: OWNER, file, offset: 0, bytes: Box::from(&b"hi"[..]) },
        &mut events,
        &mut subs,
    );
    let write = subs.pop().expect("write in flight");
    file_layer::cancel(&mut io, OWNER, &mut subs);
    let cancel = subs.pop().expect("cancel submitted");
    complete(&mut io, &env, write, Ok(Done::Count(1)), &mut events, &mut subs);
    assert_eq!(events.pop(), Some(Event::Cancelled { owner: OWNER }));
    assert!(subs.is_empty(), "the remainder is not written");
    complete(&mut io, &env, cancel, Err(crate::kernel::Error::TooLate), &mut events, &mut subs);
    assert!(io.takes());
}

fn write_opened() -> (FileIo, Token, u64) {
    let mut io = io();
    let env = file_env();
    let mut events = Queue::with_capacity(2);
    let mut subs = Queue::with_capacity(2);
    step::down(
        &mut io,
        &env,
        Request::OpenWrite { owner: OWNER, root: ROOT, name: Box::from(&b"log"[..]) },
        &mut events,
        &mut subs,
    );
    complete(&mut io, &env, subs.pop().expect("open"), Ok(Done::Fd(CHILD)), &mut events, &mut subs);
    let stat = Stat { kind: Kind::File, size: 10, mode: 0o644, owner: 1000, links: 1 };
    complete(&mut io, &env, subs.pop().expect("stat"), Ok(Done::Stat(stat)), &mut events, &mut subs);
    let Some(Event::Opened { owner: OWNER, file, len }) = events.pop() else { panic!("opened with its length") };
    (io, file, len)
}

#[test]
fn open_write_asks_the_write_open_and_tells_not_found_as_failed() {
    let mut io = io();
    let env = file_env();
    let mut events = Queue::with_capacity(2);
    let mut subs = Queue::with_capacity(2);
    let request = Request::OpenWrite { owner: OWNER, root: ROOT, name: Box::from(&b"log"[..]) };
    step::down(&mut io, &env, request, &mut events, &mut subs);
    let submit = subs.pop().expect("one open");
    let Op::Open { how, .. } = &submit.kind else { panic!("an open") };
    assert_eq!(*how, OpenHow::Write, "an existing file, to read and write, never made");
    complete(&mut io, &env, submit, Err(crate::kernel::Error::NotFound), &mut events, &mut subs);
    assert_eq!(events.pop(), Some(Event::failed(OWNER, crate::kernel::Error::NotFound)));
    assert!(io.takes(), "the driver takes the next request");
}

#[test]
fn open_write_tells_the_opened_files_length() {
    let (io, _, len) = write_opened();
    assert_eq!(len, 10, "the opened file's stat supplies its length");
    assert!(io.takes(), "the writer can send the next request");
}

#[test]
fn open_write_no_follow_asks_the_no_follow_write_open() {
    let mut io = io();
    let env = file_env();
    let mut events = Queue::with_capacity(2);
    let mut subs = Queue::with_capacity(2);
    step::down(
        &mut io,
        &env,
        Request::OpenWriteNoFollow { owner: OWNER, root: ROOT, name: Box::from(&b"log"[..]) },
        &mut events,
        &mut subs,
    );
    let submit = subs.pop().expect("one open");
    let Op::Open { how, .. } = &submit.kind else { panic!("an open") };
    assert_eq!(*how, OpenHow::WriteNoFollow, "every symbolic link is refused");
    complete(&mut io, &env, submit, Err(crate::kernel::Error::TooManyLinks), &mut events, &mut subs);
    assert!(io.takes(), "the refused open settles");
}

#[test]
fn a_cut_submits_a_truncate_and_tells_cut() {
    let (mut io, file, _) = write_opened();
    let env = file_env();
    let mut events = Queue::with_capacity(2);
    let mut subs = Queue::with_capacity(2);
    step::down(&mut io, &env, Request::Cut { owner: OWNER, file, length: 4 }, &mut events, &mut subs);
    let submit = subs.pop().expect("one truncate");
    assert_eq!(submit.kind, Op::Truncate { fd: CHILD, length: 4 });
    complete(&mut io, &env, submit, Ok(Done::Nothing), &mut events, &mut subs);
    assert_eq!(events.pop(), Some(Event::Cut { owner: OWNER }));
    assert_eq!(io.open_files(), 1, "cut keeps the file open");
    assert!(io.takes(), "the cut ends in one terminal");
}

#[test]
fn a_cut_of_a_file_io_does_not_hold_fails_not_found() {
    let mut io = io();
    let env = file_env();
    let mut events = Queue::with_capacity(2);
    let mut subs = Queue::with_capacity(2);
    step::down(&mut io, &env, Request::Cut { owner: OWNER, file: Token::new(99), length: 4 }, &mut events, &mut subs);
    assert!(subs.is_empty(), "no descriptor is known to submit");
    assert_eq!(events.pop(), Some(Event::failed(OWNER, crate::kernel::Error::NotFound)));
    assert!(io.takes(), "the refused cut is terminal");
}

#[test]
fn a_cut_past_its_deadline_waits_for_its_truncate_then_times_out() {
    let (mut io, file, _) = write_opened();
    let env = file_env();
    let mut events = Queue::with_capacity(2);
    let mut subs = Queue::with_capacity(2);
    step::down(&mut io, &env, Request::Cut { owner: OWNER, file, length: 4 }, &mut events, &mut subs);
    let submit = subs.pop().expect("truncate in flight");
    step::expire(&mut io, &Env { now: Time::ZERO.saturating_add(Duration::from_secs(1)), ..env }, &mut subs);
    assert!(subs.is_empty(), "Truncate is never cancelled");
    assert!(events.is_empty() && !io.takes(), "the deadline waits for the record to return");
    complete(&mut io, &env, submit, Ok(Done::Nothing), &mut events, &mut subs);
    assert_eq!(events.pop(), Some(Event::failed(OWNER, crate::kernel::Error::TimedOut)));
    assert!(io.takes(), "the expired cut settles after its truncate");
    assert!(events.is_empty(), "exactly one terminal");
}

const CANARY: &[u8] = b"canary-7f3a9c";

#[test]
#[expect(clippy::disallowed_macros, reason = "verify Debug hides stored file contents")]
fn debug_of_a_stored_record_shows_its_length_never_its_bytes() {
    let root = Token::new(8);
    for (request, expected) in [
        (
            Request::Store {
                owner: OWNER,
                root,
                path: Box::from(&b"tokens/a"[..]),
                bytes: Box::from(CANARY),
                expected: crate::file::Expect::Any,
                no_follow: true,
            },
            "Store { owner: Token(7), root: Token(8), path: [116, 111, 107, 101, 110, 115, 47, 97], len: 13, expected: Any, no_follow: true }",
        ),
        (
            Request::WriteAt { owner: OWNER, file: root, offset: 5, bytes: Box::from(CANARY) },
            "WriteAt { owner: Token(7), file: Token(8), offset: 5, len: 13 }",
        ),
    ] {
        let debug = alloc::format!("{request:?}");
        assert!(!debug.contains("canary-7f3a9c"));
        assert!(!debug.contains(&alloc::format!("{CANARY:?}")));
        assert_eq!(debug, expected);
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "verify Debug hides loaded and read file contents")]
fn debug_of_a_loaded_or_read_file_shows_its_length_never_its_bytes() {
    for (event, expected) in [
        (Event::Loaded { owner: OWNER, bytes: Box::from(CANARY) }, "Loaded { owner: Token(7), len: 13 }"),
        (Event::Read { owner: OWNER, bytes: Box::from(CANARY) }, "Read { owner: Token(7), len: 13 }"),
    ] {
        let debug = alloc::format!("{event:?}");
        assert!(!debug.contains("canary-7f3a9c"));
        assert!(!debug.contains(&alloc::format!("{CANARY:?}")));
        assert_eq!(debug, expected);
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "verify Debug preserves the derived form without contents")]
fn debug_of_records_without_contents_is_the_derived_form() {
    let request = Request::Load {
        owner: OWNER,
        root: Token::new(8),
        path: Box::from(&b"tokens/a"[..]),
        max: 16,
        no_follow: true,
    };
    assert_eq!(
        alloc::format!("{request:?}"),
        "Load { owner: Token(7), root: Token(8), path: [116, 111, 107, 101, 110, 115, 47, 97], max: 16, no_follow: true }"
    );
    let event = Event::Failed {
        owner: OWNER,
        error: crate::kernel::Error::Other(5),
        committed: true,
        residue: Some(crate::file::Residue { name: Box::from(&b"tmp"[..]), error: crate::kernel::Error::ReadOnly }),
    };
    assert_eq!(
        alloc::format!("{event:?}"),
        "Failed { owner: Token(7), error: Other(5), committed: true, residue: Some(Residue { name: [116, 109, 112], error: ReadOnly }) }"
    );
}

#[test]
#[expect(clippy::disallowed_macros, reason = "verify Debug hides file contents throughout reads and loads")]
fn debug_of_file_io_hides_a_load_and_a_read_in_flight() {
    let mut io = io();
    let env = file_env();
    let root = io.adopt_root(ROOT).expect("room for root");
    let mut events = Queue::with_capacity(2);
    let mut subs = Queue::with_capacity(2);
    step::down(
        &mut io,
        &env,
        Request::Load { owner: OWNER, root, path: Box::from(&b"f"[..]), max: 16, no_follow: true },
        &mut events,
        &mut subs,
    );
    complete(&mut io, &env, subs.pop().expect("open"), Ok(Done::Fd(CHILD)), &mut events, &mut subs);
    let stat = Stat { kind: Kind::File, size: 13, mode: 0o600, owner: 1000, links: 1 };
    complete(&mut io, &env, subs.pop().expect("stat"), Ok(Done::Stat(stat)), &mut events, &mut subs);
    read(&mut io, &env, subs.pop().expect("first load read"), CANARY, &mut events, &mut subs);
    let loading = alloc::format!("{io:?}");
    read(&mut io, &env, subs.pop().expect("end read"), b"", &mut events, &mut subs);
    let closing = alloc::format!("{io:?}");
    complete(&mut io, &env, subs.pop().expect("close"), Ok(Done::Nothing), &mut events, &mut subs);
    assert_eq!(events.pop(), Some(Event::Loaded { owner: OWNER, bytes: Box::from(CANARY) }));

    step::down(
        &mut io,
        &env,
        Request::OpenRead { owner: OWNER, root: ROOT, name: Box::from(&b"f"[..]) },
        &mut events,
        &mut subs,
    );
    complete(&mut io, &env, subs.pop().expect("open"), Ok(Done::Fd(CHILD)), &mut events, &mut subs);
    complete(&mut io, &env, subs.pop().expect("stat"), Ok(Done::Stat(stat)), &mut events, &mut subs);
    let Event::Opened { file, .. } = events.pop().expect("opened file") else { panic!("opened file") };
    step::down(&mut io, &env, Request::ReadAt { owner: OWNER, file, offset: 5, max: 16 }, &mut events, &mut subs);
    read(&mut io, &env, subs.pop().expect("first positioned read"), CANARY, &mut events, &mut subs);
    for debug in [loading, closing, alloc::format!("{io:?}")] {
        assert!(!debug.contains("canary-7f3a9c"));
        assert!(!debug.contains(&alloc::format!("{CANARY:?}")));
        assert!(debug.contains("len: 13"));
    }
}

#[test]
fn file_limits_need_room_but_whole_may_be_smaller_than_read() {
    let limits = file_layer::Limits { files: 3, read: 4096, entries: 4, whole: 1, timeout: Duration::from_secs(1) };
    assert!(file_layer::valid(&limits));
    assert!(FileIo::with_limits(&limits).takes());
    for empty in [
        file_layer::Limits { files: 0, ..limits },
        file_layer::Limits { read: 0, ..limits },
        file_layer::Limits { entries: 0, ..limits },
        file_layer::Limits { whole: 0, ..limits },
        file_layer::Limits { timeout: Duration::ZERO, ..limits },
    ] {
        assert!(!file_layer::valid(&empty), "{empty:?}");
    }
}

pub(super) fn file_env() -> Env<file_layer::Limits> {
    Env {
        now: Time::ZERO,
        wall: Wall::EPOCH,
        limits: file_layer::Limits { files: 3, read: 16, entries: 4, whole: 16, timeout: Duration::from_secs(1) },
    }
}

#[test]
fn whole_load_chunks_use_the_environment_read_limit() {
    let mut io = io();
    let env = Env { limits: file_layer::Limits { read: 2, ..file_env().limits }, ..file_env() };
    let root = io.adopt_root(ROOT).expect("room for root");
    let mut events = Queue::uncapped();
    let mut subs = Queue::uncapped();
    step::down(
        &mut io,
        &env,
        Request::Load { owner: OWNER, root, path: Box::from(&b"f"[..]), max: 8, no_follow: false },
        &mut events,
        &mut subs,
    );
    complete(&mut io, &env, subs.pop().expect("open"), Ok(Done::Fd(CHILD)), &mut events, &mut subs);
    complete(
        &mut io,
        &env,
        subs.pop().expect("stat"),
        Ok(Done::Stat(Stat { kind: Kind::File, size: 3, mode: 0o644, owner: 1000, links: 1 })),
        &mut events,
        &mut subs,
    );
    for bytes in [b"ab".as_slice(), b"c", b""] {
        let submit = subs.pop().expect("read");
        let Op::Read { buf, .. } = &submit.kind else { panic!("a load reads") };
        assert_eq!(buf.len(), 2, "each chunk uses the environment's read bound");
        read(&mut io, &env, submit, bytes, &mut events, &mut subs);
    }
    assert!(events.is_empty(), "the close settles before the terminal");
    complete(&mut io, &env, subs.pop().expect("close"), Ok(Done::Nothing), &mut events, &mut subs);
    assert_eq!(events.pop(), Some(Event::Loaded { owner: OWNER, bytes: Box::from(&b"abc"[..]) }));
    assert!(io.takes());
}
