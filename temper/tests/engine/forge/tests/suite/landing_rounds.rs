//! A round's own branch, preparation, record and cleanup (domain/landing.md, 5–15).
use skein_fake_forge_domain::{self as fake, api as raw};
use skein_forge::api;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{
    REPO, World, changes,
    landings::{self, CHANGE, LANDING},
    translate,
};

const MEMBER: &[u8] = b"temper/51";
const GOAL: &[u8] = b"temper/1/g99";
fn number(commit: api::Commit) -> u64 {
    fake::commit_count(&commit).expect("fake commit")
}
fn start(goal: bool) -> World {
    let mut world = landings::turn_world();
    let base = if goal { GOAL } else { b"main" };
    if goal {
        world.external(raw::Op::Git(raw::Git::Create { branch: base.into(), commit: world.branch(b"main") }), 1);
    }
    world.produce_on(MEMBER, base);
    landings::ready_change_into(&mut world, CHANGE, MEMBER, base);
    register(&mut world);
    world
}
fn register(world: &mut World) {
    let row = landings::landing_row(world.change(CHANGE));
    landings::register(world, row);
    assert_eq!(landings::step_landing_raw(world, LANDING, 100), landing::Decision::None);
    assert!(matches!(landings::step_landing_raw(world, LANDING, 101), landing::Decision::Prepare { .. }));
}
fn prepared(squash: bool, goal: bool) -> World {
    let mut world = start(goal);
    world.prepare_landing(LANDING, squash);
    landings::drive_landing(&mut world, LANDING);
    world
}
fn key(world: &World) -> [u8; 32] {
    world
        .stored()
        .iter()
        .find_map(|(_, row)| match row {
            top::Stored::Receipt { key, .. } => Some(key.key),
            _ => None,
        })
        .expect("receipt")
}
fn candidate_commit(world: &World) -> raw::CommitMessage {
    let member = &world.landing(LANDING).landing.members[0];
    let raw::Answer::Commits { commits } = world.inspect(&raw::Read::Commits {
        before: number(member.base.expect("base")),
        after: number(member.candidate.expect("candidate")),
    }) else {
        panic!("range")
    };
    commits.last().expect("candidate").clone()
}
#[test]
fn merging_and_squashing_the_same_member_have_equal_trees_and_keys() {
    let mut merged = prepared(false, false);
    let mut squashed = prepared(true, false);
    let merge = candidate_commit(&merged);
    let squash = candidate_commit(&squashed);
    assert_eq!(key(&merged), key(&squashed));
    assert_eq!(merge.parents.len(), 2);
    assert_eq!(squash.parents.as_ref(), &[number(squashed.landing(LANDING).landing.members[0].base.expect("base"))]);
    assert!(String::from_utf8_lossy(&squash.message).contains("Head: "));
    assert_eq!(
        merged.inspect(&raw::Read::Tree { commit: merge.id }),
        squashed.inspect(&raw::Read::Tree { commit: squash.id })
    );
    assert!(
        !merged
            .take_calls()
            .iter()
            .any(|op| matches!(op, api::Op::Write(api::Write::Merge { how: api::MergeHow::Recorded { .. }, .. })))
    );
    assert!(squashed.take_calls().iter().any(|op| matches!(op, api::Op::Write(api::Write::Merge { how: api::MergeHow::Recorded { commit }, .. }) if *commit == translate::commit(squash.id))));
}
#[test]
fn a_goal_train_is_recorded_without_a_test_dispatch() {
    let mut world = prepared(true, true);
    let row = world.landing(LANDING).clone();
    let member = &row.landing.members[0];
    assert_eq!(row.landing.round.combined, Some(landing::Combine::Squash));
    assert!(row.landing.round.recorded);
    assert!(
        matches!(world.change(CHANGE).change.state, change::State::Landed { merge, .. } if Some(merge) == member.landed)
    );
    let raw::Answer::Pull(pull) = world.inspect(&raw::Read::Pull { number: member.pull }) else {
        panic!("member pull")
    };
    assert_eq!(pull.merged.map(translate::commit), member.landed);
    assert!(!world.take_calls().iter().any(|op| matches!(op, api::Op::Write(api::Write::Dispatch { .. }))));
}
#[test]
fn a_catch_up_reads_its_upstream_and_is_merged_under_squash() {
    let mut world = landings::turn_world();
    world.external(raw::Op::Git(raw::Git::Create { branch: GOAL.into(), commit: world.branch(b"main") }), 1);
    let name = top::Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: top::What::Branch(Box::new([
            b"temper".as_slice().into(),
            b"1".as_slice().into(),
            b"g99".as_slice().into(),
        ])),
    };
    world.event(top::Event::Hold { entry: 0, task: 99, resource: name, from: None });
    world.push_file(b"main", b"upstream", b"upstream moved");
    let upstream = translate::commit(world.branch(b"main"));
    world.produce_on(MEMBER, b"main");
    landings::ready_change_into(&mut world, CHANGE, MEMBER, GOAL);
    register(&mut world);
    assert_eq!(world.landing(LANDING).landing.round.upstream, Some(upstream));
    world.prepare_landing(LANDING, true);
    landings::drive_landing(&mut world, LANDING);
    assert_eq!(world.landing(LANDING).landing.round.combined, Some(landing::Combine::Merge));
}
fn until_merge(world: &mut World) {
    for entry in 102..140 {
        if matches!(landings::step_landing_raw(world, LANDING, entry), landing::Decision::Merge { .. }) {
            world.run_for(4);
            return;
        }
    }
    panic!("round merge not made: {:?}", world.landing(LANDING));
}
#[test]
fn a_refused_record_holds_with_the_member_landed_and_release_records_it_again() {
    let mut world = start(true);
    world.prepare_landing(LANDING, true);
    until_merge(&mut world);
    world.permission(raw::Permission::Read);
    assert!(matches!(landings::step_landing_raw(&mut world, LANDING, 200), landing::Decision::Record { .. }));
    world.run_for(4);
    assert_eq!(
        landings::step_landing_raw(&mut world, LANDING, 201),
        landing::Decision::Hold(landing::Hold::Record { member: 0 })
    );
    assert!(world.landing(LANDING).landing.members[0].landed.is_some());
    assert!(matches!(world.change(CHANGE).change.state, change::State::Ready { .. }));
    assert!(
        String::from_utf8_lossy(&world.landing_hold_words(LANDING).expect("hold words"))
            .starts_with("turn: record of #1: ")
    );
    world.restart();
    world.permission(raw::Permission::Admin);
    world.event(top::Event::StepLanding {
        task: LANDING,
        entry: 202,
        heard: top::LandingHeard {
            job: None,
            outcomes: Box::new([]),
            dependencies: Box::new([]),
            held: Box::new([]),
            preparation: landing::Preparation::Idle,
            failure: Box::new([]),
            released: true,
            cancelled: false,
            kept: Box::new([]),
        },
    });
    world.run_for(1);
    landings::drive_landing(&mut world, LANDING);
    assert!(world.landing(LANDING).landing.round.recorded);
    assert!(matches!(world.change(CHANGE).change.state, change::State::Landed { .. }));
    assert_eq!(
        world
            .take_calls()
            .iter()
            .filter(|op| matches!(op, api::Op::Write(api::Write::Merge { how: api::MergeHow::Recorded { .. }, .. })))
            .count(),
        2
    );
}
#[test]
fn an_already_contained_member_lands_at_the_base_without_preparing() {
    let mut world = landings::turn_world();
    world.produce(MEMBER);
    landings::ready_change(&mut world, CHANGE, MEMBER);
    let head = world.branch(MEMBER);
    world.external(
        raw::Op::Git(raw::Git::Push {
            branch: b"main".as_slice().into(),
            commit: head,
            expected: Some(world.branch(b"main")),
        }),
        1,
    );
    world.run_for(4);
    let base = translate::commit(world.branch(b"main"));
    let row = landings::landing_row(world.change(CHANGE));
    world.event(top::Event::Landing { row });
    world.take_calls();
    landings::drive_landing(&mut world, LANDING);
    assert_eq!(world.landing(LANDING).landing.members[0].landed, Some(base));
    assert!(matches!(world.change(CHANGE).change.state, change::State::Landed { merge, .. } if merge == base));
    assert!(!world.take_calls().iter().any(|op| matches!(op, api::Op::Write(_))));
}
#[test]
fn an_exhausted_prepare_job_holds_its_landing_not_its_member() {
    let mut world = start(false);
    world.event(top::Event::StepLanding {
        task: LANDING,
        entry: 102,
        heard: top::LandingHeard {
            job: None,
            outcomes: Box::new([]),
            dependencies: Box::new([]),
            held: Box::new([]),
            preparation: landing::Preparation::Failed,
            failure: b"push refused".as_slice().into(),
            released: false,
            cancelled: false,
            kept: Box::new([]),
        },
    });
    world.run_for(1);
    assert!(matches!(world.landing(LANDING).landing.phase, landing::Phase::Held { why: landing::Hold::Prepare, .. }));
    assert_eq!(world.landing_hold_words(LANDING).expect("hold").as_ref(), b"prepare job: push refused");
    assert!(matches!(world.change(CHANGE).change.state, change::State::Ready { .. }));
}
#[test]
fn a_restart_finds_uncertain_opening_and_record_by_reading_the_forge() {
    for record in [false, true] {
        let mut world = start(true);
        world.prepare_landing(LANDING, true);
        if record {
            until_merge(&mut world);
        }
        world.land_writes_late(skein_lib::Duration::from_secs(3));
        let decision = landings::step_landing_raw(&mut world, LANDING, 200);
        assert_eq!(matches!(decision, landing::Decision::Record { .. }), record);
        assert_eq!(decision == landing::Decision::Open, !record);
        world.run_for(4);
        world.answer_writes();
        world.restart();
        landings::drive_landing(&mut world, LANDING);
        assert!(world.landing(LANDING).landing.round.recorded);
        let writes = world.take_calls();
        assert_eq!(writes.iter().filter(|op| matches!(op, api::Op::Write(api::Write::OpenPull { .. }))).count(), 1);
        assert_eq!(
            writes
                .iter()
                .filter(|op| matches!(
                    op,
                    api::Op::Write(api::Write::Merge { how: api::MergeHow::Recorded { .. }, .. })
                ))
                .count(),
            1
        );
    }
}
#[test]
fn a_head_moved_at_the_turn_closes_the_round_and_returns_to_gating() {
    let mut world = start(false);
    world.prepare_landing(LANDING, false);
    let name = top::Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: top::What::Branch(Box::new([b"temper".as_slice().into(), b"51".as_slice().into()])),
    };
    world.event(top::Event::Names { task: 99, resources: Box::new([name.clone()]) });
    world.event(top::Event::Hold { entry: 0, task: 99, resource: name.clone(), from: None });
    world.run_for(1);
    for entry in 102..140 {
        landings::step_landing_raw(&mut world, LANDING, entry);
        if matches!(world.landing(LANDING).landing.phase, landing::Phase::Turn { .. }) {
            break;
        }
    }
    assert!(matches!(world.landing(LANDING).landing.phase, landing::Phase::Turn { .. }));
    let base = world.branch(b"main");
    world.event(top::Event::Claim {
        designs: Box::new([]),
        task: 99,
        attempt: 1,
        writes: Box::new([name.clone()]),
        holders: Box::new([]),
    });
    world.run_for(1);
    world.push_file(MEMBER, b"next", b"new delivery");
    let head = translate::commit(world.branch(MEMBER));
    world.event(top::Event::Answered { task: 99, attempt: 1, pushed: Box::new([(name, head, true)]) });
    world.run_for(1);
    landings::drive_landing(&mut world, LANDING);
    assert!(world.landing(LANDING).landing.members[0].landed.is_none());
    landings::close_landing(&mut world, LANDING);
    assert_eq!(world.branch(b"main"), base);
    assert!(
        matches!(world.change(CHANGE).change.state, change::State::Ready { head: latest, .. } if latest == head),
        "{:?}",
        world.change(CHANGE).change.state
    );
    assert_eq!(world.change(CHANGE).change.landing, None);
    let writes = world.take_calls();
    assert!(writes.iter().any(|op| matches!(op, api::Op::Write(api::Write::Close { .. }))));
    assert!(writes.iter().any(|op| matches!(op, api::Op::Write(api::Write::DeleteBranch { .. }))));
    assert!(!writes.iter().any(|op| matches!(op, api::Op::Write(api::Write::Merge { .. }))));
}
#[test]
fn a_conflict_closes_the_round_and_resolves_then_lands_in_a_later_landing() {
    let mut world = start(false);
    let base = world.landing(LANDING).landing.members[0].base.expect("base");
    world.event(top::Event::StepLanding {
        task: LANDING,
        entry: 102,
        heard: top::LandingHeard {
            job: None,
            outcomes: Box::new([]),
            dependencies: Box::new([]),
            held: Box::new([]),
            preparation: landing::Preparation::Conflict,
            failure: b"file".as_slice().into(),
            released: false,
            cancelled: false,
            kept: Box::new([]),
        },
    });
    world.run_for(1);
    assert_eq!(world.landing(LANDING).landing.members[0].left, Some(landing::Leave::Conflict));
    assert_eq!(world.landing_leave_words(LANDING).as_deref(), Some(b"conflict with the branch: file".as_slice()));
    let mut closed = false;
    for entry in 103..120 {
        let decision = landings::step_landing_raw(&mut world, LANDING, entry);
        if decision == landing::Decision::Finish {
            closed = true;
            break;
        }
    }
    assert!(closed);
    assert!(world.landing(LANDING).landing.members[0].landed.is_none());
    let choice = changes::change_step_task(&mut world, CHANGE, 200, change::Status::Passed, Box::new([]), false, false);
    assert_eq!(choice, change::Decision::Delegate(change::Delegate::Resolve { base }));
    world.resolve(MEMBER, b"resolved");
    for entry in 201..210 {
        changes::change_step_task(&mut world, CHANGE, entry, change::Status::Passed, Box::new([]), false, false);
        if matches!(world.change(CHANGE).change.state, change::State::Ready { .. }) {
            break;
        }
    }
    assert!(matches!(world.change(CHANGE).change.state, change::State::Ready { .. }));
    assert_eq!(world.change(CHANGE).change.landing, None);
    let mut row = landings::landing_row(world.change(CHANGE));
    row.landing.task = 62;
    row.landing.round = landing::Round::new(b"temper/1/l62".as_slice().into());
    row.landing.members[0].branch.clone_from(&row.landing.round.branch);
    world.event(top::Event::Landing { row });
    landings::drive_landing(&mut world, 62);
    assert!(world.landing(62).landing.members[0].landed.is_some());
}

#[test]
fn cancelling_after_the_prefix_is_recorded_tells_the_change_landed_as_its_pull_stands() {
    let mut world = start(true);
    world.prepare_landing(LANDING, true);
    until_merge(&mut world);
    assert!(matches!(landings::step_landing_raw(&mut world, LANDING, 200), landing::Decision::Record { .. }));
    let prefix = world.landing(LANDING).landing.members[0].landed.expect("recorded prefix");
    let pull = world.landing(LANDING).landing.members[0].pull;
    let before = world.inspect(&raw::Read::Pull { number: pull });
    world.take_calls();
    world.event(top::Event::SettleEffects { task: LANDING, root: LANDING, ending: top::ReleaseEnding::Cancelled });
    assert!(matches!(world.change(CHANGE).change.state, change::State::Landed { merge, .. } if merge == prefix));
    assert_eq!(world.change(CHANGE).change.landing, None);
    assert_eq!(world.inspect(&raw::Read::Pull { number: pull }), before);
    assert!(world.take_seen().iter().any(|request| matches!(
        request,
        top::Request::LandingNews { task: LANDING, member: 0, step: top::LandingNews::Landed }
    )));
    assert!(!world.take_calls().iter().any(|op| matches!(op, api::Op::Write(_))));
}

#[test]
fn a_successful_prepare_report_takes_its_branch_without_repeating_the_round() {
    for squash in [false, true] {
        let mut world = start(false);
        let prefix = translate::commit(world.prepare_landing(LANDING, squash));
        // A refused range is not evidence that this job's delivered branch is absent.
        world.script_ancestry(Some(Err(api::Error::TooLarge(api::Excess::unnamed()))));
        let report = if squash {
            br#"{"combined":[{"member":1,"by":"squash"}]}"#.as_slice()
        } else {
            br#"{"combined":[{"member":1,"by":"merge"}]}"#.as_slice()
        };
        let decision = landings::step_landing_heard(
            &mut world,
            LANDING,
            102,
            top::LandingHeard {
                job: None,
                outcomes: top::preparation::outcomes(report, 1).expect("successful report"),
                dependencies: Box::new([]),
                held: Box::new([]),
                preparation: landing::Preparation::Idle,
                failure: report.into(),
                released: false,
                cancelled: false,
                kept: Box::new([]),
            },
        );
        assert_eq!(decision, landing::Decision::Open);
        assert_eq!(world.landing(LANDING).landing.preparation, 1);
        assert_eq!(world.landing(LANDING).landing.members[0].candidate, Some(prefix));
        assert_eq!(
            world.landing(LANDING).landing.members[0].combined,
            Some(if squash { landing::Combine::Squash } else { landing::Combine::Merge })
        );
        world.script_ancestry(None);
        world.restart();
        landings::drive_landing(&mut world, LANDING);
        assert_eq!(world.landing(LANDING).landing.members[0].landed, Some(prefix));
    }
}

#[test]
fn a_successful_prepare_report_is_forgotten_when_the_round_changes() {
    let mut world = start(false);
    world.prepare_landing(LANDING, false);
    let report = br#"{"combined":[{"member":1,"by":"merge"}]}"#.as_slice();
    assert_eq!(
        landings::step_landing_heard(
            &mut world,
            LANDING,
            102,
            top::LandingHeard {
                job: None,
                outcomes: top::preparation::outcomes(report, 1).expect("successful report"),
                dependencies: Box::new([]),
                held: Box::new([]),
                preparation: landing::Preparation::Idle,
                failure: report.into(),
                released: false,
                cancelled: false,
                kept: Box::new([]),
            }
        ),
        landing::Decision::Open
    );
    world.push_file(b"main", b"next-base", b"new base");
    world.restart();
    let mut prepared = false;
    for entry in 103..130 {
        if matches!(landings::step_landing_raw(&mut world, LANDING, entry), landing::Decision::Prepare { .. }) {
            prepared = true;
            break;
        }
    }
    assert!(prepared, "a changed round needs its own prepare");
    assert_eq!(world.landing(LANDING).landing.preparation, 2);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PrepareCut {
    None,
    Released,
    Delete,
    Stale,
    Asked,
    Pushed,
    BeforeRelease,
    RepeatedReport,
    RetainedReport,
}

#[test]
fn a_prepare_ending_after_its_base_moves_starts_one_fresh_round() {
    for cut in [
        PrepareCut::None,
        PrepareCut::Released,
        PrepareCut::Delete,
        PrepareCut::Stale,
        PrepareCut::Asked,
        PrepareCut::Pushed,
        PrepareCut::BeforeRelease,
        PrepareCut::RepeatedReport,
        PrepareCut::RetainedReport,
    ] {
        stale_prepare_round(cut);
    }
}

fn successful_prepare(released: bool) -> top::LandingHeard {
    let report = br#"{"combined":[{"member":1,"by":"merge"}]}"#.as_slice();
    top::LandingHeard {
        job: None,
        outcomes: top::preparation::outcomes(report, 1).expect("successful report"),
        dependencies: Box::new([]),
        held: Box::new([]),
        preparation: landing::Preparation::Idle,
        failure: report.into(),
        released,
        cancelled: false,
        kept: Box::new([]),
    }
}

fn stale_prepare_round(cut: PrepareCut) {
    let mut world = start(false);
    let mut row = world.landing(LANDING).clone();
    row.landing.phase = landing::Phase::Held { was: Box::new(landing::Phase::Preparing), why: landing::Hold::Prepare };
    world.event(top::Event::Restore { record: top::Stored::Landing(row) });
    world.prepare_landing(LANDING, false);
    world.push_file(b"main", b"new-base", b"outside push");
    let base = translate::commit(world.branch(b"main"));
    if cut == PrepareCut::BeforeRelease {
        world.restart();
    }
    landings::step_landing_heard(&mut world, LANDING, 102, successful_prepare(true));
    if cut == PrepareCut::Released {
        world.restart();
    }
    let mut prepares = 0;
    let mut restarted = false;
    for entry in 103..145 {
        let mut decision = landings::step_landing_raw(&mut world, LANDING, entry);
        if !restarted
            && (cut == PrepareCut::Delete && matches!(decision, landing::Decision::Delete { .. })
                || cut == PrepareCut::Stale && decision == landing::Decision::Stale)
        {
            restarted = true;
            world.restart();
        }
        if cut == PrepareCut::RepeatedReport && decision == landing::Decision::Stale {
            decision = landings::step_landing_heard(&mut world, LANDING, entry + 200, successful_prepare(false));
            assert!(
                matches!(decision, landing::Decision::Prepare { .. }),
                "{decision:?}: {:?}",
                world.landing(LANDING)
            );
        }
        if cut == PrepareCut::RetainedReport && decision == landing::Decision::Stale {
            world.restart();
            let mut row = world.landing(LANDING).clone();
            row.landing.round.failure = successful_prepare(false).failure;
            row.job = Some(top::history::JobRecord {
                step: top::history::JobStep {
                    task: 70,
                    attempt: 1,
                    job: top::history::Job::Prepare,
                    outcome: Some(top::history::JobOutcome::Done),
                    head: None,
                },
                round: 1,
                members: Box::new([0]),
                head: None,
                written: true,
            });
            // The restore input is the stalled durable boundary: no base, an old job and its report.
            world.event(top::Event::Restore { record: top::Stored::Landing(row) });
        }
        if matches!(decision, landing::Decision::Prepare { .. }) {
            prepares += 1;
            assert_eq!(world.landing(LANDING).landing.members[0].base, Some(base));
            assert_eq!(world.landing(LANDING).landing.preparation, 2);
            if cut == PrepareCut::Asked {
                world.restart();
            }
            assert_eq!(
                landings::step_landing_raw(&mut world, LANDING, entry + 100),
                landing::Decision::Wait { until: None }
            );
            world.prepare_landing(LANDING, false);
            if cut == PrepareCut::Pushed {
                world.restart();
            }
        }
        if decision == landing::Decision::Finish {
            break;
        }
    }
    let row = world.landing(LANDING);
    assert_eq!(row.landing.phase, landing::Phase::Landed, "{row:?}");
    assert_eq!(row.landing.preparation, 2);
    assert_eq!(prepares, 1);
    assert_eq!(row.landing.members[0].base, Some(base));
}
