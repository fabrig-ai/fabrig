//! Signal and restart boundaries, independently decoded (temper/docs/design/protocol/README.md, 6).
use super::{AgentRecord, Store};
use crate::world::{Counts, Proc, Stopped, Story};
use jig_domain_core::{CoreRecord, Record};
use jig_domain_tasks::{self as tasks, TaskRecord};
use temper_engine_domain::Record as Row;

fn task(store: &Store, number: u64) -> &TaskRecord {
    store
        .rows
        .values()
        .find_map(|row| match row {
            Row::Core(Record::Tasks(tasks::Stored::Live(task))) if task.number == number => Some(task),
            _ => None,
        })
        .expect("stopped task remains live")
}

fn producer(store: &Store) -> Option<u64> {
    store.rows.values().find_map(|row| match row {
        Row::Forge { row } => match row.as_ref() {
            temper_engine_domain_forge::Stored::Change(change) => change.delegate.as_ref().map(|(task, _)| *task),
            _ => None,
        },
        Row::Core(_) => None,
    })
}

fn counts(store: &Store, number: u64) -> Option<Counts> {
    store.rows.values().find_map(|row| match row {
        Row::Forge { row } => match row.as_ref() {
            temper_engine_domain_forge::Stored::Change(row)
                if row.delegate.as_ref().is_some_and(|(task, _)| *task == number) =>
            {
                let change = &row.change;
                Some(Counts {
                    repairs: change.repairs,
                    resolutions: change.resolutions,
                    updates: change.updates,
                    gate_failures: change.gate_failures.iter().map(|gate| (gate.number, gate.count)).collect(),
                })
            }
            _ => None,
        },
        Row::Core(_) => None,
    })
}

pub(super) fn ready(scenario: Story, store: &Store, other: &[Proc]) -> bool {
    let number = if matches!(scenario, Story::CodingAtStop) {
        let Some(number) = producer(store) else { return false };
        let row = task(store, number);
        if row.saved.is_empty()
            || row.turn == 0
            || !matches!(row.phase, tasks::Phase::Active(tasks::Active::Running { .. }))
        {
            return false;
        }
        let queried = other.iter().any(|proc| {
            let Proc::Peer(peer) = proc else { return false };
            peer.observations().iter().any(|seen| {
                let skein_fake_peers::llm::Observation::Query { query, .. } = seen else { return false };
                query
                    .messages
                    .iter()
                    .flat_map(|message| message.parts.iter())
                    .filter(|part| {
                        matches!(part,
                            skein_fake_llm_domain::api::Part::ToolCall { name, .. } if name.as_ref() == b"shell"
                        )
                    })
                    .count()
                    >= 2
            })
        });
        if !queried {
            return false;
        }
        number
    } else {
        1
    };
    store.rows.values().any(|row| matches!(row,
        Row::Core(Record::Core(CoreRecord::Turn(turn))) if turn.task == number && turn.attempt == task(store, number).attempt && turn.turn > 0
    ))
}

pub(super) fn at_signal(scenario: Story, store: &Store) -> Stopped {
    let number = if matches!(scenario, Story::CodingAtStop) { producer(store).expect("producer") } else { 1 };
    let row = task(store, number);
    Stopped {
        task: number,
        attempt: row.attempt,
        interrupted: 0,
        tries: [row.tries; 2],
        saved: [row.saved.clone(), row.saved.clone()],
        change: counts(store, number).map(|counts| [counts.clone(), counts]),
        next: None,
        writers: Vec::new(),
        workspace: None,
        saved_head: [None; 2],
        checkout_head: None,
        checkout_file: None,
    }
}

pub(super) fn interrupted(stopped: &Stopped, store: &Store) -> u32 {
    u32::try_from(store.rows.values().filter(|row| matches!(row,
        Row::Core(Record::Core(CoreRecord::Terminal(end))) if end.task == stopped.task && end.attempt == stopped.attempt && end.end == tasks::End::Recheck
    )).count()).expect("interrupted terminals")
}

pub(super) fn next<'a>(stopped: &Stopped, store: &'a Store) -> Option<(u64, &'a Option<smith_channel::Workspace>)> {
    store.agent_records.iter().find_map(|record| match record {
        AgentRecord::Start { task, attempt, workspace, .. } if *task == stopped.task && *attempt > stopped.attempt => {
            Some((*attempt, workspace))
        }
        _ => None,
    })
}

fn accounting(stopped: &mut Stopped, store: &Store) {
    let row = task(store, stopped.task);
    stopped.tries[1] = row.tries;
    stopped.saved[1].clone_from(&row.saved);
    if let Some(counts) = stopped.change.as_mut() {
        counts[1] = self::counts(store, stopped.task).expect("change remains");
    }
}

pub(super) fn at_restart(stopped: &mut Stopped, store: &Store) {
    accounting(stopped, store);
    let row = task(store, stopped.task);
    // A lost terminal closes a failing row before its retries can hide the cause.
    let (attempt, workspace) = next(stopped, store).unwrap_or_else(|| {
        panic!(
            "stopped task {} attempt {}: tries.lost == {}; Recheck terminals == {}; expected lost 0 and Recheck 1",
            stopped.task, stopped.attempt, stopped.tries[1].lost, stopped.interrupted
        )
    });
    stopped.next = Some(attempt);
    stopped.workspace.clone_from(workspace);
    // Check every held write resource, including a missing slot, against the new claim.
    for holding in &row.holdings {
        if let tasks::Holding::Write { resource, .. } = holding {
            let slot = store
                .rows
                .values()
                .find_map(|row| match row {
                    Row::Core(Record::Tasks(tasks::Stored::Writer(slot))) if &slot.resource == resource => Some(slot),
                    _ => None,
                })
                .expect("new claim owns each required writer slot");
            assert_eq!(slot.writer, tasks::Writer::Run { task: stopped.task, attempt });
        }
    }
    stopped.writers = store
        .rows
        .values()
        .filter_map(|row| match row {
            Row::Core(Record::Tasks(tasks::Stored::Writer(slot))) => match slot.writer {
                tasks::Writer::Run { task, attempt } if task == stopped.task => Some((attempt, slot.lost)),
                _ => None,
            },
            _ => None,
        })
        .collect();
}

/// Read the prepared checkout before the restarted agent can change its tree.
pub(super) fn checkout(stopped: &mut Stopped, disk: &mut skein_fake_machine::Machine) {
    let Some(workspace) = &stopped.workspace else { return };
    let directory = workspace.directories().as_slice().first().expect("prepared repository");
    let at = directory.path().strip_prefix(b"/worker/").expect("checkout below worker root");
    let root = disk.reopen_root(1);
    let mut read = |suffix: &[u8]| {
        let file =
            disk.open(root, &[at, suffix].concat(), skein_fake_machine::How::ReadNoFollow).expect("checkout file");
        let bytes = disk.read(file, 0, 4096).expect("checkout bytes");
        disk.close(file);
        bytes
    };
    stopped.checkout_head = Some(u64::from_le_bytes(read(b"/.git/temper-head").try_into().expect("counted git head")));
    stopped.checkout_file = Some(read(b"/conflict.txt"));
    disk.close(root);
}

/// Keep accounting through the restart's final drain as well as its Start boundary.
pub(super) fn after_drain(stopped: &mut Stopped, store: &Store) {
    accounting(stopped, store);
    assert!(
        !store.rows.values().any(|row| matches!(row,
            Row::Core(Record::Tasks(tasks::Stored::Writer(slot)))
                if slot.writer == tasks::Writer::Run { task: stopped.task, attempt: stopped.attempt }
        )),
        "no writer slot retains the stopped attempt"
    );
}
