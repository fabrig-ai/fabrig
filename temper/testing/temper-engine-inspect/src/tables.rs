use crate::{View, landing_ended, task_name};
use skein_lib::Time;
use skein_world::inspect::{Entry, Inspection, Refused, Release, Table};
use temper_engine_domain_forge::{LimitSet, What};

fn writes(view: &View<'_>) -> Table {
    let forge = view.forge;
    let mut writes = Table {
        name: "forge write entries",
        capacity: u64::from(forge.inspection_entry_capacity()),
        entries: Vec::new(),
        refused: Vec::new(),
    };
    for number in forge.inspection_entries() {
        let Some(entry) = forge.inspection_entry(number) else { continue };
        writes.entries.push(Entry {
            key: format!("entry {number} (task {})", entry.task),
            rule: "the write's settlement, or its owner's end with the entry withdrawn",
            release: if view.ended(entry.task) && forge.inspection_client().inspection_entry(number).is_none() {
                Release::Due
            } else {
                Release::Pending
            },
        });
    }
    for task in forge.inspection_reservations() {
        writes.entries.push(Entry {
            key: format!("reserved for task {task}"),
            rule: "the write's settlement, or its owner's end with the entry withdrawn",
            release: if view.ended(task) { Release::Due } else { Release::Pending },
        });
    }
    for (task, row) in forge.inspection_changes() {
        if row.room.is_some() && refused_room(view, *task) {
            writes.refused.push(refused(*task));
        }
    }
    for (task, row) in forge.inspection_landings() {
        if row.room.is_some() && refused_room(view, *task) {
            writes.refused.push(refused(*task));
        }
    }
    for (task, row) in forge.inspection_releases() {
        if row.room.is_some() && refused_room(view, *task) {
            writes.refused.push(refused(*task));
        }
    }
    for task in forge.inspection_making_waiters() {
        if refused_room(view, task) {
            writes.refused.push(refused(task));
        }
    }

    writes
}

// The line passes over tasks held for something other than this table's room.
fn refused_room(view: &View<'_>, task: u64) -> bool {
    if view.forge.room_asked(task) {
        return false;
    }
    let Some(root) = view.root else { return true };
    if root.tasks().task(task).is_some_and(|row| matches!(row.phase, jig_domain_tasks::Phase::Held { .. })) {
        return root.forge_limit_holds().iter().any(|hold| hold.task == task && hold.set == LimitSet::Entries);
    }
    true
}

fn reads(view: &View<'_>) -> Table {
    let forge = view.forge;
    let client = forge.inspection_client();
    let mut reads = Table {
        name: "forge read admissions",
        capacity: u64::from(client.inspection_call_capacity()),
        entries: Vec::new(),
        refused: Vec::new(),
    };
    let mut calls = client.inspection_calls();
    calls.sort_unstable();
    for number in calls {
        let task = match client.inspection_call_owner(number) {
            Some(skein_forge_client::CallOwner::Read(_)) => view.read_task(number),
            Some(skein_forge_client::CallOwner::Entry(entry)) => forge.inspection_entry(*entry).map(|entry| entry.task),
            _ => None,
        };
        reads.entries.push(Entry {
            key: match task {
                Some(task) => format!("call {number} (task {task})"),
                None => format!("call {number}"),
            },
            rule: "the read's answer",
            release: if client.inspection_answered_call(number) { Release::Due } else { Release::Pending },
        });
    }
    for task in forge
        .inspection_landing_read_waiters()
        .iter()
        .chain(forge.inspection_lost_read_waiters().iter())
        .chain(forge.inspection_making_read_waiters().iter())
        .chain(forge.inspection_brief_read_waiters().iter())
    {
        reads.refused.push(refused(*task));
    }

    reads
}

fn holds(view: &View<'_>) -> Table {
    let forge = view.forge;
    let mut holds = Table {
        name: "branch holds",
        capacity: u64::from(forge.inspection_holds().capacity()),
        entries: Vec::new(),
        refused: Vec::new(),
    };
    for (name, hold) in forge.inspection_holds() {
        let prefix =
            match forge.repository(skein_forge::api::Repository { forge: name.forge, repository: name.repository }) {
                Some(repository) => format!(
                    "forge/{}/{}/{}",
                    String::from_utf8_lossy(&repository.host),
                    String::from_utf8_lossy(&repository.owner),
                    String::from_utf8_lossy(&repository.name)
                ),
                None => format!("forge/{}/repository/{}", name.forge, name.repository),
            };
        let resource = match &name.what {
            What::Branch(parts) => format!(
                "{prefix}/branch/{}",
                parts.iter().map(|part| String::from_utf8_lossy(part)).collect::<Vec<_>>().join("/")
            ),
            What::Repository => prefix,
            What::Pull(number) => format!("{prefix}/pull/{number}"),
            What::Issue(number) => format!("{prefix}/issue/{number}"),
        };
        let ended_forge = forge.forge_ended(name.forge);
        holds.entries.push(Entry {
            key: format!("{resource} (task {})", hold.task),
            rule: if ended_forge {
                "the forge's return or a person's act"
            } else {
                "the closing settlement of the landing or change that holds it"
            },
            // An ended forge cannot settle the holder's closing. Only the root
            // has the task's terminal, as distinct from its landed phase.
            release: if !ended_forge
                && view.root.is_some()
                && view.ended(hold.task)
                && forge.inspection_releases().get(&hold.task).is_none_or(|release| release.failed)
            {
                Release::Due
            } else {
                Release::Pending
            },
        });
    }
    holds
}

fn landings(view: &View<'_>) -> Table {
    let forge = view.forge;
    let mut landings = Table {
        name: "landing rows",
        capacity: u64::from(forge.inspection_landings().capacity()),
        entries: Vec::new(),
        refused: Vec::new(),
    };
    for (task, row) in forge.inspection_landings() {
        let release = if landing_ended(&row.landing.phase) || view.ended(*task) {
            let unhanded = row.asks.iter().enumerate().any(|(index, ask)| {
                let Some(ask) = ask else { return false };
                let Some(member) = row.landing.members.get(index) else { return true };
                !forge.change(member.change).is_some_and(|change| change.change.landing_ask.as_deref() == Some(ask))
            });
            if !row.entries.is_empty() || unhanded { Release::Due } else { Release::Kept }
        } else {
            Release::Pending
        };
        landings.entries.push(Entry {
            key: format!("landing {task}"),
            rule: "the closing settlement of the landing",
            release,
        });
    }
    landings
}

pub(super) fn inspect(view: &View<'_>, into: &mut Inspection) {
    let mut writes = writes(view);
    let mut holds = holds(view);
    if let Some(root) = view.root {
        for hold in root.forge_limit_holds() {
            match hold.set {
                LimitSet::Entries => writes.refused.push(refused(hold.task)),
                LimitSet::Holds => holds.refused.push(refused(hold.task)),
                _ => {}
            }
        }
    }
    into.table(writes);
    into.table(reads(view));
    into.table(holds);
    into.table(landings(view));
}

fn refused(task: u64) -> Refused {
    Refused { key: task_name(task), since: Time::ZERO, bound: None }
}
