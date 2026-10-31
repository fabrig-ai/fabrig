//! Rebuild live projections once, then name the indexed ended owners (store.md 7.2).
use crate::start::{LiveSet, Phase, StartMode};
use crate::{Census, Core, Limits, ReadResults};
use alloc::boxed::Box;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::List;

#[derive(Debug)]
pub(crate) struct Progress {
    unread_result: u64,
    read_conversation: u64,
}

impl Core {
    fn project_live_inbox(&mut self, limits: &Limits) {
        for task in self.tasks.view_tasks() {
            let row = self.tasks.task(task.number).expect("live view names its owner");
            let entries = self.waiting_entries(limits, row);
            if let Some(party) = self.conversation_requester(row) {
                let input = people::RestoreProjection::ChatState {
                    party,
                    project: row.project,
                    task: row.number,
                    last_reply: row.last_reply,
                    live: true,
                    seen: None,
                    replies: super::reply_refs(limits, row, party),
                };
                self.project(limits, input);
            }
            self.project(limits, people::RestoreProjection::Waiting { task: task.number, entries });
        }
        let mut proposals = List::with_capacity(self.tasks.person_proposals().len());
        for (number, _) in self.tasks.person_proposals() {
            proposals.push(*number).expect("one key per pending proposal");
        }
        for number in proposals.into_boxed() {
            let proposal = self.tasks.person_proposals().get(&number).expect("snapshot proposal");
            let input = people::RestoreProjection::Waiting {
                task: proposal.first(),
                entries: self.person_proposal_entries(proposal),
            };
            self.project(limits, input);
        }
    }

    pub fn census(&mut self, limits: &Limits) -> Census {
        self.start_input();
        let selected = match self.start.mode {
            StartMode::Default => {
                assert!(
                    self.restart_step() == Some(crate::RestartStep::LoadInbox) && self.people.ready(),
                    "people restored before census"
                );
                false
            }
            StartMode::LoadAndLink => {
                assert!(
                    self.start_phase() == Some(Phase::LoadInbox) && self.start_failure().is_none(),
                    "census requires unrefused LoadInbox"
                );
                true
            }
        };
        self.project_live_inbox(limits);
        if selected && self.start_failure().is_some() {
            return Census { questions: Box::new([]), results: Box::new([]) };
        }
        let mut questions = List::with_capacity(super::question_capacity(limits).expect("validated question bound"));
        for task in self.tasks.view_tasks() {
            let row = self.tasks.task(task.number).expect("live view owner");
            if !super::questions_live(&row.phase) {
                continue;
            }
            for credit in &row.questions {
                if questions.push((row.number, credit.line)).is_err() {
                    assert!(selected, "reserved answer slot bounds original questions");
                    self.start_room(
                        LiveSet::Census,
                        crate::Key::Tasks(tasks::Key::Live(row.number)),
                        questions.len(),
                        questions.capacity(),
                    );
                    return Census { questions: questions.into_boxed(), results: Box::new([]) };
                }
            }
            if let jig_domain_tasks::Party::Person(_) = row.requester {
                for line in &row.question_lines {
                    let mut outgoing = false;
                    for credit in &row.questions {
                        outgoing |= credit.line == line.line;
                    }
                    if line.answer.is_none() && !outgoing && questions.push((row.number, line.line)).is_err() {
                        assert!(selected, "reserved notice slot bounds received questions");
                        self.start_room(
                            LiveSet::Census,
                            crate::Key::Tasks(tasks::Key::Live(row.number)),
                            questions.len(),
                            questions.capacity(),
                        );
                        return Census { questions: questions.into_boxed(), results: Box::new([]) };
                    }
                }
            }
        }
        let mut results = List::with_capacity(limits.people.people);
        for party in &self.people.parties() {
            if selected
                && (!self.start_room(
                    LiveSet::Census,
                    crate::Key::People(people::Key::Person(*party)),
                    self.census_progress.len(),
                    self.census_progress.capacity(),
                ) || !self.start_room(
                    LiveSet::ReadResults,
                    crate::Key::People(people::Key::Person(*party)),
                    results.len(),
                    results.capacity(),
                ))
            {
                break;
            }
            self.census_progress
                .insert(*party, Progress { unread_result: 0, read_conversation: 0 })
                .expect("bounded census parties");
            results
                .push(ReadResults {
                    party: *party,
                    after: self.people.results_seen(*party).expect("known restored party"),
                })
                .expect("bounded restored parties");
        }
        Census { questions: questions.into_boxed(), results: results.into_boxed() }
    }
}

impl Core {
    fn project_ended_row(
        &mut self,
        limits: &Limits,
        row: &tasks::TaskRecord,
        party: u64,
        at: skein_lib::Wall,
        seen: Option<u64>,
    ) {
        self.project(
            limits,
            people::RestoreProjection::Result {
                party,
                entry: people::ResultRef {
                    task: row.number,
                    project: row.project,
                    at,
                    position: row.result_position,
                    forgettable: (row.flags.retention == tasks::Retention::ForgottenOnceRead).then_some(
                        people::Forgettable {
                            label: row.label,
                            listed: row.flags.listed,
                            turns_read: match seen {
                                Some(through) => through >= row.last_reply,
                                None => false,
                            },
                            last_reply: row.flags.conversation.then_some(row.last_reply),
                        },
                    ),
                },
            },
        );
        if let Some(through) = seen {
            self.project(
                limits,
                people::RestoreProjection::ChatState {
                    party,
                    project: row.project,
                    task: row.number,
                    last_reply: row.last_reply,
                    live: false,
                    seen: Some(through),
                    replies: super::reply_refs(limits, row, party),
                },
            );
        }
    }

    #[must_use]
    pub fn restore_inbox_row(&mut self, limits: &Limits, record: &crate::Record) -> bool {
        self.start_input();
        let selected = match self.start.mode {
            StartMode::Default => {
                if self.restart_step() != Some(crate::RestartStep::LoadInbox) || !self.people.ready() {
                    return false;
                }
                false
            }
            StartMode::LoadAndLink => {
                assert!(
                    self.start_phase() == Some(Phase::LoadInbox) && self.start_failure().is_none(),
                    "inbox row requires unrefused LoadInbox"
                );
                true
            }
        };
        self.load_census_row(limits, record, selected)
    }

    fn load_census_question(&mut self, limits: &Limits, line: &tasks::ActivityLine, selected: bool) -> bool {
        if selected {
            self.prune_questions(line.task);
            if !self.question_activity.contains_key(&(line.task, line.position))
                && !self.start_room(
                    LiveSet::Census,
                    crate::Key::Tasks(tasks::Key::Activity { task: line.task, position: line.position }),
                    self.question_activity.len(),
                    self.question_activity.capacity(),
                )
            {
                return false;
            }
            self.question_activity.insert((line.task, line.position), line.clone()).expect("question fit checked");
        } else if !self.remember_question(line) {
            return false;
        }
        let owner = self.tasks.task(line.task).expect("named question owner");
        let entries = self.waiting_entries(limits, owner);
        self.project(limits, people::RestoreProjection::Waiting { task: line.task, entries });
        self.start_failure().is_none()
    }

    fn load_census_row(&mut self, limits: &Limits, record: &crate::Record, selected: bool) -> bool {
        if let crate::Record::Tasks(tasks::Stored::Activity(line)) = record {
            return self.load_census_question(limits, line, selected);
        }
        if let crate::Record::People(people::Stored::EndedRead { task, party, through }) = record {
            if let Some((previous, party, _)) = self.census_position {
                assert!(
                    !selected,
                    "{:?} must precede its ended owner before {:?}",
                    crate::Key::People(people::Key::EndedRead { task: previous, party }),
                    record.key()
                );
                return false;
            }
            self.census_position = Some((*task, *party, *through));
            return true;
        }
        let crate::Record::Tasks(tasks::Stored::Ended(row)) = record else { return false };
        let seen = match self.census_position.take() {
            Some((task, party, through))
                if task == row.number && row.requester == tasks::Party::Person(party) && row.flags.conversation =>
            {
                Some(through)
            }
            Some((task, party, _)) => {
                assert!(
                    !selected,
                    "{:?} links mismatched {:?}",
                    crate::Key::People(people::Key::EndedRead { task, party }),
                    record.key()
                );
                return false;
            }
            None => None,
        };
        if !selected && (!super::matches_ended(&row.phase) || !super::valid_header(self, limits, row)) {
            return false;
        }
        match row.requester {
            tasks::Party::Person(party) => {
                if !selected && self.people.identity(party).is_none() {
                    return false;
                }
                let at = match row.ended_at {
                    Some(at) => at,
                    None if selected => row.changed,
                    None => return false,
                };
                if !selected && (row.result_position == 0 || row.result_position > self.counters.deployment().messages)
                {
                    return false;
                }
                if let Some(progress) = self.census_progress.get_mut(&party) {
                    let mark =
                        if seen.is_none() { &mut progress.unread_result } else { &mut progress.read_conversation };
                    let position = if seen.is_none() { row.result_position } else { row.number };
                    if position <= *mark {
                        return true;
                    }
                    *mark = position;
                } else {
                    assert!(selected, "party census initialised");
                }
                self.project_ended_row(limits, row, party, at, seen);
            }
            tasks::Party::Task(_) | tasks::Party::Deployment { .. } => {}
        }
        self.project(limits, people::RestoreProjection::EndedConversation { task: row.number });
        let visible = match row.requester {
            tasks::Party::Person(party) => self.people.role(party, row.project).is_some(),
            tasks::Party::Task(_) | tasks::Party::Deployment { .. } => false,
        };
        if visible && (seen.is_some() || !self.archived_chats.contains_key(&row.number)) {
            if selected
                && !self.archived_chats.contains_key(&row.number)
                && !self.start_room(
                    LiveSet::Conversations,
                    record.key(),
                    self.archived_chats.len(),
                    self.archived_chats.capacity(),
                )
            {
                return false;
            }
            self.cache_archived_chat(limits, row, seen);
        }
        self.start_failure().is_none()
    }
}
