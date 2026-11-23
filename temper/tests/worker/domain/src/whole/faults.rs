//! Fault compositions over the whole worker (temper/docs/design/testing/worker.md, 3–4).
use super::{ATTEMPT, Case, RUN, World, assignment, translate};
use skein_lib::{Duration, Time, Token};
use skein_world::{
    faults::{FaultLedger, Injection, Struck},
    streams::{Stream, Streams},
};
use temper_worker_domain::{
    Event, agent,
    checkout::git::{Done, Fault, Op},
    host, wire,
};
use temper_worker_inspect::faults::{DISK_FULL, FETCH_CUT, PLACEMENT_BUSY};

#[derive(Default)]
pub(super) struct Faults {
    pub cut: Option<bool>,
    pub held: Option<(Token, Op, Time)>,
    pub fill: Option<bool>,
    pub capacity: Option<u64>,
    pub full: Option<Injection>,
    pub ledger: FaultLedger,
    pub makes: u32,
}

impl Faults {
    pub fn perform(
        &mut self,
        world: &mut super::Forge,
        disk: &mut super::Checkout,
        owner: Token,
        op: Op,
        now: Time,
        deadline: Time,
    ) -> Option<Done> {
        if matches!(op, Op::Make { .. }) {
            self.makes += 1;
        }
        let checkout = matches!(op, Op::CheckOut { .. });
        let fetch = matches!(op, Op::Fetch { .. });
        if self.cut.is_some_and(|at_checkout| if at_checkout { checkout } else { fetch }) {
            self.cut = None;
            self.held = Some((owner, op, deadline));
            return None;
        }
        if self.fill.is_some_and(|at_checkout| if at_checkout { checkout } else { fetch }) {
            self.fill = None;
            self.capacity = Some(translate::used(disk));
        }
        let writes = matches!(
            op,
            Op::Store { .. }
                | Op::Clone { .. }
                | Op::Fetch { .. }
                | Op::Counts { .. }
                | Op::Point { .. }
                | Op::CheckOut { .. }
                | Op::Merge { .. }
                | Op::Commit { .. }
                | Op::Reset { .. }
                | Op::Reds { .. }
        );
        world.at(now);
        let done = translate::perform_with_capacity(world, disk, op, self.capacity);
        if let Done::FailedWithOutput { diagnostic, .. } = &done {
            if diagnostic.output() == b"No space left on device" && self.full.is_none() {
                self.full = Some(self.ledger.inject(DISK_FULL, now, struck(format!("hold {} fetch", owner.raw()))));
            }
        } else if writes
            && !matches!(done, Done::Failed { .. } | Done::FailedWithOutput { .. })
            && let Some(entry) = self.full.take()
        {
            self.ledger.stop(entry, now).expect("one end after the fault began");
        }
        Some(done)
    }
}

fn fault_world(seed: u64, slots: u32) -> World {
    let mut world = World::with_slots(Streams::new(seed).seed_of(Stream::Workload, "worker forge"), slots);
    world.seed = seed;
    world
}

fn struck(request: String) -> Struck {
    Struck { request: Some(request), ..Struck::default() }
}

/// A killed fetch leaves partial state; the next attempt must prepare afresh.
pub fn cut_prepare(seed: u64, checkout: bool) -> Result<FaultLedger, String> {
    let mut world = fault_world(seed, 1);
    world.faults.cut = Some(checkout);
    world.connect()?;
    world.assign(wire::Start::Base { branch: b"main".as_slice().into() })?;
    assert!(world.process.is_none());
    let (owner, op, deadline) = world.faults.held.take().expect("the fetch is in flight at the quiet point");
    world.env.now = deadline;
    let entry = world.faults.ledger.inject(FETCH_CUT, deadline, struck(format!("hold {} fetch", owner.raw())));
    let done = translate::cut(&mut world.forge, &mut world.disk, op);
    assert_eq!(done, Done::Failed { fault: Fault::TimedOut });
    world.faults.ledger.stop(entry, deadline)?;
    world.event(Event::Done { owner, done })?;
    let (answer, _, _) = world.answer.take().expect("prepare failed");
    assert!(matches!(answer.ending, host::Ending::Failed { failure: host::Failure::Unprepared(_), .. }));
    assert_eq!(world.domain.workspaces(), 0);
    world.event(Event::Link(host::LinkDown::Acknowledge { run: RUN, attempt: ATTEMPT }))?;
    world.attempt = Token::new(14);
    world.start()?;
    assert_eq!(world.faults.makes, 2, "the failed workspace is made afresh");
    world.story(Case::Delivered)?;
    assert_eq!(world.domain.workspaces(), 0);
    world.event(Event::Link(host::LinkDown::Acknowledge { run: RUN, attempt: world.attempt }))?;
    Ok(world.faults.ledger)
}

/// Space refusal fails the prepare, carrying git's diagnostic, without retaining a hold.
pub fn full_prepare(seed: u64, checkout: bool) -> Result<FaultLedger, String> {
    let mut world = fault_world(seed, 1);
    // The diagnostic is the subject of this story, not the fixture's tiny text bound.
    world.env.limits.host.detail_bytes = 256;
    world.faults.fill = Some(checkout);
    world.connect()?;
    world.assign(wire::Start::Base { branch: b"main".as_slice().into() })?;
    let (answer, _, _) = world.answer.take().expect("prepare failed for space");
    let host::Ending::Failed { failure: host::Failure::Unprepared(_), detail, .. } = answer.ending else {
        panic!("disk refusal fails preparation")
    };
    assert!(String::from_utf8_lossy(&detail).contains("No space left on device"), "{detail:?}");
    assert!(world.process.is_none());
    assert_eq!(world.domain.workspaces(), 0);
    assert!(world.domain.checkout().held().is_empty());
    world.event(Event::Link(host::LinkDown::Acknowledge { run: RUN, attempt: ATTEMPT }))?;
    assert_eq!(world.domain.host().hosted(), 0);
    Ok(world.faults.ledger)
}

/// Two runs announce long delivery checks. One holds the build place and one waits.
/// Only the world completes the checks, just as the deployment's external builder does.
pub fn busy_placement(seed: u64, first_run: bool, backoff: Duration) -> Result<FaultLedger, String> {
    let mut world = fault_world(seed, 2);
    let second = Token::new(82);
    let newcomer = Token::new(83);
    world.start()?;
    world.say(agent::Up::Long { operation: agent::Operation::Checks, span: Duration::from_secs(60) })?;
    assert!(world.builder.ask(RUN, world.env.now));
    let first_process = world.process.take();
    world.reading = false;
    world.run = second;
    world.assign(wire::Start::Base { branch: b"main".as_slice().into() })?;
    world.say(agent::Up::Admitted)?;
    world.say(agent::Up::Long { operation: agent::Operation::Checks, span: Duration::from_secs(60) })?;
    assert!(!world.builder.ask(second, world.env.now));
    world.check()?;
    let second_process = world.process;
    let before = (world.domain.workspaces(), world.faults.makes, world.domain.checkout().held().len());
    let place = |world: &World| {
        assignment(world.first, newcomer, newcomer.raw(), wire::Start::Base { branch: b"main".as_slice().into() })
    };
    world.event(place(&world))?;
    assert_eq!(world.busy, [(newcomer, ATTEMPT)]);
    let busy = world.faults.ledger.inject(
        PLACEMENT_BUSY,
        world.env.now,
        struck(format!("run {} attempt {}", newcomer.raw(), ATTEMPT.raw())),
    );
    assert_eq!(
        before,
        (world.domain.workspaces(), world.faults.makes, world.domain.checkout().held().len()),
        "busy takes no workspace or prepare"
    );
    let mut found = temper_worker_inspect::inspection();
    temper_worker_inspect::inspect_root(&world.domain, &world.env.limits, &mut found);
    let slots = found.tables().iter().find(|table| table.name == "run slots").expect("slots");
    assert_eq!(slots.entries.len(), 2);
    assert!(
        slots.entries.iter().all(|entry| !entry.key.starts_with("run 83 ")),
        "no refused attempt in the slot table"
    );
    world.env.now = world.env.now.saturating_add(backoff);
    world.event(place(&world))?;
    assert_eq!(world.busy.len(), 2, "backoff alone does not make room");
    assert_eq!(world.builder.finish(RUN), Some(second));
    world.run = RUN;
    world.process = first_process;
    world.reading = true;
    world.say(agent::Up::LongDone)?;
    world.finish()?;
    world.gone()?;
    world.answer.take().expect("first run ended");
    world.builder.ended(RUN);
    world.event(Event::Link(host::LinkDown::Acknowledge { run: RUN, attempt: ATTEMPT }))?;
    world.run = newcomer;
    world.process = None;
    world.reading = false;
    world.env.now = world.env.now.saturating_add(backoff);
    world.event(place(&world))?;
    assert!(world.process.is_some(), "the placed-again attempt is admitted");
    assert_eq!(world.busy.len(), 2);
    world.faults.ledger.stop(busy, world.env.now)?;
    world.say(agent::Up::Admitted)?;
    // Vary completion order without changing which queued check is granted first.
    if first_run {
        world.finish()?;
        world.gone()?;
        world.answer.take().expect("new run ended");
        world.event(Event::Link(host::LinkDown::Acknowledge { run: newcomer, attempt: ATTEMPT }))?;
    }
    let new_process = world.process;
    world.run = second;
    world.process = second_process;
    world.reading = true;
    assert_eq!(world.builder.finish(second), None);
    world.say(agent::Up::LongDone)?;
    world.finish()?;
    world.gone()?;
    world.answer.take().expect("second run ended");
    world.builder.ended(second);
    world.event(Event::Link(host::LinkDown::Acknowledge { run: second, attempt: ATTEMPT }))?;
    if !first_run {
        world.run = newcomer;
        world.process = new_process;
        world.reading = true;
        world.finish()?;
        world.gone()?;
        world.answer.take().expect("new run ended");
        world.event(Event::Link(host::LinkDown::Acknowledge { run: newcomer, attempt: ATTEMPT }))?;
    }
    assert_eq!(world.domain.host().hosted(), 0);
    assert_eq!(world.domain.workspaces(), 0);
    Ok(world.faults.ledger)
}

/// Existing workload choices remain unchanged; these new draws have separate streams.
pub fn seeded(seed: u64) -> Result<FaultLedger, String> {
    let streams = Streams::new(seed);
    let mut workload = streams.rng(Stream::Workload, "worker");
    let cases = [
        Case::Delivered,
        Case::CancelledDelivery,
        Case::ContactWithin,
        Case::ContactPast,
        Case::Drained,
        Case::DrainExpired,
        Case::LinkDrained,
        Case::LinkDrainExpired,
    ];
    let case = cases[usize::try_from(workload.next_u64() % 8).expect("eight cases")];
    fault_world(seed, 1).run(case)?;
    let mut faults = streams.rng(Stream::Faults, "worker");
    let mut ledger = FaultLedger::new();
    // Every seed composes the three faults, with independent fault locations and order.
    for run in [
        cut_prepare(seed, faults.next_u64() % 2 == 0)?,
        full_prepare(seed, faults.next_u64() % 2 == 0)?,
        busy_placement(seed, faults.next_u64() % 2 == 0, Duration::from_secs(1 + faults.next_u64() % 3))?,
    ] {
        for entry in run.entries() {
            let injected = ledger.inject(entry.kind, entry.from, entry.struck.clone());
            if let Some(until) = entry.until {
                ledger.stop(injected, until)?;
            }
        }
    }
    Ok(ledger)
}
