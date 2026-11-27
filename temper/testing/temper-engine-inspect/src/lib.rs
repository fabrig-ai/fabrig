//! The engine's frozen waits and bounded tables (temper/docs/design/testing/engine-checks.md, 2).
mod landings;
mod tables;
mod waits;

pub use tables::check_room_census;

use jig_domain_tasks as tasks;
use skein_world::inspect::{Inspection, Wake};
use temper_engine_domain::Domain;
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_change::State;
use temper_engine_domain_forge_landing::Phase;

/// Every documented landing wait, change step and making wait, in their design's words.
pub const KINDS: &[&str] = &[
    "landing: release",
    "landing: ready",
    "landing: turn",
    "landing: prepare",
    "landing: prefixes",
    "landing: open",
    "landing: close",
    "landing: delete",
    "landing: post",
    "landing: run",
    "landing: receipt",
    "landing: judgement",
    "landing: merge",
    "landing: record",
    "landing: ancestry",
    "landing: read",
    "landing: room",
    "change: producing",
    "change: opening",
    "change: checking",
    "change: gating",
    "change: queued",
    "change: first",
    "change: landing",
    "change: ready",
    "change: preparing",
    "change: judging",
    "change: updating",
    "change: resolving",
    "change: repairing",
    "change: cancelling gates",
    "change: held",
    "change: room",
    "making: room",
];

/// The root's forge waits and tables, using its task arena for ownership and prepare jobs.
pub fn inspect(root: &Domain, into: &mut Inspection) {
    let view = View::new(root.forge(), Some(root));
    landings::inspect(&view, into, |task| root.tasks().procedure_due(task).is_some());
    waits::inspect(&view, into);
    tables::inspect(&view, into);
}

/// The forge component alone, as driven by the forge world.
pub fn forge(forge: &forge::Domain, into: &mut Inspection) {
    forge_with_due(forge, into, |_| false);
}

/// Include wakes already delivered to the forge world's scripted root.
pub fn forge_with_due(forge: &forge::Domain, into: &mut Inspection, due: impl Fn(u64) -> bool) {
    let view = View::new(forge, None);
    landings::inspect(&view, into, due);
    waits::inspect(&view, into);
    tables::inspect(&view, into);
}

struct View<'a> {
    forge: &'a forge::Domain,
    root: Option<&'a Domain>,
    calls: Vec<Call>,
}

struct Call {
    number: u64,
    task: Option<u64>,
    read: bool,
    answered: bool,
}

impl<'a> View<'a> {
    fn new(forge: &'a forge::Domain, root: Option<&'a Domain>) -> Self {
        use skein_forge_client::CallOwner;
        let mut facts = forge.inspection_client().inspection_call_facts();
        facts.sort_unstable_by_key(|(number, _, _)| *number);
        let calls = facts
            .iter()
            .map(|&(number, owner, answered)| {
                let task = match owner {
                    CallOwner::Read(owner) => {
                        forge.inspection_read_task(*owner).or_else(|| forge.inspection_brief_task(*owner))
                    }
                    CallOwner::Entry(entry) => forge.inspection_entry(*entry).map(|entry| entry.task),
                    CallOwner::Resource(_) | CallOwner::Repository(_) => None,
                };
                Call { number, task, read: matches!(owner, CallOwner::Read(_)), answered }
            })
            .collect();
        Self { forge, root, calls }
    }

    fn ended(&self, task: u64) -> bool {
        if let Some(root) = self.root {
            return root.tasks().task(task).is_none_or(|row| matches!(row.phase, tasks::Phase::Ended(_)));
        }
        if let Some(row) = self.forge.landing(task) {
            return landing_ended(&row.landing.phase);
        }
        self.forge.change(task).is_some_and(|row| matches!(row.change.state, State::Ended))
    }

    fn waiter(&self, task: u64) -> Wake {
        if self.ended(task) { Wake::Ended(task_name(task)) } else { Wake::Waiter(task_name(task)) }
    }

    fn prepare(&self, row: &forge::LandingRow) -> Vec<Wake> {
        if let Some(root) = self.root
            && let Some(task) = root.tasks().task(row.landing.task)
        {
            for child in task.delegates.iter().rev() {
                if let Some(child) = root.tasks().task(*child)
                    && child.executor == (tasks::Executor::Job { kind: forge::PREPARE_KIND })
                    && !matches!(child.phase, tasks::Phase::Ended(_))
                {
                    return vec![Wake::Waiter(task_name(child.number))];
                }
            }
        }
        // Preparing persists while the answered round opens its pull or retires
        // a stale prefix. The outstanding write, not the ended job, wakes it.
        let writes: Vec<_> = row
            .entries
            .iter()
            .filter(|entry| {
                entry.preparation == Some(row.landing.preparation) && self.forge.inspection_entry(entry.entry).is_some()
            })
            .map(|entry| Wake::Party(format!("forge: entry {}", entry.entry)))
            .collect();
        if !writes.is_empty() {
            return writes;
        }
        if self.root.is_none() && row.job.as_ref().is_none_or(|job| job.step.outcome.is_none()) {
            return vec![Wake::Party("prepare job".into())];
        }
        vec![Wake::Ended("prepare job".into())]
    }

    fn read_wakes(&self, task: u64) -> Vec<Wake> {
        let mut wakes = Vec::new();
        for call in &self.calls {
            if call.read && !call.answered && call.task == Some(task) {
                wakes.push(Wake::Party(format!("forge: call {}", call.number)));
            }
        }
        // A gather refused client admission retains its read-line timer even
        // after its merge write ended and before another call is admitted.
        if self.root.is_none_or(|root| root.tasks().procedure_due(task).is_some())
            && let Some(at) = self.forge.inspection_landing_read_retry(task)
        {
            wakes.push(Wake::Timer(at));
        }
        // Restart freshness is owned by the client's resource reader, not a
        // change's read token (domain/forge/catching-up.md, 2 and 4).
        for resource in self.forge.inspection_restart_reads(task) {
            wakes.push(Wake::Party(format!(
                "forge: restart read {}/{}/{:?}",
                resource.repository.forge, resource.repository.repository, resource.what
            )));
        }
        for owner in self.forge.inspection_retry_readers() {
            if self.forge.inspection_read_task(owner) == Some(task)
                && let Some(at) = self.forge.inspection_read_retry(owner)
            {
                wakes.push(Wake::Timer(at));
            }
        }
        wakes
    }

    fn answerer(&self, task: u64) -> Wake {
        if let Some(root) = self.root
            && let Some(row) = root.tasks().task(task)
            && let tasks::Party::Task(requester) = row.requester
        {
            return self.waiter(requester);
        }
        Wake::Party("person".into())
    }
}

fn task_name(task: u64) -> String {
    format!("task {task}")
}

fn landing_ended(phase: &Phase) -> bool {
    matches!(phase, Phase::Landed | Phase::Cancelled)
}
