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
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    let mut row = fixtures::opening_row();
    row.pending = None;
    row.pull = Some(1);
    outputs(&mut d, Event::Restore { record: Stored::Change(row) });
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
    for (before, operation, resource) in [
        (&[][..], b"read branch".as_slice(), run_branch(b"51")),
        (&[fixtures::HEAD][..], b"read base", base),
        (&[fixtures::HEAD, fixtures::BASE][..], b"read pull", pull),
    ] {
        let mut d = gathering();
        for head in before {
            fixtures::answer_next(&mut d, Ok(Answer::Commit(*head)));
        }
        let out = fixtures::answer_next(&mut d, Err(skein_forge::api::Error::Forbidden));
        assert!(
            out.contains(&Request::ResourceRefused {
                task: fixtures::CHANGE,
                operation,
                resource: Some(resource),
                cause: RefusalCause::Backend(skein_forge::api::Error::Forbidden),
            }),
            "{operation:?}: {out:?}"
        );
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
