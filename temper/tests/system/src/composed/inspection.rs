//! Cross-root quiet-point checks (temper/docs/design/testing/checks.md, 6.1).
use super::{World, engine, host, process::Process, worker};
use jig_inspect::{HOST_CALLS, PLACEMENTS};
use skein_world::inspect::{Inspection, Moment, Related, check_relations, check_tables, check_waits};
use worker::agent;

/// Inspection buffers belong to the world, not to any of the roots it drives.
pub(super) struct Scratch {
    jig: Inspection,
    links: Inspection,
    worker: Inspection,
    hubs: Vec<Inspection>,
    smiths: Vec<Inspection>,
    smith_count: usize,
}

impl Default for Scratch {
    fn default() -> Self {
        Self {
            jig: Inspection::new("jig", jig_inspect::KINDS),
            links: Inspection::new("links", &[]),
            worker: temper_worker_inspect::inspection(),
            hubs: Vec::new(),
            smiths: Vec::new(),
            smith_count: 0,
        }
    }
}

impl Scratch {
    fn observe(&mut self, world: &World, moment: Moment) -> Result<(), String> {
        self.jig.reset();
        jig_inspect::core(world.root.core(), &mut self.jig);
        self.links.reset();
        world.inspect_links(&mut self.links);
        self.hubs.resize_with(world.lanes.len(), || Inspection::new("jig host", &[]));
        self.smith_count = 0;
        for (lane, hub) in world.lanes.iter().zip(&mut self.hubs) {
            hub.reset();
            jig_inspect::host(lane.worker.host(), hub);
            // Checkout tokens are local to each lane. No checkout wait names a waiter outside its worker.
            self.worker.reset();
            temper_worker_inspect::inspect(&lane.worker, &lane.worker_env.limits, &lane.disk, &mut self.worker);
            shared(&self.worker, moment)?;
        }
        for process in world.lanes.iter().map(|lane| &lane.process).chain(world.parked.values()) {
            let Some((smith, task, _)) = process.activation() else { continue };
            if self.smith_count == self.smiths.len() {
                self.smiths.push(Inspection::new("smith", smith_inspect::KINDS));
            }
            let inspection = &mut self.smiths[self.smith_count];
            self.smith_count += 1;
            inspection.reset();
            smith_inspect::inspect(smith.domain(), inspection);
            smith_inspect::host_calls(smith.domain(), task, inspection);
            // The process drives its own clock. Its waits name only local or outside-party wakes.
            shared(inspection, Moment { at: smith.now(), ..moment })?;
        }
        Ok(())
    }

    fn related(&self, links: &Inspection, moment: Moment) -> Result<(), String> {
        let inspections: Vec<_> =
            [&self.jig, links].into_iter().chain(&self.hubs).chain(&self.smiths[..self.smith_count]).collect();
        check_relations(&inspections, &[&HOST_CALLS, &PLACEMENTS], moment)
    }
}

impl World {
    pub(super) fn composed_checks(&mut self) -> Result<(), String> {
        let moment = self.inspection_moment();
        let mut scratch = std::mem::take(&mut self.inspections);
        let result = scratch
            .observe(self, moment)
            .and_then(|()| self.checks.quiet_with(&self.root, moment.at, moment.iteration, &[&scratch.jig]))
            .and_then(|()| scratch.related(&scratch.links, moment));
        self.inspections = scratch;
        result?;
        self.workload_bounds();
        Ok(())
    }

    /// At a held host call's quiet point, remove its matching relay from a copy of the link inspection.
    pub fn without_held_host_relay(&self) -> Result<(), String> {
        assert!(self.engine_quiet() && self.lanes[0].process.host_call_held());
        let scratch = &self.inspections;
        let mut links = Inspection::new("links", &[]);
        let mut calls = scratch.links.relateds().iter().filter(|row| row.relation == HOST_CALLS.name);
        let call = calls.next().expect("the snapshot holds a host call in flight");
        assert!(calls.next().is_none(), "the snapshot holds just one host call in flight");
        for row in scratch.links.relateds().iter().filter(|row| *row != call) {
            links.related(row.clone());
        }
        scratch.related(&links, self.inspection_moment())
    }

    fn inspection_moment(&self) -> Moment {
        Moment { seed: self.seed, at: self.env.now, iteration: self.iteration }
    }

    fn inspect_links(&self, into: &mut Inspection) {
        for lane in &self.lanes {
            for event in &lane.worker_events {
                worker_event(event, &lane.process, into);
            }
            process_link(&lane.process, into);
        }
        for process in self.parked.values() {
            process_link(process, into);
        }
        // The engine inbox and both outboxes have drained at this quiet point.
        for link in &self.absence.outbound {
            engine_link(link, into);
        }
        if let Some(story) = &self.merge_job {
            if let Some((_, assignment)) = &story.pending {
                placement(assignment.task, assignment.attempt, into);
            }
            if let Some(link) = &story.held {
                engine_link(link, into);
            }
        }
        // The replay fixture holds the first answer until the retransmit crosses.
        if let Some((_, Some(down))) = &self.replay {
            link_down(down, into);
        }
    }
}

fn shared(inspection: &Inspection, moment: Moment) -> Result<(), String> {
    let only = [inspection];
    inspection.check_violation(moment)?;
    check_waits(&only, moment)?;
    check_tables(&only, moment)
}

fn worker_event(event: &worker::Event, process: &Process, into: &mut Inspection) {
    match event {
        worker::Event::Assign { assignment, .. } => {
            placement(assignment.assignment.run.raw(), assignment.assignment.attempt.raw(), into);
        }
        worker::Event::AssignJob { job, .. } => placement(job.run.raw(), job.attempt.raw(), into),
        worker::Event::Link(down) => link_down(down, into),
        worker::Event::RelayCancelled { run, attempt, call } => encoded_call(run.raw(), attempt.raw(), call, into),
        worker::Event::Process(event) => {
            if let agent::process::Event::Received { message, .. } = event.as_ref()
                && let Some((_, task, attempt)) = process.activation()
            {
                process_up(message, task, attempt, into);
            }
        }
        worker::Event::Encoded { encoded: worker::Encoded::CallName(name), .. } => {
            if let Some((smith, task, attempt)) = process.activation()
                && let Some(call) = jig_protocol_charter::call_key(name, attempt)
                && smith.host_submissions().last().is_some_and(|submitted| {
                    (submitted.name.activation, submitted.name.completion, submitted.name.position)
                        == (attempt, call.completion, call.position)
                })
            {
                host_call(task, attempt, call.completion, call.position, into);
            }
        }
        _ => {}
    }
}

fn process_link(process: &Process, into: &mut Inspection) {
    let Some((smith, task, attempt)) = process.activation() else { return };
    for message in process.channel_up() {
        process_up(message, task, attempt, into);
    }
    if let Some(call) = smith.queued_host_reply() {
        host_call(task, call.activation, call.completion, call.position, into);
    }
}

fn process_up(message: &agent::Up, task: u64, attempt: u64, into: &mut Inspection) {
    match message {
        agent::Up::Call { name, ask: agent::Ask::Host { .. }, .. } => {
            host_call(task, name.activation, name.completion, name.position, into);
        }
        agent::Up::Answer { .. } => placement(task, attempt, into),
        _ => {}
    }
}

fn engine_link(link: &engine::Link, into: &mut Inspection) {
    if let engine::Link::Up { up, .. } = link {
        match up {
            host::LinkUp::Call { run, attempt, call, .. } => encoded_call(run.raw(), attempt.raw(), call, into),
            host::LinkUp::Answer { run, attempt, .. } | host::LinkUp::JobAnswer { run, attempt, .. } => {
                placement(run.raw(), attempt.raw(), into);
            }
            _ => {}
        }
    }
}

fn link_down(down: &host::LinkDown, into: &mut Inspection) {
    match down {
        host::LinkDown::Assign { assignment } => {
            placement(assignment.assignment.run.raw(), assignment.assignment.attempt.raw(), into);
        }
        host::LinkDown::AssignJob { job } => placement(job.run.raw(), job.attempt.raw(), into),
        host::LinkDown::Relayed { run, attempt, call, .. } => encoded_call(run.raw(), attempt.raw(), call, into),
        _ => {}
    }
}

fn encoded_call(task: u64, attempt: u64, name: &[u8], into: &mut Inspection) {
    if let Some(call) = jig_protocol_charter::call_key(name, attempt) {
        host_call(task, attempt, call.completion, call.position, into);
    }
}

fn host_call(task: u64, attempt: u64, completion: u32, position: u32, into: &mut Inspection) {
    into.related(Related {
        relation: HOST_CALLS.name,
        key: format!("task {task} attempt {attempt} completion {completion} position {position}"),
        holds: "a relay in flight",
    });
}

fn placement(run: u64, attempt: u64, into: &mut Inspection) {
    into.related(Related {
        relation: PLACEMENTS.name,
        key: format!("run {run} attempt {attempt}"),
        holds: "a relay in flight",
    });
}
