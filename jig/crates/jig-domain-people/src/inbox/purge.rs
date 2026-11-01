//! Removal of volatile inbox references (jig/docs/design/domain/people.md, 6).
use super::{Domain, Entry, List, Queue, Request, Update, Whom};
use crate::Key;

#[derive(Clone, Copy)]
pub(in crate::domain) enum Subject {
    Task(u64),
    Project(u32),
}

impl Subject {
    fn contains(self, task: u64, project: u32) -> bool {
        match self {
            Self::Task(number) => task == number,
            Self::Project(number) => project == number,
        }
    }

    fn entry(self, entry: Entry) -> bool {
        self.contains(entry.task, entry.project) || self.whom(entry.whom)
    }

    fn whom(self, whom: Whom) -> bool {
        match whom {
            Whom::Role { project, .. } => match self {
                Self::Project(number) => project == number,
                Self::Task(_) => false,
            },
            Whom::Person(_) => false,
        }
    }
}

pub(in crate::domain) fn erase_ended_positions(domain: &Domain, task: u64, out: &mut Queue<Request>) {
    for (party, _) in &domain.people {
        let mut known = domain.conversations_seen.contains_key(&(*party, task));
        if let Some(chat) = domain.chat_totals.get(&task) {
            known |= chat.party == *party;
        }
        if let Some(entries) = domain.replies.get(&task) {
            for entry in entries {
                match entry.whom {
                    Whom::Person(owner) => known |= owner == *party,
                    Whom::Role { .. } => {}
                }
            }
        }
        if let Some(entries) = domain.unread.get(party) {
            for entry in entries {
                known |= entry.task == task;
            }
        }
        for ((owner, _), entries) in &domain.result_census {
            if owner == party {
                for entry in entries {
                    known |= entry.task == task;
                }
            }
        }
        if let Some(entries) = domain.recent.get(party) {
            for entry in entries {
                known |= match entry.update {
                    Update::Result(row) => row.task == task,
                    Update::Reply(row) => row.task == task,
                    Update::ChatReply(row) => row.entry.task == task,
                };
            }
        }
        if known {
            out.push(Request::Erase { key: Key::EndedRead { task, party: *party } });
        }
    }
}

pub(in crate::domain) fn purge_project(domain: &mut Domain, project: u32, out: &mut Queue<Request>) {
    let mut positions = List::with_capacity(domain.conversations_seen.capacity());
    for (key, _) in &domain.conversations_seen {
        if let Some(chat) = domain.chat_totals.get(&key.1)
            && chat.project == project
        {
            positions.push(*key).expect("subset of live positions");
        }
    }
    for (party, task) in positions.into_boxed() {
        let _old = domain.conversations_seen.remove(&(party, task));
        out.push(Request::Erase { key: Key::ReadConversation { party, task } });
    }
    let mut chats = List::with_capacity(domain.chat_totals.capacity());
    for (task, chat) in &domain.chat_totals {
        if chat.project == project {
            chats.push(*task).expect("subset of live chats");
        }
    }
    for task in chats.into_boxed() {
        let _old = domain.chat_totals.remove(&task);
    }
    let mut aggregates = List::with_capacity(domain.chat_updates.capacity());
    for (key, _) in &domain.chat_updates {
        if key.1 == project {
            aggregates.push(*key).expect("subset of project aggregates");
        }
    }
    for key in aggregates.into_boxed() {
        let _old = domain.chat_updates.remove(&key);
    }
    purge_references(domain, Subject::Project(project));
}

pub(in crate::domain) fn purge_references(domain: &mut Domain, subject: Subject) {
    purge_waiting(domain, subject);
    purge_needs(domain, subject);
    purge_replies(domain, subject);
    purge_recent(domain, subject);
    purge_unread(domain, subject);
    purge_result_census(domain, subject);
    purge_refusals(domain, subject);
}

fn purge_waiting(domain: &mut Domain, subject: Subject) {
    let mut keys = List::with_capacity(domain.waiting.capacity());
    for (key, _) in &domain.waiting {
        keys.push(*key).expect("one key per waiting row");
    }
    for key in keys.into_boxed() {
        let old = domain.waiting.remove(&key).expect("snapshot key");
        if subject.whom(key) {
            continue;
        }
        let mut kept = List::with_capacity(u32::try_from(old.len()).expect("bounded waiting row"));
        for entry in old {
            let erased = subject.entry(entry);
            if !erased {
                kept.push(entry).expect("waiting subset");
            }
        }
        if !kept.is_empty() {
            domain.waiting.insert(key, kept.into_boxed()).expect("removed waiting slot");
        }
    }
}

fn purge_needs(domain: &mut Domain, subject: Subject) {
    let mut keys = List::with_capacity(domain.needs.capacity());
    for (key, _) in &domain.needs {
        keys.push(*key).expect("one key per needs row");
    }
    for key in keys.into_boxed() {
        let old = domain.needs.remove(&key).expect("snapshot key");
        let mut kept = List::with_capacity(u32::try_from(old.len()).expect("bounded needs row"));
        for entry in old {
            let erased = subject.contains(key, entry.project) || subject.whom(entry.whom);
            if !erased {
                kept.push(entry).expect("needs subset");
            }
        }
        if !kept.is_empty() {
            domain.needs.insert(key, kept.into_boxed()).expect("removed needs slot");
        }
    }
}

fn purge_replies(domain: &mut Domain, subject: Subject) {
    let mut keys = List::with_capacity(domain.replies.capacity());
    for (key, _) in &domain.replies {
        keys.push(*key).expect("one key per replies row");
    }
    for key in keys.into_boxed() {
        let old = domain.replies.remove(&key).expect("snapshot key");
        let mut kept = List::with_capacity(u32::try_from(old.len()).expect("bounded replies row"));
        for entry in old {
            let erased = subject.entry(entry);
            if !erased {
                kept.push(entry).expect("replies subset");
            }
        }
        if !kept.is_empty() {
            domain.replies.insert(key, kept.into_boxed()).expect("removed replies slot");
        }
    }
}

fn purge_recent(domain: &mut Domain, subject: Subject) {
    let mut keys = List::with_capacity(domain.recent.capacity());
    for (key, _) in &domain.recent {
        keys.push(*key).expect("one key per recent row");
    }
    for key in keys.into_boxed() {
        let old = domain.recent.remove(&key).expect("snapshot key");
        let mut kept = List::with_capacity(u32::try_from(old.len()).expect("bounded recent row"));
        for entry in old {
            let erased = match entry.update {
                Update::Result(row) => subject.contains(row.task, row.project),
                Update::Reply(row) => subject.entry(row),
                Update::ChatReply(row) => subject.entry(row.entry),
            };
            if !erased {
                kept.push(entry).expect("recent subset");
            }
        }
        if !kept.is_empty() {
            domain.recent.insert(key, kept.into_boxed()).expect("removed recent slot");
        }
    }
}

fn purge_unread(domain: &mut Domain, subject: Subject) {
    let mut keys = List::with_capacity(domain.unread.capacity());
    for (key, _) in &domain.unread {
        keys.push(*key).expect("one key per unread row");
    }
    for key in keys.into_boxed() {
        let old = domain.unread.remove(&key).expect("snapshot key");
        let mut kept = List::with_capacity(u32::try_from(old.len()).expect("bounded unread row"));
        for entry in old {
            let erased = subject.contains(entry.task, entry.project);
            if !erased {
                kept.push(entry).expect("unread subset");
            }
        }
        if !kept.is_empty() {
            domain.unread.insert(key, kept.into_boxed()).expect("removed unread slot");
        }
    }
}

fn purge_result_census(domain: &mut Domain, subject: Subject) {
    let mut keys = List::with_capacity(domain.result_census.capacity());
    for (key, _) in &domain.result_census {
        keys.push(*key).expect("one key per result_census row");
    }
    for key in keys.into_boxed() {
        let old = domain.result_census.remove(&key).expect("snapshot key");
        if match subject {
            Subject::Task(_) => false,
            Subject::Project(project) => key.1 == project,
        } {
            continue;
        }
        let mut kept = List::with_capacity(u32::try_from(old.len()).expect("bounded result_census row"));
        for entry in old {
            let erased = subject.contains(entry.task, entry.project);
            if !erased {
                kept.push(entry).expect("result_census subset");
            }
        }
        if !kept.is_empty() {
            domain.result_census.insert(key, kept.into_boxed()).expect("removed result_census slot");
        }
    }
}

fn purge_refusals(domain: &mut Domain, subject: Subject) {
    for _ in 0..domain.room_refusals.len() {
        let entry = domain.room_refusals.pop().expect("snapshot queue length");
        let erased = match subject {
            Subject::Project(project) => entry.project == Some(project),
            Subject::Task(task) => match entry.action {
                crate::CapacityAction::Escalation { task: number }
                | crate::CapacityAction::Words { task: number }
                | crate::CapacityAction::Question { task: number, .. }
                | crate::CapacityAction::Amend { task: number }
                | crate::CapacityAction::Move { task: number }
                | crate::CapacityAction::TakePerson { task: number }
                | crate::CapacityAction::HandBackPerson { task: number }
                | crate::CapacityAction::AnswerPerson { task: number }
                | crate::CapacityAction::Stop { task: number }
                | crate::CapacityAction::Cancel { task: number }
                | crate::CapacityAction::Release { task: number }
                | crate::CapacityAction::MarkSeen { seen: crate::Seen::Conversation { task: number, .. } } => {
                    number == task
                }
                crate::CapacityAction::Proposal { proposer, .. } => match proposer {
                    crate::Proposer::Task(number) => number == task,
                    crate::Proposer::Party(_) => false,
                },
                crate::CapacityAction::Compound
                | crate::CapacityAction::Create
                | crate::CapacityAction::Application { .. }
                | crate::CapacityAction::Note { .. }
                | crate::CapacityAction::Service
                | crate::CapacityAction::Adoption
                | crate::CapacityAction::Roles
                | crate::CapacityAction::Policy
                | crate::CapacityAction::Pool { .. }
                | crate::CapacityAction::Lift { .. }
                | crate::CapacityAction::MarkSeen { seen: crate::Seen::Results { .. } }
                | crate::CapacityAction::AccountSignIn
                | crate::CapacityAction::AccountSignOut
                | crate::CapacityAction::AccountChange => false,
            },
        };
        if !erased {
            domain.room_refusals.push(entry);
        }
    }
}
