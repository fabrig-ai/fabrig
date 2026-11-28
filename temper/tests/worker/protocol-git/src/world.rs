//! One component, scripted requests above and scripted records below.
use crate::{
    fixtures,
    io::{Below, Input, Invocation, Node, Script},
    referee::{Expected, Referee, Verdict},
};
use skein_lib::{Duration, Rng, Time, Token};
use skein_world::{
    faults::{FaultLedger, Struck},
    inspect::{Moment, check_tables, check_waits},
    streams::{Stream, Streams},
};
use std::collections::BTreeMap;
use temper_worker_domain_checkout::git;
use temper_worker_inspect::{faults::TRASH_STUCK, inspect_trash, inspection};
use temper_worker_protocol_git::{Component, Event, Request};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stop {
    None,
    Deadline(u32),
    Cancel(u32),
    ExitFirst,
    FileDeadline(u32),
    FileCancel(u32),
}
#[derive(Debug)]
pub struct Scenario {
    pub op: git::Op,
    pub script: Script,
    pub stop: Stop,
    pub expected: Expected,
    pub identity: bool,
    pub files: BTreeMap<Vec<u8>, Node>,
}
#[derive(Debug)]
pub struct Outcome {
    pub verdict: Verdict,
    pub terminals: Vec<(Token, git::Done)>,
    pub spawns: u32,
    pub signals: u32,
    pub view_steps: u32,
    pub tree: BTreeMap<Vec<u8>, Node>,
    pub kept: Box<[git::Kept]>,
}
impl Outcome {
    #[must_use]
    pub fn faults(&self) -> Vec<(Token, Option<git::Fault>)> {
        self.terminals
            .iter()
            .map(|(owner, done)| {
                (
                    *owner,
                    match done {
                        git::Done::Failed { fault } | git::Done::FailedWithOutput { fault, .. } => Some(*fault),
                        git::Done::ParentUnreachable { .. }
                        | git::Done::Looked { .. }
                        | git::Done::Results { .. }
                        | git::Done::Paths { .. }
                        | git::Done::Message { .. }
                        | git::Done::Brings { .. }
                        | git::Done::IsAncestor { .. }
                        | git::Done::Merged
                        | git::Done::Conflicted { .. }
                        | git::Done::Succeeded
                        | git::Done::Fetched { .. }
                        | git::Done::Committed { .. }
                        | git::Done::Unchanged
                        | git::Done::Exists
                        | git::Done::Rejected => None,
                    },
                )
            })
            .collect()
    }
}
fn deliver(component: &mut Component, below: &mut Below, now: Time, input: Input) {
    match input {
        Input::Io(event) => {
            assert!(component.accepts(&event), "owned io event");
            component.up(now, event, &mut below.out);
        }
        Input::Kernel(complete) => {
            assert!(component.owns(complete.op));
            component.complete(now, complete, &mut below.out);
        }
    }
    component.reclaim();
}

/// Drain submitted work, without advancing the world's time to future deadlines.
pub fn settle(component: &mut Component, below: &mut Below, now: Time, draw: &mut u64) {
    for _ in 0..10_000 {
        below.drain();
        let Some(input) = below.next(draw) else {
            return;
        };
        deliver(component, below, now, input);
    }
    panic!("git world did not settle");
}

/// A started worker root, retained across operations and their sweep passes.
pub struct World {
    pub component: Component,
    pub below: Below,
    pub ledger: FaultLedger,
    fault_rng: Option<Rng>,
    seed: u64,
    draw: u64,
    now: Time,
    iteration: u64,
    kept: Box<[git::Kept]>,
}
impl World {
    #[must_use]
    pub fn new(seed: u64, tree: BTreeMap<Vec<u8>, Node>) -> Self {
        let mut below =
            Below::new(Script { invocations: [Invocation::captured("version")].into(), ..Script::default() });
        below.tree.extend(tree);
        let mut component =
            Component::new(&fixtures::limits(), &fixtures::checkout(), fixtures::settings()).expect("valid limits");
        let mut draw = seed;
        component.start(Time::ZERO, &mut below.out);
        settle(&mut component, &mut below, Time::ZERO, &mut draw);
        assert_eq!(below.out.events.pop(), Some(Event::Started));
        while let Some(event) = below.out.events.pop() {
            assert!(matches!(event, Event::Unswept { .. }), "unexpected startup event: {event:?}");
        }
        let kept = component.take_kept();
        Self {
            kept,
            component,
            below,
            ledger: FaultLedger::new(),
            fault_rng: None,
            seed,
            draw,
            now: Time::ZERO,
            iteration: 0,
        }
    }

    #[must_use]
    pub fn with_trash_faults(streams: Streams) -> Self {
        let mut world = Self::new(streams.seed(), BTreeMap::new());
        world.draw = streams.seed_of(Stream::Scheduling, "worker");
        world.fault_rng = Some(streams.rng(Stream::Faults, "worker"));
        world
    }

    fn mark_trash(&mut self, scenario: &Scenario) {
        if scenario.op.kind() != git::Kind::Make || self.below.remove_failure.is_some() {
            return;
        }
        let Some(rng) = &mut self.fault_rng else { return };
        let candidates: Vec<_> = self
            .below
            .tree
            .iter()
            .filter(|(path, node)| path.starts_with(b"trash/") && matches!(node, Node::File(_)))
            .map(|(path, _)| path.clone())
            .collect();
        if candidates.is_empty() {
            return;
        }
        let index = usize::try_from(rng.below(skein_lib::len_u64(candidates.len()))).expect("candidate index");
        self.below.remove_failure = Some((candidates[index].clone(), skein_io::kernel::Error::Permission));
        self.ledger.inject(
            TRASH_STUCK,
            self.now,
            Struck { request: Some(format!("hold {} make", fixtures::OWNER.raw())), ..Struck::default() },
        );
    }

    /// Run through the terminal and all file operations then due, before inspection.
    pub fn run(&mut self, scenario: &Scenario) -> Result<Outcome, String> {
        self.below.reset(fixtures::copy_script(&scenario.script));
        self.below.tree.extend(scenario.files.clone());
        let deadline = self.now.checked_add(Duration::from_nanos(fixtures::DEADLINE.as_nanos())).expect("short run");
        self.component.down(
            self.now,
            Request::Io {
                owner: fixtures::OWNER,
                op: scenario.op.clone(),
                deadline,
                identity: scenario.identity.then(fixtures::identity),
            },
            &mut self.below.out,
        );
        let mut referee = Referee::default();
        let mut stopped = false;
        for _ in 0..100_000 {
            self.iteration += 1;
            self.below.drain();
            self.mark_trash(scenario);
            referee.check(&self.below);
            if let Some(event) = self.below.out.events.pop() {
                referee.observe(event, &self.below, fixtures::OWNER, &scenario.expected);
            }
            if !stopped
                && match scenario.stop {
                    Stop::Deadline(at) | Stop::Cancel(at) => self.below.spawns == at,
                    Stop::FileDeadline(at) | Stop::FileCancel(at) => self.below.view_steps == at,
                    Stop::None | Stop::ExitFirst => false,
                }
            {
                stopped = true;
                referee.stop(self.below.spawns);
                match scenario.stop {
                    Stop::Deadline(_) | Stop::FileDeadline(_) => {
                        self.now = deadline;
                        self.component.fire(self.now, &mut self.below.out);
                    }
                    Stop::Cancel(_) | Stop::FileCancel(_) => {
                        self.component.down(
                            self.now,
                            Request::CancelIo { owner: fixtures::OWNER },
                            &mut self.below.out,
                        );
                    }
                    Stop::None | Stop::ExitFirst => unreachable!("stop selected"),
                }
            }
            let Some(input) = self.below.next(&mut self.draw) else {
                break;
            };
            if !stopped
                && scenario.stop == Stop::ExitFirst
                && self.below.script.invocations.is_empty()
                && self.below.closed_children == self.below.spawns
                && self.below.spawns > 0
            {
                stopped = true;
                referee.stop(self.below.spawns);
                self.component.down(self.now, Request::CancelIo { owner: fixtures::OWNER }, &mut self.below.out);
            }
            deliver(&mut self.component, &mut self.below, self.now, input);
        }
        self.below.drain();
        referee.check(&self.below);
        if let Some(event) = self.below.out.events.pop() {
            referee.observe(event, &self.below, fixtures::OWNER, &scenario.expected);
        }
        self.check_trash()?;
        Ok(Outcome {
            verdict: referee.finish(&scenario.expected),
            terminals: referee.terminals,
            spawns: self.below.spawns,
            signals: self.below.signals,
            view_steps: self.below.view_steps,
            tree: self.below.tree.clone(),
            kept: self.kept.clone(),
        })
    }

    pub fn check_trash(&self) -> Result<(), String> {
        let names = self.below.trash();
        let names: Vec<_> = names.iter().map(Vec::as_slice).collect();
        let mut inspected = inspection();
        inspect_trash(&self.component, &names, &mut inspected);
        let moment = Moment { seed: self.seed, at: self.now, iteration: self.iteration };
        check_waits(&[&inspected], moment)?;
        check_tables(&[&inspected], moment)
    }

    pub fn shutdown(mut self) {
        self.component.down(self.now, Request::Shutdown, &mut self.below.out);
        settle(&mut self.component, &mut self.below, self.now, &mut self.draw);
        assert!(self.component.is_empty() && self.below.settled(), "teardown has no live resources");
    }
}

/// Run to the terminal, then shut down and settle every owned descriptor.
#[must_use]
pub fn run(scenario: &Scenario, seed: u64) -> Outcome {
    run_on_tree(scenario, seed, BTreeMap::new())
}
/// Start a new component over an existing worker root, then run and shut it down.
#[must_use]
pub fn run_on_tree(scenario: &Scenario, seed: u64, tree: BTreeMap<Vec<u8>, Node>) -> Outcome {
    let mut world = World::new(seed, tree);
    let outcome = world.run(scenario).expect("worker inspection");
    world.shutdown();
    outcome
}
