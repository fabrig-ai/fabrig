//! Nonrecursive chip facts (`client/domain/objects.md`, 3 and 5). Durable
//! lifecycle clocks remain distinct from best-effort engine receipt clocks.

use crate::{Core, Limits, documents::Missing};
pub use jig_domain_core_vocabulary::documents::chip_phase::*;
use jig_domain_fleet as fleet;
use jig_domain_link::Operation;
use jig_domain_tasks as tasks;
use skein_lib::{List, Token};

fn current_attempt(row: &tasks::TaskRecord) -> Option<u64> {
    match row.phase {
        tasks::Phase::Active(tasks::Active::Claimed { attempt } | tasks::Active::Running { attempt }) => Some(attempt),
        tasks::Phase::Waiting
        | tasks::Phase::Active(
            tasks::Active::Idle | tasks::Active::Due | tasks::Active::Preparing | tasks::Active::BackingOff { .. },
        )
        | tasks::Phase::Closing(_)
        | tasks::Phase::Held { .. }
        | tasks::Phase::Ended(_) => None,
    }
}

impl Core {
    /// Whether the task's current run is past Starting (the Task document's chip.work).
    #[must_use]
    pub fn run_past_starting(&self, task: u64) -> bool {
        let Some(row) = self.tasks.task(task) else { return false };
        let Some(attempt) = current_attempt(row) else { return false };
        let Some(observation) = self.fleet.observation(Token::new(task), Token::new(attempt)) else { return false };
        match observation.activity {
            fleet::Activity::Preparing | fleet::Activity::Starting => false,
            fleet::Activity::Active
            | fleet::Activity::Writing
            | fleet::Activity::Long { .. }
            | fleet::Activity::Tool { .. }
            | fleet::Activity::Waiting
            | fleet::Activity::Ending => true,
        }
    }
}

#[expect(clippy::match_same_arms, reason = "activity variants retain their domain order")]
pub(super) fn observe(
    core: &Core,
    limits: &Limits,
    row: &tasks::TaskRecord,
    details: &mut ChipDetails,
) -> Result<(), Missing> {
    details.dependencies = super::task_tree::dependency_facts(core, limits, row);
    for index in 0..details.resources.len() {
        let holding = row.holdings.get(skein_lib::length::usize_of(index)).expect("one resource per holding");
        let name = match holding {
            tasks::Holding::Write { resource, .. } => resource,
            tasks::Holding::Slot { pool, .. } => pool,
        };
        let resource = details.resources.get_mut(index).expect("bounded resource index");
        resource.holder = core.tasks.holder(name);
        if let Some(slot) = core.tasks.writer(name) {
            resource.writer = Some(slot.writer);
            resource.writer_lost = slot.lost;
        }
    }
    details.resources_observed = true;
    let closing = match &row.phase {
        tasks::Phase::Closing(_) => true,
        tasks::Phase::Held { was, .. } => match was {
            tasks::Was::Closing(_) => true,
            tasks::Was::Waiting | tasks::Was::Active(_) => false,
        },
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Ended(_) => false,
    };
    if closing {
        let mut connectors = List::with_capacity(limits.documents.lines);
        for ((task, connector), ()) in &core.closing_connectors {
            if *task == row.number {
                if connectors.room() == 0 {
                    return Err(Missing::TooLarge);
                }
                connectors.push(*connector).expect("whole connector list room checked");
            }
        }
        details.closing_connectors = Some(connectors);
    }
    let current = current_attempt(row);
    if let Some(attempt) = current
        && let Some(observation) = core.fleet.observation(Token::new(row.number), Token::new(attempt))
    {
        let work = match &observation.activity {
            fleet::Activity::Preparing => ChipWork::Preparing,
            fleet::Activity::Starting => ChipWork::Starting,
            fleet::Activity::Active => ChipWork::Active,
            fleet::Activity::Writing | fleet::Activity::Long { operation: Operation::Completion, .. } => {
                ChipWork::Writing
            }
            fleet::Activity::Long { operation: Operation::Checks | Operation::Command, .. } => ChipWork::Active,
            fleet::Activity::Waiting => ChipWork::Waiting,
            fleet::Activity::Ending => ChipWork::Ending,
            fleet::Activity::Tool { call, tool } => {
                let bound = skein_lib::length::usize_of(limits.documents.text_bytes);
                if call.len() > bound || tool.len() > bound {
                    return Err(Missing::TooLarge);
                }
                ChipWork::Tool { call: skein_lib::bytes::copy_of(call), tool: skein_lib::bytes::copy_of(tool) }
            }
        };
        details.observation = Some(ChipObservation {
            channel: observation.channel,
            work,
            since: observation.since,
            reported_at: observation.reported_at,
            started: observation.started,
        });
    }
    Ok(())
}
