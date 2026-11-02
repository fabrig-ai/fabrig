//! Job stories on the host's seeded world (jig/docs/design/domain/hosts.md, 6.9).
use super::*;
use crate::World;
use crate::world::{Arrival, Engine, Limits, Parent, Settings, engine, parent};
use host::{JobAssignment, JobBound, JobEnd, JobEnding, JobFailure, JobInvalid, JobKind, JobRefusal, Phase};
use jig_domain_host::{LinkDown, Reason};
use skein_lib::Duration;

const RUN: Token = Token::new(900);
const ATTEMPT: Token = Token::new(901);
const BOUND: JobBound = JobBound { kind: JobKind(7), spec: 4, result: 1, deadline: Duration::from_secs(10) };
const ITERATIONS: u32 = 10_000;

fn assignment() -> JobAssignment {
    JobAssignment {
        run: RUN,
        attempt: ATTEMPT,
        kind: BOUND.kind,
        spec: Box::from(&b"spec"[..]),
        deadline: BOUND.deadline,
        workspace: Some(host::Workspace { workstream: 7, items: Token::new(902) }),
        grants: Box::new([]),
    }
}
fn world(seed: u64) -> World {
    let calm = Settings::calm(seed);
    let settings = Settings {
        host: Limits { slots: 2, ..calm.host },
        engine: engine::Script { assignments: 0, ..calm.engine },
        parent: parent::Script { changes: 1000, ..calm.parent },
        ..calm
    };
    let mut world = World::with_trace(&settings, true);
    world.job_kinds = Box::new([BOUND]);
    let event = world.job_opened();
    send(&mut world, event);
    world
}
fn send(world: &mut World, event: Event) {
    world.stage.push(Arrival { event, stale: false, duplicate: false });
}
fn assign(world: &mut World, job: JobAssignment) {
    send(world, Event::Down(LinkDown::AssignJob { job }));
}
fn advance(world: &mut World, until: fn(&World) -> bool) {
    for _ in 0..ITERATIONS {
        if until(world) {
            return;
        }
        world.iterate();
        if until(world) {
            return;
        }
        if !world.has_work_now() {
            world.now = world.wire.next_time().expect("a job has scheduled work");
        }
    }
    panic!("job story did not reach its phase");
}
fn is_active(world: &World) -> bool {
    world
        .admitted
        .get(&(RUN, ATTEMPT))
        .and_then(|owner| world.host.hosting(*owner))
        .is_some_and(|entry| entry.phase == Phase::Active)
}
fn is_delivering(world: &World) -> bool {
    world.host.calls() > 0
}
fn cancel(world: &mut World) {
    send(world, Event::Down(LinkDown::Cancel { run: RUN, attempt: ATTEMPT }));
}
fn result(world: &World) -> &JobEnding {
    world.job_answers.get(&(RUN, ATTEMPT)).expect("job answered")
}
fn failure(world: &World) -> JobFailure {
    match result(world) {
        JobEnding::Failed { failure, .. } => *failure,
        other @ (JobEnding::Refused(_) | JobEnding::Finished { .. }) => panic!("failed job: {other:?}"),
    }
}
fn delivery_end() -> JobEnd {
    JobEnd::Finished { result: Box::from(&b"r"[..]), delivery: Some(Box::from(&b"made"[..])) }
}

/// jig/docs/design/domain/hosts.md, 6.9: the job uses its prepared workspace, answers, and frees its slot on ACK.
#[test]
fn a_job_of_a_declared_kind_runs_in_its_workspace_and_answers_its_result() {
    let mut w = world(7);
    assign(&mut w, assignment());
    w.run(ITERATIONS);
    assert_eq!(
        result(&w),
        &JobEnding::Finished { result: Box::from(&b"r"[..]), work: host::Work { left: None, saved: None } }
    );
    assert_eq!((w.parent.tally().prepares, w.parent.tally().releases, w.host.hosted()), (1, 1, 0));
    assert_eq!(w.parent.tally().saves, 0);
}

/// jig/docs/design/domain/hosts.md, 6.9: invalid jobs never prepare and retain nothing.
#[test]
fn a_job_of_a_kind_not_declared_is_refused_as_invalid() {
    let mut kind = assignment();
    kind.kind = JobKind(8);
    let mut spec = assignment();
    spec.spec = Box::from(&b"spec!"[..]);
    let mut deadline = assignment();
    deadline.deadline = Duration::from_secs(11);
    let mut grant = assignment();
    grant.grants = Box::new([host::Grant {
        form: host::Form::Credential(host::Scheme::Key),
        account: 1,
        generation: 1,
        valid: Duration::from_secs(1),
    }]);
    for (job, invalid) in [
        (kind, JobInvalid::Kind),
        (spec, JobInvalid::Spec),
        (deadline, JobInvalid::Deadline),
        (grant, JobInvalid::Grants),
    ] {
        let mut w = world(8);
        assign(&mut w, job);
        w.run(ITERATIONS);
        assert_eq!(result(&w), &JobEnding::Refused(JobRefusal::Invalid(invalid)));
        assert_eq!((w.parent.tally().prepares, w.host.kept_answers()), (0, 0));
    }
}

/// jig/docs/design/domain/hosts.md, 6.9: delivery and Gone precede release and answer.
#[test]
fn a_finished_job_delivers_through_its_workspace_before_release() {
    let mut w = world(9);
    w.parent.job_end = Some(delivery_end());
    assign(&mut w, assignment());
    w.run(ITERATIONS);
    let JobEnding::Finished { result, work } = result(&w) else { panic!("finished job") };
    assert_eq!(&**result, b"r");
    assert!(work.left.is_some());
    assert_eq!(work.saved, None);
    let trace = w.trace();
    let delivered = trace.iter().position(|line| line.contains("DeliverJob")).expect("delivered");
    let released = trace.iter().position(|line| line.contains("Release")).expect("released");
    assert!(delivered < released);
}

/// jig/docs/design/domain/hosts.md, 6.9: Stale is a typed delivery failure.
#[test]
fn a_stale_job_delivery_fails_undelivered() {
    let mut w = world(10);
    w.parent = Parent::new(parent::Script { moved: 1000, changes: 1000, ..w.settings.parent }, w.settings.host, 10);
    w.parent.job_end = Some(delivery_end());
    assign(&mut w, assignment());
    w.run(ITERATIONS);
    assert_eq!(failure(&w), JobFailure::Undelivered(host::DeliveryOutcome::Stale));
}

/// jig/docs/design/domain/hosts.md, 6.9: cancellation stops the capability and waits for Gone.
#[test]
fn a_cancelled_job_is_stopped_and_answered_once_its_processes_are_gone() {
    let mut w = world(11);
    assign(&mut w, assignment());
    advance(&mut w, is_active);
    cancel(&mut w);
    w.iterate();
    assert_eq!(w.parent.tally().releases, 0);
    w.run(ITERATIONS);
    assert_eq!(failure(&w), JobFailure::Cancelled(Reason::Engine));
    assert_eq!(w.parent.tally().stops, 1);
}

/// jig/docs/design/domain/hosts.md, 6.9: a begun delivery is waited for after cancel.
#[test]
fn a_cancelled_job_waits_for_its_delivery() {
    let mut w = world(12);
    w.parent.job_end = Some(delivery_end());
    assign(&mut w, assignment());
    advance(&mut w, is_delivering);
    cancel(&mut w);
    w.iterate();
    assert_eq!(w.parent.tally().releases, 0);
    w.run(ITERATIONS);
    assert_eq!(failure(&w), JobFailure::Cancelled(Reason::Engine));
    let JobEnding::Failed { work, .. } = result(&w) else { panic!("cancelled job") };
    assert!(work.left.is_some());
}

/// jig/docs/design/domain/hosts.md, 6.9: deadline is distinct from the job's own failure.
#[test]
fn a_job_past_its_deadline_fails_deadline() {
    let mut w = world(13);
    w.parent.job_end = Some(JobEnd::Deadline);
    assign(&mut w, assignment());
    w.run(ITERATIONS);
    assert_eq!(failure(&w), JobFailure::Deadline);
}

/// jig/docs/design/domain/hosts.md, 6.9: an oversized result never reaches the workspace's delivery.
#[test]
fn a_result_past_its_kinds_bound_fails_and_delivers_nothing() {
    let mut w = world(14);
    w.parent.job_end =
        Some(JobEnd::Finished { result: Box::from(&b"rr"[..]), delivery: Some(Box::from(&b"made"[..])) });
    assign(&mut w, assignment());
    w.run(ITERATIONS);
    assert_eq!(failure(&w), JobFailure::Result);
    assert!(!w.trace().iter().any(|line| line.contains("DeliverJob")));
}

/// jig/docs/design/domain/hosts.md, 6.9: a redial lists kinds and hosted jobs, then replays kept answers.
#[test]
fn the_hello_states_the_kinds_and_lists_a_hosted_job() {
    let mut w = world(15);
    assign(&mut w, assignment());
    advance(&mut w, is_active);
    let event = w.job_opened();
    send(&mut w, event);
    w.iterate();
    let owner = *w.admitted.get(&(RUN, ATTEMPT)).expect("hosted job");
    assert_eq!(w.host.hosting(owner).expect("phase").phase, Phase::Active);
    // Keep the answer by delaying the engine's ACK over the network.
    advance(&mut w, |world| world.host.kept_answers() == 1);
    let before = w.trace().iter().filter(|line| line.contains("host -> Up(JobAnswer")).count();
    let event = w.job_opened();
    send(&mut w, event);
    w.iterate();
    assert_eq!(&*w.job_kinds, [BOUND]);
    assert!(w.host.answer_at(0).is_some());
    assert!(w.trace().iter().any(|line| line.contains("Answered")));
    assert_eq!(w.trace().iter().filter(|line| line.contains("host -> Up(JobAnswer")).count(), before);
    w.run(ITERATIONS);
    assert_eq!(w.trace().iter().filter(|line| line.contains("engine <- Up(JobAnswer")).count(), 2);
}

/// jig/docs/design/domain/hosts.md, 6.9: runs and jobs share two slots and replay the same trace after restart.
#[test]
fn a_seeded_world_mixes_a_run_and_a_job_and_replays() {
    fn run(seed: u64) -> World {
        let mut w = world(seed);
        w.engine =
            Engine::new(engine::Script { assignments: 1, events: 0, ..w.settings.engine }, w.settings.host, seed);
        let begin = w.engine.begin();
        w.acts(begin);
        assign(&mut w, assignment());
        w.run(ITERATIONS);
        w
    }
    let first = run(16);
    let replay = run(16);
    assert_eq!(first.trace(), replay.trace());
    assert_eq!(first.stats(), replay.stats());
    assert_eq!(first.job_answers, replay.job_answers);
    assert_eq!(first.stats().peak, 2);
    assert_eq!(first.engine.tally().assignments, 1);
    assert!(matches!(result(&first), JobEnding::Finished { .. }));
}
