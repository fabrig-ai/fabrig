//! Loads of durable rows outside the working set: what a run's preparation
//! reads (its transcript and inputs), notes, and
//! the core's historical queries (jig's `domain/engine/executors-transcripts.md`, 1).
//! Window pages carry whole or cut turns (temper/docs/design/domain/root.md, 7).

use crate::boundary::{Read, Request, Work};
use crate::domain::Domain;
use crate::restart::request_load;
use crate::{Key, Range, Record, loads};
use alloc::boxed::Box;
use jig_domain_notes as notes;
use jig_domain_tasks as tasks;
use skein_lib::{Id, List, Queue, ReplyTo, Token};

/// What a live load waiter reads, if it still reads anything.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Waiting {
    ChangeDelegate,
    History,
    ChangeFrom,
    GoalWords,
    Notes,
    Document,
    CloserReport,
    Remember,
    Parts,
}

pub(crate) fn waiting(domain: &Domain, waiter: Token) -> Option<Waiting> {
    match domain.archive_reads.get(Id::from_token(waiter))? {
        Some(Read::ChangeDelegate { .. }) => Some(Waiting::ChangeDelegate),
        Some(Read::History { .. }) => Some(Waiting::History),
        Some(Read::ChangeFrom { .. }) => Some(Waiting::ChangeFrom),
        Some(Read::GoalWords(_)) => Some(Waiting::GoalWords),
        Some(Read::Notes { .. } | Read::SettingsNote { .. }) => Some(Waiting::Notes),
        Some(Read::Document(_)) => Some(Waiting::Document),
        Some(Read::CloserReport(_)) => Some(Waiting::CloserReport),
        Some(Read::Parts(_)) => Some(Waiting::Parts),
        Some(Read::Remember(_)) => Some(Waiting::Remember),
        None => None,
    }
}

/// Free a reserved read slot the core no longer needs.
pub(crate) fn retire_read(domain: &mut Domain, waiter: Token) {
    if take_read(domain, waiter).is_some() {
        domain.archive_reads.retire(Id::from_token(waiter));
    }
}

pub(crate) fn take_note_owner(domain: &mut Domain, waiter: Token) -> Option<Token> {
    match take_read(domain, waiter) {
        Some(Read::Notes { owner }) => Some(owner),
        Some(
            Read::ChangeDelegate { .. }
            | Read::History { .. }
            | Read::Parts(_)
            | Read::CloserReport(_)
            | Read::Document(_)
            | Read::Remember(_)
            | Read::SettingsNote { .. }
            | Read::ChangeFrom { .. }
            | Read::GoalWords(_),
        )
        | None => None,
    }
}

pub(crate) fn note_failed(domain: &mut Domain, waiter: Token) {
    if waiting(domain, waiter) == Some(Waiting::CloserReport) {
        crate::route::forge_route::goal_report::failed(domain, waiter);
        return;
    }
    if waiting(domain, waiter) == Some(Waiting::Remember) {
        crate::route::remember::failed(domain, waiter);
        return;
    }
    if let Some(Some(Read::SettingsNote { to, .. })) = domain.archive_reads.get(Id::from_token(waiter)) {
        let to = *to;
        retire_read(domain, waiter);
        crate::route::host_route::note_refused(domain, ReplyTo::new(to), b"settings note could not be loaded");
        return;
    }
    let Some(owner) = take_note_owner(domain, waiter) else { return };
    domain.archive_reads.retire(Id::from_token(waiter));
    domain.work.push(Work::Core(jig_domain_core::Event::Notes(jig_domain_notes::Event::LoadFailed { owner })));
}

pub(crate) fn note_loaded(
    domain: &mut Domain,
    waiter: Token,
    rows: Box<[Record]>,
    next: Option<Key>,
    cut: Option<loads::Cut>,
) {
    if let Some(Some(Read::SettingsNote { .. })) = domain.archive_reads.get(Id::from_token(waiter)) {
        let Some(Read::SettingsNote { to, call }) = take_read(domain, waiter) else { unreachable!("note waiter") };
        domain.archive_reads.retire(Id::from_token(waiter));
        crate::route::host_route::settings_note_loaded(
            domain,
            ReplyTo::new(to),
            *call,
            rows,
            next.is_some() || cut.is_some(),
        );
        return;
    }
    ordinary_note_loaded(domain, waiter, rows, next, cut);
}

fn ordinary_note_loaded(
    domain: &mut Domain,
    waiter: Token,
    rows: Box<[Record]>,
    next: Option<Key>,
    cut: Option<loads::Cut>,
) {
    if cut.is_some() {
        note_failed(domain, waiter);
        return;
    }
    let mut kept = List::with_capacity(u32::try_from(rows.len()).expect("bounded note page"));
    for row in rows {
        match row {
            Record::Core(jig_domain_core::Record::Notes(record)) => {
                let valid = match &record {
                    notes::Record::Entry(entry) => notes::valid_entry(entry, &domain.limits.notes),
                    notes::Record::Line(line) => notes::valid_line(line, &domain.limits.notes),
                };
                if !valid {
                    note_failed(domain, waiter);
                    return;
                }
                kept.push(record).expect("note page room");
            }
            Record::Core(
                jig_domain_core::Record::Core(
                    jig_domain_core::CoreRecord::Settings(_)
                    | jig_domain_core::CoreRecord::Part(_)
                    | jig_domain_core::CoreRecord::Call(_)
                    | jig_domain_core::CoreRecord::Deployment(_)
                    | jig_domain_core::CoreRecord::EscalationDecision(_)
                    | jig_domain_core::CoreRecord::ProposalDecision(_)
                    | jig_domain_core::CoreRecord::Turn(_)
                    | jig_domain_core::CoreRecord::RunProof(_)
                    | jig_domain_core::CoreRecord::Terminal(_)
                    | jig_domain_core::CoreRecord::Projection(_),
                )
                | jig_domain_core::Record::Tasks(_)
                | jig_domain_core::Record::People(_)
                | jig_domain_core::Record::Accounts(_),
            )
            | Record::Forge { .. } => {
                note_failed(domain, waiter);
                return;
            }
        }
    }
    let Some(owner) = take_note_owner(domain, waiter) else { return };
    domain.archive_reads.retire(Id::from_token(waiter));
    domain.work.push(Work::Core(jig_domain_core::Event::Notes(jig_domain_notes::Event::Loaded {
        owner,
        rows: jig_domain_notes::Rows { records: kept },
        more: next.is_some(),
    })));
}

/// A provenance lookup is temper's own read, not an engine store load.
pub(crate) fn change_from_loaded(domain: &mut Domain, waiter: Token, rows: Option<Box<[Record]>>) {
    let Some(Read::ChangeFrom { task }) = take_read(domain, waiter) else { return };
    domain.archive_reads.retire(Id::from_token(waiter));
    let mut row = None;
    if let Some(rows) = rows
        && rows.len() == 1
        && let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(ended)))) = rows.into_iter().next()
    {
        row = Some(ended);
    }
    domain.work.push(Work::ChangeFromLoaded { task, row });
}

pub(crate) fn take_read(domain: &mut Domain, waiter: Token) -> Option<Read> {
    let entry = domain.archive_reads.get_mut(Id::from_token(waiter))?;
    entry.take()
}

pub(crate) fn goal_failed(domain: &mut Domain, waiter: Token) {
    let Some(Read::GoalWords(read)) = take_read(domain, waiter) else { return };
    domain.archive_reads.retire(Id::from_token(waiter));
    domain.work.push(Work::Core(jig_domain_core::Event::NamedAnswer {
        to: ReplyTo::new(read.to),
        key: read.key,
        part: jig_domain_core::CallPart::DelegationRefused(tasks::Problem::new(None, tasks::Refusal::Read)),
    }));
}

pub(crate) fn goal_loaded(domain: &mut Domain, waiter: Token, rows: Box<[Record]>, out: &mut Queue<Request>) {
    let Some(Some(Read::GoalWords(read))) = domain.archive_reads.get_mut(Id::from_token(waiter)) else { return };
    if rows.len() != 1 {
        goal_failed(domain, waiter);
        return;
    }
    let row = rows.into_iter().next().expect("one requested activity line");
    let Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Activity(line))) = row else {
        goal_failed(domain, waiter);
        return;
    };
    let mut found = false;
    match line.line {
        tasks::Line::Words { by, words, .. } => {
            if by == tasks::Party::Person(read.person) {
                read.person_words = words;
                found = true;
            }
        }
        tasks::Line::Heard(_)
        | tasks::Line::Notified { .. }
        | tasks::Line::RunBegan { .. }
        | tasks::Line::RunEnded { .. }
        | tasks::Line::Turn { .. }
        | tasks::Line::Decided { .. }
        | tasks::Line::Milestone { .. }
        | tasks::Line::NowWritten { .. }
        | tasks::Line::Stood { .. }
        | tasks::Line::TalkOpened { .. }
        | tasks::Line::TalkEnded { .. }
        | tasks::Line::SessionEnded { .. }
        | tasks::Line::Kept { .. } => {}
    }
    read.position = read.position.checked_sub(1).expect("positive requested activity position");
    if found || read.position == 0 {
        let Some(Read::GoalWords(read)) = take_read(domain, waiter) else { unreachable!("completed goal words") };
        domain.archive_reads.retire(Id::from_token(waiter));
        domain.work.push(Work::AgreedGoal(read));
    } else {
        let range = Range::Core(crate::CoreRange::RequesterWord { task: read.key.task, position: read.position });
        request_load(domain, waiter, range, None, out);
    }
}

pub(crate) fn history_failed(domain: &mut Domain, waiter: Token) {
    let Some(Read::History { load, .. }) = take_read(domain, waiter) else { return };
    domain.archive_reads.retire(Id::from_token(waiter));
    domain.work.push(Work::Forge(temper_engine_domain_forge::Event::HistoryUnloaded { load }));
}

pub(crate) fn history_loaded(
    domain: &mut Domain,
    waiter: Token,
    rows: Box<[Record]>,
    next: Option<Key>,
    cut: Option<loads::Cut>,
) {
    if cut.is_some() {
        history_failed(domain, waiter);
        return;
    }
    let mut stored = List::with_capacity(u32::try_from(rows.len()).expect("bounded history page"));
    for row in rows {
        let Record::Forge { row: record } = row else {
            history_failed(domain, waiter);
            return;
        };
        stored.push(*record).expect("one stored row per page row");
    }
    let next = match next {
        Some(Key::Forge(key)) => Some(key),
        Some(Key::Core(_) | Key::Wires | Key::Opening(_)) => {
            history_failed(domain, waiter);
            return;
        }
        None => None,
    };
    let Some(Read::History { load, .. }) = take_read(domain, waiter) else { return };
    domain.archive_reads.retire(Id::from_token(waiter));
    domain.work.push(Work::Forge(temper_engine_domain_forge::Event::HistoryLoaded {
        load,
        rows: stored.into_boxed(),
        next,
    }));
}
