use jig_domain_tasks::{
    self as tasks, Active, Control, Ending, Event, Executor, Hold, HoldKind, Holding, KeptStart, Kind, MessageKind,
    Name, Party, Phase, ProcedureDecision, ProcedureEnd, Refusal, Taken, TaskResult,
};
use jig_tasks_world::{LIMITS, Reply, World, task};
use skein_lib::ReplyTo;

const PROCEDURE: u64 = 1;
const DEPENDENCY: u64 = 2;
const HOLDER: u64 = 3;
const REQUESTER: Party = Party::Person(9);
const START: KeptStart = KeptStart { connector: 1, code: 1 };

fn configured() -> World {
    let mut world = World::new(890, LIMITS);
    world.set_kept_starts(Box::new([START]));
    world
}

fn procedure(dependencies: &[u64]) -> tasks::New {
    let mut new = task(PROCEDURE, dependencies);
    new.executor = Executor::Procedure { connector: START.connector, code: START.code };
    new
}

fn pending() -> World {
    let mut world = configured();
    world.make(REQUESTER, vec![procedure(&[DEPENDENCY]), task(DEPENDENCY, &[])]);
    world
}

fn response(world: &mut World, event: impl FnOnce(ReplyTo) -> Event) -> Reply {
    let to = world.to();
    let call = to.into_token();
    world.send(event(ReplyTo::new(call)));
    world.replies[&call.raw()].clone()
}

fn step(world: &mut World, decision: ProcedureDecision) -> Reply {
    let step = world.record(PROCEDURE).attempt + 1;
    let read = world.contexts[&PROCEDURE].inbox.last().map(|word| word.number);
    response(world, |reply_to| Event::Procedure { reply_to, task: PROCEDURE, step, read, decision })
}

fn done(world: &mut World, number: u64) {
    world.claim_fresh(number);
    world.finish(number);
    world.settle(number);
}

fn result() -> ProcedureEnd {
    ProcedureEnd::Result(TaskResult::Report { words: Box::new([7]) })
}

#[test]
fn a_kept_start_procedure_made_with_a_pending_dependency_is_active_and_kept() {
    let mut world = pending();
    assert_eq!(world.record(PROCEDURE).phase, Phase::Active(Active::Due));
    assert!(world.record(PROCEDURE).holds_taken);
    assert!(world.activations.contains(&PROCEDURE));
    assert_eq!(world.contexts[&PROCEDURE].pending.as_ref(), [DEPENDENCY]);
    assert_eq!(world.record(PROCEDURE).due_reason, tasks::Why::Made);
    let before = world.records.clone();
    let decision = ProcedureDecision { batch: Box::new([task(4, &[])]), ..ProcedureDecision::new(result()) };
    assert!(matches!(step(&mut world, decision), Reply::Refused(problem) if problem.why == Refusal::Kept));
    assert_eq!(world.records, before, "the done step and its batch are refused whole");
    assert_eq!(step(&mut world, ProcedureDecision::new(ProcedureEnd::Hold(Hold::Drift))), Reply::Done);
    assert!(matches!(world.record(PROCEDURE).phase, Phase::Held { .. }));
    assert_eq!(
        response(&mut world, |reply_to| Event::Control {
            reply_to,
            by: REQUESTER,
            task: PROCEDURE,
            control: Control::Release { again: None, message: None },
        }),
        Reply::Done
    );
    assert_eq!(world.record(PROCEDURE).waiting_on.as_ref(), [DEPENDENCY]);
    assert_eq!(
        step(&mut world, ProcedureDecision::new(ProcedureEnd::Result(TaskResult::Failure { reason: Box::new([8]) }))),
        Reply::Done
    );
    world.settle(PROCEDURE);
    assert!(matches!(world.results[&PROCEDURE], Ending::Failed { .. }));
}

#[test]
fn a_kept_start_procedure_goes_on_when_its_dependency_is_done() {
    let mut world = pending();
    assert_eq!(step(&mut world, ProcedureDecision::new(ProcedureEnd::Wait)), Reply::Done);
    world.activations.remove(&PROCEDURE);
    done(&mut world, DEPENDENCY);
    let notices: Vec<_> =
        world.record(PROCEDURE).inbox.iter().filter(|word| matches!(word.kind, MessageKind::Met { .. })).collect();
    assert_eq!(notices.len(), 1);
    assert_eq!(notices[0].kind, MessageKind::Met { dependency: DEPENDENCY, removed: false });
    assert_eq!(notices[0].from, Party::Task(DEPENDENCY));
    assert!(world.activations.contains(&PROCEDURE));
    assert!(world.contexts[&PROCEDURE].pending.is_empty());
    assert_eq!(step(&mut world, ProcedureDecision::new(result())), Reply::Done);
    world.settle(PROCEDURE);
    assert!(matches!(world.results[&PROCEDURE], Ending::Done(_)));
}

#[test]
fn a_procedure_of_another_code_made_with_a_pending_dependency_waits() {
    for executor in [
        Executor::Procedure { connector: START.connector, code: START.code + 1 },
        Executor::Procedure { connector: START.connector + 1, code: START.code },
        Executor::Agent { charter: 1 },
        Executor::Person(tasks::PersonAddress::Person(9)),
    ] {
        let mut world = configured();
        let mut new = task(PROCEDURE, &[DEPENDENCY]);
        new.executor = executor;
        world.make(REQUESTER, vec![new, task(DEPENDENCY, &[])]);
        assert_eq!(world.record(PROCEDURE).phase, Phase::Waiting);
        assert!(!world.record(PROCEDURE).holds_taken);
        assert!(!world.activations.contains(&PROCEDURE));
    }
}

#[test]
fn a_kept_start_with_no_pending_dependency_can_end_done() {
    let mut world = configured();
    assert_eq!(world.make(REQUESTER, vec![procedure(&[])]), Reply::Made(vec![PROCEDURE]));
    assert_eq!(world.record(PROCEDURE).phase, Phase::Active(Active::Due));
    assert!(world.contexts[&PROCEDURE].pending.is_empty());
    assert_eq!(step(&mut world, ProcedureDecision::new(result())), Reply::Done);
    world.settle(PROCEDURE);
    assert!(matches!(world.results[&PROCEDURE], Ending::Done(_)));
}

fn waiting_for_holds() -> World {
    let mut world = configured();
    world.configure_holds(1, vec![Kind { connector: 1, kind: 1, hold: HoldKind::Exclusive { taken: Taken::Waits } }]);
    let holding = Holding::Write { resource: Name { connector: 1, path: Box::new([Box::new([1])]) }, kind: 1 };
    let mut holder = task(HOLDER, &[]);
    holder.holdings = Box::new([holding.clone()]);
    world.make(REQUESTER, vec![holder, task(DEPENDENCY, &[])]);
    let mut new = procedure(&[DEPENDENCY]);
    new.holdings = Box::new([holding]);
    world.make(REQUESTER, vec![new]);
    world
}

#[test]
fn a_kept_start_waits_for_its_holds_then_starts_with_its_dependency_pending() {
    let mut world = waiting_for_holds();
    assert_eq!(world.record(PROCEDURE).phase, Phase::Waiting);
    assert!(!world.record(PROCEDURE).holds_taken);
    assert!(world.record(PROCEDURE).hold_wait_since.is_some());
    assert!(!world.activations.contains(&PROCEDURE));
    let before = world.records.clone();
    world.restart();
    assert_eq!(world.records, before);
    done(&mut world, HOLDER);
    assert_eq!(world.record(PROCEDURE).phase, Phase::Active(Active::Due));
    assert!(world.record(PROCEDURE).holds_taken);
    assert_eq!(world.contexts[&PROCEDURE].pending.as_ref(), [DEPENDENCY]);
}

#[test]
fn a_kept_start_holds_queue_keeps_its_wait_bound_with_a_pending_dependency() {
    for dependency_ends in [false, true] {
        let mut world = waiting_for_holds();
        let bound_since = world.record(PROCEDURE).hold_bound_since;
        let half = skein_lib::Duration::from_nanos(LIMITS.hold_wait.as_nanos() / 2);
        world.elapse(half);
        if dependency_ends {
            done(&mut world, DEPENDENCY);
        }
        assert_eq!(world.record(PROCEDURE).hold_bound_since, bound_since);
        assert_eq!(world.record(PROCEDURE).phase, Phase::Waiting);
        world.restart();
        world.elapse(half);
        assert!(matches!(world.record(PROCEDURE).phase, Phase::Held { ref why, .. } if *why == Hold::HoldsWaited));
    }
}

#[test]
fn a_kept_starts_job_with_a_pending_dependency_waits() {
    let mut world = configured();
    world.make(REQUESTER, vec![procedure(&[])]);
    let mut dependency = task(DEPENDENCY, &[]);
    dependency.funder = tasks::Funder::Task(PROCEDURE);
    let job_number = 4;
    let mut job = task(job_number, &[DEPENDENCY]);
    job.executor = Executor::Job { kind: START.code };
    job.spec.job = Some(Box::new(tasks::JobSpec { bytes: Box::new([1]), deadline: skein_lib::Duration::from_secs(7) }));
    job.spec.words = Box::new([]);
    job.contract = tasks::Contract::Job { bytes: 16 };
    job.numbers = tasks::Numbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 };
    job.authority.budget.spend = 0;
    job.funder = tasks::Funder::Task(PROCEDURE);
    assert_eq!(
        step(
            &mut world,
            ProcedureDecision { batch: Box::new([dependency, job]), ..ProcedureDecision::new(ProcedureEnd::Wait) }
        ),
        Reply::Made(vec![DEPENDENCY, job_number])
    );
    assert_eq!(world.record(job_number).phase, Phase::Waiting);
    assert!(!world.activations.contains(&job_number));
}

#[test]
fn a_kept_start_restores_active_and_kept_until_its_dependency_is_done() {
    let mut world = pending();
    let before = world.records.clone();
    world.restart();
    assert_eq!(world.records, before);
    assert_eq!(world.record(PROCEDURE).phase, Phase::Active(Active::Due));
    assert_eq!(world.contexts[&PROCEDURE].pending.as_ref(), [DEPENDENCY]);
    assert!(
        matches!(step(&mut world, ProcedureDecision::new(result())), Reply::Refused(problem) if problem.why == Refusal::Kept)
    );
    done(&mut world, DEPENDENCY);
    assert_eq!(step(&mut world, ProcedureDecision::new(result())), Reply::Done);
    world.settle(PROCEDURE);
    assert!(matches!(world.results[&PROCEDURE], Ending::Done(_)));
}
