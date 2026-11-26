//! Cleanup failures keep their operation, resource and cause (temper/docs/design/domain/forge/releases.md, 2).
use super::{Domain, Name, Queue, ReleaseEnding, ReleaseRow, Request, Stored, Writer, emit, repository_reads};
use alloc::boxed::Box;
use skein_forge::api::{Bound, Excess, Repository, Write};
use skein_lib::{Decimal, bytes};

pub(super) fn report(out: &mut Queue<Request>, task: u64, entry: Option<u64>, words: Box<[u8]>) {
    emit(out, Request::ReleaseFailed { task, entry, words });
}

pub(super) fn held(d: &mut Domain, row: &mut ReleaseRow, words: Box<[u8]>, out: &mut Queue<Request>) {
    row.failed = true;
    d.release_reads.remove(&row.task);
    d.release_waits.remove(&row.task);
    d.releases.insert(row.task, row.clone()).expect("release slot checked");
    emit(out, Request::Save { record: Stored::Release(row.clone()), read_afresh: None, release: None });
    report(out, row.task, None, words);
}

pub(super) fn task_words(task: u64, why: &[u8]) -> Box<[u8]> {
    bytes::concat(&[b"release of task ", Decimal::of(task).as_bytes(), b": ", why])
}

pub(super) fn conflict(row: &ReleaseRow, root: u64, ending: ReleaseEnding) -> Box<[u8]> {
    let why = bytes::concat(&[
        b"under way for root ",
        Decimal::of(row.root).as_bytes(),
        b" as ",
        ending_words(row.ending),
        b"; asked by root ",
        Decimal::of(root).as_bytes(),
        b" as ",
        ending_words(ending),
    ]);
    task_words(row.task, &why)
}

fn ending_words(ending: ReleaseEnding) -> &'static [u8] {
    match ending {
        ReleaseEnding::Done => b"done",
        ReleaseEnding::Cancelled => b"cancelled",
        ReleaseEnding::Failed => b"failed",
    }
}

pub(super) fn full(task: u64, configured: u32, out: &mut Queue<Request>) {
    let configured = u64::from(configured);
    let why =
        crate::RefusalCause::Limit { member: "forge.tasks", configured, needed: configured.saturating_add(1) }.words();
    report(out, task, None, task_words(task, &why));
}

pub(super) fn writing(d: &Domain, name: &Name, writer: Writer) -> Box<[u8]> {
    let why = match writer {
        Writer::Entry(entry) => bytes::concat(&[b"still written by outbox entry ", Decimal::of(entry).as_bytes()]),
        Writer::Run { task, attempt } | Writer::Lost { task, attempt } => bytes::concat(&[
            b"still written by run ",
            Decimal::of(attempt).as_bytes(),
            b" of task ",
            Decimal::of(task).as_bytes(),
        ]),
    };
    branch(d, b"delete branch", name, &why)
}

pub(super) fn reread(d: &Domain, name: &Name, task: u64, attempt: u64, answer: &[u8]) -> Box<[u8]> {
    let why = bytes::concat(&[
        b"the reread after run ",
        Decimal::of(attempt).as_bytes(),
        b" of task ",
        Decimal::of(task).as_bytes(),
        b" failed: ",
        answer,
    ]);
    branch(d, b"read branch", name, &why)
}

pub(super) fn branch(d: &Domain, operation: &[u8], name: &Name, why: &[u8]) -> Box<[u8]> {
    crate::effect_words::branch_account(operation, name, d.repository(repository_reads::repository(name)), why)
}

pub(super) fn not_sent(d: &Domain, repository: Repository, write: &Write, why: &[u8]) -> Box<[u8]> {
    crate::effect_words::write_account(
        write,
        repository,
        d.repository(repository),
        &bytes::concat(&[b"not sent: ", why]),
    )
}

pub(super) fn op_bound(max: u32, measured: Option<u64>) -> Box<[u8]> {
    let max = u64::from(max);
    let excess = match measured {
        Some(size) if size > max => Excess::exact(Bound::OpBytes, max, size),
        Some(_) | None => Excess::unnamed(),
    };
    bytes::concat(&[b"past ", &crate::bound_words(excess)])
}
