//! A hint invalidates the procedures whose fresh facts it can concern.
use skein_forge::api::{Change, Hint, Op, Read};
use temper_engine_domain_forge as forge;
use temper_engine_domain_world::forge_routes::{hint_step, hint_world, shared_goal_hint_world};
use temper_engine_forge_world::{REPO, World, translate};

fn hint(world: &mut World, change: Change) -> Vec<u64> {
    world.take_seen();
    world.event(forge::Event::Hint { hint: Hint { repository: REPO, change, key: None } });
    world.run_for(1);
    world
        .take_seen()
        .iter()
        .filter_map(|request| if let forge::Request::WakeProcedure { task } = request { Some(*task) } else { None })
        .collect()
}

fn reads_for(world: &World, task: u64) -> Vec<Op> {
    let Some(forge::Stored::Change(row)) = world.stored().get(&forge::Key::Change(task)) else {
        panic!("live change {task}")
    };
    let base = translate::commit(world.branch(&row.base));
    let head = translate::commit(world.branch(&row.branch));
    vec![
        Op::Read(Read::Branch { branch: row.branch.clone() }),
        Op::Read(Read::Branch { branch: row.base.clone() }),
        Op::Read(match row.pull {
            Some(number) => Read::Pull { number, ci: true },
            None => Read::PullFor { head: row.branch.clone(), base: row.base.clone() },
        }),
        Op::Read(Read::Checks { commit: base }),
        Op::Read(Read::Commits { before: head, after: base, page: 1 }),
        Op::Read(Read::Reviews { number: row.pull.expect("opened pull"), page: 1, skip: 0 }),
    ]
}

#[test]
fn a_repository_hint_wakes_only_the_changes_it_concerns() {
    let mut world = hint_world();
    for (change, expected) in [
        (Change::Branch(b"temper/a".as_slice().into()), vec![51]),
        (
            Change::BranchMoved {
                branch: b"release".as_slice().into(),
                head: translate::commit(world.branch(b"release")),
                actor: 2,
            },
            vec![52],
        ),
        (Change::Branch(b"unmatched-or-renamed".as_slice().into()), vec![51, 52]),
        (Change::Branch(Box::new([])), vec![51, 52]),
        (Change::Item(1), vec![51]),
        (Change::Item(2), vec![52]),
    ] {
        let wakes = hint(&mut world, change.clone());
        assert_eq!(wakes, expected, "{change:?}");
        assert!(world.take_calls().is_empty(), "no resources held by this scripted parent");
        for task in wakes {
            let expected = reads_for(&world, task);
            hint_step(&mut world, task, 100 + task);
            assert_eq!(world.take_calls(), expected, "fresh reads for {task}, {change:?}");
        }
    }
    // An unchanged commit's CI is read alone, not a reason to gather every
    // procedure fact again (keeping-up.md, 2, A commit's news).
    for branch in [b"temper/a".as_slice(), b"release"] {
        let commit = translate::commit(world.branch(branch));
        assert_eq!(hint(&mut world, Change::Commit(commit)).len(), 0);
        assert_eq!(world.take_calls(), [Op::Read(Read::Checks { commit })]);
    }
}

#[test]
fn only_a_relevant_hint_restarts_an_unfinished_fresh_read_chain() {
    use skein_lib::Duration;
    use temper_engine_domain_forge_change as change;
    for (branch, expected_wake, passes) in [(b"temper/a".as_slice(), 51, 1), (b"release".as_slice(), 52, 2)] {
        let mut world = hint_world();
        let expected: Vec<_> = (0..passes).flat_map(|_| reads_for(&world, 52)).collect();
        world.slow_calls(Duration::from_millis(200));
        let step = |entry| forge::Event::StepChange {
            merge_job: false,
            task: 52,
            entry,
            heard: change::Heard {
                delegate_running: false,
                delegate_live: false,
                review_request: None,
                start_conflict: None,
                lacking: None,
                spend: None,
                delegate: change::Status::Unknown,
                effect: change::EffectResult::None,
                released: false,
                cancelled: false,
            },
            gates: Box::new([]),
            queue_repair_active: false,
            pending: Box::new([]),
            kept: true,
        };
        world.event(step(150));
        assert_eq!(hint(&mut world, Change::Branch(branch.into())), [expected_wake]);
        if expected_wake == 52 {
            world.event(step(151));
        }
        world.run_for(3);
        assert_eq!(world.take_calls(), expected, "hint on {}", String::from_utf8_lossy(branch));
    }
}

#[test]
fn both_fresh_base_facts_remain_dependencies_when_the_pull_and_branch_disagree() {
    use skein_lib::{Duration, Wall};
    use temper_engine_domain_forge_change as change;
    let mut world = hint_world();
    let before = translate::commit(world.branch(b"release"));
    world.slow_calls(Duration::from_secs(1));
    world.event(forge::Event::StepChange {
        merge_job: false,
        task: 52,
        entry: 152,
        heard: change::Heard {
            delegate_running: false,
            delegate_live: false,
            review_request: None,
            start_conflict: None,
            lacking: None,
            spend: None,
            delegate: change::Status::Unknown,
            effect: change::EffectResult::None,
            released: false,
            cancelled: false,
        },
        gates: Box::new([]),
        queue_repair_active: false,
        pending: Box::new([]),
        kept: true,
    });
    // The base branch's read has been sent and snapshots the old tip, but
    // the pull read has not been sent. Move the base between those reads.
    world.run_to(Wall::from_nanos(world.wall().as_nanos() + Duration::from_millis(1500).as_nanos()));
    world.push(b"release", b"base moved between reads");
    let after = translate::commit(world.branch(b"release"));
    assert_ne!(before, after);
    world.run_for(6);
    let calls = world.take_calls();
    assert!(calls.contains(&Op::Read(Read::Checks { commit: before })), "CI used the branch read's tip");
    for (commit, source) in [(before, "the decision's CI/compare base"), (after, "the pull's newer base")] {
        assert!(hint(&mut world, Change::Commit(commit)).is_empty(), "CI is read before waking: {source}");
        assert!(world.take_calls().contains(&Op::Read(Read::Checks { commit })), "read {source}");
    }
}

#[test]
fn a_named_issue_still_gets_its_hint_reads_and_wakes_its_change() {
    let mut world = hint_world();
    let issue = world.issue(b"dependency");
    world.event(forge::Event::Names {
        task: 52,
        resources: Box::new([forge::Name {
            forge: REPO.forge,
            repository: REPO.repository,
            what: forge::What::Issue(issue),
        }]),
    });
    world.run_for(1);
    world.take_calls();
    assert_eq!(hint(&mut world, Change::Item(issue)), [52]);
    let calls = world.take_calls();
    assert!(
        calls.iter().any(|op| matches!(op, Op::Read(Read::Item { number, .. }) if *number == issue)),
        "hint keep reads: {calls:?}"
    );
    // An identical second hint must read the held item again too.
    world.run_for(1);
    world.take_calls();
    assert_eq!(hint(&mut world, Change::Item(issue)), [52]);
    let calls = world.take_calls();
    assert!(
        calls.iter().any(|op| matches!(op, Op::Read(Read::Item { number, .. }) if *number == issue)),
        "repeated hint keep reads: {calls:?}"
    );
}

#[test]
fn subscribed_ci_and_unknown_change_facts_keep_their_wakes() {
    let mut world = hint_world();
    let head = translate::commit(world.branch(b"temper/b"));
    world.event(forge::Event::Subscribe {
        subscription: forge::Subscriber {
            task: 51,
            number: 1,
            topic: forge::Topic::Ci { repository: REPO, head },
            own_change: None,
            goal_tasks: None,
            paths: Box::new([]),
        },
    });
    world.run_for(1);
    assert_eq!(
        hint(&mut world, Change::Commit(head)),
        [51],
        "only the subscriber wakes before unchanged head CI answers"
    );

    let Some(forge::Stored::Change(mut row)) = world.stored().get(&forge::Key::Change(51)).cloned() else {
        panic!("first live change")
    };
    row.task = 53;
    row.change.task = 53;
    row.branch = b"temper/not-yet-pushed".as_slice().into();
    row.pull = None;
    row.change.last_head = None;
    row.change.state = temper_engine_domain_forge_change::State::Producing { requested: false };
    world.event(forge::Event::Change { row: *row });
    assert_eq!(hint(&mut world, Change::Item(2)), [52], "an unpushed change has no pull");
    assert_eq!(hint(&mut world, Change::Commit(head)), [51], "only the subscriber wakes; an unpushed change has no CI");
    assert_eq!(hint(&mut world, Change::Item(0)), [51, 52], "unnamed item concerns only changes with heads");
    assert_eq!(hint(&mut world, Change::Commit([0; 32])), [51, 52], "unnamed commit concerns only changes with heads");
    for (number, topic, change, expected) in [
        (2, forge::Topic::Ci { repository: REPO, head }, Change::Commit(head), vec![51, 53]),
        (3, forge::Topic::Pull { repository: REPO, number: 99 }, Change::Item(99), vec![53]),
    ] {
        world.event(forge::Event::Subscribe {
            subscription: forge::Subscriber {
                task: 53,
                number,
                topic,
                own_change: None,
                goal_tasks: None,
                paths: Box::new([]),
            },
        });
        assert_eq!(hint(&mut world, change), expected, "an unpushed change keeps its subscribed inputs");
    }
    let dependency = world.issue(b"unpushed dependency");
    world.event(forge::Event::Names {
        task: 53,
        resources: Box::new([forge::Name {
            forge: REPO.forge,
            repository: REPO.repository,
            what: forge::What::Issue(dependency),
        }]),
    });
    assert_eq!(hint(&mut world, Change::Item(dependency)), [53], "named inputs still wake an unpushed change");
    let mut unknown = temper_engine_forge_world::changes::change_row();
    unknown.task = 54;
    unknown.change.task = 54;
    unknown.change.last_head = Some([54; 32]);
    world.event(forge::Event::Change { row: unknown });
    assert_eq!(hint(&mut world, Change::Item(98)), [54], "a pushed change with unknown pull still matches");
    assert_eq!(hint(&mut world, Change::Commit([98; 32])), [54], "a pushed change with unknown base still matches");
}

fn waiting_step(world: &mut World, entry: u64) {
    use temper_engine_domain_forge_change as change;
    world.event(forge::Event::StepChange {
        task: 53,
        entry,
        heard: change::Heard {
            delegate_running: false,
            delegate_live: false,
            review_request: None,
            start_conflict: None,
            lacking: None,
            spend: None,
            delegate: change::Status::Unknown,
            effect: change::EffectResult::None,
            released: false,
            cancelled: false,
        },
        gates: Box::new([]),
        queue_repair_active: false,
        pending: Box::new([change::Pending { task: 40, kind: change::PendingKind::Other }]),
        kept: false,
        merge_job: false,
    });
    world.run_for(1);
}

fn unpushed_profile() -> World {
    let mut world = hint_world();
    world.fallback(skein_lib::Duration::from_secs(300));
    let mut unpushed = temper_engine_forge_world::changes::change_row();
    unpushed.task = 53;
    unpushed.change.task = 53;
    unpushed.branch = b"temper/unpushed".as_slice().into();
    world.event(forge::Event::Change { row: unpushed });
    waiting_step(&mut world, 153);
    assert_eq!(
        world.take_calls(),
        [Op::Read(Read::Branch { branch: b"temper/unpushed".as_slice().into() })],
        "first step reads only the absent branch",
    );
    waiting_step(&mut world, 154);
    assert!(world.take_calls().is_empty(), "a dependency wake before fallback reads nothing");
    world
}

#[test]
#[expect(clippy::too_many_lines, reason = "the request profile keeps shared-base, unpushed and landing cases together")]
fn two_change_hint_request_profile() {
    use temper_engine_domain_forge_change as change;
    const LANDING_REQUEST_BOUND: usize = 18;
    let mut world = hint_world();
    world.fallback(skein_lib::Duration::from_secs(300));
    for (change, expected_wakes, common_goal) in [
        (Change::Branch(b"temper/a".as_slice().into()), vec![51], false),
        (Change::Branch(b"release".as_slice().into()), vec![52], false),
        (Change::Branch(b"unmatched-or-renamed".as_slice().into()), vec![51, 52], false),
        (Change::Branch(b"release".as_slice().into()), vec![51, 52], true),
    ] {
        if common_goal {
            world = shared_goal_hint_world();
        }
        let wakes = hint(&mut world, change.clone());
        assert_eq!(wakes, expected_wakes, "{change:?}: woken changes");
        let expected_reads: Vec<_> = wakes.iter().flat_map(|task| reads_for(&world, *task)).collect();
        if common_goal {
            world.slow_calls(skein_lib::Duration::from_secs(2));
        }
        for task in wakes {
            hint_step(&mut world, task, 100 + task);
        }
        if common_goal {
            world.run_for(20);
            world.slow_calls(skein_lib::Duration::ZERO);
        }
        let calls = world.take_calls();
        if common_goal {
            let base = Op::Read(Read::Branch { branch: b"release".as_slice().into() });
            assert_eq!(calls.iter().filter(|op| **op == base).count(), 1, "two changes share their fresh base read");
            assert_eq!(calls.len(), expected_reads.len() - 1, "one shared request saved");
            for op in expected_reads {
                assert!(calls.contains(&op), "gather still read {op:?}");
            }
        } else {
            assert_eq!(calls.len(), expected_reads.len(), "{change:?}: fresh-read request count");
            assert_eq!(calls, expected_reads, "{change:?}: only the woken changes' fresh reads");
        }
    }
    let branch = Op::Read(Read::Branch { branch: b"temper/unpushed".as_slice().into() });
    let head = translate::commit(world.branch(b"temper/a"));
    for (change, expected_wakes, expected_reads) in [
        (Change::Commit(head), vec![], vec![Op::Read(Read::Checks { commit: head })]),
        (Change::Commit([99; 32]), vec![], vec![]),
        (Change::Item(99), vec![], vec![]),
        (Change::Branch(b"unmatched".as_slice().into()), vec![51, 52], vec![]),
        (Change::Branch(b"temper/unpushed".as_slice().into()), vec![53], vec![branch.clone()]),
        (Change::Branch(Box::new([])), vec![51, 52, 53], vec![branch]),
    ] {
        let mut unpushed = unpushed_profile();
        let wakes = hint(&mut unpushed, change.clone());
        assert_eq!(wakes, expected_wakes, "{change:?}");
        if wakes.contains(&53) {
            waiting_step(&mut unpushed, 155);
        }
        assert_eq!(unpushed.take_calls(), expected_reads, "{change:?}: unpushed change reads no other facts");
    }
    let mut world = hint_world();
    let mut landing_requests = [0; 2];
    for (at, task) in [51, 52].into_iter().enumerate() {
        let Some(forge::Stored::Change(row)) = world.stored().get(&forge::Key::Change(task)) else {
            panic!("change {task}")
        };
        let gates = Box::new([change::GateReport {
            number: 9,
            head: row.change.last_head.expect("pushed head"),
            status: change::Status::Passed,
        }]);
        assert_eq!(
            temper_engine_forge_world::changes::change_step_task(
                &mut world,
                task,
                200 + task,
                change::Status::Unknown,
                gates.clone(),
                false,
                false,
            ),
            change::Decision::Ready
        );
        assert!(matches!(
            temper_engine_forge_world::changes::change_step_task(
                &mut world,
                task,
                300 + task,
                change::Status::Unknown,
                gates,
                false,
                false,
            ),
            change::Decision::Effect(change::Effect::Merge { .. })
        ));
        assert!(
            world
                .stored()
                .iter()
                .any(|(_, row)| matches!(row, forge::Stored::Landed { task: landed, .. } if *landed == task))
        );
        // Keep reads may interleave with the landing's fresh reads and merge.
        // Bound their total; their order is not the hint-wake contract
        // (temper/docs/design/domain/forge/keeping-up.md, 2 and 3).
        let count = world.take_calls().len();
        assert!(
            count <= LANDING_REQUEST_BOUND,
            "landing {task}: {count} requests exceeds bound {LANDING_REQUEST_BOUND}"
        );
        landing_requests[at] = count;
    }
    let [first, second] = landing_requests;
    assert!(second <= first, "landing 52 sends {second} requests to landing 51's {first}");
}
