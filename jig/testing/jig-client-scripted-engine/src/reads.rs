//! Retained rows and scoped opaque cursors; client/domain/contract/README.md, 4.
use crate::{Bounds, Engine, Invalid, Up};
use jig_client_domain::contract::{
    ActivityPage, Cursor, DecodedActivityLine, EndedDelegate, EndedPage, FundingRecord, HistoryPage, HistoryRow,
    InboxEntry, InboxGroup, InboxPage, InboxReadGroup, Note, NotePage, NoteScope, ObjectKey, Read, ReadNone,
    ReadResult, ReadValue, Side, SpendPage,
};
use skein_lib::{List, Time, Token};

/// An authoritative retained row collection, not a protocol page.
/// Each row passes through the wire codec and link; the protocol chooses bounded pages.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rows {
    Tasks(Vec<jig_client_domain::contract::TaskLine>),
    Ended(Vec<EndedDelegate>),
    History(Vec<HistoryRow>),
    Activity(Vec<DecodedActivityLine>),
    Inbox(Vec<InboxEntry>),
    Spend(Vec<FundingRecord>),
    Notes(Vec<Note>),
}
/// A party's page source. The read has no cursor and supplies its exact filters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Source {
    pub party: u64,
    pub read: Read,
    pub rows: Rows,
}
impl Source {
    pub(crate) fn validate(&self, bounds: &Bounds) -> core::result::Result<(), Invalid> {
        let (read, cursor) = plain(self.read.clone());
        if cursor.is_some() || crate::codec::selector(&read, bounds).is_err() || !self.rows.matches(&read) {
            return Err(Invalid::Identity);
        }
        self.rows.validate(bounds)?;
        if let Rows::Inbox(rows) = &self.rows {
            for row in rows {
                match (&row.group(), &row.reply) {
                    (InboxGroup::Replies, Some(line))
                        if row.card_key() == ObjectKey::Task(line.chip.task) && row.project == Some(line.project) => {}
                    (InboxGroup::Replies, None | Some(_))
                    | (
                        InboxGroup::Questions
                        | InboxGroup::Proposals
                        | InboxGroup::Escalations
                        | InboxGroup::PersonTasks
                        | InboxGroup::Results,
                        Some(_),
                    ) => return Err(Invalid::Identity),
                    (
                        InboxGroup::Questions
                        | InboxGroup::Proposals
                        | InboxGroup::Escalations
                        | InboxGroup::PersonTasks
                        | InboxGroup::Results,
                        None,
                    ) => {}
                }
            }
        }
        Ok(())
    }
}
impl Rows {
    pub(crate) fn validate(&self, bounds: &Bounds) -> core::result::Result<(), Invalid> {
        for start in 0..self.len() {
            crate::codec::read(&self.page(start, start + 1, None, None), bounds)?;
        }
        Ok(())
    }

    fn matches(&self, read: &Read) -> bool {
        matches!(
            (self, read),
            (Self::Tasks(_), Read::Tasks { .. })
                | (Self::Ended(_), Read::Ended { .. })
                | (Self::History(_), Read::History { .. })
                | (Self::Activity(_), Read::Activity { .. } | Read::ActivityBeside { .. })
                | (Self::Inbox(_), Read::Inbox { .. })
                | (Self::Spend(_), Read::Spend { .. })
                | (Self::Notes(_), Read::Notes { .. })
        )
    }
    fn len(&self) -> usize {
        match self {
            Self::Tasks(rows) => rows.len(),
            Self::Ended(rows) => rows.len(),
            Self::History(rows) => rows.len(),
            Self::Activity(rows) => rows.len(),
            Self::Inbox(rows) => rows.len(),
            Self::Spend(rows) => rows.len(),
            Self::Notes(rows) => rows.len(),
        }
    }
    fn page(&self, start: usize, end: usize, before: Option<Cursor>, after: Option<Cursor>) -> ReadValue {
        match self {
            Self::Tasks(rows) => ReadValue::Tasks(Box::new(jig_client_domain::contract::TaskPage {
                live: list(&rows[start..end]),
                live_after: after,
                ended: jig_client_domain::contract::EndedGroup::None,
            })),
            Self::Ended(rows) => {
                ReadValue::EndedPage(Box::new(EndedPage { cut: false, rows: list(&rows[start..end]), before, after }))
            }
            Self::History(rows) => {
                ReadValue::History(Box::new(HistoryPage { cut: false, rows: list(&rows[start..end]), before, after }))
            }
            Self::Activity(rows) => {
                ReadValue::Activity(Box::new(ActivityPage { cut: false, rows: list(&rows[start..end]), before, after }))
            }
            Self::Inbox(rows) => ReadValue::Inbox(Box::new(InboxPage {
                selection: None,
                cut: false,
                rows: list(&rows[start..end]),
                before,
                after,
            })),
            Self::Spend(rows) => {
                ReadValue::Spend(Box::new(SpendPage { cut: false, rows: list(&rows[start..end]), before, after }))
            }
            Self::Notes(rows) => {
                ReadValue::Notes(Box::new(NotePage { cut: false, rows: list(&rows[start..end]), before, after }))
            }
        }
    }
}
#[derive(Debug)]
pub(crate) struct Position {
    party: u64,
    read: Read,
    index: usize,
}
fn cursor(number: u64, bounds: &Bounds) -> Cursor {
    Cursor::new(number.to_be_bytes().to_vec().into_boxed_slice(), bounds.text).expect("cursor bound checked")
}
pub(crate) fn list<T: Clone>(rows: &[T]) -> List<T> {
    List::from_boxed(rows.to_vec().into_boxed_slice())
}
pub(crate) fn plain(mut read: Read) -> (Read, Option<Cursor>) {
    let position = match &mut read {
        Read::Ended { after, .. }
        | Read::History { after, .. }
        | Read::Inbox { after, .. }
        | Read::Spend { after, .. }
        | Read::Notes { after, .. } => after.take(),
        Read::Tasks { live, ended, .. } => ended.take().or_else(|| live.take()),
        Read::Activity { before, .. } => before.take(),
        Read::ActivityBeside { .. }
        | Read::Me
        | Read::Turn { .. }
        | Read::TaskText { .. }
        | Read::Task { .. }
        | Read::Proposal { .. }
        | Read::Escalation { .. }
        | Read::Question { .. }
        | Read::PersonTask { .. }
        | Read::Result { .. }
        | Read::Project { .. }
        | Read::Settings
        | Read::Setting(_)
        | Read::Revision(_)
        | Read::Revisions(_)
        | Read::Deployment => None,
    };
    (read, position)
}
impl<A: crate::Application> Engine<A> {
    /// Captures one read at the latest numbered commit. A token receives one terminal.
    pub fn read(&mut self, now: Time, sign_in: u64, token: Token, read: Read) -> core::result::Result<(), Invalid> {
        if crate::codec::selector(&read, &self.bounds).is_err() {
            return Err(Invalid::Payload);
        }
        if !self.read_tokens.insert(token) {
            return Err(Invalid::Identity);
        }
        self.reads.insert(token, sign_in);
        let terminal = if !self.online(now) {
            ReadResult::None(ReadNone::Unreachable)
        } else if let Some(party) = self.truth.sign_ins.get(&sign_in).copied() {
            self.read_value(party, read).map_or_else(ReadResult::None, ReadResult::Value)
        } else {
            ReadResult::None(ReadNone::SignedOut)
        };
        let terminal = match terminal {
            ReadResult::Value(value) => ReadResult::Value(value),
            ReadResult::None(why) => ReadResult::None(crate::codec::none(why)),
        };
        self.queue(now, self.next_commit - 1, Up::Read { token, terminal });
        Ok(())
    }
    pub(crate) fn read_value(&mut self, party: u64, read: Read) -> core::result::Result<ReadValue, ReadNone> {
        let (query, position) = plain(read);
        if !self.read_visible(party, &query) {
            return Err(ReadNone::Unknown);
        }
        let object = match &query {
            Read::Me => {
                return self.checked_read(self.truth.frames.get(&party).cloned().map(|v| ReadValue::Me(Box::new(v))));
            }
            Read::Turn { task, attempt, turn } => return self.read_turn(party, *task, *attempt, *turn),
            Read::TaskText { .. } => return Err(ReadNone::Unknown),
            Read::Task { task } => Some(ObjectKey::Task(*task)),
            Read::Proposal { proposal } => Some(ObjectKey::Proposal(*proposal)),
            Read::Escalation { task } => Some(ObjectKey::Escalation(*task)),
            Read::Question { task, message } => Some(ObjectKey::Question { task: *task, message: *message }),
            Read::PersonTask { task } => Some(ObjectKey::PersonTask(*task)),
            Read::Result { task } => Some(ObjectKey::Result(*task)),
            Read::Project { project } => {
                return self
                    .checked_read(self.truth.projects.get(project).cloned().map(|v| ReadValue::Project(Box::new(v))));
            }
            Read::Settings => {
                return self.checked_read(self.truth.settings.clone().map(|v| ReadValue::Settings(Box::new(v))));
            }
            Read::Setting(query) => {
                return self.checked_read(
                    self.truth
                        .setting_values
                        .iter()
                        .find(|v| v.setting == query.setting && v.revision == query.revision)
                        .cloned()
                        .map(|v| ReadValue::Setting(Box::new(v))),
                );
            }
            Read::Revision(query) => {
                return self.checked_read(Some(ReadValue::Revision(Box::new(
                    self.truth
                        .revisions
                        .get(&query.number)
                        .cloned()
                        .unwrap_or(jig_client_domain::contract::RevisionAnswer::Unknown),
                ))));
            }
            Read::Revisions(query) => {
                return self.checked_read(
                    self.truth.revision_pages.get(&query.before).cloned().map(|v| ReadValue::Revisions(Box::new(v))),
                );
            }
            Read::Deployment => {
                return self.checked_read(self.truth.deployment.clone().map(|v| ReadValue::Deployment(Box::new(v))));
            }
            Read::Tasks { .. }
            | Read::Ended { .. }
            | Read::History { .. }
            | Read::Activity { .. }
            | Read::ActivityBeside { .. }
            | Read::Inbox { .. }
            | Read::Spend { .. }
            | Read::Notes { .. } => None,
        };
        if let Some(key) = object {
            return self.checked_read(self.truth.objects.get(&key).cloned());
        }
        self.read_page(party, query, position)
    }
    fn read_page(
        &mut self,
        party: u64,
        query: Read,
        position: Option<Cursor>,
    ) -> core::result::Result<ReadValue, ReadNone> {
        let rows = if let Read::Tasks { query, .. } = &query {
            Rows::Tasks(self.task_lines(party, query))
        } else {
            self.rows_for(party, &query)
        };
        let backward = matches!(query, Read::Activity { .. } | Read::ActivityBeside { side: Side::Earlier, .. });
        let beside =
            if let Read::ActivityBeside { position, side, .. } = &query { Some((*position, *side)) } else { None };
        let index = if let Some(cursor) = position {
            let bytes: [u8; 8] = cursor.value.bytes().try_into().map_err(|_| ReadNone::Gone)?;
            let saved = self.positions.get(&u64::from_be_bytes(bytes)).ok_or(ReadNone::Gone)?;
            if saved.party != party || saved.read != query {
                return Err(ReadNone::Gone);
            }
            saved.index.min(rows.len())
        } else if let (Some((position, side)), Rows::Activity(rows)) = (beside, &rows) {
            rows.partition_point(|row| match side {
                Side::Earlier => row.source.position < position,
                Side::Later => row.source.position <= position,
            })
        } else if backward {
            rows.len()
        } else {
            0
        };
        let maximum = skein_lib::length::usize_of(self.bounds.rows);
        let available = if backward { index } else { rows.len() - index };
        let mut count = maximum.min(available);
        if count == 0 && available != 0 {
            return Err(ReadNone::Busy);
        }
        loop {
            let (start, end) = if backward { (index - count, index) } else { (index, index + count) };
            let before = (start != 0 && !matches!(beside, Some((_, Side::Later))))
                .then(|| cursor(self.next_position, &self.bounds));
            let after = (end != rows.len() && !matches!(beside, Some((_, Side::Earlier))))
                .then(|| cursor(self.next_position + 1, &self.bounds));
            let mut page = if let (Rows::Tasks(rows), Read::Tasks { query, .. }) = (&rows, &query)
                && query.phase == Some(jig_client_domain::contract::PhaseClass::Ended)
            {
                ReadValue::Tasks(Box::new(jig_client_domain::contract::TaskPage {
                    live: list(&[]),
                    live_after: None,
                    ended: jig_client_domain::contract::EndedGroup::Rows {
                        rows: list(&rows[start..end]),
                        after: after.clone(),
                        cut: false,
                    },
                }))
            } else {
                rows.page(start, end, before.clone(), after.clone())
            };
            if backward && let ReadValue::Activity(page) = &mut page {
                let mut rows = core::mem::replace(&mut page.rows, List::with_capacity(0)).into_boxed();
                rows.reverse();
                page.rows = List::from_boxed(rows);
            }
            if let ReadValue::Inbox(page) = &mut page {
                page.selection = self.truth.inbox_selection.get(&party).map(|rows| {
                    list(
                        &rows
                            .iter()
                            .filter(|selection| page.rows.iter().any(|row| row.cursor == selection.cursor))
                            .cloned()
                            .collect::<Vec<_>>(),
                    )
                });
            }
            if let Ok(page) = crate::codec::read(&page, &self.bounds) {
                let cursor_query = if let Read::ActivityBeside { task, .. } = query {
                    Read::Activity { task, before: None }
                } else {
                    query.clone()
                };
                if before.is_some() {
                    self.positions.insert(self.next_position, Position { party, read: cursor_query, index: start });
                }
                if after.is_some() {
                    self.positions.insert(self.next_position + 1, Position { party, read: query, index: end });
                }
                self.next_position += 2;
                return Ok(page);
            }
            if count == 0 {
                return Err(ReadNone::Busy);
            }
            count -= 1;
            if count == 0 && available != 0 {
                return Err(ReadNone::Busy);
            }
        }
    }
    fn read_turn(&self, party: u64, task: u64, attempt: u64, turn: u32) -> core::result::Result<ReadValue, ReadNone> {
        let Rows::Activity(rows) = self.rows_for(party, &Read::Activity { task, before: None }) else {
            unreachable!("activity rows")
        };
        for row in rows {
            let source = row.source;
            if source.task == task
                && matches!(source.line, jig_client_domain::contract::Activity::Turn { attempt: found_attempt, turn: found_turn, .. } if (found_attempt, found_turn) == (attempt, turn))
            {
                let archive = row.decoded.ok_or(ReadNone::Malformed)?;
                if (archive.task, archive.name.attempt, archive.name.number) != (task, attempt, turn)
                    || matches!(archive.turn, jig_client_domain::contract::TurnRead::Undecodable { .. })
                {
                    return Err(ReadNone::Malformed);
                }
                return self.checked_read(Some(ReadValue::Turn(Box::new(archive))));
            }
        }
        Err(ReadNone::Unknown)
    }
    pub(crate) fn rows_for(&self, party: u64, query: &Read) -> Rows {
        let broad = if let Read::Inbox { group, .. } = query {
            Read::Inbox { group: *group, project: None, kind: None, goal: None, after: None }
        } else if let Read::ActivityBeside { task, .. } = query {
            Read::Activity { task: *task, before: None }
        } else {
            query.clone()
        };
        let source = self
            .truth
            .sources
            .iter()
            .find(|source| source.party == party && source.read == *query)
            .or_else(|| self.truth.sources.iter().find(|source| source.party == party && source.read == broad));
        let rows = source.map_or_else(
            || {
                if matches!(query, Read::Notes { .. }) {
                    Rows::Notes(self.truth.notes.values().cloned().collect())
                } else {
                    empty(query)
                }
            },
            |source| source.rows.clone(),
        );
        match rows {
            Rows::Inbox(rows) => Rows::Inbox(rows.into_iter().filter(|row| {
                if row.project.is_some_and(|project| !self.truth.visible(party,project)) { return false; }
                match query {
                    Read::Inbox { group, project, kind, goal, .. } => *group == InboxReadGroup::of(row.group()) && project.is_none_or(|p| Some(p) == row.project) && kind.is_none_or(|k| k == row.group()) && goal.is_none_or(|g| self.truth.inbox_selection.get(&party).is_some_and(|selections| selections.iter().any(|selection| selection.cursor == row.cursor && selection.goals.iter().any(|goal| *goal == g)))),
                    Read::TaskText { .. } | Read::Tasks { .. } | Read::Me | Read::Task { .. } | Read::Proposal { .. } | Read::Escalation { .. } | Read::Question { .. }
                    | Read::Turn { .. } | Read::PersonTask { .. } | Read::Result { .. } | Read::Ended { .. } | Read::History { .. } | Read::Activity { .. } | Read::ActivityBeside { .. } | Read::Spend { .. } | Read::Project { .. } | Read::Notes { .. } | Read::Settings | Read::Setting(_) | Read::Revision(_) | Read::Revisions(_) | Read::Deployment => false,
                }
            }).collect()),
            Rows::Notes(rows) => Rows::Notes(rows.into_iter().filter_map(|mut row| {
                if let Read::Notes { scope: NoteScope::Tasks { project }, description, .. } = query {
                    if description.is_some() || !matches!(row.scope, NoteScope::Task { project: owner, .. } if owner == *project) {
                        return None;
                    }
                    row.body = jig_client_domain::contract::DocumentText { bytes: jig_client_domain::contract::Text::new(Box::new([]), 0).expect("empty text"), cut: true };
                    return Some(row);
                }
                matches!(query, Read::Notes { scope, description, .. } if *scope == row.scope && description.as_ref().is_none_or(|needle| needle.bytes().is_empty() || row.description.bytes().windows(needle.bytes().len()).any(|window| window == needle.bytes()))).then_some(row)
            }).collect()),
            Rows::Tasks(rows) => Rows::Tasks(rows),
            Rows::Ended(rows) => Rows::Ended(rows), Rows::History(rows) => Rows::History(rows),
            Rows::Activity(rows) => Rows::Activity(rows), Rows::Spend(rows) => Rows::Spend(rows),
        }
    }
    fn checked_read(&self, value: Option<ReadValue>) -> core::result::Result<ReadValue, ReadNone> {
        let value = value.ok_or(ReadNone::Unknown)?;
        crate::codec::read(&value, &self.bounds).map_err(|_| ReadNone::Busy)
    }
    pub(crate) fn read_visible(&self, party: u64, read: &Read) -> bool {
        let key = match read {
            Read::TaskText { task, .. }
            | Read::Task { task }
            | Read::Turn { task, .. }
            | Read::Ended { task, .. }
            | Read::History { task, .. }
            | Read::Activity { task, .. }
            | Read::ActivityBeside { task, .. } => Some(ObjectKey::Task(*task)),
            Read::Proposal { proposal } => Some(ObjectKey::Proposal(*proposal)),
            Read::Escalation { task } => Some(ObjectKey::Escalation(*task)),
            Read::Question { task, message } => Some(ObjectKey::Question { task: *task, message: *message }),
            Read::PersonTask { task } => Some(ObjectKey::PersonTask(*task)),
            Read::Result { task } => Some(ObjectKey::Result(*task)),
            Read::Project { project } | Read::Spend { project, .. } => {
                return self.truth.visible(party, *project);
            }
            Read::Inbox { project, .. } => {
                return project.is_none_or(|p| self.truth.visible(party, p));
            }
            Read::Notes { scope, .. } => {
                return match scope {
                    NoteScope::Party { .. } => false,
                    NoteScope::Deployment => self.truth.frames.get(&party).is_some_and(|f| f.deployment_role.is_some()),
                    NoteScope::Project(project)
                    | NoteScope::Resource { project, .. }
                    | NoteScope::Task { project, .. }
                    | NoteScope::Tasks { project } => self.truth.visible(party, *project),
                };
            }
            Read::Tasks { query, .. } => {
                return query.project.map_or_else(
                    || self.truth.frames.contains_key(&party),
                    |project| self.truth.visible(party, project),
                );
            }
            Read::Me => return self.truth.frames.contains_key(&party),
            Read::Settings | Read::Setting(_) | Read::Revision(_) | Read::Revisions(_) | Read::Deployment => {
                return self.truth.frames.get(&party).is_some_and(|f| f.deployment_role.is_some());
            }
        };
        key.and_then(|key| self.truth.project(key)).is_some_and(|project| self.truth.visible(party, project))
    }
}
fn empty(read: &Read) -> Rows {
    match read {
        Read::Tasks { .. } => Rows::Tasks(vec![]),
        Read::Ended { .. } => Rows::Ended(vec![]),
        Read::History { .. } => Rows::History(vec![]),
        Read::Activity { .. } | Read::ActivityBeside { .. } => Rows::Activity(vec![]),
        Read::Inbox { .. } => Rows::Inbox(vec![]),
        Read::Spend { .. } => Rows::Spend(vec![]),
        Read::Notes { .. } => Rows::Notes(vec![]),
        Read::Me
        | Read::Turn { .. }
        | Read::TaskText { .. }
        | Read::Task { .. }
        | Read::Proposal { .. }
        | Read::Escalation { .. }
        | Read::Question { .. }
        | Read::PersonTask { .. }
        | Read::Result { .. }
        | Read::Project { .. }
        | Read::Settings
        | Read::Setting(_)
        | Read::Revision(_)
        | Read::Revisions(_)
        | Read::Deployment => unreachable!("documents do not request a row source"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jig_client_domain::contract::{DocumentText, NoteAuthor, Revision, Text};

    const ACTIVITY_TASK: u64 = 8;
    const ACTIVITY_PARTY: u64 = 1;
    const ACTIVITY_LINES: u64 = 5;
    const PAGE_ROWS: u32 = 2;
    fn activity_engine() -> Engine {
        let mut script = crate::Script::default();
        script.bounds.rows = PAGE_ROWS;
        script.truth.sources.push(Source {
            party: ACTIVITY_PARTY,
            read: Read::Activity { task: ACTIVITY_TASK, before: None },
            rows: Rows::Activity(
                (1..=ACTIVITY_LINES)
                    .map(|position| DecodedActivityLine {
                        source: Box::new(jig_client_domain::contract::ActivityLine {
                            task: ACTIVITY_TASK,
                            position,
                            at: skein_lib::Wall::EPOCH,
                            line: jig_client_domain::contract::Activity::Milestone { position },
                        }),
                        decoded: None,
                    })
                    .collect(),
            ),
        });
        Engine::new(script, 1).expect("activity script")
    }
    #[test]
    fn beside_pages_exclude_the_line_and_end_at_either_boundary() {
        for (side, position, expected, before, after) in [
            (Side::Earlier, 3, vec![2, 1], false, false),
            (Side::Later, 2, vec![3, 4], false, true),
            (Side::Earlier, 1, vec![], false, false),
            (Side::Later, ACTIVITY_LINES, vec![], false, false),
            (Side::Later, 0, vec![1, 2], false, true),
            (Side::Earlier, u64::MAX, vec![5, 4], true, false),
        ] {
            let mut engine = activity_engine();
            let read = Read::ActivityBeside { task: ACTIVITY_TASK, position, side };
            let ReadValue::Activity(page) = engine.read_page(ACTIVITY_PARTY, read, None).unwrap() else {
                panic!("activity page")
            };
            assert_eq!(
                page.rows.iter().map(|row| row.source.position).collect::<Vec<_>>(),
                expected,
                "{side:?} at {position}"
            );
            assert_eq!((page.before.is_some(), page.after.is_some()), (before, after), "{side:?} at {position}");
        }
    }
    #[test]
    fn an_earlier_beside_cursor_continues_as_an_activity_read() {
        let mut engine = activity_engine();
        let read = Read::ActivityBeside { task: ACTIVITY_TASK, position: ACTIVITY_LINES, side: Side::Earlier };
        let ReadValue::Activity(page) = engine.read_page(ACTIVITY_PARTY, read, None).unwrap() else {
            panic!("activity page")
        };
        let read = Read::Activity { task: ACTIVITY_TASK, before: None };
        let ReadValue::Activity(page) = engine.read_page(ACTIVITY_PARTY, read, page.before).unwrap() else {
            panic!("activity page")
        };
        assert_eq!(page.rows.iter().map(|row| row.source.position).collect::<Vec<_>>(), vec![2, 1]);
        assert!(page.before.is_none());
    }
    #[test]
    fn tasks_select_goal_notes_without_bodies() {
        let mut script = crate::Script::default();
        for (number, scope) in [
            (1, NoteScope::Deployment),
            (2, NoteScope::Project(1)),
            (
                3,
                NoteScope::Resource {
                    project: 1,
                    connector: 1,
                    pattern: jig_client_domain::contract::Pattern {
                        segments: List::with_capacity(0),
                        last: jig_client_domain::contract::PatternEnd::Open(Box::new([])),
                    },
                },
            ),
            (4, NoteScope::Task { project: 2, task: 4 }),
            (5, NoteScope::Task { project: 1, task: 5 }),
            (6, NoteScope::Task { project: 1, task: 6 }),
        ] {
            script.truth.notes.insert(
                number,
                Note {
                    number,
                    scope,
                    revision: Revision { value: 1 },
                    description: DocumentText::complete(Text::new(Box::new(*b"draft"), 5).unwrap()),
                    body: DocumentText::complete(Text::new(Box::new(*b"body"), 4).unwrap()),
                    writer: NoteAuthor::Task { task: number, attempt: 1 },
                    tasks: List::with_capacity(0),
                    changed: None,
                    acts: List::with_capacity(0),
                    cut: false,
                },
            );
        }
        let engine = Engine::new(script, 1).unwrap();
        let query = Read::Notes { scope: NoteScope::Tasks { project: 1 }, description: None, after: None };
        let Rows::Notes(rows) = engine.rows_for(1, &query) else { panic!("notes rows") };
        assert_eq!(rows.len(), 2);
        for (row, task) in rows.iter().zip([5, 6]) {
            assert_eq!(row.scope, NoteScope::Task { project: 1, task });
            assert_eq!(row.body.bytes(), []);
            assert!(row.body.cut);
            assert!(!row.cut);
        }
        let query = Read::Notes {
            scope: NoteScope::Tasks { project: 1 },
            description: Some(Text::new(Box::new([]), 0).unwrap()),
            after: None,
        };
        let Rows::Notes(rows) = engine.rows_for(1, &query) else { panic!("notes rows") };
        assert_eq!(rows.len(), 0);
    }
}
