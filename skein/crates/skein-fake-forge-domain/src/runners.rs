//! Ephemeral, forge-wide dispatch admission (testing/fakes/forge.md, 2.3).
use crate::{Domain, Fault, Fell, Limits, store::Repository};
use skein_lib::{Id, Map, Time};

#[derive(Debug)]
pub(crate) struct Pool {
    pub(crate) limit: Option<u32>,
    held: Map<u64, u32>,
    waiting: Map<u64, Waiting>,
    order: u64,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Waiting {
    pub(crate) repository: Id<Repository>,
    pub(crate) run: u64,
    pub(crate) job: u64,
    at: Time,
    order: u64,
}

impl Pool {
    pub(crate) fn new(limits: &Limits) -> Self {
        let capacity = limits
            .repositories
            .checked_mul(limits.workflow_runs)
            .expect("validated run bound")
            .checked_mul(limits.contexts)
            .expect("validated job bound");
        Self { limit: None, held: Map::with_capacity(capacity), waiting: Map::with_capacity(capacity), order: 0 }
    }

    pub(crate) fn capacity(&self) -> u32 {
        self.waiting.capacity()
    }

    pub(crate) fn waiting_count(&self) -> u32 {
        self.waiting.len()
    }

    pub(crate) fn available(&self) -> bool {
        self.limit.is_none() || self.free().is_some()
    }

    pub(crate) fn waiting(&self, job: u64) -> bool {
        self.waiting.contains_key(&job)
    }

    pub(crate) fn runner(&self, job: u64) -> Option<u32> {
        self.held.get(&job).copied()
    }

    fn free(&self) -> Option<u32> {
        let limit = self.limit?;
        for runner in 1..=limit.min(self.held.len().saturating_add(1)) {
            let mut held = false;
            for (_, &used) in &self.held {
                if used == runner {
                    held = true;
                    break;
                }
            }
            if !held {
                return Some(runner);
            }
        }
        None
    }

    pub(crate) fn next(&self) -> Option<Waiting> {
        let mut next: Option<Waiting> = None;
        for (_, entry) in &self.waiting {
            let earlier = match next {
                Some(first) => entry.order < first.order,
                None => true,
            };
            if earlier {
                next = Some(*entry);
            }
        }
        next
    }

    pub(crate) fn forget(&mut self, job: u64) {
        self.release(job);
    }

    pub(crate) fn release(&mut self, job: u64) -> Option<Waiting> {
        self.held.remove(&job);
        self.waiting.remove(&job)
    }
}

/// Only jobs past their declared wait, labels and needs enter this queue.
pub(crate) fn take(domain: &mut Domain, at: Time, repository: Id<Repository>, run: u64, job: u64) -> bool {
    let pool = &mut domain.runners;
    let runner = pool.free();
    if pool.limit.is_some() {
        let first = pool.next();
        let behind = match first {
            Some(first) => first.job != job,
            None => false,
        };
        if runner.is_none() || behind {
            if !pool.waiting(job) {
                pool.order = pool.order.checked_add(1).expect("ready order fits");
                pool.waiting
                    .insert(job, Waiting { repository, run, job, at, order: pool.order })
                    .expect("one queue entry per retained job");
            }
            return false;
        }
        pool.held.insert(job, runner.expect("free runner")).expect("one runner per retained job");
    }
    if let Some(wait) = pool.waiting.remove(&job) {
        end_wait(domain, wait, at, runner);
    }
    true
}

pub(crate) fn end_wait(domain: &mut Domain, wait: Waiting, at: Time, runner: Option<u32>) {
    let mut fell = Fell::repository(domain, wait.at, Fault::RunnerWait, wait.repository);
    fell.until = Some(at);
    fell.run = Some(wait.run);
    fell.commit = Some(
        domain
            .repositories
            .get(wait.repository)
            .expect("repository")
            .dispatched
            .get(&wait.run)
            .expect("waiting run")
            .run
            .commit,
    );
    fell.runner = runner;
    domain.record_fell(fell);
}
