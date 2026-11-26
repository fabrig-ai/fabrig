//! Change wait reports (temper/docs/design/domain/forge/waits.md, 4).
use super::*;
use fixtures::{BASE, CHANGE, HEAD, OPEN};
use jig_domain_core_vocabulary::waits::{AtBound, ReportedBound, Thing, Unbounded};
use skein_forge::api::{Answer, Check, Ci, Error, Op, Read, RunState};
use temper_client_protocol_wire::waits as code;
use temper_engine_domain_forge_change::{self as change, Decision, Delegate, Hold, State, Status};

const CHILD: u64 = CHANGE + 1;
const DEPENDENCY: u64 = CHILD + 1;
const PULL: u64 = 1;
const RUN: u64 = 7;
const EARLIER: Duration = Duration::from_secs(5);

pub(super) fn started() -> Domain {
    let mut d = fixtures::connected_domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::CatchUp);
    d
}

fn row(state: State) -> ChangeRow {
    let mut row = fixtures::opening_row();
    row.change.state = state;
    row.effect = change::EffectResult::None;
    row.pending = None;
    row
}

fn opened(state: State) -> ChangeRow {
    let mut row = row(state);
    row.pull = Some(PULL);
    row
}

fn at(start: Wall, duration: Duration, then: AtBound) -> ReportedBound {
    ReportedBound::At { at: Wall::from_nanos(start.as_nanos().saturating_add(duration.as_nanos())), then }
}

fn earlier() -> Wall {
    Wall::from_nanos(env().wall.as_nanos().saturating_sub(EARLIER.as_nanos()))
}

fn outputs_at(d: &mut Domain, clock: &Env<Limits>, event: Event) -> Box<[Request]> {
    let mut out = Queue::uncapped();
    step(d, clock, event, &mut out);
    take(out)
}

fn take(mut out: Queue<Request>) -> Box<[Request]> {
    let mut requests = skein_lib::List::with_capacity(out.len());
    for _ in 0..out.len() {
        requests.push(out.pop().expect("counted request")).expect("room for queued requests");
    }
    requests.into_boxed()
}

#[test]
fn opening_reports_the_in_flight_entries_stall() {
    let mut d = started();
    let mut clock = env();
    let row = fixtures::opening_row();
    let ReportedBound::At { at: bound, .. } = at(row.change.since, clock.limits.change_policy.stall, AtBound::Hold)
    else {
        panic!("an opening stall")
    };
    let since = row.change.since;
    let initial = outputs(&mut d, Event::Change { row });
    clock.now = clock.now.saturating_add(clock.limits.change_policy.stall);
    clock.wall = bound;
    outputs_at(&mut d, &clock, fixtures::change_step());
    let finished = answer_gather(&mut d, &clock, None);

    assert_eq!(
        wait_report(&initial, CHANGE),
        Some(
            [step_wait(
                code::CHANGE_OPENING,
                since,
                ReportedBound::At { at: bound, then: AtBound::Hold },
                &[Thing { what: code::THING_ENTRY, number: OPEN, owner: Some(CHANGE) }],
            )]
            .as_slice()
        )
    );
    let mut held = false;
    for request in &finished {
        if let Request::ChangeDecision { task: CHANGE, decision: Decision::Hold(Hold::Stalled), .. } = request {
            held = true;
        }
    }
    assert!(held, "the reported instant holds at opening");
    assert_eq!(wait_report(&finished, CHANGE), Some([].as_slice()));
}

fn answer_gather(d: &mut Domain, clock: &Env<Limits>, execution: Option<RunState>) -> Box<[Request]> {
    const GATHER_READS: u32 = 9; // branch, base, pull, checks, execution run, base CI, ancestry, paths, reviews
    let link = skein_forge::api::JobAttempt { head: HEAD, run: RUN, job: 41, attempt: 2 };
    for _ in 0..GATHER_READS {
        let (call, op) = fixtures::next_at(d, clock);
        let answer = match op {
            Op::Read(Read::Branch { branch }) => {
                Ok(Answer::Commit(if branch.as_ref() == b"main" { BASE } else { HEAD }))
            }
            Op::Read(Read::Pull { .. } | Read::PullFor { .. }) => match execution {
                Some(_) => {
                    let mut pull = fixtures::open_pull();
                    pull.ci = Ci::Pending;
                    Ok(Answer::Pull(pull))
                }
                None => Err(Error::Missing),
            },
            Op::Read(Read::Checks { commit }) if commit == HEAD && execution.is_some() => {
                let Answer::Checks(mut rows) = fixtures::checks(Check::Pending) else { panic!("check rows") };
                rows[0].job = Some(link);
                Ok(Answer::Checks(rows))
            }
            Op::Read(Read::Checks { .. }) => Ok(fixtures::checks(Check::Passed)),
            Op::Read(Read::Run { run: RUN }) => {
                let mut run = fixtures::failed_run(link, None);
                run.jobs[0].state = execution.expect("an execution reading");
                Ok(Answer::Run(run))
            }
            Op::Read(Read::Commits { .. }) => Ok(Answer::Commits { commits: Box::new([]), more: false }),
            Op::Read(Read::Reviews { .. }) => Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }),
            other @ (Op::Read(_) | Op::Write(_)) => panic!("unexpected gather operation {other:?}"),
        };
        let step = take(fixtures::answer_at(d, clock, call, answer));
        for request in &step {
            if let Request::ChangeDecision { .. } = request {
                return step;
            }
        }
    }
    panic!("a finished gather")
}

#[test]
fn checking_reports_the_bound_of_its_latest_execution_reading() {
    let mut d = started();
    let since = earlier();
    let mut row = opened(State::Checking { head: HEAD });
    row.change.since = since;
    outputs(&mut d, Event::Change { row });
    let mut clock = env();
    let things = [wait_thing(code::THING_PULL, PULL), wait_thing(code::THING_RUN, RUN)];
    for (reading, remaining) in
        [(RunState::Running, LIMITS.change_policy.stall), (RunState::Waiting, LIMITS.change_policy.runner_wait)]
    {
        outputs_at(&mut d, &clock, fixtures::change_step());
        let decided = answer_gather(&mut d, &clock, Some(reading));
        assert_eq!(
            wait_report(&decided, CHANGE),
            Some(
                [step_wait(code::CHANGE_CHECKING, since, at(clock.wall, remaining, AtBound::Hold), &things,)]
                    .as_slice()
            )
        );
        fixtures::advance(&mut d, &mut clock, EARLIER);
    }
}

#[test]
fn a_gather_reports_its_own_clock_even_at_queued() {
    let mut d = started();
    let since = earlier();
    let mut row = opened(State::Queued { head: HEAD, ready_since: since });
    row.change.since = since;
    row.change.ready_since = Some(since);
    outputs(&mut d, Event::Change { row });
    let gathering = outputs(&mut d, fixtures::change_step());
    assert_eq!(
        wait_report(&gathering, CHANGE),
        Some(
            [step_wait(
                code::CHANGE_QUEUED,
                env().wall,
                at(env().wall, LIMITS.change_policy.stall, AtBound::Hold),
                &[wait_thing(code::THING_PULL, PULL)],
            )]
            .as_slice()
        )
    );
}

#[test]
fn producing_reports_the_first_pending_dependency() {
    let mut d = started();
    outputs(&mut d, Event::Change { row: row(State::Producing { requested: false }) });
    let mut event = fixtures::change_step();
    let Event::StepChange { pending, kept, .. } = &mut event else { panic!("change step") };
    *pending = Box::new([change::Pending { task: DEPENDENCY, kind: change::PendingKind::Other }]);
    *kept = true;
    let wait = outputs(&mut d, event);
    let met = outputs(&mut d, Event::DependenciesMet { task: CHANGE });
    assert_eq!(
        wait_report(&wait, CHANGE),
        Some(
            [step_wait(
                code::CHANGE_PRODUCING,
                env().wall,
                ReportedBound::None(Unbounded::Behind(DEPENDENCY)),
                &[wait_thing(code::THING_TASK, DEPENDENCY)],
            )]
            .as_slice()
        )
    );
    assert_eq!(
        wait_report(&met, CHANGE),
        Some(
            [step_wait(
                code::CHANGE_PRODUCING,
                env().wall,
                at(env().wall, LIMITS.change_policy.stall, AtBound::Hold),
                &[],
            )]
            .as_slice()
        )
    );
}

#[test]
fn live_children_clear_the_report_until_their_ends() {
    let gate = Delegate::Gate { head: HEAD, number: 1 };
    for kind in [
        Delegate::Produce,
        Delegate::Repair(change::Repair::Ci),
        Delegate::Resolve { base: BASE },
        Delegate::Merge { base: BASE, from: Some(HEAD) },
        gate,
    ] {
        let mut d = started();
        outputs(&mut d, Event::Change { row: opened(State::Checking { head: HEAD }) });
        let admitted = outputs(&mut d, Event::Delegated { task: CHANGE, child: CHILD, kind });
        let ended = outputs(&mut d, terminal(CHILD));
        assert_eq!(wait_report(&admitted, CHANGE), Some([].as_slice()), "{kind:?}");
        assert_eq!(
            wait_report(&ended, CHANGE),
            Some(
                [step_wait(
                    code::CHANGE_CHECKING,
                    env().wall,
                    at(env().wall, LIMITS.change_policy.stall, AtBound::Hold),
                    &[wait_thing(code::THING_PULL, PULL)],
                )]
                .as_slice()
            ),
            "{kind:?}"
        );
    }
}

fn terminal(child: u64) -> Event {
    Event::DelegateResult {
        task: CHANGE,
        child,
        ended_at: Some(env().wall),
        status: Status::Passed,
        no_change: false,
        flake: false,
        no_test: false,
        failed: false,
        lost: false,
        cancelled: false,
        words: Box::new([]),
    }
}

#[test]
fn a_live_asker_and_the_held_change_report_none() {
    let mut d = started();
    outputs(&mut d, Event::Change { row: row(State::Producing { requested: true }) });
    let questions = change::Questions {
        open: Box::new([change::Open {
            asker: CHILD,
            number: 1,
            line: b"Which branch?".as_slice().into(),
            words: b"Which branch?".as_slice().into(),
            stands: change::Stands::Asked,
        }]),
        ..change::Questions::default()
    };
    outputs(&mut d, Event::Relayed { task: CHANGE, questions: questions.clone() });
    let opened = outputs(&mut d, Event::Delegated { task: CHANGE, child: CHILD, kind: Delegate::Produce });
    let answered = outputs(&mut d, Event::Relayed { task: CHANGE, questions: change::Questions::default() });
    outputs(&mut d, terminal(CHILD));
    let held = outputs(&mut d, Event::RelayRefused { task: CHANGE, questions, words: b"Refused".as_slice().into() });
    assert_eq!(wait_report(&opened, CHANGE), Some([].as_slice()), "a live child asking");
    assert_eq!(wait_report(&answered, CHANGE), None, "answering doesn't end the asker");
    assert_eq!(wait_report(&held, CHANGE), Some([].as_slice()));
}

#[test]
fn forge_connection_loss_clears_the_wait_until_recovery() {
    let mut d = started();
    outputs(&mut d, Event::Change { row: fixtures::opening_row() });
    let forge = REPO.forge;
    let lost = outputs(&mut d, Event::Connection { forge, state: Connection::NotConnected });
    let found = outputs(
        &mut d,
        Event::Connection { forge, state: Connection::Connected { id: 7, login: b"bot".as_slice().into() } },
    );
    assert_eq!(wait_report(&lost, CHANGE), Some([].as_slice()));
    assert_eq!(
        wait_report(&found, CHANGE),
        Some(
            [step_wait(
                code::CHANGE_OPENING,
                env().wall,
                at(env().wall, LIMITS.change_policy.stall, AtBound::Hold),
                &[Thing { what: code::THING_ENTRY, number: OPEN, owner: Some(CHANGE) }],
            )]
            .as_slice()
        )
    );
}

#[test]
fn an_unreachable_repository_clears_its_gathers_wait() {
    let mut d = started();
    outputs(&mut d, Event::Change { row: opened(State::Checking { head: HEAD }) });
    fixtures::step_change(&mut d);
    let (call, _) = fixtures::next_call(&mut d);
    let lost = fixtures::answer_once(&mut d, call, Err(Error::Unavailable));
    let mut clock = env();
    fixtures::advance(&mut d, &mut clock, LIMITS.client.backoff);
    let found = answer_gather(&mut d, &clock, Some(RunState::Running));
    assert_eq!(wait_report(&lost, CHANGE), Some([].as_slice()));
    assert_eq!(
        wait_report(&found, CHANGE),
        Some(
            [step_wait(
                code::CHANGE_CHECKING,
                clock.wall,
                at(clock.wall, LIMITS.change_policy.stall, AtBound::Hold),
                &[wait_thing(code::THING_PULL, PULL), wait_thing(code::THING_RUN, RUN)],
            )]
            .as_slice()
        )
    );
}

#[test]
fn queued_reports_the_change_that_owns_its_turn() {
    let mut d = started();
    let since = env().wall;
    let mut owner = opened(State::Checking { head: HEAD });
    owner.task = CHILD;
    owner.change.task = CHILD;
    owner.change.owns_turn = true;
    outputs(&mut d, Event::Change { row: owner });
    let result = outputs(&mut d, Event::Change { row: opened(State::Queued { head: HEAD, ready_since: since }) });
    assert_eq!(
        wait_report(&result, CHANGE),
        Some(
            [step_wait(
                code::CHANGE_QUEUED,
                since,
                ReportedBound::None(Unbounded::Behind(CHILD)),
                &[wait_thing(code::THING_TASK, CHILD)],
            )]
            .as_slice()
        )
    );
}

#[test]
fn a_ready_change_reports_the_first_four_settling_changes_of_its_free_train() {
    let mut d = started();
    assert!(d.project_landing(
        5,
        change::Way::Turn,
        Box::new([]),
        temper_engine_domain_forge_landing::Project::DEFAULT,
        AfterLanding {
            wait: change::Wait::Cleared,
            verdict: temper_engine_domain_forge_tiers::Settings {
                retries: 2,
                suspects: temper_engine_domain_forge_tiers::Suspects::Bisect,
            },
            bound: Duration::from_secs(7200),
        },
    ));
    let since = env().wall;
    let mut ready = opened(State::Ready { head: HEAD, ready_since: since });
    ready.change.way = change::Way::Turn;
    let mut out = Queue::uncapped();
    // Install out of order, with the lowest task's release into another train.
    for (task, base) in [
        (CHILD + 5, b"main".as_slice()),
        (CHILD, b"other".as_slice()),
        (CHILD + 3, b"main".as_slice()),
        (CHILD + 2, b"main".as_slice()),
        (CHILD + 1, b"main".as_slice()),
        (CHILD + 4, b"main".as_slice()),
    ] {
        let mut member = opened(State::Gating { head: HEAD, asked: Box::new([]) });
        member.change.way = change::Way::Turn;
        member.change.last_head = Some(HEAD);
        member.change.task = task;
        member.task = task;
        member.base = base.into();
        outputs(&mut d, Event::Change { row: member });
        d.set_settling(task, true, &mut out);
    }
    let result = outputs(&mut d, Event::Change { row: ready });
    assert_eq!(
        wait_report(&result, CHANGE),
        Some(
            [step_wait(
                code::CHANGE_READY,
                since,
                ReportedBound::None(Unbounded::Behind(CHILD + 1)),
                &[
                    wait_thing(code::THING_TASK, CHILD + 1),
                    wait_thing(code::THING_TASK, CHILD + 2),
                    wait_thing(code::THING_TASK, CHILD + 3),
                    wait_thing(code::THING_TASK, CHILD + 4),
                ]
            )]
            .as_slice()
        )
    );
}

#[test]
fn landed_verification_gives_up_but_red_has_no_deadline() {
    const VERIFICATION: Duration = Duration::from_secs(7200); // Stock default (temper/docs/design/domain/after-landing.md, 6).
    let since = earlier();
    let pull = [wait_thing(code::THING_PULL, PULL)];
    for (state, code, bound, things) in [
        (
            State::Landed { merge: HEAD, since },
            code::CHANGE_LANDED,
            at(since, VERIFICATION, AtBound::GiveUp),
            pull.as_slice(),
        ),
        (
            State::Red { merge: HEAD, since },
            code::CHANGE_RED,
            ReportedBound::None(Unbounded::NoDeadline),
            [].as_slice(),
        ),
    ] {
        let mut d = started();
        d.verifications
            .insert(
                (REPO, HEAD),
                Verification {
                    repository: REPO,
                    commit: HEAD,
                    changes: Box::new([CHANGE]),
                    tiers: Box::new([TierVerification {
                        tier: b"tests".as_slice().into(),
                        verdict: None,
                        repaired: None,
                    }]),
                },
            )
            .expect("the verification's tier has no verdict");
        let report = outputs(&mut d, Event::Change { row: opened(state) });
        assert_eq!(wait_report(&report, CHANGE), Some([step_wait(code, since, bound, things)].as_slice()));
    }
}
