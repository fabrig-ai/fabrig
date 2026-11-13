//! The asking family's document call (temper/docs/design/domain/agent/ask-temper.md, 3).
mod sections;
pub(crate) use sections::{heading, made};

use super::{CallKey, Decision, Delivery, Domain, Env, Limits, Request, Work, emit};
use crate::boundary::Read;
use crate::{
    CallAnswer, CoreRange, DocumentAction, DocumentOutcome, DocumentRefusal, Documented, Key, Range, Record, loads,
};
use alloc::boxed::Box;
use jig_domain_core::{self as core, NamedAction, ToolKind};
use jig_domain_notes as notes;
use skein_lib::{Id, List, Queue, ReplyTo, Token};

/// The note a document call leaves: its description and whole body.
pub(crate) struct Made {
    pub description: Box<[u8]>,
    pub body: Box<[u8]>,
}

/// A document call between its scope index and entry loads.
#[derive(Hash, Debug)]
pub(crate) struct DocumentRead {
    to: Token,
    key: CallKey,
    action: DocumentAction,
    body: Box<[u8]>,
    scope: notes::Scope,
    phase: Phase,
}

#[derive(Hash, Debug)]
enum Phase {
    Index { after: Option<u64> },
    Entry { name: u64 },
}

/// Start after the asking family has authorized the call.
pub(crate) fn call(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    to: ReplyTo,
    key: CallKey,
    action: DocumentAction,
    body: Box<[u8]>,
) {
    let right = to.into_token();
    let Some(context) = domain.core.tasks.delegation(key.task) else {
        refused(domain, right, action, &body, DocumentRefusal::Unread);
        return;
    };
    let scope = notes::Scope::Goal { project: context.project, goal: key.task };
    let mut scopes = List::with_capacity(1);
    scopes.push(scope.clone()).expect("one document scope");
    for (family, kind) in [
        (crate::NoteFamily::Recall, ToolKind::Recall { by: notes::Recall::Search { scopes, query: Box::new([]) } }),
        (crate::NoteFamily::Note, ToolKind::Note { scope: scope.clone() }),
    ] {
        if let Some(part) = domain.core.authorize_tool(key, kind) {
            let flight = domain.call_renders.get_mut(&right).expect("document flight");
            let super::host_route::FlightRender::Document(document) = &mut flight.render else {
                unreachable!("document context")
            };
            document.checked = Some(family);
            domain.work.push(Work::Core(core::Event::NamedAnswer { to: ReplyTo::new(right), key, part }));
            return;
        }
    }
    let range = Range::Core(CoreRange::Notes(notes::Range::Lines { scope: scope.clone(), after: None }));
    let read = DocumentRead { to: right, key, action, body, scope, phase: Phase::Index { after: None } };
    match domain.archive_reads.insert(Some(Read::Document(Box::new(read)))) {
        Ok(id) => emit(decision, &env.limits, Delivery::Load { waiter: id.token(), range, after: None }),
        Err(Some(Read::Document(read))) => refused(domain, right, action, &read.body, DocumentRefusal::Busy),
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
                | Read::Remember(_),
            )
            | None,
        ) => unreachable!("document read insertion"),
    }
}

fn refused(domain: &mut Domain, to: Token, action: DocumentAction, body: &[u8], why: DocumentRefusal) {
    if !domain.call_renders.contains_key(&to) {
        return;
    }
    super::host_route::read(
        domain,
        ReplyTo::new(to),
        CallAnswer::Document(Box::new(Documented {
            action,
            heading: heading(action, body),
            outcome: DocumentOutcome::Refused(why),
        })),
    );
}

/// A terminal store load failure changes nothing and keeps no call record.
pub(crate) fn failed(domain: &mut Domain, waiter: Token) {
    let Some(Read::Document(read)) = crate::archive::take_read(domain, waiter) else { return };
    domain.archive_reads.retire(Id::from_token(waiter));
    refused(domain, read.to, read.action, &read.body, DocumentRefusal::Unread);
}

/// Consume a page, or load the next index page or the chosen entry.
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
    let Some(Read::Document(mut read)) = crate::archive::take_read(domain, waiter) else { return };
    if !domain.call_renders.contains_key(&read.to) {
        domain.archive_reads.retire(Id::from_token(waiter));
        return;
    }
    let result = match read.phase {
        Phase::Index { after } => index(domain, &mut read, rows, next, after),
        Phase::Entry { name } => entry(domain, &read, rows, next, name),
    };
    match result {
        Ok(Some(range)) => {
            *domain.archive_reads.get_mut(Id::from_token(waiter)).expect("document load slot") =
                Some(Read::Document(read));
            crate::restart::request_load(domain, waiter, range, None, out);
        }
        Ok(None) => {
            domain.archive_reads.retire(Id::from_token(waiter));
        }
        Err(why) => {
            domain.archive_reads.retire(Id::from_token(waiter));
            refused(domain, read.to, read.action, &read.body, why);
        }
    }
}

fn index(
    domain: &mut Domain,
    read: &mut DocumentRead,
    rows: Box<[Record]>,
    next: Option<Key>,
    after: Option<u64>,
) -> Result<Option<Range>, DocumentRefusal> {
    let mut last = after;
    let mut chosen = None;
    for row in rows {
        let Record::Core(core::Record::Notes(notes::Record::Line(line))) = row else {
            return Err(DocumentRefusal::Unread);
        };
        if !notes::valid_line(&line, &domain.limits.notes)
            || line.scope != read.scope
            || !domain.core.note_readable(read.key.task, &line.scope)
            || Some(line.name) <= last
        {
            return Err(DocumentRefusal::Unread);
        }
        last = Some(line.name);
        if chosen.is_none() && !line.description.starts_with(b"work:") {
            chosen = Some(line.name);
        }
    }
    if let Some(name) = chosen {
        read.phase = Phase::Entry { name };
        return Ok(Some(Range::Core(CoreRange::Notes(notes::Range::Entry { name }))));
    }
    if next.is_some() {
        if last == after {
            return Err(DocumentRefusal::Unread);
        }
        read.phase = Phase::Index { after: last };
        return Ok(Some(Range::Core(CoreRange::Notes(notes::Range::Lines { scope: read.scope.clone(), after: last }))));
    }
    write(domain, read, None)?;
    Ok(None)
}

fn entry(
    domain: &mut Domain,
    read: &DocumentRead,
    rows: Box<[Record]>,
    next: Option<Key>,
    name: u64,
) -> Result<Option<Range>, DocumentRefusal> {
    if next.is_some() || rows.len() != 1 {
        return Err(DocumentRefusal::Unread);
    }
    let Some(Record::Core(core::Record::Notes(notes::Record::Entry(entry)))) = rows.into_iter().next() else {
        return Err(DocumentRefusal::Unread);
    };
    if entry.name != name
        || entry.scope != read.scope
        || entry.description.starts_with(b"work:")
        || !notes::valid_entry(&entry, &domain.limits.notes)
        || !domain.core.note_readable(read.key.task, &entry.scope)
    {
        return Err(DocumentRefusal::Unread);
    }
    write(domain, read, Some(entry))?;
    Ok(None)
}

fn write(domain: &mut Domain, read: &DocumentRead, entry: Option<notes::Entry>) -> Result<(), DocumentRefusal> {
    let previous = match &entry {
        Some(entry) => Some((entry.description.as_ref(), entry.body.as_ref())),
        None => None,
    };
    let made = made(previous, read.action, &read.body, &domain.limits.notes)?;
    let (name, recalled, references) = match entry {
        Some(entry) => (entry.name, Some(entry.revision), entry.references),
        None => (0, None, List::with_capacity(0)),
    };
    domain.work.push(Work::Core(core::Event::NamedAction {
        to: ReplyTo::new(read.to),
        key: read.key,
        action: NamedAction::Note {
            entry: notes::New {
                name,
                scope: read.scope.clone(),
                description: made.description,
                body: made.body,
                references,
                author: notes::Author::Task { task: 0, attempt: 0 },
            },
            recalled,
        },
    }));
    Ok(())
}
