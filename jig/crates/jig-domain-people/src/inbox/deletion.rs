//! Erased tasks leave no inbox traces; deleted projects leave no roster
//! (temper/docs/design/draft/playground-reset.md, 5).
use super::{Domain, forget_chat, prune_unheld_roles, replace_waiting};
use crate::{Key, Limits, Request, Update};
use skein_lib::{List, Queue};

#[derive(Clone, Copy)]
enum Subject {
    Task(u64),
    Project(u32),
}

impl Subject {
    fn names(self, task: u64, project: u32) -> bool {
        match self {
            Self::Task(number) => task == number,
            Self::Project(number) => project == number,
        }
    }
}

pub(in crate::domain) fn forget_task(domain: &mut Domain, limits: &Limits, task: u64, out: &mut Queue<Request>) {
    forget_chat(domain, limits, task, out);
    replace_waiting(domain, limits, task, &[]);
    prune_updates(domain, Subject::Task(task));
}

pub(in crate::domain) fn drop_project(domain: &mut Domain, limits: &Limits, project: u32, out: &mut Queue<Request>) {
    drop(domain.roles.remove(&project));
    out.push(Request::Erase { key: Key::Roles(project) });
    prune_unheld_roles(domain, project);

    // Tasks are erased first by the parent. Clear any remaining volatile projections too.
    let mut tasks = List::with_capacity(domain.chat_totals.capacity());
    for (task, chat) in &domain.chat_totals {
        if chat.project == project {
            tasks.push(*task).expect("one live header per task");
        }
    }
    for task in tasks.into_boxed() {
        forget_chat(domain, limits, task, out);
    }
    let mut tasks = List::with_capacity(domain.needs.capacity());
    for (task, needs) in &domain.needs {
        for need in needs {
            if need.project == project {
                tasks.push(*task).expect("one project per task");
                break;
            }
        }
    }
    for task in tasks.into_boxed() {
        drop(domain.needs.remove(&task));
    }
    let mut tasks = List::with_capacity(domain.replies.capacity());
    for (task, replies) in &domain.replies {
        for reply in replies {
            if reply.project == project {
                tasks.push(*task).expect("one project per task");
                break;
            }
        }
    }
    for task in tasks.into_boxed() {
        drop(domain.replies.remove(&task));
    }
    let mut recipients = List::with_capacity(domain.waiting.capacity());
    for (whom, _) in &domain.waiting {
        recipients.push(*whom).expect("one key per recipient");
    }
    for whom in recipients.into_boxed() {
        let old = domain.waiting.remove(&whom).expect("snapshot recipient");
        let mut kept = List::with_capacity(limits.inbox_entries);
        for entry in old {
            if entry.project != project {
                kept.push(entry).expect("waiting subset");
            }
        }
        if !kept.is_empty() {
            assert!(domain.waiting.insert(whom, kept.into_boxed()).is_ok(), "existing recipient");
        }
    }
    let mut keys = List::with_capacity(domain.chat_updates.capacity());
    for (key, _) in &domain.chat_updates {
        if key.1 == project {
            keys.push(*key).expect("one aggregate per party and project");
        }
    }
    for key in keys.into_boxed() {
        let _old = domain.chat_updates.remove(&key);
    }
    prune_updates(domain, Subject::Project(project));
}

fn prune_updates(domain: &mut Domain, subject: Subject) {
    let mut keys = List::with_capacity(domain.result_census.capacity());
    for (key, _) in &domain.result_census {
        keys.push(*key).expect("one census per party and project");
    }
    for key in keys.into_boxed() {
        let old = domain.result_census.remove(&key).expect("snapshot census");
        let mut kept = List::with_capacity(u32::try_from(old.len()).expect("bounded census"));
        for entry in old {
            if !subject.names(entry.task, entry.project) {
                kept.push(entry).expect("census subset");
            }
        }
        if !kept.is_empty() {
            assert!(domain.result_census.insert(key, kept.into_boxed()).is_ok(), "existing census");
        }
    }
    let mut parties = List::with_capacity(domain.unread.capacity());
    for (party, _) in &domain.unread {
        parties.push(*party).expect("one result cache per party");
    }
    for party in parties.into_boxed() {
        let old = domain.unread.remove(&party).expect("snapshot cache");
        let mut kept = List::with_capacity(u32::try_from(old.len()).expect("bounded cache"));
        for entry in old {
            if !subject.names(entry.task, entry.project) {
                kept.push(entry).expect("result subset");
            }
        }
        if !kept.is_empty() {
            assert!(domain.unread.insert(party, kept.into_boxed()).is_ok(), "existing cache");
        }
    }
    let mut parties = List::with_capacity(domain.recent.capacity());
    for (party, _) in &domain.recent {
        parties.push(*party).expect("one recent prefix per party");
    }
    for party in parties.into_boxed() {
        let old = domain.recent.remove(&party).expect("snapshot prefix");
        let mut kept = List::with_capacity(u32::try_from(old.len()).expect("bounded prefix"));
        for item in old {
            let (task, project) = match item.update {
                Update::Result(row) => (row.task, row.project),
                Update::Reply(entry) => (entry.task, entry.project),
                Update::ChatReply(reply) => (reply.entry.task, reply.entry.project),
            };
            if !subject.names(task, project) {
                kept.push(item).expect("recent subset");
            }
        }
        if !kept.is_empty() {
            assert!(domain.recent.insert(party, kept.into_boxed()).is_ok(), "existing prefix");
        }
    }
}
