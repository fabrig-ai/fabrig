//! Watch admission is reproduced at the connector step, without a root or transport.
use super::*;
use skein_forge::api::{Answer, Op, Read};

const TASK: u64 = 7;
const NEXT: u64 = 8;
// One departing watch occupies the only resource slot until its read completes.
const RESOURCES: u32 = 1;

fn small() -> (Domain, Env<Limits>) {
    let (mut d, env) = small_restored(None);
    send(&mut d, &env, Event::ReadAfresh);
    send(&mut d, &env, Event::SettleOutbox);
    (d, env)
}

fn small_restored(names: Option<Stored>) -> (Domain, Env<Limits>) {
    let mut env = env();
    env.limits.client.resources = RESOURCES;
    env.limits.holds = RESOURCES;
    let mut d = domain_with_limits(&env.limits);
    d.restore_begin(Wall::EPOCH);
    send(&mut d, &env, Event::Restore { record: Stored::Repository(adopted()) });
    if let Some(record) = names {
        send(&mut d, &env, Event::Restore { record });
    }
    send(&mut d, &env, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    (d, env)
}

fn send(d: &mut Domain, env: &Env<Limits>, event: Event) -> Queue<Request> {
    let mut out = Queue::uncapped();
    step(d, env, event, &mut out);
    out
}

fn departing() -> (Domain, Env<Limits>, Token) {
    let (mut d, env) = small();
    send(&mut d, &env, Event::Names { task: TASK, resources: Box::new([branch()]) });
    let mut out = Queue::uncapped();
    let mut call = None;
    for _ in 0..CLIENT.calls {
        resume(&mut d, &env, &mut out);
        while let Some(request) = out.pop() {
            if let Request::Call { call: token, op: Op::Read(Read::Branch { .. }), .. } = request {
                call = Some(token);
            }
        }
    }
    send(&mut d, &env, Event::Unnamed { task: TASK });
    (d, env, call.expect("departing branch read is in flight"))
}

#[test]
fn departing_watch_pressure_retries_without_a_persons_release() {
    let (mut d, mut env, call) = departing();
    let name = run_branch(b"8");
    send(&mut d, &env, Event::Names { task: NEXT, resources: Box::new([name.clone()]) });
    send(
        &mut d,
        &env,
        Event::Client(skein_forge_client::Event::Answered {
            free: 0,
            rate: None,
            call,
            cost: 1,
            result: Ok(Answer::Commit(fixtures::HEAD)),
        }),
    );
    env.now = env.now.saturating_add(Duration::from_secs(1));
    let mut out = Queue::uncapped();
    fire(&mut d, &env, &mut out);
    assert_eq!(d.names(NEXT), Some([name].as_slice()), "the desired watch enters automatically");
    resume(&mut d, &env, &mut out);
    let mut watching = false;
    for request in &out {
        if let Request::Save { record: Stored::Client(skein_forge_client::Stored::Live(_)), .. }
        | Request::Call { op: Op::Read(Read::Branch { .. }), .. } = request
        {
            watching = true;
        }
    }
    assert!(watching, "the client starts watching the replacement");
}

#[test]
fn a_branch_first_watched_after_its_push_keeps_the_recorded_head_for_drift() {
    let (mut d, env) = small();
    let name = branch();
    send(&mut d, &env, Event::Hold { entry: 0, task: TASK, resource: name.clone(), from: None });
    send(
        &mut d,
        &env,
        Event::Claim {
            task: TASK,
            attempt: 1,
            writes: Box::new([name.clone()]),
            designs: Box::new([]),
            holders: Box::new([]),
        },
    );
    send(&mut d, &env, Event::Answered { task: TASK, attempt: 1, pushed: Box::new([(name.clone(), fixtures::HEAD)]) });
    assert_eq!(d.branch_head(&name), Some(fixtures::HEAD));
    let mut out = send(&mut d, &env, Event::Names { task: TASK, resources: Box::new([name]) });
    resume(&mut d, &env, &mut out);
    let mut seeded = false;
    for request in &out {
        if let Request::Save { record: Stored::Client(skein_forge_client::Stored::Live(row)), .. } = request {
            seeded |= row.pushed == Some(fixtures::HEAD) && row.cached.tip == Some(fixtures::HEAD);
        }
    }
    assert!(seeded, "the new keep must compare future reads with the head pushed before it was watched: {out:?}");
}

#[test]
fn oversized_watch_sets_name_the_bound_and_required_usage() {
    let (mut d, env) = small();
    let names = Box::new([branch(), run_branch(b"8")]);
    let out = send(&mut d, &env, Event::Names { task: TASK, resources: names });
    let expected = Request::ResourceRefused {
        task: TASK,
        operation: b"watch resources",
        resource: Some(branch()),
        cause: RefusalCause::Limit { member: "forge.client.resources", configured: 1, needed: 2 },
    };
    let mut refused = false;
    for request in &out {
        refused |= *request == expected;
    }
    assert!(refused, "{out:?}");
    assert_eq!(d.names(TASK), None, "an oversized batch keeps no desired names");
}

fn gathering() -> Domain {
    let mut d = fixtures::connected_domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    let mut row = fixtures::opening_row();
    row.pending = None;
    row.pull = Some(1);
    outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(row)) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    fixtures::step_change(&mut d);
    d
}

#[test]
fn branch_base_and_pull_reads_retain_their_target_and_cause() {
    let base = Name { what: What::Branch(Box::new([Box::from(b"main".as_slice())])), ..branch() };
    let pull = Name { what: What::Pull(1), ..branch() };
    for error in [skein_forge::api::Error::Forbidden, skein_forge::api::Error::NotInstalled] {
        for (before, operation, resource) in [
            (&[][..], b"read branch".as_slice(), run_branch(b"51")),
            (&[fixtures::HEAD][..], b"read base", base.clone()),
            (&[fixtures::HEAD, fixtures::BASE][..], b"read pull", pull.clone()),
        ] {
            let mut d = gathering();
            for head in before {
                fixtures::answer_next(&mut d, Ok(Answer::Commit(*head)));
            }
            let out = fixtures::answer_next(&mut d, Err(error));
            assert!(
                out.contains(&Request::ResourceRefused {
                    task: fixtures::CHANGE,
                    operation,
                    resource: Some(resource),
                    cause: RefusalCause::Backend(error),
                }),
                "{operation:?}: {out:?}"
            );
        }
    }
}

#[test]
fn change_reads_ride_outages_with_capped_waits_and_reset_on_an_answer() {
    use skein_forge::api::Error;
    for error in [Error::Unavailable, Error::Timeout] {
        for before in [&[][..], &[fixtures::HEAD][..], &[fixtures::HEAD, fixtures::BASE][..]] {
            let mut d = gathering();
            let mut env = env();
            env.limits.client.read_attempts = 1;
            for head in before {
                fixtures::answer_next(&mut d, Ok(Answer::Commit(*head)));
            }
            let (mut call, op) = fixtures::next_at(&mut d, &env);
            for seconds in [1, 2, 4, 8, 16, 32, 40, 40] {
                let out = fixtures::answer_at(&mut d, &env, call, Err(error));
                for request in &out {
                    assert!(!matches!(request, Request::ResourceRefused { .. }), "{out:?}");
                }
                let mut early = Queue::uncapped();
                fixtures::advance(&mut d, &mut env, Duration::from_nanos(Duration::from_secs(seconds).as_nanos() - 1));
                resume(&mut d, &env, &mut early);
                for request in &early {
                    assert!(!matches!(request, Request::Call { .. }), "no early re-ask: {early:?}");
                }
                fixtures::advance(&mut d, &mut env, Duration::from_nanos(1));
                let next = fixtures::next_at(&mut d, &env);
                assert_eq!(next.1, op, "the same fresh read, not a new chain");
                call = next.0;
            }
            let answer = match before.len() {
                0 => Answer::Commit(fixtures::HEAD),
                1 => Answer::Commit(fixtures::BASE),
                _ => Answer::Pull(fixtures::open_pull()),
            };
            fixtures::answer_at(&mut d, &env, call, Ok(answer));
            let (call, next_op) = fixtures::next_at(&mut d, &env);
            assert_ne!(next_op, op, "the change steps on the answer");
            fixtures::answer_at(&mut d, &env, call, Err(error));
            fixtures::advance(&mut d, &mut env, CLIENT.backoff);
            assert_eq!(fixtures::next_at(&mut d, &env).1, next_op, "an answer resets the wait");
        }
    }
}

fn assert_connection_wait(out: &Queue<Request>) {
    let mut waiting = false;
    for request in out {
        if let Request::ChangeDecision { decision, .. } = request {
            assert_eq!(*decision, temper_engine_domain_forge_change::Decision::Wait { until: None });
        }
        waiting |= *request
            == Request::ChangeDecision {
                task: fixtures::CHANGE,
                decision: temper_engine_domain_forge_change::Decision::Wait { until: None },
                entry: None,
                evidence: None,
            };
        assert!(!matches!(request, Request::ResourceRefused { .. } | Request::Call { .. }), "{out:?}");
    }
    assert!(waiting, "{out:?}");
}

fn assert_no_connection_poll(d: &mut Domain, env: &mut Env<Limits>) {
    let mut out = fixtures::advance(d, env, env.limits.client.slow.saturating_add(Duration::from_secs(100)));
    for _ in 0..env.limits.client.pending {
        resume(d, env, &mut out);
    }
    for request in &out {
        assert!(!matches!(request, Request::Call { .. } | Request::WakeProcedure { .. }), "{out:?}");
    }
    assert_eq!(d.next_deadline(), None, "no polling timer");
}

fn reconnect_change(d: &mut Domain, env: &Env<Limits>) -> Token {
    let connected = Connection::Connected { id: 7, login: b"bot".as_slice().into() };
    let mut other = send(d, env, Event::Connection { forge: 2, state: connected.clone() });
    for _ in 0..env.limits.client.pending {
        resume(d, env, &mut other);
    }
    for request in &other {
        assert_ne!(*request, Request::WakeProcedure { task: fixtures::CHANGE }, "another forge: {other:?}");
    }
    let mut out = send(d, env, Event::Connection { forge: 1, state: connected });
    resume(d, env, &mut out);
    let mut woke = false;
    for request in &out {
        woke |= *request == Request::WakeProcedure { task: fixtures::CHANGE };
    }
    assert!(woke, "{out:?}");
    send(d, env, fixtures::change_step());
    let (call, op) = fixtures::next_at(d, env);
    assert_eq!(op, Op::Read(Read::Branch { branch: Box::from(b"temper/51".as_slice()) }));
    call
}

#[test]
fn change_reads_answered_not_connected_wait_and_restart_from_the_branch() {
    use skein_forge::api::{Check, Ci, CommitMessage, Error};
    let mut pull = fixtures::open_pull();
    pull.ci = Ci::Failed;
    let answers = [
        Answer::Commit(fixtures::HEAD),
        Answer::Commit(fixtures::BASE),
        Answer::Pull(pull),
        fixtures::checks(Check::Failed),
        fixtures::checks(Check::Passed),
        Answer::Commits { commits: Box::new([]), more: false },
    ];
    for (stage, catch_up, before, reverse) in [
        ("branch", None, 0, false),
        ("base", None, 1, false),
        ("pull", None, 2, false),
        ("ci", None, 3, false),
        ("base ci", None, 4, false),
        ("compare", None, 5, false),
        ("contains head", None, 6, true),
        ("reviews", None, 6, false),
        ("catch up", Some(fixtures::BASE), 2, false),
    ] {
        let mut d = gathering();
        let mut env = env();
        send(&mut d, &env, Event::Connection { forge: 1, state: Connection::Connecting });
        d.changes.get_mut(&fixtures::CHANGE).expect("change").catch_up = catch_up;
        for (index, answer) in answers.iter().take(before).enumerate() {
            let answer = if reverse && index == 5 {
                Answer::Commits {
                    commits: Box::new([CommitMessage {
                        id: fixtures::BASE,
                        parents: Box::new([]),
                        message: Box::new([]),
                    }]),
                    more: false,
                }
            } else {
                answer.clone()
            };
            fixtures::answer_next(&mut d, Ok(answer));
        }
        let (call, _) = fixtures::next_at(&mut d, &env);
        let out = fixtures::answer_at(&mut d, &env, call, Err(Error::NotConnected));
        assert_connection_wait(&out);
        assert!(d.steps.is_empty(), "partial {stage} facts dropped");
        // A wake must not restart even while routing still says Connecting.
        assert_connection_wait(&send(&mut d, &env, fixtures::change_step()));
        d.wait_change(&env, fixtures::CHANGE, None);
        assert_no_connection_poll(&mut d, &mut env);
        let call = reconnect_change(&mut d, &env);
        fixtures::answer_at(&mut d, &env, call, Ok(Answer::Commit(fixtures::HEAD)));
        let (call, _) = fixtures::next_at(&mut d, &env);
        fixtures::answer_at(&mut d, &env, call, Ok(Answer::Commit(fixtures::BASE)));
        if catch_up.is_some() {
            let (call, _) = fixtures::next_at(&mut d, &env);
            fixtures::answer_at(&mut d, &env, call, Ok(Answer::Commits { commits: Box::new([]), more: false }));
        }
        let mut out = Queue::uncapped();
        for answer in
            answers.iter().skip(2).cloned().chain([Answer::Reviews { reviews: Box::new([]), more: false, cut: false }])
        {
            let (call, _) = fixtures::next_at(&mut d, &env);
            out = fixtures::answer_at(&mut d, &env, call, Ok(answer));
        }
        let mut decided = false;
        for request in &out {
            decided |= matches!(request, Request::ChangeDecision { .. });
        }
        assert!(decided, "a complete fresh chain decides: {out:?}");
    }
}

#[test]
fn change_steps_wait_without_reads_until_their_forge_connects() {
    for state in [
        None,
        Some(Connection::NotConnected),
        Some(Connection::NotSetUp),
        Some(Connection::SettingUp { step: None }),
        Some(Connection::SetUpFailed { why: b"failed".as_slice().into() }),
        Some(Connection::Stopped { reason: b"stopped".as_slice().into() }),
        Some(Connection::Connecting),
    ] {
        let mut d = domain();
        outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
        outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(fixtures::opening_row())) });
        outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        outputs(&mut d, Event::ReadAfresh);
        outputs(&mut d, Event::SettleOutbox);
        let mut env = env();
        let connecting = state == Some(Connection::Connecting);
        if let Some(state) = state {
            send(&mut d, &env, Event::Connection { forge: 1, state });
        }
        let out = send(&mut d, &env, fixtures::change_step());
        if connecting {
            let (call, _) = fixtures::next_at(&mut d, &env);
            assert_connection_wait(&fixtures::answer_at(
                &mut d,
                &env,
                call,
                Err(skein_forge::api::Error::NotConnected),
            ));
        } else {
            assert_connection_wait(&out);
        }
        d.wait_change(&env, fixtures::CHANGE, None);
        assert_no_connection_poll(&mut d, &mut env);
        assert_connection_wait(&send(&mut d, &env, fixtures::change_step()));
        reconnect_change(&mut d, &env);
    }
}

#[test]
fn cancelled_watch_admission_does_not_return_after_pressure_clears() {
    let (mut d, mut env, call) = departing();
    send(&mut d, &env, Event::Names { task: NEXT, resources: Box::new([run_branch(b"8")]) });
    send(&mut d, &env, Event::Unnamed { task: NEXT });
    send(
        &mut d,
        &env,
        Event::Client(skein_forge_client::Event::Answered {
            free: 0,
            rate: None,
            call,
            cost: 1,
            result: Ok(Answer::Commit(fixtures::HEAD)),
        }),
    );
    env.now = env.now.saturating_add(Duration::from_secs(1));
    fire(&mut d, &env, &mut Queue::uncapped());
    assert_eq!(d.names(NEXT), None);
    assert_eq!(d.next_deadline(), None, "the removed desired set leaves no retry timer");
}

#[test]
fn pending_watch_names_resume_after_restart_without_client_rows() {
    let (mut d, env, _) = departing();
    let resources = Box::new([run_branch(b"8")]);
    let out = send(&mut d, &env, Event::Names { task: NEXT, resources: resources.clone() });
    let mut saved = None;
    for request in &out {
        if let Request::Save { record: Stored::Names { task: NEXT, .. }, .. } = request {
            saved = Some(Stored::Names { task: NEXT, resources: resources.clone() });
        }
    }
    let (mut restarted, env) = small_restored(Some(saved.expect("desired names are durable while waiting")));
    let mut out = send(&mut restarted, &env, Event::ReadAfresh);
    resume(&mut restarted, &env, &mut out);
    let mut watching = false;
    for request in &out {
        if let Request::Save { record: Stored::Client(skein_forge_client::Stored::Live(_)), .. } = request {
            watching = true;
        }
    }
    assert!(watching, "restart admits the desired watch even without a saved client row");
}
