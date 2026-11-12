//! Configured limits refuse at the step, retaining the requested usage.
use super::*;
use temper_engine_domain_forge_change as change;

const TASK: u64 = 7;

fn send(d: &mut Domain, env: &Env<Limits>, event: Event) -> Queue<Request> {
    let mut out = Queue::uncapped();
    step(d, env, event, &mut out);
    out
}

fn adopted_domain() -> Domain {
    let mut d = fixtures::connected_domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    d
}

fn contains(out: &Queue<Request>, expected: &Request) -> bool {
    for request in out {
        if request == expected {
            return true;
        }
    }
    false
}

fn limit(out: &Queue<Request>, task: u64, member: &'static str, configured: u64, needed: u64) {
    let mut found = false;
    for request in out {
        if let Request::ResourceRefused { task: owner, cause, .. } = request {
            found |= *owner == task && *cause == RefusalCause::Limit { member, configured, needed };
        }
    }
    assert!(found, "{out:?}");
    assert!(!contains(out, &Request::Refused { task }), "a limit must keep its cause");
}

fn claim_event(writes: Box<[Name]>, designs: Box<[Name]>) -> Event {
    Event::Claim { task: TASK, attempt: 1, writes, designs, holders: Box::new([]) }
}

#[test]
fn slot_and_design_claims_name_the_requested_name_count() {
    let mut d = adopted_domain();
    let env = env();
    let names = vec![branch(); usize::try_from(env.limits.resources_per_task + 1).unwrap()].into_boxed_slice();
    for event in [claim_event(names.clone(), Box::new([])), claim_event(Box::new([]), names)] {
        let out = send(&mut d, &env, event);
        limit(
            &out,
            TASK,
            "forge.resources_per_task",
            u64::from(env.limits.resources_per_task),
            u64::from(env.limits.resources_per_task) + 1,
        );
    }
}

#[test]
fn a_full_hold_table_names_the_next_hold() {
    let mut d = adopted_domain();
    let mut env = env();
    env.limits.holds = 1;
    send(&mut d, &env, Event::Hold { entry: 0, task: TASK, resource: branch(), from: None });
    let resource = run_branch(b"8");
    let out = send(&mut d, &env, Event::Hold { entry: 0, task: TASK, resource: resource.clone(), from: None });
    limit(&out, TASK, "forge.holds", 1, 2);
    assert!(d.hold(&resource).is_none());
    let out = send(&mut d, &env, Event::Hold { entry: 0, task: TASK, resource: branch(), from: None });
    for request in &out {
        assert!(!matches!(request, Request::ResourceRefused { .. }), "replacing a hold consumes no room");
    }
}

#[test]
fn a_full_design_writer_set_names_the_next_writer() {
    let mut d = adopted_domain();
    let name = Name {
        what: What::Branch(Box::new([
            Box::from(b"temper".as_slice()),
            Box::from(b"7".as_slice()),
            Box::from(b"g7".as_slice()),
        ])),
        ..branch()
    };
    outputs(
        &mut d,
        Event::Restore {
            record: Stored::Hold(Hold {
                name: name.clone(),
                task: TASK,
                writer: None,
                design_writers: Box::new([DesignWriter { task: 8, attempt: 1, rereading: false }]),
                drift: None,
            }),
        },
    );
    outputs(
        &mut d,
        Event::Restore {
            record: Stored::BranchHead(BranchHead { name: name.clone(), commit: fixtures::HEAD, owned: true }),
        },
    );
    let mut env = env();
    env.limits.holds = 1;
    let out = send(&mut d, &env, claim_event(Box::new([]), Box::new([name])));
    limit(&out, TASK, "forge.holds", 1, 2);
}

#[test]
fn hold_names_count_each_segments_separator_and_distinguish_bad_names() {
    let mut d = adopted_domain();
    let env = env();
    let name = Name {
        what: What::Branch(Box::new([
            Box::from(b"temper".as_slice()),
            vec![b'x'; usize::try_from(env.limits.name_bytes - 7).unwrap()].into_boxed_slice(),
        ])),
        ..branch()
    };
    let out = send(&mut d, &env, Event::Hold { entry: 0, task: TASK, resource: name, from: None });
    limit(&out, TASK, "forge.name_bytes", u64::from(env.limits.name_bytes), u64::from(env.limits.name_bytes) + 1);
    for (name, cause) in [
        (Name { repository: 99, ..branch() }, RefusalCause::Unadopted),
        (
            Name {
                what: What::Branch(Box::new([Box::from(b"temper".as_slice()), Box::from(b"bad/name".as_slice())])),
                ..branch()
            },
            RefusalCause::InvalidName,
        ),
    ] {
        let out = send(&mut d, &env, Event::Hold { entry: 0, task: TASK, resource: name.clone(), from: None });
        assert!(
            contains(
                &out,
                &Request::ResourceRefused {
                    task: TASK,
                    operation: b"hold resource",
                    resource: Some(name.clone()),
                    cause
                }
            ),
            "{out:?}"
        );
    }
}

#[test]
fn unwatching_names_the_watch_that_does_not_fit() {
    for landing in [false, true] {
        let mut d = adopted_domain();
        if landing {
            d.names.insert(8, Box::new([branch()])).unwrap();
            let subscription = subscription();
            d.subscriptions.insert((subscription.task, subscription.topic.clone()), subscription).unwrap();
        } else {
            d.names.insert(8, Box::new([branch(), run_branch(b"8")])).unwrap();
        }
        let mut env = env();
        env.limits.client.resources = 1;
        let out = send(&mut d, &env, Event::Unnamed { task: TASK });
        limit(&out, TASK, "forge.client.resources", 1, 2);
    }
}

fn entry(number: u64) -> skein_forge_client::Entry {
    skein_forge_client::Entry {
        number,
        task: TASK,
        repository: REPO,
        effect: skein_forge_client::Effect {
            condition: skein_forge_client::Condition::None,
            write: skein_forge::api::Write::Post { number: 1, key: Box::from(b"test".as_slice()), body: Box::new([]) },
        },
        start: None,
        attempt: None,
        failures: 0,
        queued: false,
    }
}

#[test]
fn a_full_outbox_names_the_next_entry_but_a_duplicate_stays_bare() {
    let mut d = adopted_domain();
    let mut env = env();
    env.limits.entries = 1;
    send(&mut d, &env, Event::Enqueue { entry: entry(1) });
    let out = send(&mut d, &env, Event::Enqueue { entry: entry(2) });
    limit(&out, TASK, "forge.entries", 1, 2);
    let duplicate = send(&mut d, &env, Event::Enqueue { entry: entry(1) });
    assert!(contains(&duplicate, &Request::Refused { task: TASK }), "{duplicate:?}");
}

fn reviewing() -> Domain {
    let mut d = adopted_domain();
    let mut row = fixtures::opening_row();
    row.delegate = Some((52, change::Delegate::Gate { number: 1, head: fixtures::HEAD }));
    outputs(&mut d, Event::Change { row });
    d
}

fn review(words: &[u8]) -> Event {
    Event::DelegateResult {
        task: fixtures::CHANGE,
        child: 52,
        status: change::Status::Passed,
        no_change: false,
        failed: false,
        cancelled: false,
        words: words.into(),
    }
}

#[test]
fn a_gate_review_names_the_next_kept_verdict() {
    let mut d = reviewing();
    d.changes.get_mut(&fixtures::CHANGE).unwrap().verdicts =
        Box::new([change::GateReport { number: 2, head: fixtures::HEAD, status: change::Status::Passed }]);
    let mut env = env();
    env.limits.change_policy.gates = 1;
    let out = send(&mut d, &env, review(b"ok"));
    limit(&out, fixtures::CHANGE, "forge.change_policy.gates", 1, 2);
}

#[test]
fn a_gate_review_names_the_least_room_for_its_text_or_cut_mark() {
    for (words, needed) in [(b"ok".as_slice(), 2), (b"a long review that needs a cut mark".as_slice(), 16)] {
        let mut d = reviewing();
        let mut env = env();
        env.limits.client.answer_bytes = 1;
        let out = send(&mut d, &env, review(words));
        limit(&out, fixtures::CHANGE, "forge.client.answer_bytes", 1, needed);
    }
}

#[test]
fn a_step_names_its_gate_count_and_the_pending_step_table() {
    let mut d = adopted_domain();
    outputs(&mut d, Event::Change { row: fixtures::opening_row() });
    let mut env = env();
    env.limits.change_policy.gates = 1;
    let Event::StepChange { merge_job, task, entry, heard, queue_repair_active, pending, kept, .. } =
        fixtures::change_step()
    else {
        panic!("step fixture")
    };
    let gates = Box::new([change::GateReport { number: 1, head: fixtures::HEAD, status: change::Status::Passed }; 2]);
    let out = send(
        &mut d,
        &env,
        Event::StepChange { merge_job, task, entry, heard, gates, queue_repair_active, pending, kept },
    );
    limit(&out, fixtures::CHANGE, "forge.change_policy.gates", 1, 2);
    // One pending read from another change fills the step table, not the retained change table.
    send(&mut d, &env, fixtures::change_step());
    let mut other = fixtures::opening_row();
    other.task = 53;
    other.change.task = 53;
    outputs(&mut d, Event::Change { row: other });
    env.limits.changes = 1;
    let Event::StepChange { merge_job, entry, heard, gates, queue_repair_active, pending, kept, .. } =
        fixtures::change_step()
    else {
        panic!("step fixture")
    };
    let out = send(
        &mut d,
        &env,
        Event::StepChange { task: 53, merge_job, entry, heard, gates, queue_repair_active, pending, kept },
    );
    limit(&out, 53, "forge.changes", 1, 2);
}

fn subscription() -> Subscriber {
    Subscriber {
        task: TASK,
        number: 1,
        topic: Topic::Landings { repository: REPO, branch: Box::from(b"main".as_slice()) },
        own_change: None,
        goal_tasks: None,
        paths: Box::new([]),
    }
}

#[test]
fn a_subscription_names_one_path_past_the_configured_count() {
    let mut d = adopted_domain();
    let env = env();
    let mut subscription = subscription();
    subscription.paths =
        vec![Box::from(b"src".as_slice()); usize::try_from(env.limits.paths_per_subscription + 1).unwrap()]
            .into_boxed_slice();
    let out = send(&mut d, &env, Event::Subscribe { subscription });
    limit(
        &out,
        TASK,
        "forge.paths_per_subscription",
        u64::from(env.limits.paths_per_subscription),
        u64::from(env.limits.paths_per_subscription) + 1,
    );
    assert!(d.subscriptions.is_empty());
}

#[test]
fn a_subscription_names_its_goal_task_count() {
    let mut d = adopted_domain();
    let env = env();
    let mut subscription = subscription();
    subscription.goal_tasks =
        Some(vec![8; usize::try_from(env.limits.issue_policy.plan_items + 1).unwrap()].into_boxed_slice());
    let out = send(&mut d, &env, Event::Subscribe { subscription });
    limit(
        &out,
        TASK,
        "forge.issue_policy.plan_items",
        u64::from(env.limits.issue_policy.plan_items),
        u64::from(env.limits.issue_policy.plan_items) + 1,
    );
}

#[test]
fn a_subscription_names_the_next_table_row() {
    let mut d = adopted_domain();
    let mut env = env();
    env.limits.subscriptions = 1;
    send(&mut d, &env, Event::Subscribe { subscription: subscription() });
    let mut subscription = subscription();
    subscription.task = 8;
    let out = send(&mut d, &env, Event::Subscribe { subscription });
    limit(&out, 8, "forge.subscriptions", 1, 2);
}

#[test]
fn a_subscription_names_oversized_paths_and_branch_names() {
    let mut d = adopted_domain();
    let env = env();
    let large = vec![b'x'; usize::try_from(env.limits.name_bytes + 1).unwrap()].into_boxed_slice();
    for subscription in [
        Subscriber { paths: Box::new([large.clone()]), ..subscription() },
        Subscriber { topic: Topic::Landings { repository: REPO, branch: large }, ..subscription() },
    ] {
        let out = send(&mut d, &env, Event::Subscribe { subscription });
        limit(&out, TASK, "forge.name_bytes", u64::from(env.limits.name_bytes), u64::from(env.limits.name_bytes) + 1);
    }
}

#[test]
fn a_landing_subscription_keeps_each_watch_union_limit() {
    let mut d = adopted_domain();
    for (member, resources, repositories, op_bytes, name_bytes, configured, needed) in [
        ("forge.client.resources", 1, 2, 512, 128, 1, 2),
        ("forge.client.repositories", 8, 1, 512, 128, 1, 2),
        ("forge.client.op_bytes", 8, 2, 3, 128, 3, 4),
        ("forge.name_bytes", 8, 2, 512, 4, 4, 5),
    ] {
        let mut other = adopted();
        other.provider.repository = 3;
        outputs(&mut d, Event::Restore { record: Stored::Repository(other) });
        d.names.insert(8, Box::new([Name { repository: 3, ..branch() }])).unwrap();
        let mut env = env();
        env.limits.client.resources = resources;
        env.limits.client.repositories = repositories;
        env.limits.client.op_bytes = op_bytes;
        env.limits.name_bytes = name_bytes;
        // The existing watch must not exceed op_bytes before main does.
        d.names.insert(8, Box::new([Name { forge: 1, repository: 3, what: What::Repository }])).unwrap();
        let out = send(&mut d, &env, Event::Subscribe { subscription: subscription() });
        limit(&out, TASK, member, configured, needed);
    }
}

#[test]
fn emitting_a_change_effect_names_the_full_outbox() {
    use skein_forge::api::{Answer, Check, Error};
    let mut d = adopted_domain();
    let mut row = fixtures::opening_row();
    row.pending = None;
    row.effect = change::EffectResult::None;
    row.change.state = change::State::Opening { requested: false };
    outputs(&mut d, Event::Change { row });
    outputs(&mut d, Event::Enqueue { entry: entry(1) });
    fixtures::step_change(&mut d);
    for answer in [
        Ok(Answer::Commit(fixtures::HEAD)),
        Ok(Answer::Commit(fixtures::BASE)),
        Err(Error::Missing),
        Ok(fixtures::checks(Check::Passed)),
        Ok(fixtures::checks(Check::Passed)),
    ] {
        fixtures::answer_next(&mut d, answer);
    }
    let (call, _) = fixtures::next_call(&mut d);
    let mut env = env();
    env.limits.entries = 1;
    let out = fixtures::answer_at(&mut d, &env, call, Ok(Answer::Commits { commits: Box::new([]), more: false }));
    limit(&out, fixtures::CHANGE, "forge.entries", 1, 2);
}
