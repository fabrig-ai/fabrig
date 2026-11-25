use crate::{View, landing_ended, task_name};
use skein_lib::Time;
use skein_world::inspect::{Entry, Inspection, Refused, Release, Table};
use std::fmt::Write as _;
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
            // Before handoff there is no client write to have settled
            // (testing/engine-checks.md, 2.3; domain/forge/room.md, 2).
            release: if view.ended(entry.task)
                && forge.inspection_entry_handed(number)
                && forge.inspection_client().inspection_entry(number).is_none()
            {
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
    for call in &view.calls {
        let number = call.number;
        reads.entries.push(Entry {
            key: match call.task {
                Some(task) => format!("call {number} (task {task})"),
                None => format!("call {number}"),
            },
            rule: "the read's answer",
            release: if call.answered { Release::Due } else { Release::Pending },
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
        entries: Vec::with_capacity(skein_lib::usize_of(forge.inspection_holds().len())),
        refused: Vec::new(),
    };
    for (name, hold) in forge.inspection_holds() {
        let mut key =
            match forge.repository(skein_forge::api::Repository { forge: name.forge, repository: name.repository }) {
                Some(repository) => format!(
                    "forge/{}/{}/{}",
                    String::from_utf8_lossy(&repository.host),
                    String::from_utf8_lossy(&repository.owner),
                    String::from_utf8_lossy(&repository.name)
                ),
                None => format!("forge/{}/repository/{}", name.forge, name.repository),
            };
        match &name.what {
            What::Branch(parts) => {
                key.push_str("/branch/");
                let mut parts = parts.iter();
                if let Some(first) = parts.next() {
                    key.push_str(&String::from_utf8_lossy(first));
                }
                for part in parts {
                    key.push('/');
                    key.push_str(&String::from_utf8_lossy(part));
                }
            }
            What::Repository => {}
            What::Pull(number) => write!(key, "/pull/{number}").expect("write into a string"),
            What::Issue(number) => write!(key, "/issue/{number}").expect("write into a string"),
        }
        write!(key, " (task {})", hold.task).expect("write into a string");
        let ended_forge = forge.forge_ended(name.forge);
        holds.entries.push(Entry {
            key,
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
            // An ended landing retains its sent writes until their outcomes
            // settle (domain/forge/room.md, 2; testing/engine-checks.md, 2.3).
            // A reference whose outbox entry is gone has outlived that rule.
            let retired = row.entries.iter().any(|entry| forge.inspection_entry(entry.entry).is_none());
            if retired || unhanded {
                Release::Due
            } else if row.entries.is_empty() {
                Release::Kept
            } else {
                Release::Pending
            }
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
