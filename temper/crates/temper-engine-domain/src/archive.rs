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
    Transcript,
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
        Some(Read::Transcript { .. }) => Some(Waiting::Transcript),
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
            | Read::Transcript { .. }
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

pub(crate) fn transcript_failed(domain: &mut Domain, waiter: Token) {
    let Some(Read::Transcript { task }) = take_read(domain, waiter) else { return };
    domain.archive_reads.retire(Id::from_token(waiter));
    // A stale read fails no current preparation.
    if domain.engine.core().transcripts.contains_key(&task) {
        domain.work.push(Work::Core(jig_domain_core::Event::TranscriptLoaded {
            task,
            load: jig_domain_core::TranscriptLoad::Failed,
        }));
    }
}

pub(crate) fn transcript_unread(domain: &mut Domain, waiter: Token) {
    let Some(Read::Transcript { task }) = take_read(domain, waiter) else { return };
    domain.archive_reads.retire(Id::from_token(waiter));
    // The core owns the closed/open window policy (jig's `domain/engine/executors-transcripts.md`, 1).
    if domain.engine.core().transcripts.contains_key(&task) {
        domain.work.push(Work::Core(jig_domain_core::Event::TranscriptLoaded {
            task,
            load: jig_domain_core::TranscriptLoad::Unread,
        }));
    }
}

pub(crate) fn transcript_loaded(
    domain: &mut Domain,
    waiter: Token,
    rows: Box<[Record]>,
    texts: Option<crate::TranscriptTexts>,
    next: Option<Key>,
    cut: Option<loads::Cut>,
    out: &mut Queue<Request>,
) {
    if cut.is_some() {
        transcript_failed(domain, waiter);
        return;
    }
    let Some(Some(Read::Transcript { task })) = domain.archive_reads.get(Id::from_token(waiter)) else { return };
    let task = *task;
    // A message can invalidate preparation while this store read is in
    // flight; its old answer belongs to no current preparation (jig's
    // `examples/ops/domain/src/restart.rs`).
    if !domain.engine.core().transcripts.contains_key(&task) {
        retire_read(domain, waiter);
        return;
    }
    let Some(load) = transcript_page(rows, texts, next.is_none()) else {
        transcript_failed(domain, waiter);
        return;
    };
    domain.work.push(Work::Core(jig_domain_core::Event::TranscriptLoaded { task, load }));
    if let Some(after) = next {
        let mut issued = Queue::with_capacity(1);
        loads::next(&mut domain.loads, waiter, after, domain.limits.loads.rows, &mut issued)
            .expect("uncut transcript continuation keeps its load slot");
        match issued.pop().expect("continuation issued") {
            loads::Request::Load { owner, range, after, most, bytes } => {
                out.push(Request::Store(crate::StoreRequest::Load { owner, range, after, most, bytes }));
            }
            loads::Request::Loaded { .. } | loads::Request::Unloaded { .. } => {
                unreachable!("continuation only issues IO")
            }
        }
        return;
    }
    let Some(Read::Transcript { .. }) = take_read(domain, waiter) else { unreachable!("finished transcript read") };
    domain.archive_reads.retire(Id::from_token(waiter));
}

/// Validate row/text pairing before handing a page to the core (temper/docs/design/domain/root.md, 7).
fn transcript_page(
    rows: Box<[Record]>,
    texts: Option<crate::TranscriptTexts>,
    done: bool,
) -> Option<jig_domain_core::TranscriptLoad> {
    let count = rows.len();
    if let Some(texts) = &texts
        && texts.len() != count
    {
        return None;
    }
    let mut texts = match texts {
        Some(texts) => texts,
        None => Box::new([]),
    }
    .into_iter();
    let mut loaded = List::with_capacity(u32::try_from(rows.len()).expect("bounded transcript page"));
    for row in rows {
        let turn = match row {
            Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Turn(turn))) => turn,
            Record::Core(
                jig_domain_core::Record::Core(
                    jig_domain_core::CoreRecord::ProposalDecision(_)
                    | jig_domain_core::CoreRecord::Settings(_)
                    | jig_domain_core::CoreRecord::Part(_)
                    | jig_domain_core::CoreRecord::Call(_)
                    | jig_domain_core::CoreRecord::EscalationDecision(_)
                    | jig_domain_core::CoreRecord::Deployment(_)
                    | jig_domain_core::CoreRecord::RunProof(_)
                    | jig_domain_core::CoreRecord::Terminal(_)
                    | jig_domain_core::CoreRecord::Projection(_),
                )
                | jig_domain_core::Record::Tasks(_)
                | jig_domain_core::Record::People(_)
                | jig_domain_core::Record::Accounts(_)
                | jig_domain_core::Record::Notes(_),
            )
            | Record::Forge { .. } => {
                return None;
            }
        };
        let text = match texts.next() {
            Some(crate::TurnText::Cut(bytes)) => {
                if count != 1 || !turn.transcript.is_empty() {
                    return None;
                }
                return Some(jig_domain_core::TranscriptLoad::Cut { turn, bytes: u64::from(bytes), done });
            }
            Some(crate::TurnText::Rendered(text)) => {
                if text.len() > usize::try_from(crate::TRANSCRIPT_TAIL_BYTES).expect("tail bound fits usize") {
                    return None;
                }
                Some(text)
            }
            Some(crate::TurnText::Unrendered) | None => None,
        };
        loaded.push((turn, text)).expect("one transcript item per row");
    }
    Some(jig_domain_core::TranscriptLoad::Page { rows: loaded.into_boxed(), done })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::resume_fixture::root;
    use jig_domain_core as core;

    const TASK: u64 = 1;
    const PREVIOUS_ATTEMPT: u64 = 1;

    fn transcript_read(domain: &mut Domain) -> Token {
        domain.archive_reads.insert(Some(Read::Transcript { task: TASK })).expect("reserved transcript read").token()
    }

    #[test]
    fn a_transcript_continuation_keeps_its_slot_while_every_other_slot_is_taken() {
        let (mut domain, env) = root();
        domain.engine.core_mut().begin_transcript(TASK, PREVIOUS_ATTEMPT, Some((PREVIOUS_ATTEMPT, 1)));
        let waiter = transcript_read(&mut domain);
        let range = Range::Core(crate::CoreRange::TaskTranscript { task: TASK, from: (PREVIOUS_ATTEMPT, 1) });
        let mut issued = Queue::with_capacity(1);
        let owner = loads::begin(&mut domain.loads, waiter, range.clone(), None, 1, &mut issued)
            .expect("first transcript page admitted");
        issued.pop().expect("first page issued");
        for number in 1..env.limits.loads.loads {
            loads::begin(
                &mut domain.loads,
                Token::new(u64::MAX - u64::from(number)),
                Range::Core(crate::CoreRange::Tasks),
                None,
                1,
                &mut issued,
            )
            .expect("other slot held");
            issued.pop().expect("other load issued");
        }
        assert!(!domain.loads.has_room());
        let row = crate::TurnRecord {
            task: TASK,
            attempt: PREVIOUS_ATTEMPT,
            turn: 1,
            spent: 1,
            read: None,
            at: skein_lib::Wall::EPOCH,
            opens_window: true,
            transcript: Box::new(*b"x"),
        };
        let rows = Box::new([Record::Core(core::Record::Core(core::CoreRecord::Turn(row)))]);
        let after = Key::Core(core::Key::Core(core::CoreKey::Turn { task: TASK, attempt: PREVIOUS_ATTEMPT, turn: 1 }));
        loads::loaded(&mut domain.loads, owner, rows, Some(after.clone()), &mut issued);
        let Some(loads::Request::Loaded { waiter: found, rows, next, cut }) = issued.pop() else {
            panic!("page delivered to its waiter");
        };
        assert_eq!(found, waiter);
        let mut out = Queue::with_capacity(1);
        transcript_loaded(&mut domain, waiter, rows, None, next, cut, &mut out);
        assert_eq!(
            out.pop(),
            Some(Request::Store(crate::StoreRequest::Load {
                owner,
                range,
                after: Some(after),
                most: env.limits.loads.rows,
                bytes: env.limits.loads.reply_bytes,
            }))
        );
        let Some(Work::Core(core::Event::TranscriptLoaded {
            task,
            load: core::TranscriptLoad::Page { rows, done: false },
        })) = domain.work.pop()
        else {
            panic!("the core hears its page, never Failed for room");
        };
        assert_eq!(task, TASK);
        assert_eq!(rows.len(), 1);
        assert!(domain.work.is_empty());
        assert!(out.is_empty());
    }
}
