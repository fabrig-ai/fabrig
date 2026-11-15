//! Failed prepare evidence (domain/worker/checkouts.md, 1.1 and 1.2).

use alloc::boxed::Box;
use skein_lib::{Decimal, Time, Writer};

use super::{Holding, PrepareStep, Timing, candidate, hex_length, nth, put_hex};
use crate::boundary::Start;
use crate::git::Done;

// domain/worker/checkouts.md, 1.2: fields keep their room; words take the rest.
const DETAIL_BYTES: usize = 2048;
const NAME_BYTES: usize = 256;

#[expect(clippy::too_many_lines, reason = "each preparation stage and its repository stay in one evidence table")]
pub(super) fn preparation_detail(
    holding: &Holding,
    step: PrepareStep,
    done: &Done,
    timing: Timing,
    now: Time,
) -> Box<[u8]> {
    let (fault, command, output, conflicted) = match done {
        Done::Failed { fault } => (fault.name(), None, b"".as_slice(), false),
        Done::FailedWithOutput { fault, diagnostic } => {
            (fault.name(), diagnostic.command(), diagnostic.output(), false)
        }
        Done::Conflicted { .. } => (b"Conflicted".as_slice(), None, b"".as_slice(), true),
        Done::Succeeded
        | Done::Fetched { .. }
        | Done::Committed { .. }
        | Done::Unchanged
        | Done::Exists
        | Done::Rejected
        | Done::Results { .. }
        | Done::Paths { .. }
        | Done::Looked { .. }
        | Done::Brings { .. }
        | Done::IsAncestor { .. }
        | Done::Merged => return Box::new([]),
    };
    let mut conflict_head = None;
    let (stage, repository): (&[u8], Option<u32>) = match step {
        PrepareStep::Candidate { repository, step } => (
            match step {
                candidate::Step::Head => b"fetch member",
                candidate::Step::Upstream => b"fetch upstream",
                candidate::Step::Base => b"fetch round base",
                candidate::Step::Checkout => b"check out start",
                candidate::Step::Advance | candidate::Step::Restore => b"check out prefix",
                candidate::Step::MainCheckout => b"check out round base",
                candidate::Step::MainMerge => b"merge onto round base",
                candidate::Step::Merge => b"merge member",
                candidate::Step::Brings => b"compare upstream",
                candidate::Step::Commit { .. } => b"commit candidate",
                candidate::Step::Existing => b"fetch earlier prefix",
                candidate::Step::SameTree => b"compare earlier prefix tree",
                candidate::Step::Parent | candidate::Step::Member => b"check earlier prefix ancestry",
            },
            Some(repository),
        ),
        PrepareStep::Reds { .. } => return Box::new([]),
        PrepareStep::Store { repository, .. } => (b"store", Some(repository)),
        PrepareStep::Look { repository, .. } => (b"look", Some(repository)),
        PrepareStep::Make => (b"make", None),
        PrepareStep::Clone { repository, .. } => (b"clone", Some(repository)),
        PrepareStep::Fetch { repository, .. } => (b"fetch", Some(repository)),
        PrepareStep::Default { repository, .. } => (b"fetch default", Some(repository)),
        PrepareStep::Create { repository, .. } => (b"create base", Some(repository)),
        PrepareStep::Refetch { repository, .. } => (b"refetch base", Some(repository)),
        PrepareStep::CheckOut { repository, .. } => (b"check out", Some(repository)),
        PrepareStep::FetchPush { repository, .. } => (b"fetch push", Some(repository)),
        PrepareStep::FetchLanding { repository, .. } => (b"fetch landing", Some(repository)),
        PrepareStep::PointLanding { repository, .. } | PrepareStep::PinReplaced { repository, .. } => {
            (b"point landing", Some(repository))
        }
        PrepareStep::FetchBase { repository, .. } => (b"fetch base", Some(repository)),
        PrepareStep::PointBase { repository, .. } => (b"point base", Some(repository)),
        PrepareStep::CheckOutMerge { repository, .. } => (b"check out merge", Some(repository)),
        PrepareStep::CompareBase { repository, .. } => (b"compare base", Some(repository)),
        PrepareStep::CompareHead { repository, .. } => (b"compare head", Some(repository)),
        PrepareStep::Merge { repository, theirs, head: Some(_) } if conflicted => {
            conflict_head = Some((repository, theirs));
            (b"merge", Some(repository))
        }
        PrepareStep::Merge { repository, .. } => (b"merge", Some(repository)),
        PrepareStep::CommitHead { repository, .. } => (b"commit head", Some(repository)),
        PrepareStep::CheckOutHead { repository, .. } => (b"check out head", Some(repository)),
        PrepareStep::CommitMerge { repository, .. } => (b"commit merge", Some(repository)),
        PrepareStep::CheckOutMerged { repository, .. } => (b"check out merged", Some(repository)),
    };
    let workspace = Decimal::of(holding.workspace.token().raw());
    let place = Decimal::of(u64::from(repository.unwrap_or(0)));
    let (place, space, name) = match repository {
        Some(repository) => {
            let name = &nth(holding, repository).name;
            (place.as_bytes(), b" ".as_slice(), name.get(..name.len().min(NAME_BYTES)).expect("bounded name"))
        }
        None => (b"none".as_slice(), b"".as_slice(), b"".as_slice()),
    };
    let operation = Decimal::of(now.saturating_since(timing.started).as_millis());
    let deadline = Decimal::of(timing.timeout.as_millis());
    let prepare = Decimal::of(now.saturating_since(holding.arrived).as_millis());
    let queued = Decimal::of(holding.admitted.saturating_since(holding.arrived).as_millis());
    let fields: &[&[u8]] = &[
        stage,
        b": ",
        fault,
        b" (workspace ",
        workspace.as_bytes(),
        b", repository ",
        place,
        space,
        name,
        b", command ",
        command.unwrap_or(b"none"),
        b", operation ",
        operation.as_bytes(),
        b" ms of ",
        deadline.as_bytes(),
        b" ms, prepare ",
        prepare.as_bytes(),
        b" ms, queued ",
        queued.as_bytes(),
        b" ms)",
    ];
    if let Some((repository, theirs)) = conflict_head {
        let branch = match &nth(holding, repository).start {
            Start::Merged { branch, .. } => branch,
            Start::Base { .. }
            | Start::Branch { .. }
            | Start::Commit { .. }
            | Start::Saved { .. }
            | Start::Merge { .. }
            | Start::Prepare { .. } => unreachable!("only a merged start refuses a head's conflict"),
        };
        let mut hex = Writer::new(hex_length(theirs));
        put_hex(&mut hex, theirs);
        let hex = hex.finish();
        return write_detail(fields, &[b"head ", &hex, b" does not merge into ", branch]);
    }
    write_detail(fields, &[output])
}

/// Measure fields first and keep the beginning of words in the room left.
fn write_detail(fields: &[&[u8]], words: &[&[u8]]) -> Box<[u8]> {
    let mut size = 0_usize;
    for field in fields {
        size = size.checked_add(field.len()).expect("bounded fields");
    }
    let mut word_bytes = 0_usize;
    for word in words {
        word_bytes = word_bytes.saturating_add(word.len());
    }
    let separator = if word_bytes == 0 { b"".as_slice() } else { b": ".as_slice() };
    size = size.checked_add(separator.len()).expect("bounded fields and separator");
    let room = DETAIL_BYTES.checked_sub(size).expect("fixed fields and bounded names fit");
    let kept = word_bytes.min(room);
    let mut text = Writer::new(size.checked_add(kept).expect("bounded detail"));
    for field in fields {
        text.put(field).expect("measured field");
    }
    text.put(separator).expect("measured separator");
    let mut remaining = kept;
    for word in words {
        let count = word.len().min(remaining);
        text.put(word.get(..count).expect("bounded words")).expect("measured words");
        remaining = remaining.checked_sub(count).expect("within remaining room");
    }
    text.finish()
}
