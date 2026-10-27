//! Raw race completions at the adapter's seam, without a real ring.

use io_uring::{opcode, squeue};
use skein_io::kernel::{Complete, Done, Error, Fd, Op, OpenHow};
use skein_lib::Token;

use super::super::{AGAIN, Flight, Prepared, Reaped, Table, prepare, user_data};

const GENERATION: u32 = 7;
const OPEN: Token = Token::new(1);
const ROOT: Fd = Fd::new(123);
const OPENED: Fd = Fd::new(456);

/// The Linux SQE prefix through `user_data`, including `OpenAt2`'s pointers.
#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
struct OpenEntry {
    opcode: u8,
    flags: u8,
    priority: u16,
    root: i32,
    how: u64,
    path: u64,
    how_len: u32,
    open_flags: u32,
    data: u64,
}

fn entry_fields(entry: &squeue::Entry) -> OpenEntry {
    assert_eq!(size_of::<squeue::Entry>(), 64, "a Linux SQE is 64 bytes");
    assert_eq!(size_of::<OpenEntry>(), 40, "the SQE prefix has no padding");
    // SAFETY: Entry transparently wraps a Linux SQE. OpenAt2's builder
    // initializes this integer-only prefix, whose C layout follows the
    // kernel ABI. Read only the prefix, without dereferencing its pointers.
    unsafe { std::ptr::read_unaligned(std::ptr::from_ref(entry).cast::<OpenEntry>()) }
}

fn operation() -> Op {
    // A path-only success needs no descriptor inspection, so the entire
    // completion stream can be injected without opening a file or ring.
    Op::Open { root: ROOT, path: Box::from(&b"sub/../file"[..]), how: OpenHow::PathNoFollow }
}

fn pending() -> (Table, OpenEntry) {
    let mut table = Table::new(1);
    assert_eq!(table.free.pop(), Some(0), "reserve the only slot");
    let slot = table.slots.first_mut().expect("one slot");
    slot.generation = GENERATION;
    let flight = slot.flight.insert(Flight::new(OPEN, operation()));
    let Prepared::Entry(entry) = prepare(flight, &table.tokens, &mut table.cancelled, &mut table.listing) else {
        panic!("an Open prepares an OpenAt2 entry");
    };
    let data = user_data(0, GENERATION);
    table.tokens.insert(OPEN, data);
    let original = entry_fields(&entry.user_data(data));
    assert_eq!(original.opcode, opcode::OpenAt2::CODE, "submit an OpenAt2");
    assert_eq!(original.root, ROOT.raw(), "resolve beneath the requested root");
    (table, original)
}

fn race(table: &mut Table, original: &OpenEntry, count: u32) {
    let Reaped::Again(index) = table.complete(user_data(0, GENERATION), -libc::EAGAIN) else {
        panic!("race {count} must resubmit instead of completing");
    };
    assert_eq!(index, 0, "retry the same slot");
    assert!(table.free.is_empty(), "the slot stays reserved");
    assert!(table.ready.is_empty(), "no outward completion while retrying");
    assert_eq!(table.tokens.get(&OPEN), Some(&user_data(0, GENERATION)), "keep the token and generation");
    let entry = table.resubmission(index);
    let retried = entry_fields(&entry);
    assert_eq!(&retried, original, "the resubmitted OpenAt2 keeps its fields and pointers");
    assert_eq!(retried.data, user_data(0, GENERATION), "submit under the same slot and generation");
    let slot = table.slots.first_mut().expect("the slot stays allocated");
    assert_eq!(slot.generation, GENERATION, "no retirement between attempts");
    let flight = slot.flight.as_mut().expect("the Open stays in flight");
    assert_eq!(flight.kind, operation(), "keep the root, path and opening mode");
    assert_eq!(flight.again, count, "one retry per race");
    assert_eq!(&*flight.path, b"sub/../file\0", "keep the terminated path");
    assert_eq!(
        retried.path,
        u64::try_from(flight.path.as_ptr().addr()).expect("a path pointer fits u64"),
        "the entry points to the retained path"
    );
    assert_eq!(
        retried.how,
        u64::try_from(flight.open_how.get().addr()).expect("an open_how pointer fits u64"),
        "the entry points to the retained open flags"
    );
    let flags = u64::try_from(libc::O_PATH | libc::O_NOFOLLOW | libc::O_CLOEXEC).expect("positive open flags");
    let expected =
        io_uring::types::OpenHow::new().flags(flags).mode(0).resolve(libc::RESOLVE_BENEATH | libc::RESOLVE_NO_SYMLINKS);
    assert_eq!(format!("{:?}", flight.open_how.get_mut()), format!("{expected:?}"), "keep the open flags");
}

fn finished(table: &mut Table, raw: i32, expected: &Result<Done, Error>) {
    let Reaped::Complete(Complete { op, kind, result }) = table.complete(user_data(0, GENERATION), raw) else {
        panic!("a terminal answer must complete the Open");
    };
    assert_eq!(op, OPEN, "complete the original token");
    assert_eq!(kind, operation(), "return the original record");
    assert_eq!(&result, expected, "decode the terminal kernel answer");
    assert!(table.tokens.is_empty(), "retire the token");
    assert!(table.cancelled.is_empty(), "retire any cancellation state");
    assert_eq!(table.free, [0], "release exactly one slot");
    let slot = table.slots.first().expect("one slot");
    assert!(slot.flight.is_none(), "the operation is no longer in flight");
    assert_eq!(slot.generation, 8, "advance the generation exactly once");
}

#[test]
fn an_open_through_dot_dot_is_resubmitted_after_each_race_then_completes() {
    assert_eq!(AGAIN, 16, "the kernel contract allows sixteen resubmissions");
    for races in [1, 2, 16] {
        let (mut table, original) = pending();
        for count in 1..=races {
            race(&mut table, &original, count);
        }
        finished(&mut table, OPENED.raw(), &Ok(Done::Fd(OPENED)));
    }
}

#[test]
fn an_open_through_dot_dot_still_racing_after_sixteen_retries_completes_with_eagain() {
    let (mut table, original) = pending();
    for count in 1..=16 {
        race(&mut table, &original, count);
    }
    finished(&mut table, -libc::EAGAIN, &Err(Error::Other(libc::EAGAIN)));
}

#[test]
fn a_cancelled_open_through_dot_dot_is_not_resubmitted_after_a_race() {
    for races in [0, 1, 2, 16] {
        let (mut table, original) = pending();
        for count in 1..=races {
            race(&mut table, &original, count);
        }
        table.cancelled.insert(OPEN);
        finished(&mut table, -libc::EAGAIN, &Err(Error::Other(libc::EAGAIN)));
    }
}
