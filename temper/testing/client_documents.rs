//! Owner document fixtures for Temper's client consumers.
//! See temper/docs/design/client/README.md, 10.
use jig_client::contract as c;
use skein_lib::{List, Wall};

pub fn entry(
    object: c::ObjectKey,
    project: u32,
    position: u64,
    at: Wall,
    reply: Option<Box<c::TaskLine>>,
) -> c::InboxEntry {
    let (number, object) = match object {
        c::ObjectKey::Question { task, message } => (task, c::InboxObject::Question { message }),
        c::ObjectKey::Proposal(key) => {
            let number = match key.proposer {
                c::Proposer::Task(n) | c::Proposer::Party(n) => n,
            };
            (number, c::InboxObject::Proposal { proposer: key.proposer, number: key.number })
        }
        c::ObjectKey::Escalation(task) => (task, c::InboxObject::Escalation { revision: position }),
        c::ObjectKey::PersonTask(task) => (task, c::InboxObject::PersonTask),
        c::ObjectKey::Result(task) => (task, c::InboxObject::Result { position }),
        c::ObjectKey::Task(task) => (task, c::InboxObject::Reply { message: position }),
        c::ObjectKey::Note { .. } => unreachable!("notes do not enter the inbox"),
    };
    c::InboxEntry {
        task: c::TaskLink::Stored { number, status: None },
        object,
        project: Some(project),
        activity: Some(position),
        at: Some(at),
        unread: None,
        cursor: c::InboxCursor::Inbox { task: number, object, at: Some(at) },
        reply,
    }
}

pub fn inbox(
    entries: List<c::InboxEntry>,
    cached_needs: u32,
    cached_updates: c::Count,
    results_seen: u64,
    more: bool,
) -> c::Inbox {
    let mut needs = List::with_capacity(entries.len());
    let mut updates = List::with_capacity(entries.len());
    for row in entries.into_boxed() {
        match c::InboxReadGroup::of(row.group()) {
            c::InboxReadGroup::Needs => needs.push(row).expect("bounded needs"),
            c::InboxReadGroup::Updates => updates.push(row).expect("bounded updates"),
        }
    }
    c::Inbox {
        party: 1,
        needs,
        updates,
        next: if more { Some(c::InboxCursor::Number { number: 1 }) } else { None },
        cached_needs,
        cached_updates,
        results_seen,
        updates_time_complete: true,
        selection: None,
    }
}

pub fn line(position: u64, at: Wall, event: c::Activity) -> c::DecodedActivityLine {
    c::DecodedActivityLine { source: Box::new(c::ActivityLine { task: 9, position, at, line: event }), decoded: None }
}

pub fn turn(position: u64, at: Wall, name: c::TurnName, turn: c::TurnRead) -> c::DecodedActivityLine {
    let read = match &turn {
        c::TurnRead::Decoded(value) => value.read_through,
        c::TurnRead::Undecodable { .. } => None,
    };
    c::DecodedActivityLine {
        source: Box::new(c::ActivityLine {
            task: 9,
            position,
            at,
            line: c::Activity::Turn { reply: position, attempt: name.attempt, turn: name.number, read },
        }),
        decoded: Some(c::ArchivedTurn { task: 9, name, cumulative: position, read, at, turn }),
    }
}

pub fn result(position: u64, at: Wall, result: c::Result) -> c::DecodedActivityLine {
    let task = result.task.task;
    let words =
        if let Some(c::ResultValue::Report(words)) = result.value { words.bytes } else { result.task.title.bytes };
    line(
        position,
        at,
        c::Activity::Heard(c::Word {
            number: position,
            from: c::Requester::Task(task),
            kind: c::MessageKind::Result(c::WordResult::Report),
            words,
            at,
            hits: 1,
            eligible: true,
        }),
    )
}
