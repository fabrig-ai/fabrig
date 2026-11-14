use super::*;

const TASK: u64 = 1;
const ATTEMPT: u64 = 2;
const HEAD: skein_forge::api::Commit = [7; 32];
const RESTART: Clock = Clock { now: 1 };

#[test]
fn a_commit_feed_matches_whole_projection_through_stories_and_cold_restarts() {
    // Unit-test builds compare every fed snapshot with the whole projection,
    // before connector evidence and before the referee receives it.
    for story in [stories::Story::RepairCi, stories::Story::GoalIssue] {
        let outcome =
            stories::run(story, jig_conformance::Cut::AfterCommit(20), 71).expect("whole and keyed evidence agree");
        assert!(outcome.crashes.contains(&20), "the comparison includes a cold restart");
    }
}

#[test]
fn shared_policy_refreshes_adapter_edits_and_replaces_referee_edits() {
    let mut world = jig_conformance::Harness::<Temper>::new(Config::new(71, None), 71);
    world.drain().expect("faithful startup");
    let before = world.referee.policy.clone();
    world.peers.policy.delivery_bound = 1234;
    world.iterate().expect("edited scenario policy");
    assert_eq!(world.referee.policy.delivery_bound, 1234);
    assert_eq!(before.delivery_bound, 60_000_000_000);
    world.referee.policy.delivery_bound = 999;
    world.iterate().expect("refresh the complete independent policy again");
    assert_eq!(world.referee.policy.delivery_bound, 1234);
    assert_eq!(world.peers.policy.delivery_bound, 1234);
}

#[test]
fn indexed_effect_evidence_matches_whole_rows_for_each_origin_and_cold_retry() {
    // Unit-test builds compare effect identities, outcomes and issue evidence
    // with full walks. These stories exercise call, projection and procedure keys.
    for (story, cut) in [
        // After the uncertain creation's entry retired; its scripted crash covers the retry.
        (stories::Story::IssueCreation, 17),
        // The goal issue's creation attempt is saved but has not reached the forge.
        (stories::Story::GoalIssue, 19),
        (stories::Story::LostMerge, 20),
    ] {
        let outcome = stories::run(story, jig_conformance::Cut::AfterCommit(cut), 71)
            .expect("indexed and whole effect evidence agree");
        assert!(outcome.crashes.contains(&cut));
    }
}

#[test]
fn a_worker_reports_its_unchanged_push_head_after_an_engine_restart() {
    let config = Config::new(71, Some(jig_conformance::scenarios::Scenario::Grace));
    let mut peers = Temper::peers(&config, 71);
    peers.reported.insert(TASK, HEAD);
    Temper::cold(&mut peers, &Store::new(), RESTART).expect("cold peer evidence");
    let event = link::worker_event(
        &mut peers,
        workers::Up::Answer {
            channel: link::CHANNEL,
            task: TASK,
            attempt: ATTEMPT,
            spent: 0,
            end: workers::End::Parked,
            read: None,
            refused: Box::new([]),
        },
    );
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Up { parts, .. }, .. }) = event else {
        panic!("worker answer reaches the engine");
    };
    assert_eq!(
        parts.pushed,
        Box::new([forge::Pushed {
            tag: forge::tag(temper_engine_forge_world::REPO.forge, temper_engine_forge_world::REPO.repository),
            commit: HEAD,
            checked: true
        }]) as Box<[forge::Pushed]>
    );
}

#[test]
fn ci_evidence_follows_native_commit_and_pull_reads() {
    use skein_fake_forge_domain::api as raw;
    use skein_forge::api;
    use temper_engine_forge_world::translate;

    let clock = Clock { now: 1_000_000_000 };
    let mut forge = systems::FakeForge::new(91);
    forge.create(clock, b"change");
    let head = forge.advance(clock, b"change", b"new head");
    forge.poll(clock);
    let raw_head = forge.domain.branch(b"org/repo", b"change").expect("change head");
    let raw::Answer::Created(number) = forge.external(
        clock,
        raw::Op::Write(raw::Write::OpenPull {
            title: b"change".as_slice().into(),
            body: Box::new([]),
            head: b"change".as_slice().into(),
            base: b"main".as_slice().into(),
            opens_empty: false,
        }),
    ) else {
        panic!("opened pull");
    };
    let reads = [
        api::Read::Statuses { commit: head, page: 1 },
        api::Read::Checks { commit: head },
        api::Read::PullFor { head: b"change".as_slice().into(), base: b"main".as_slice().into() },
        api::Read::Pull { number, ci: false },
    ];
    let store = Store::new();
    for state in [raw::Check::Passed, raw::Check::Pending, raw::Check::Failed] {
        forge.external(
            clock,
            raw::Op::Write(raw::Write::Status {
                commit: raw_head,
                context: b"build".as_slice().into(),
                state,
                description: Box::new([]),
                link: Box::new([]),
            }),
        );
        for read in &reads {
            forge.take_observed();
            let events =
                forge.call(&store, clock, Token::new(1), temper_engine_forge_world::REPO, api::Op::Read(read.clone()));
            assert!(events.is_empty());
            forge.poll(clock);
            let observed = forge.take_observed();
            let [r::Observed::Read { resource, state: observed_state, at }] = observed.as_slice() else {
                panic!("one CI observation for {read:?}: {observed:?}");
            };
            let mut expected = observations::repository();
            expected.path.extend([b"branch".to_vec(), b"main".to_vec()]);
            assert_eq!(*resource, expected);
            assert_eq!(*observed_state, (state == raw::Check::Passed).then(|| observations::value(&head)));
            assert_eq!(*at, clock.now);
        }
    }
    // Reading the base's passing CI must not stand in for the change head.
    let base = translate::commit(forge.domain.branch(b"org/repo", b"main").expect("main head"));
    forge.call(
        &store,
        clock,
        Token::new(1),
        temper_engine_forge_world::REPO,
        api::Op::Read(api::Read::Checks { commit: base }),
    );
    forge.poll(clock);
    assert_eq!(forge.take_observed().len(), 0);
}

#[test]
fn expense_observations_keep_lost_turns_before_sorted_task_totals() {
    use jig_fake_workers::Expense;
    let mut actual = vec![r::Observed::Restart];
    observations::expenses(&std::collections::BTreeMap::new(), &mut actual);
    assert_eq!(actual, vec![r::Observed::Restart]);
    let expenses = std::collections::BTreeMap::from([
        ((3, 8), Expense { spent: 7, lost: 4, lost_turns: 2, ..Expense::default() }),
        ((1, 4), Expense { spent: 11, kept: 11, ..Expense::default() }),
        ((3, 7), Expense { spent: 5, lost: 2, lost_turns: 1, ..Expense::default() }),
        ((2, 1), Expense::default()),
    ]);
    observations::expenses(&expenses, &mut actual);
    assert_eq!(
        actual,
        vec![
            r::Observed::Restart,
            r::Observed::LostTurns { task: 3, attempt: 7, turns: 1, spent: 2 },
            r::Observed::LostTurns { task: 3, attempt: 8, turns: 2, spent: 4 },
            r::Observed::Spend { task: 1, spent: 11, lost_bound: 0 },
            r::Observed::Spend { task: 2, spent: 0, lost_bound: 0 },
            r::Observed::Spend { task: 3, spent: 12, lost_bound: 6 },
        ]
    );
}
