//! Interrupted transfers at the adapter's seam (shell.md, 7.4), without a ring.

use io_uring::{opcode, squeue};
use skein_io::kernel::{Complete, Done, Error, Fd, Op, ServiceSignal, Signalled};
use skein_lib::Token;

use super::super::{Flight, Prepared, Reaped, Table, prepare, user_data};

const GENERATION: u32 = 7;
const TRANSFER: Token = Token::new(1);
const FD: Fd = Fd::new(123);
const BYTES: &[u8] = b"abcdef";
const BUFFER_BYTES: usize = BYTES.len();
const BUFFER_COUNT: u32 = 6; // The six fixture bytes.
// A prefix already transferred, leaving four of the fixture's six bytes.
const FROM: u32 = 2;
const REMAINING_COUNT: u32 = 4; // Six fixture bytes after the two-byte prefix.
const AT: u64 = 987;
// Two bytes is a short count for both the full buffer and its remaining suffix.
const COUNT: i32 = 2;
// Cross the Open/EAGAIN bound to check that EINTR has no such bound (shell.md, 7.4).
const INTERRUPTIONS: u32 = 17;

/// The Linux SQE prefix through `user_data`, including transfer buffer and offset.
#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
struct TransferEntry {
    opcode: u8,
    flags: u8,
    priority: u16,
    fd: i32,
    offset: u64,
    buffer: u64,
    len: u32,
    op_flags: u32,
    data: u64,
}

fn entry_fields(entry: &squeue::Entry) -> TransferEntry {
    assert_eq!(size_of::<squeue::Entry>(), 64, "a Linux SQE is 64 bytes");
    assert_eq!(size_of::<TransferEntry>(), 40, "the SQE prefix has no padding");
    // SAFETY: Entry transparently wraps a Linux SQE. Transfer builders
    // initialize this integer-only prefix with the kernel ABI's C layout.
    // No buffer pointer is dereferenced.
    unsafe { std::ptr::read_unaligned(std::ptr::from_ref(entry).cast::<TransferEntry>()) }
}

fn writes(from: u32, at: u64) -> [(Op, u8, u64); 4] {
    [
        (Op::PipeWrite { fd: FD, bytes: Box::from(BYTES), from }, opcode::Write::CODE, u64::MAX),
        (Op::Write { fd: FD, bytes: Box::from(BYTES), from, at }, opcode::Write::CODE, at),
        (Op::Append { fd: FD, bytes: Box::from(BYTES), from }, opcode::Write::CODE, u64::MAX),
        (Op::Send { fd: FD, bytes: Box::from(BYTES), from }, opcode::Send::CODE, 0),
    ]
}

fn reads(at: u64) -> [(Op, u8, u64); 3] {
    [
        (Op::PipeRead { fd: FD, buf: vec![0; BUFFER_BYTES].into_boxed_slice() }, opcode::Read::CODE, u64::MAX),
        (Op::Read { fd: FD, buf: vec![0; BUFFER_BYTES].into_boxed_slice(), at }, opcode::Read::CODE, at),
        (Op::Recv { fd: FD, buf: vec![0; BUFFER_BYTES].into_boxed_slice() }, opcode::Recv::CODE, 0),
    ]
}

fn pending(kind: Op) -> (Table, TransferEntry) {
    let mut table = Table::new(1);
    let _reserved = table.free.pop().expect("reserve the only slot");
    let slot = table.slots.first_mut().expect("one slot");
    slot.generation = GENERATION;
    let flight = slot.flight.insert(Flight::new(TRANSFER, kind));
    let Prepared::Entry(entry) = prepare(flight, &table.tokens, &mut table.cancelled, &mut table.listing) else {
        panic!("a transfer prepares a ring entry");
    };
    let data = user_data(0, GENERATION);
    table.tokens.insert(TRANSFER, data);
    (table, entry_fields(&entry.user_data(data)))
}

fn interrupted(table: &mut Table, original: &TransferEntry) {
    let Reaped::Again(index) = table.complete(user_data(0, GENERATION), -libc::EINTR) else {
        panic!("an interrupted transfer resubmits without completing");
    };
    assert_eq!(index, 0, "retry the same slot");
    assert!(table.free.is_empty(), "keep the slot reserved");
    assert!(table.ready.is_empty(), "no outward completion while retrying");
    assert_eq!(table.tokens.len(), 1, "keep exactly one token in flight");
    assert_eq!(table.tokens.get(&TRANSFER), Some(&user_data(0, GENERATION)), "keep the token and generation");
    assert_eq!(entry_fields(&table.resubmission(index)), *original, "keep every SQE field and pointer");
    assert_eq!(table.resubmitted, 0, "the Open/EAGAIN counter excludes interrupted transfers");
}

fn finished(table: &mut Table, raw: i32, expected: &Result<Done, Error>) -> Op {
    let Reaped::Complete(Complete { op, kind, result }) = table.complete(user_data(0, GENERATION), raw) else {
        panic!("a terminal answer completes the transfer");
    };
    assert_eq!(op, TRANSFER, "complete the original token");
    assert_eq!(&result, expected, "decode the terminal kernel answer");
    assert!(table.tokens.is_empty(), "retire the token");
    assert!(table.cancelled.is_empty(), "retire any cancellation state");
    assert!(table.ready.is_empty(), "no duplicate outward completion");
    assert_eq!(table.free, [0], "release exactly one slot");
    let slot = table.slots.first().expect("one slot");
    assert!(slot.flight.is_none(), "the operation is no longer in flight");
    assert_eq!(slot.generation, 8, "advance the generation exactly once");
    kind
}

#[test]
fn interrupted_writes_resubmit_unchanged_then_complete_once() {
    for ((kind, code, offset), (expected, _, _)) in writes(0, 0).into_iter().zip(writes(0, 0)) {
        let (mut table, original) = pending(kind);
        assert_eq!(original.opcode, code, "submit the requested write opcode");
        assert_eq!(original.fd, FD.raw(), "write to the requested descriptor");
        assert_eq!(original.offset, offset, "keep the requested file position");
        assert_eq!(original.len, BUFFER_COUNT, "write the six fixture bytes");
        assert_eq!(original.data, user_data(0, GENERATION), "submit the original slot and generation");
        interrupted(&mut table, &original);
        assert_eq!(finished(&mut table, COUNT, &Ok(Done::Count(2))), expected, "hand back the original record");
    }
}

#[test]
fn interrupted_writes_keep_nonzero_buffer_and_file_offsets() {
    for ((kind, _, offset), (expected, _, _)) in writes(FROM, AT).into_iter().zip(writes(FROM, AT)) {
        let (mut table, original) = pending(kind);
        assert_eq!(original.offset, offset, "keep the nonzero file position");
        assert_eq!(original.len, REMAINING_COUNT, "the two-byte prefix leaves four bytes");
        interrupted(&mut table, &original);
        assert_eq!(finished(&mut table, COUNT, &Ok(Done::Count(2))), expected, "retain bytes and offsets");
    }
}

#[test]
fn interrupted_reads_resubmit_unchanged_then_complete_once() {
    for ((kind, code, offset), (expected, _, _)) in reads(AT).into_iter().zip(reads(AT)) {
        let (mut table, original) = pending(kind);
        assert_eq!(original.opcode, code, "submit the requested read opcode");
        assert_eq!(original.fd, FD.raw(), "read from the requested descriptor");
        assert_eq!(original.offset, offset, "keep the requested file position");
        interrupted(&mut table, &original);
        assert_eq!(finished(&mut table, COUNT, &Ok(Done::Count(2))), expected, "hand back the original read buffer");
    }
}

#[test]
fn interrupted_signal_reads_keep_the_record_then_decode_success() {
    let (mut table, original) = pending(Op::ReadSignal { fd: FD });
    interrupted(&mut table, &original);
    let flight = table.slots.first_mut().expect("one slot").flight.as_mut().expect("read in flight");
    flight.signal_info.get_mut().ssi_signo = u32::try_from(libc::SIGTERM).expect("positive signal");
    let count = i32::try_from(size_of::<libc::signalfd_siginfo>()).expect("signal record fits i32");
    let expected = Ok(Done::Signalled(Signalled::Termination { signal: ServiceSignal::Terminate, sender: None }));
    assert_eq!(finished(&mut table, count, &expected), Op::ReadSignal { fd: FD }, "hand back the signal read");
}

#[test]
fn requested_cancellation_completes_interrupted_transfers_as_cancelled() {
    for retries in [0, 1] {
        for (kind, _, _) in writes(FROM, AT).into_iter().chain(reads(AT)).chain([(
            Op::ReadSignal { fd: FD },
            opcode::Read::CODE,
            u64::MAX,
        )]) {
            let (mut table, original) = pending(kind);
            for _ in 0..retries {
                interrupted(&mut table, &original);
            }
            table.cancelled.insert(TRANSFER);
            let _record = finished(&mut table, -libc::EINTR, &Err(Error::Cancelled));
        }
    }
}

#[test]
fn genuine_transfer_errors_complete_without_retrying() {
    for (kind, _, _) in
        writes(FROM, AT).into_iter().chain(reads(AT)).chain([(Op::ReadSignal { fd: FD }, opcode::Read::CODE, u64::MAX)])
    {
        let (mut table, _original) = pending(kind);
        let _record = finished(&mut table, -libc::EIO, &Err(Error::Other(libc::EIO)));
    }
    for (kind, _, _) in writes(FROM, AT) {
        let (mut table, original) = pending(kind);
        interrupted(&mut table, &original);
        let _record = finished(&mut table, -libc::EIO, &Err(Error::Other(libc::EIO)));
    }
}

#[test]
fn broken_pipe_errors_keep_their_operation_specific_mapping() {
    for (kind, expected) in [
        (Op::PipeWrite { fd: FD, bytes: Box::from(BYTES), from: FROM }, Error::BrokenPipe),
        (Op::Send { fd: FD, bytes: Box::from(BYTES), from: FROM }, Error::BrokenPipe),
        (Op::Write { fd: FD, bytes: Box::from(BYTES), from: FROM, at: AT }, Error::Other(libc::EPIPE)),
        (Op::Append { fd: FD, bytes: Box::from(BYTES), from: FROM }, Error::Other(libc::EPIPE)),
    ] {
        let (mut table, _original) = pending(kind);
        let _record = finished(&mut table, -libc::EPIPE, &Err(expected));
    }
}

#[test]
fn interrupted_transfers_retry_past_the_open_race_bound() {
    for (kind, _, _) in writes(FROM, AT).into_iter().chain(reads(AT)) {
        let (mut table, original) = pending(kind);
        for _ in 0..INTERRUPTIONS {
            interrupted(&mut table, &original);
        }
        let _record = finished(&mut table, COUNT, &Ok(Done::Count(2)));
    }
}

#[test]
fn short_transfers_complete_without_continuing_the_record() {
    for (kind, _, _) in writes(FROM, AT).into_iter().chain(reads(AT)) {
        let (mut table, _original) = pending(kind);
        let _record = finished(&mut table, COUNT, &Ok(Done::Count(2)));
    }
}
