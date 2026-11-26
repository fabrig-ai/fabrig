//! Cold store rehearsals (`temper/docs/design/testing/store-check.md`, 2–4).

use jig_domain_core::engine::store as engine_store;
use std::collections::VecDeque;

use jig_conformance::store_check::{StoreChecks, finding};
use skein_lib::{Env, Queue, Time, Wall};
use temper_engine_domain::{self as engine, Config, Event, Limits, Request, StoreInput, StoreRequest};

use crate::{
    oracle::config,
    store::{Image, Store},
};

/// Rehearse a start over an independent durable image, through the opening.
pub fn store(store: &Store, config: &Config, limits: &Limits, wall: Wall, commit: u64) -> Result<(), String> {
    rehearse(store.image(), config::copy(config), limits, wall, commit)
}

fn rehearse(image: Image, mut config: Config, limits: &Limits, wall: Wall, commit: u64) -> Result<(), String> {
    StoreChecks::rehearse(|| {
        // The nominal fixture workers offer the scenario's full fleet capacity, as in Driver::cold.
        for worker in &mut config.core.defaults.workers {
            worker.places = limits.fleet.slots;
        }
        let mut root = engine::Domain::new(config, limits).expect("valid engine configuration");
        root.rehearse();
        let store = image.open();
        let pending = store.pending_generation;
        let mut start = Rehearsal {
            root,
            env: Env { now: Time::from_nanos(wall.as_nanos()), wall, limits: *limits },
            out: Queue::uncapped(),
            events: VecDeque::from([Event::Restart { pending }]),
            store,
        };
        let mut quiet = 0;
        loop {
            if start.root.rehearsed() {
                return Ok(());
            }
            if let Some(refusal) = start.root.start_stop() {
                let phase = start.root.start_phase().map_or_else(|| "None".to_owned(), |phase| format!("{phase:?}"));
                return Err(finding(commit, &phase, &format!("{refusal:?}")));
            }
            if quiet == 2 {
                let phase = start.root.start_phase().map_or_else(|| "None".to_owned(), |phase| format!("{phase:?}"));
                return Err(format!("store check: commit {commit}: start stalled at {phase}"));
            }
            if !start.advance() && start.events.is_empty() && start.store.pending.is_empty() {
                quiet += 1;
            } else {
                quiet = 0;
            }
        }
    })
}

/// The start talks only to its store and its own durable continuations.
struct Rehearsal {
    root: engine::Domain,
    env: Env<Limits>,
    out: Queue<Request>,
    events: VecDeque<Event>,
    store: Store,
}

impl Rehearsal {
    fn advance(&mut self) -> bool {
        let mut progress = false;
        if let Some(event) = self.events.pop_front() {
            progress |= self.step(event);
        }
        if let Some(number) = self.store.tick(false).expect("rehearsal store commit") {
            progress |= self.step(Event::Store(StoreInput::Committed { number, fill: 0 }));
        }
        crate::poll(&mut self.root, &self.env, &mut self.out);
        progress |= self.collect();
        self.root.reclaim();
        progress
    }

    fn step(&mut self, event: Event) -> bool {
        if let Err(event) = engine::try_step(&mut self.root, &self.env, event) {
            self.events.push_back(event);
        }
        engine::release(&mut self.root, &self.env, &mut self.out);
        let progress = self.collect();
        self.root.reclaim();
        progress
    }

    fn collect(&mut self) -> bool {
        let progress = !self.out.is_empty();
        while let Some(request) = self.out.pop() {
            match request {
                Request::ToChild(released) => self.events.push_back(Event::Released(released)),
                Request::Store(StoreRequest::Catalogue { names }) => self.store.catalogue(names),
                Request::Store(StoreRequest::Commit { number, writes }) => {
                    self.store.pending.push_back((number, writes));
                }
                Request::Store(StoreRequest::Switch) => self.events.push_back(Event::Store(self.store.switch())),
                Request::Store(StoreRequest::Load { owner, range, after, most, bytes }) => {
                    let terminal = self.store.page_terminal(&range, after.as_ref(), most);
                    self.events.extend(self.store.take_missing(owner).into_iter().map(Event::Store));
                    assert!(bytes >= self.env.limits.loads.bytes);
                    self.events.push_back(Event::Store(crate::direct::page_at_owner(terminal, owner, most)));
                }
                Request::Store(StoreRequest::Engine(engine_store::Out::Load {
                    load,
                    range: engine_store::Range::Read(range),
                })) => {
                    let result = self.store.read(range);
                    self.events.push_back(Event::Store(StoreInput::Engine(engine_store::In::Loaded { load, result })));
                }
                Request::Store(StoreRequest::PartRead { waiter, range }) => {
                    let result = self.store.read(range);
                    self.events.push_back(Event::Store(StoreInput::PartLoaded { waiter, result }));
                }
                Request::Store(StoreRequest::Checkpoint { token }) => {
                    self.events.push_back(Event::Store(self.store.checkpoint(token)));
                }
                // Startup can announce settings and request credentials, but
                // no runtime peer's reply is needed to reach the opening.
                _ => {}
            }
        }
        progress
    }
}

/// An engine world's store-check selection and the image it owns until the run ends.
pub struct Run {
    checks: StoreChecks,
    kept: Option<Box<Kept>>,
    ended: bool,
}

struct Kept {
    image: Image,
    // Present only if the world's configuration changed since this commit.
    config: Option<Config>,
    limits: Limits,
    wall: Wall,
}

impl Run {
    #[must_use]
    pub fn new(checks: StoreChecks) -> Self {
        Self { checks, kept: None, ended: false }
    }

    #[must_use]
    pub fn counts(&self) -> &StoreChecks {
        &self.checks
    }

    /// Count only a newly durable commit, including one whose acknowledgement is held.
    pub fn committed(
        &mut self,
        store: &Store,
        config: &Config,
        env: &Env<Limits>,
        previous: u64,
    ) -> Result<(), String> {
        let commit = store.applied;
        if self.ended || commit == previous {
            return Ok(());
        }
        if self.checks.due(commit) {
            return self::store(store, config, &env.limits, env.wall, commit);
        }
        if self.checks.keeps() {
            self.kept = Some(Box::new(Kept { image: store.image(), config: None, limits: env.limits, wall: env.wall }));
        }
        Ok(())
    }

    /// Keep the old configuration only if the kept commit still uses it.
    pub fn replace_config(&mut self, config: &mut Config, next: Config) {
        if let Some(kept) = &mut self.kept
            && kept.config.is_none()
        {
            kept.config = Some(std::mem::replace(config, next));
        } else {
            *config = next;
        }
    }

    /// Rehearse the image already taken, without forking or changing the live store.
    pub fn finish(&mut self, config: &Config) -> Result<(), String> {
        self.ended = true;
        if let Some(commit) = self.checks.end() {
            let Kept { image, config: old, limits, wall } = *self.kept.take().expect("kept commit has its image");
            rehearse(image, old.unwrap_or_else(|| config::copy(config)), &limits, wall, commit)?;
        }
        Ok(())
    }
}
