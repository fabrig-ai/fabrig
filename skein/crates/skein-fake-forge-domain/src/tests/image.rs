//! Restarts keep the lasting forge, not its in-flight deliveries or callers.
use super::*;
use crate::{
    api::{Report, ReportFormat, Workflow},
    image::Refusal,
};

fn restored(h: &Harness) -> Harness {
    Harness {
        domain: Domain::from_image(&h.env.limits, &h.domain.image().unwrap()).unwrap(),
        env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: h.env.limits.clone() },
        out: Queue::uncapped(),
        hooks: Queue::with_capacity(64),
        answered: Queue::with_capacity(LIMITS.calls),
        calls: h.calls,
    }
}
fn workflow() -> Workflow {
    Workflow {
        file: copy_of(b"wf"),
        jobs: names(&[b"test"]),
        latency_min: Duration::from_secs(5),
        latency_max: Duration::from_secs(5),
        silent: 0,
        passes: 0,
        cue: Some(Cue::Message { fail: copy_of(b"bad") }),
        fails: names(&[b"unit"]),
        flaky: names(&[b"flaky"]),
        select: Some(copy_of(b"only")),
        report: Some(Report { artifact: copy_of(b"report"), format: ReportFormat::Lines, deflate: true }),
    }
}
fn story() -> (Harness, u64, u64) {
    let mut configured = setup();
    configured.checks.cues = Box::new([crate::api::ContextCue {
        context: copy_of(b"ci"),
        cue: Cue::File { path: copy_of(b"ci"), green: copy_of(b"green") },
    }]);
    configured.workflows = Box::new([workflow()]);
    let mut h = Harness::with(calm(), configured);
    let mut gone = setup();
    gone.id = 3;
    gone.name = copy_of(b"ai/gone");
    crate::repository(&mut h.domain, &calm(), gone);
    crate::grant(&mut h.domain, b"ai/gone", ADMIN, Permission::Admin);
    h.call_on(b"ai/gone", ADMIN, write(Write::DeleteRepository)).unwrap();
    let issue = h.issue(ENGINE, b"issue");
    let comment = h.comment(PERSON, issue, b"first");
    h.ok(ENGINE, edit(comment, b"edited"));
    let deleted_comment = h.comment(PERSON, issue, b"gone");
    h.ok(ENGINE, write(Write::DeleteComment { id: deleted_comment }));
    h.ok(ENGINE, set_labels(issue, &[b"bug"]));
    let head = h.commit(FIRST, &[(b"src", b"one")]);
    h.push(ENGINE, b"work", head).unwrap();
    let Answer::Created(pull) = h.open(b"work").unwrap() else { panic!("opened pull") };
    h.ok(MAINTAINER, review(pull, Verdict::Approve));
    h.ok(PERSON, write(Write::Review { number: pull, verdict: None, body: copy_of(b"pending") }));
    h.ok(ENGINE, write(Write::SetDependencies { number: pull, dependencies: Box::new([issue]) }));
    h.ok(MAINTAINER, status(head, b"manual", Check::Passed));
    h.settle();
    h.ok(ENGINE, merge(pull, head));
    h.ok(
        ENGINE,
        write(Write::Dispatch {
            workflow: copy_of(b"wf"),
            branch: copy_of(MAIN),
            inputs: Box::new([(copy_of(b"only"), copy_of(b"unit"))]),
        }),
    );
    h.ok(ENGINE, put(b"Home", b"old"));
    h.ok(ENGINE, put(b"Home", b"new"));
    h.ok(
        ADMIN,
        write(Write::SetProtection {
            protection: Some(Protection {
                linear: true,
                code_owners: true,
                bypassing_users: Box::new([ADMIN]),
                queue: false,
                branch: copy_of(MAIN),
                contexts: names(&[b"ci"]),
                approvals: 1,
                dismiss_stale: true,
            }),
        }),
    );
    crate::settings(
        &mut h.domain,
        REPOSITORY,
        crate::api::Settings { default: copy_of(MAIN), merge: false, rebase: true, squash: true },
    );
    h.settle();
    (h, head, pull)
}
fn same_read(h: &Harness, other: &Harness, input: Read) {
    assert_eq!(h.inspect(&input), other.inspect(&input), "{input:?}");
}
#[test]
fn an_image_round_trips_the_lasting_forge_and_rebuilds_its_indexes() {
    let (h, head, pull) = story();
    let other = restored(&h);
    for input in [
        Read::Locate,
        Read::Labels,
        Read::Settings,
        Read::Collaborators,
        Read::Permission { user: PERSON },
        Read::Permission { user: STRANGER },
        Read::Item { number: 1, after: 0, since: None },
        Read::Comment { id: 1 },
        Read::Comments { number: 1, since: Some(Time::ZERO) },
        Read::Dependencies { number: pull },
        Read::Pull { number: pull },
        Read::Statuses { commit: head },
        Read::Checks { commit: head },
        Read::Tree { commit: head },
        Read::Branch { branch: copy_of(MAIN) },
        Read::Branch { branch: copy_of(b"work") },
        Read::Pages { after: None },
        Read::Page { name: copy_of(b"Home") },
        Read::Protection { branch: copy_of(MAIN) },
    ] {
        same_read(&h, &other, input);
    }
    for page in [1, 2] {
        same_read(
            &h,
            &other,
            Read::Items {
                state: None,
                kind: None,
                labels: names(&[]),
                author: None,
                order: Order::LeastUpdated,
                since: Time::ZERO,
                page,
                limit: 1,
            },
        );
        same_read(&h, &other, Read::Runs { workflow: None, commit: None, number: None, page, limit: 2 });
    }
    let r = h.domain.repositories.get(h.domain.id(REPOSITORY)).unwrap();
    for (_, kept) in r.runs.iter().chain(r.dispatched.iter()) {
        for input in
            [Read::Run { run: kept.run.id }, Read::Jobs { run: kept.run.id }, Read::Artifacts { run: kept.run.id }]
        {
            same_read(&h, &other, input);
        }
        for job in &kept.jobs {
            same_read(&h, &other, Read::Job { job: job.id });
            same_read(&h, &other, Read::Log { job: job.id, attempt: job.attempt, step: None });
        }
        if let Some(artifact) = &kept.artifact {
            same_read(&h, &other, Read::Artifact { run: kept.run.id, name: artifact.name.clone() });
        }
    }
    assert!(h.domain.branches(REPOSITORY).iter().eq(other.domain.branches(REPOSITORY).iter()));
    assert_eq!(other.domain.inspect(&calm(), b"ai/gone", &Read::Locate), Err(Error::Missing(What::Repository)));
    assert_eq!(other.domain.repositories.len(), 2, "gone slots stay occupied");
    assert!(other.domain.repositories.iter().last().unwrap().1.gone);
    assert_eq!(h.domain.image().unwrap(), other.domain.image().unwrap(), "lasting records are canonical");
}
#[test]
fn restored_counters_make_the_same_next_commit_and_object_numbers() {
    let (mut h, head, pull) = story();
    let mut other = restored(&h);
    other.env.now = h.env.now;
    let next = h.commit(head, &[(b"src", b"two")]);
    assert_eq!(other.commit(head, &[(b"src", b"two")]), next, "commit counter continues");
    assert_eq!(h.push(ENGINE, b"work", next), other.push(ENGINE, b"work", next));
    assert_eq!(h.comment(PERSON, 1, b"next"), other.comment(PERSON, 1, b"next"), "comment ids continue past deletion");
    let opened = h.open(b"work").unwrap();
    assert_eq!(opened, other.open(b"work").unwrap());
    let Answer::Created(number) = opened else { panic!("new pull") };
    assert_eq!(
        h.ok(MAINTAINER, review(number, Verdict::Approve)),
        other.ok(MAINTAINER, review(number, Verdict::Approve))
    );
    same_read(&h, &other, Read::Pull { number: pull });
    assert_eq!(h.ok(ENGINE, put(b"Home", b"next")), other.ok(ENGINE, put(b"Home", b"next")));
    h.settle();
    other.settle();
    same_read(&h, &other, Read::Runs { workflow: None, commit: None, number: None, page: 1, limit: 2 });
    same_read(&h, &other, Read::Checks { commit: next });
}
#[test]
fn the_restored_rng_decides_faults_and_latencies_from_the_saved_stream() {
    let config = Config {
        unavailable: 250,
        timeouts: 250,
        late: 250,
        latency_min: Duration::from_millis(3),
        latency_max: Duration::from_secs(1),
        ..calm()
    };
    let mut h = Harness::new(config);
    for _ in 0_u32..7 {
        let _answer = h.call(PERSON, read(Read::Labels));
    }
    h.settle();
    let mut other = restored(&h);
    other.env.now = h.env.now;
    for _ in 0_u32..16 {
        assert_eq!(h.call(PERSON, read(Read::Labels)), other.call(PERSON, read(Read::Labels)));
        assert_eq!(h.env.now, other.env.now, "the latency draw is the same too");
    }
}
#[test]
fn restored_comments_never_precede_the_images_last_clock_reading() {
    let mut h = Harness::new(Config { skew: Skew::Ahead(Duration::from_secs(20)), ..calm() });
    h.env.now = Time::from_nanos(Duration::from_secs(100).as_nanos());
    let issue = h.issue(ENGINE, b"issue");
    h.settle();
    let reading = h.domain.clock;
    let mut other = restored(&h);
    let id = other.comment(PERSON, issue, b"after restart");
    let Answer::Comment { comment, .. } = other.inspect(&Read::Comment { id }).unwrap() else { panic!("comment") };
    assert!(comment.created >= reading, "{} >= {}", comment.created.as_nanos(), reading.as_nanos());
}
#[test]
fn an_undecided_ci_run_keeps_its_time_left_but_unreleased_hooks_are_dropped() {
    let config = Config { hook_min: Duration::from_secs(30), hook_max: Duration::from_secs(30), ..calm() };
    let mut h = Harness::new(config);
    h.env.now = Time::from_nanos(Duration::from_secs(100).as_nanos());
    let head = h.commit(FIRST, &[(b"src", b"one")]);
    h.push(ENGINE, b"work", head).unwrap();
    assert_eq!(h.env.now, Time::from_nanos(Duration::from_secs(101).as_nanos()));
    assert_eq!(h.domain.deliveries(), 1);
    let mut other = restored(&h);
    assert_eq!(other.domain.deliveries(), 0);
    assert_eq!(other.domain.next_deadline(), Some(Time::from_nanos(Duration::from_secs(4).as_nanos())));
    other.until(Time::from_nanos(Duration::from_secs(3).as_nanos()));
    let pending = other.inspect(&Read::Checks { commit: head }).unwrap();
    let Answer::Checks(checks) = pending else { panic!("checks") };
    assert_eq!(checks.first().unwrap().status.state, Check::Pending);
    other.until(Time::from_nanos(Duration::from_secs(4).as_nanos()));
    let Answer::Checks(checks) = other.inspect(&Read::Checks { commit: head }).unwrap() else { panic!("checks") };
    assert_eq!(checks.first().unwrap().status.state, Check::Passed);
    other.settle();
    let hooks = other.heard();
    for (change, _) in &hooks {
        assert_ne!(*change, Change::Push, "the saved push hook is never delivered");
    }
    assert_eq!(hooks.as_slice(), [(Change::Job, None)], "only newly scheduled hooks survive");
}
#[test]
fn an_image_is_busy_only_until_every_reply_has_been_emitted() {
    let mut h = Harness::new(calm());
    let event = Event::Call {
        reply_to: ReplyTo::new(Token::new(1)),
        user: PERSON,
        repository: repository(),
        op: read(Read::Labels),
    };
    step(&mut h.domain, &h.env, event, &mut h.out);
    assert_eq!(h.domain.image(), Err(Refusal::Busy));
    h.env.now = h.domain.next_deadline().unwrap();
    fire(&mut h.domain, &h.env, &mut h.out);
    assert_eq!(h.domain.calls(), 1, "answered, not yet reclaimed");
    assert!(h.domain.image().is_ok(), "an emitted reply is not busy");
    h.domain.reclaim();
    step(
        &mut h.domain,
        &h.env,
        Event::Call {
            reply_to: ReplyTo::new(Token::new(2)),
            user: ENGINE,
            repository: repository(),
            op: write(Write::Copy {
                owner: copy_of(b"org"),
                name: copy_of(b"copy"),
                fill: crate::api::Fill::Slow(Duration::from_secs(10)),
            }),
        },
        &mut h.out,
    );
    assert_eq!(h.domain.image(), Err(Refusal::Busy), "an unfinished copy still owes its reply");
}
#[test]
fn malformed_and_over_limit_images_build_nothing() {
    let (h, _, _) = story();
    let bytes = h.domain.image().unwrap();
    for end in 0..bytes.len() {
        assert!(malformed(Domain::from_image(&calm(), &bytes[..end]).err()), "truncated at {end}");
    }
    assert!(malformed(Domain::from_image(&calm(), b"arbitrary bytes").err()));
    let trailing = skein_lib::bytes::concat(&[&bytes, b"trailing"]);
    assert_eq!(Domain::from_image(&calm(), &trailing).err(), Some(Refusal::Malformed("trailing bytes")));
    let smaller = Config { limits: Limits { repositories: 1, ..LIMITS }, ..calm() };
    assert_eq!(Domain::from_image(&smaller, &bytes).err(), Some(Refusal::Limits("repositories")));
    let smaller = Config { limits: Limits { comments: 0, ..LIMITS }, ..calm() };
    assert_eq!(Domain::from_image(&smaller, &bytes).err(), Some(Refusal::Limits("comments")));
}
#[test]
fn an_answered_late_landing_survives_without_a_second_reply() {
    let mut h = Harness::new(calm());
    let issue = h.issue(ENGINE, b"issue");
    h.settle();
    h.env.limits.landing = 1000;
    h.env.limits.land_min = Duration::from_secs(3);
    h.env.limits.land_max = Duration::from_secs(3);
    assert_eq!(h.call(PERSON, write(Write::Comment { number: issue, body: copy_of(b"late") })), Err(Error::Timeout));
    let mut other = restored(&h);
    other.until(Time::from_nanos(Duration::from_secs(2).as_nanos()));
    assert!(other.item(issue).1.is_empty());
    other.until(Time::from_nanos(Duration::from_secs(3).as_nanos()));
    assert_eq!(other.item(issue).1.first().unwrap().body.as_ref(), b"late");
    other.settle();
    assert!(other.answered.is_empty(), "the reply right was consumed before the image");
}

fn malformed(error: Option<Refusal>) -> bool {
    match error {
        Some(Refusal::Malformed(_)) => true,
        Some(Refusal::Busy | Refusal::Limits(_)) | None => false,
    }
}

#[test]
fn a_ci_rerun_and_a_dispatched_job_keep_their_remaining_delays() {
    let mut configured = setup();
    configured.checks.reruns = 1000;
    configured.workflows = Box::new([workflow()]);
    let mut h = Harness::with(calm(), configured);
    h.env.now = Time::from_nanos(Duration::from_secs(100).as_nanos());
    let head = h.commit(FIRST, &[(b"src", b"one")]);
    h.push(ENGINE, b"work", head).unwrap();
    h.until(Time::from_nanos(Duration::from_secs(105).as_nanos()));
    h.ok(ENGINE, write(Write::Dispatch { workflow: copy_of(b"wf"), branch: copy_of(b"work"), inputs: Box::new([]) }));
    // At 106: the CI rerun and the dispatch both have four seconds left.
    let mut other = restored(&h);
    other.until(Time::from_nanos(Duration::from_secs(3).as_nanos()));
    let Answer::Run(run) = other.inspect(&Read::Run { run: 2 }).unwrap() else { panic!("run") };
    assert_eq!(run.state, crate::api::RunState::Running);
    other.until(Time::from_nanos(Duration::from_secs(4).as_nanos()));
    let Answer::Run(run) = other.inspect(&Read::Run { run: 2 }).unwrap() else { panic!("run") };
    assert_eq!(run.state, crate::api::RunState::Completed);
    let Answer::Jobs(jobs) = other.inspect(&Read::Jobs { run: 1 }).unwrap() else { panic!("jobs") };
    assert_eq!(jobs.first().unwrap().attempt, 2);
    assert_eq!(jobs.first().unwrap().state, crate::api::RunState::Running);
    other.until(Time::from_nanos(Duration::from_secs(9).as_nanos()));
    let Answer::Jobs(jobs) = other.inspect(&Read::Jobs { run: 1 }).unwrap() else { panic!("jobs") };
    assert_eq!(jobs.first().unwrap().state, crate::api::RunState::Completed);
}
#[test]
fn a_queues_pending_wake_and_group_verdict_survive_the_image() {
    let config = Config {
        limits: Limits { name_bytes: 128, branches: 8, ..LIMITS },
        latency_min: Duration::ZERO,
        latency_max: Duration::ZERO,
        ..calm()
    };
    let mut configured = setup();
    configured.protection = Some(Protection {
        linear: false,
        code_owners: false,
        bypassing_users: Box::new([]),
        queue: true,
        branch: copy_of(MAIN),
        contexts: names(&[b"ci"]),
        approvals: 0,
        dismiss_stale: true,
    });
    let mut h = Harness::with(config, configured);
    let head = h.commit(FIRST, &[(b"src", b"one")]);
    h.push(ENGINE, b"work", head).unwrap();
    h.open(b"work").unwrap();
    h.ok(ENGINE, write(Write::Enqueue { number: 1, head }));
    assert_eq!(h.domain.next_deadline(), Some(Time::ZERO), "queue wake is still due");
    let mut other = restored(&h);
    assert_eq!(h.domain.queue_entries(REPOSITORY).unwrap(), other.domain.queue_entries(REPOSITORY).unwrap());
    assert_eq!(other.domain.next_deadline(), Some(Time::ZERO));
    same_read(&h, &other, Read::Pull { number: 1 });
    h.settle();
    other.settle();
    same_read(&h, &other, Read::Pull { number: 1 });
    assert!(other.pull(1).queue_merged);
    assert_eq!(h.branch(MAIN), other.branch(MAIN));
}
#[test]
fn rate_windows_and_undrained_observations_start_empty_after_restore() {
    let mut h = Harness::new(Config { rate_limit: 1, ..calm() });
    h.issue(ENGINE, b"first");
    assert_eq!(h.domain.windows.len(), 1);
    let mut other = restored(&h);
    assert!(other.domain.windows.is_empty());
    assert!(other.domain.pop_observation().is_none());
    assert!(h.domain.pop_observation().is_some(), "the original had undrained observations");
    assert_eq!(other.issue(ENGINE, b"second"), 2, "the rate window starts again");
    assert!(h.call(ENGINE, create(b"second", b"body", &[])).is_err(), "the old window is still running");
}
#[test]
fn references_and_nested_limits_are_checked_before_a_domain_is_returned() {
    let (h, _, _) = story();
    let bytes = h.domain.image().unwrap();
    let smaller = Config { limits: Limits { reviews: 1, ..LIMITS }, ..calm() };
    assert_eq!(Domain::from_image(&smaller, &bytes).err(), Some(Refusal::Limits("reviews")));
    let smaller = Config { limits: Limits { contexts: 1, ..LIMITS }, ..calm() };
    assert_eq!(Domain::from_image(&smaller, &bytes).err(), Some(Refusal::Limits("status contexts")));
    let smaller = Config { limits: Limits { content_bytes: 2, ..LIMITS }, ..calm() };
    assert!(matches_limits(Domain::from_image(&smaller, &bytes).err()));
    for corruption in 0_u32..5 {
        let (mut h, head, pull) = story();
        let id = h.domain.id(REPOSITORY);
        match corruption {
            0 => {
                h.domain
                    .repositories
                    .get_mut(id)
                    .unwrap()
                    .items
                    .get_mut(&pull)
                    .unwrap()
                    .pull
                    .as_mut()
                    .unwrap()
                    .commit = 999;
            }
            1 => h.domain.repositories.get_mut(id).unwrap().runs.get_mut(&head).unwrap().run.commit = 999,
            2 => h.domain.commits.get_mut(&FIRST).unwrap().parent = Some(999),
            3 => h.domain.comments = 0,
            4 => {
                h.domain.repositories.get_mut(id).unwrap().gone = true;
                h.domain
                    .timers
                    .arm(crate::domain::Alarm::Check { repository: id, commit: 999, context: 0 }, Time::ZERO)
                    .unwrap();
            }
            _ => unreachable!("five reference corruptions"),
        }
        assert!(malformed(Domain::from_image(&calm(), &h.domain.image().unwrap()).err()), "corruption {corruption}");
    }
}
fn matches_limits(error: Option<Refusal>) -> bool {
    match error {
        Some(Refusal::Limits(_)) => true,
        Some(Refusal::Busy | Refusal::Malformed(_)) | None => false,
    }
}
#[test]
fn retained_queue_admissions_need_not_follow_a_pulls_new_head_or_open_state() {
    let config = Config {
        limits: Limits { name_bytes: 128, branches: 8, ..LIMITS },
        latency_min: Duration::ZERO,
        latency_max: Duration::ZERO,
        ..calm()
    };
    let mut configured = setup();
    configured.checks.silent = 1000;
    configured.protection = Some(Protection {
        linear: false,
        code_owners: false,
        bypassing_users: Box::new([]),
        queue: true,
        branch: copy_of(MAIN),
        contexts: names(&[b"ci"]),
        approvals: 0,
        dismiss_stale: false,
    });
    let mut h = Harness::with(config, configured);
    let head = h.commit(FIRST, &[(b"src", b"one")]);
    h.push(ENGINE, b"work", head).unwrap();
    h.open(b"work").unwrap();
    h.ok(ENGINE, write(Write::Enqueue { number: 1, head }));
    let advanced = h.commit(head, &[(b"src", b"two")]);
    h.push(ENGINE, b"work", advanced).unwrap();
    h.ok(ENGINE, write(Write::Close { number: 1 }));
    let other = restored(&h);
    assert_eq!(other.domain.queue_entries(REPOSITORY).unwrap().first().unwrap().head, head);
    assert_eq!(other.pull(1).commit, advanced);
    assert_eq!(other.pull(1).state, State::Closed);
    same_read(&h, &other, Read::Pull { number: 1 });
}
#[test]
fn generated_ci_text_is_retained_even_when_incoming_bodies_have_no_room() {
    let config = Config { limits: Limits { body_bytes: 0, ..LIMITS }, ..calm() };
    let mut h = Harness::new(config);
    let head = h.commit(FIRST, &[(b"src", b"one")]);
    h.push(ENGINE, b"work", head).unwrap();
    let mut other = restored(&h);
    same_read(&h, &other, Read::Checks { commit: head });
    other.settle();
    let _image = other.domain.image().unwrap();
    let again = restored(&other);
    same_read(&other, &again, Read::Checks { commit: head });
}

#[test]
fn late_branch_creation_and_recorded_merge_survive_an_image() {
    use crate::api::{ManualMerges, MergeHow};
    for record in [false, true] {
        let mut h = Harness::new(calm());
        let head = h.commit(FIRST, &[(b"README", b"work")]);
        h.push(ENGINE, b"work", head).unwrap();
        let Answer::Created(number) = h.open(b"work").unwrap() else { panic!("pull") };
        let op = if record {
            crate::manual_merges(
                &mut h.domain,
                REPOSITORY,
                ManualMerges { allow_manual_merge: true, autodetect_manual_merge: false },
            );
            write(Write::Merge { number, head, how: MergeHow::Recorded { commit: FIRST } })
        } else {
            Op::Git(Git::PushNew { branch: copy_of(b"landing"), commit: head })
        };
        h.settle();
        h.env.limits.landing = 1000;
        h.env.limits.land_min = Duration::from_secs(3);
        h.env.limits.land_max = Duration::from_secs(3);
        assert_eq!(h.call(ENGINE, op), Err(Error::Timeout));
        let mut other = restored(&h);
        other.until(Time::from_nanos(Duration::from_secs(3).as_nanos()));
        if record {
            assert_eq!(other.pull(number).merged, Some(FIRST));
            assert_eq!(other.branch(MAIN), Some(FIRST));
        } else {
            assert_eq!(other.branch(b"landing"), Some(head));
        }
        other.settle();
        assert!(other.answered.is_empty(), "restoration retains no reply right");
    }
}
