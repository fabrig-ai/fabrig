//! The asking family's memory call (temper/docs/design/domain/agent/ask-temper.md, 4).
mod flight;
pub(crate) use flight::RememberFlight;

use super::{CallKey, Decision, Delivery, Domain, Env, Limits, Request, Work, emit};
use crate::boundary::Read;
use crate::{CallAnswer, CoreRange, Key, Range, Record, RememberAction, RememberRefusal, Remembered, loads};
use alloc::boxed::Box;
use jig_domain_core::{self as core, NamedAction};
use jig_domain_notes as notes;
use skein_lib::{Id, List, Queue, ReplyTo, Token};

/// A remember call loading its person's whole index and, for a change, its entry.
#[derive(Hash, Debug)]
pub(crate) struct RememberRead {
    to: Token,
    key: CallKey,
    action: RememberAction,
    scope: notes::Scope,
    lines: List<notes::Line>,
    phase: Phase,
}

#[derive(Hash, Debug)]
enum Phase {
    Index { after: Option<u64> },
    Entry { place: u32 },
}

pub(crate) fn call(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    to: ReplyTo,
    key: CallKey,
    action: RememberAction,
) {
    let right = to.into_token();
    let Some(context) = domain.core.tasks.delegation(key.task) else {
        refused(domain, right, RememberRefusal::Unread);
        return;
    };
    let party = match context.requester {
        jig_domain_tasks::Party::Person(party) => party,
        jig_domain_tasks::Party::Task(_) | jig_domain_tasks::Party::Deployment { .. } => {
            refused(domain, right, RememberRefusal::NoPerson);
            return;
        }
    };
    let scope = notes::Scope::Party { party };
    let range = Range::Core(CoreRange::Notes(notes::Range::Lines { scope: scope.clone(), after: None }));
    let read = RememberRead {
        to: right,
        key,
        action,
        scope,
        lines: List::with_capacity(env.limits.notes.party_entries),
        phase: Phase::Index { after: None },
    };
    match domain.archive_reads.insert(Some(Read::Remember(Box::new(read)))) {
        Ok(id) => emit(decision, &env.limits, Delivery::Load { waiter: id.token(), range, after: None }),
        Err(Some(Read::Remember(_))) => refused(domain, right, RememberRefusal::Busy),
        Err(
            Some(
                Read::Escalation(_)
                | Read::Proposal(_)
                | Read::TaskProposal { .. }
                | Read::Ended(_)
                | Read::Transcript { .. }
                | Read::InputCheck(_)
                | Read::GoalWords(_)
                | Read::History { .. }
                | Read::Notes { .. }
                | Read::SettingsNote { .. }
                | Read::Document(_),
            )
            | None,
        ) => unreachable!("remember read insertion"),
    }
}

fn refused(domain: &mut Domain, to: Token, why: RememberRefusal) {
    if domain.call_renders.contains_key(&to) {
        super::host_route::read(domain, ReplyTo::new(to), CallAnswer::Remember(Box::new(Remembered::Refused(why))));
    }
}

/// A terminal load failure keeps no call record.
pub(crate) fn failed(domain: &mut Domain, waiter: Token) {
    let Some(Read::Remember(read)) = crate::archive::take_read(domain, waiter) else { return };
    domain.archive_reads.retire(Id::from_token(waiter));
    refused(domain, read.to, RememberRefusal::Unread);
}

/// Join every index page before checking and writing the call.
pub(crate) fn loaded(
    domain: &mut Domain,
    waiter: Token,
    rows: Box<[Record]>,
    next: Option<Key>,
    cut: Option<loads::Cut>,
    out: &mut Queue<Request>,
) {
    if cut.is_some() {
        failed(domain, waiter);
        return;
    }
    let Some(Read::Remember(mut read)) = crate::archive::take_read(domain, waiter) else { return };
    if !domain.call_renders.contains_key(&read.to) {
        domain.archive_reads.retire(Id::from_token(waiter));
        return;
    }
    let result = match read.phase {
        Phase::Index { after } => index(domain, &mut read, rows, next, after),
        Phase::Entry { place } => entry(domain, &read, rows, next, place),
    };
    match result {
        Ok(Some(range)) => {
            *domain.archive_reads.get_mut(Id::from_token(waiter)).expect("remember load slot") =
                Some(Read::Remember(read));
            crate::restart::request_load(domain, waiter, range, None, out);
        }
        Ok(None) => domain.archive_reads.retire(Id::from_token(waiter)),
        Err(why) => {
            domain.archive_reads.retire(Id::from_token(waiter));
            refused(domain, read.to, why);
        }
    }
}

fn index(
    domain: &mut Domain,
    read: &mut RememberRead,
    rows: Box<[Record]>,
    next: Option<Key>,
    after: Option<u64>,
) -> Result<Option<Range>, RememberRefusal> {
    let mut last = after;
    for row in rows {
        let Record::Core(core::Record::Notes(notes::Record::Line(line))) = row else {
            return Err(RememberRefusal::Unread);
        };
        if !notes::valid_line(&line, &domain.limits.notes)
            || line.scope != read.scope
            || Some(line.name) <= last
            || line.description.len() > skein_lib::usize_of(domain.limits.notes.party_bytes)
        {
            return Err(RememberRefusal::Unread);
        }
        last = Some(line.name);
        if read.lines.push(line).is_err() {
            return Err(RememberRefusal::Unread);
        }
    }
    if next.is_some() {
        if last == after {
            return Err(RememberRefusal::Unread);
        }
        read.phase = Phase::Index { after: last };
        return Ok(Some(Range::Core(CoreRange::Notes(notes::Range::Lines { scope: read.scope.clone(), after: last }))));
    }
    check(read, &domain.limits.notes)?;
    match read.action {
        RememberAction::Change { line, .. } => {
            let place = line.checked_sub(1).expect("checked place");
            let name = read.lines.get(place).expect("checked line").name;
            read.phase = Phase::Entry { place };
            Ok(Some(Range::Core(CoreRange::Notes(notes::Range::Entry { name }))))
        }
        RememberAction::Add { .. } | RememberAction::Drop { .. } => {
            write(domain, read, None);
            Ok(None)
        }
    }
}

fn check(read: &mut RememberRead, limits: &notes::Limits) -> Result<(), RememberRefusal> {
    match &mut read.action {
        RememberAction::Add { text } | RememberAction::Change { text, .. } => {
            let trimmed = text.trim_ascii();
            if trimmed.is_empty() || text.contains(&b'\n') || text.contains(&b'\r') {
                return Err(RememberRefusal::NotOneLine);
            }
            if trimmed.len() > skein_lib::usize_of(limits.party_bytes) {
                return Err(RememberRefusal::TooLong {
                    bytes: u32::try_from(trimmed.len()).unwrap_or(u32::MAX),
                    bound: limits.party_bytes,
                });
            }
            *text = trimmed.into();
        }
        RememberAction::Drop { .. } => {}
    }
    match read.action {
        RememberAction::Add { .. } => {
            if read.lines.len() >= limits.party_entries {
                return Err(RememberRefusal::Full { bound: limits.party_entries });
            }
        }
        RememberAction::Change { line, .. } | RememberAction::Drop { line } => {
            if line == 0 || line > read.lines.len() {
                return Err(RememberRefusal::NoLine { line, lines: read.lines.len() });
            }
        }
    }
    Ok(())
}

fn entry(
    domain: &mut Domain,
    read: &RememberRead,
    rows: Box<[Record]>,
    next: Option<Key>,
    place: u32,
) -> Result<Option<Range>, RememberRefusal> {
    if next.is_some() || rows.len() != 1 {
        return Err(RememberRefusal::Unread);
    }
    let Some(Record::Core(core::Record::Notes(notes::Record::Entry(entry)))) = rows.into_iter().next() else {
        return Err(RememberRefusal::Unread);
    };
    let line = read.lines.get(place).expect("checked line");
    if entry.name != line.name
        || entry.scope != read.scope
        || !entry.body.is_empty()
        || !notes::valid_entry(&entry, &domain.limits.notes)
    {
        return Err(RememberRefusal::Unread);
    }
    write(domain, read, Some(entry));
    Ok(None)
}

fn write(domain: &mut Domain, read: &RememberRead, entry: Option<notes::Entry>) {
    let (action, line) = match &read.action {
        RememberAction::Add { text } => {
            let mut references = List::with_capacity(1);
            references.push(read.key.task).expect("one chat reference");
            (
                NamedAction::Note {
                    entry: notes::New {
                        name: 0,
                        scope: read.scope.clone(),
                        description: text.clone(),
                        body: Box::new([]),
                        references,
                        author: notes::Author::Task { task: 0, attempt: 0 },
                    },
                    recalled: None,
                },
                read.lines.len().checked_add(1).expect("admitted list"),
            )
        }
        RememberAction::Change { line, text } => {
            let entry = entry.expect("loaded change entry");
            let recalled = read.lines.get(line.checked_sub(1).expect("checked place")).expect("checked line").revision;
            (
                NamedAction::Note {
                    entry: notes::New {
                        name: entry.name,
                        scope: read.scope.clone(),
                        description: text.clone(),
                        body: Box::new([]),
                        references: entry.references,
                        author: notes::Author::Task { task: 0, attempt: 0 },
                    },
                    recalled: Some(recalled),
                },
                *line,
            )
        }
        RememberAction::Drop { line } => {
            let entry = read.lines.get(line.checked_sub(1).expect("checked place")).expect("checked line");
            (NamedAction::NoteDelete { scope: read.scope.clone(), name: entry.name, recalled: entry.revision }, *line)
        }
    };
    let flight = domain.call_renders.get_mut(&read.to).expect("remember flight");
    flight.render = super::host_route::FlightRender::Remember(Box::new(RememberFlight::new(
        &read.action,
        line,
        &read.lines,
        domain.limits.notes.party_entries,
    )));
    domain.work.push(Work::Core(core::Event::NamedAction { to: ReplyTo::new(read.to), key: read.key, action }));
}
