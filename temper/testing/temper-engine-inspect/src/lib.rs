//! The engine's frozen waits and bounded tables (temper/docs/design/testing/engine-checks.md, 2).
mod tables;
mod waits;

use jig_domain_tasks as tasks;
use skein_world::inspect::{Inspection, Wake};
use temper_engine_domain::Domain;
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_change::State;
use temper_engine_domain_forge_landing::Phase;

/// Every documented landing wait and change step, in their design's words.
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
];

/// The root's forge waits and tables, using its task arena for ownership and prepare jobs.
pub fn inspect(root: &Domain, into: &mut Inspection) {
    let view = View { forge: root.forge(), root: Some(root) };
    waits::inspect(&view, into);
    tables::inspect(&view, into);
}

/// The forge component alone, as driven by the forge world.
pub fn forge(forge: &forge::Domain, into: &mut Inspection) {
    let view = View { forge, root: None };
    waits::inspect(&view, into);
    tables::inspect(&view, into);
}

struct View<'a> {
    forge: &'a forge::Domain,
    root: Option<&'a Domain>,
}

impl View<'_> {
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

    fn prepare(&self, task: u64) -> Wake {
        let Some(root) = self.root else { return Wake::Party("prepare job".into()) };
        if let Some(row) = root.tasks().task(task) {
            for child in row.delegates.iter().rev() {
                if let Some(child) = root.tasks().task(*child)
                    && child.executor == (tasks::Executor::Job { kind: forge::PREPARE_KIND })
                    && !matches!(child.phase, tasks::Phase::Ended(_))
                {
                    return Wake::Waiter(task_name(child.number));
                }
            }
        }
        Wake::Ended("prepare job".into())
    }

    fn read_task(&self, call: u64) -> Option<u64> {
        let client = self.forge.inspection_client();
        match client.inspection_call_owner(call) {
            Some(skein_forge_client::CallOwner::Read(owner)) => {
                self.forge.inspection_read_task(*owner).or_else(|| self.forge.inspection_brief_task(*owner))
            }
            _ => None,
        }
    }

    fn read_wakes(&self, task: u64) -> Vec<Wake> {
        let client = self.forge.inspection_client();
        let mut calls = client.inspection_calls();
        calls.sort_unstable();
        let mut wakes = Vec::new();
        for call in calls {
            if !client.inspection_answered_call(call) && self.read_task(call) == Some(task) {
                wakes.push(Wake::Party(format!("forge: call {call}")));
            }
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
