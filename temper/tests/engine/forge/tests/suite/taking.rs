//! Taking and its meeting with landing (temper/docs/design/domain/forge/change/opened.md, 4).
use skein_forge::api::{Commit, Error, Op, Read, Write};
use skein_lib::Duration;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_forge_world::{REPO, World, changes, landings, translate};
const TASK: u64 = 51;
const SOURCE: &[u8] = b"docs/x";
const OWN: &[u8] = b"temper/51/c51";

fn name(branch: &[u8]) -> top::Name {
    top::Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: top::What::Branch(branch.split(|byte| *byte == b'/').map(Into::into).collect()),
    }
}
fn register(world: &mut World, at: Option<Commit>) {
    let mut row = changes::change_row();
    row.source = Some(top::ChangeSource { branch: SOURCE.into(), at });
    row.branch = OWN.into();
    row.change.state = change::State::Taking { requested: false };
    row.change.since = world.wall();
    world.event(top::Event::Change { row });
    world.event(top::Event::Names { task: TASK, resources: Box::new([name(b"main")]) });
    world.event(top::Event::Hold { task: TASK, resource: name(OWN), entry: 0, from: None });
    world.take_calls();
    world.observations();
}
fn setup() -> (World, Commit) {
    let mut world = World::turn();
    world.adopt();
    assert!(world.project_landing(5, change::Way::Turn, Box::new([])));
    world.landing_requirements(b"main", true, true);
    world.produce_on(SOURCE, b"main");
    let head = translate::commit(world.branch(SOURCE));
    register(&mut world, None);
    (world, head)
}
fn step(world: &mut World, entry: u64) -> change::Decision {
    changes::change_step(world, entry, change::Status::Unknown, Box::new([]))
}
fn ask(world: &mut World, entry: u64) {
    world.event(top::Event::StepChange {
        task: TASK,
        entry,
        merge_job: false,
        kept: false,
        pending: Box::new([]),
        gates: Box::new([]),
        queue_repair_active: false,
        heard: change::Heard {
            review_request: None,
            start_conflict: None,
            lacking: None,
            spend: None,
            delegate: change::Status::Unknown,
            delegate_running: false,
            delegate_live: false,
            effect: change::EffectResult::None,
            released: false,
            cancelled: false,
        },
    });
}
fn release(world: &mut World, ending: top::ReleaseEnding) {
    world.event(top::Event::SettleEffects { task: TASK, root: TASK, ending });
    for entry in 900..950 {
        world.run_for(1);
        for request in world.take_seen() {
            match request {
                top::Request::ContinueRelease { task: TASK } => {
                    world.event(top::Event::ContinueRelease { task: TASK, entry });
                }
                top::Request::EffectsSettled { task: TASK } => {
                    world.event(top::Event::Release { task: TASK, root: TASK, ending, reason: None, entry });
                }
                _ => {}
            }
        }
        if world.hold(&name(OWN)).is_none() {
            return;
        }
    }
    panic!("release did not settle: {:?}", world.stored().get(&top::Key::Release(TASK)));
}
fn source_reads(calls: &[Op]) -> usize {
    calls.iter().filter(|op| **op == Op::Read(Read::Branch { branch: SOURCE.into() })).count()
}
fn untouched(world: &mut World) {
    assert!(world.hold(&name(SOURCE)).is_none());
    for op in world.take_calls() {
        if let Op::Write(Write::CreateBranch { branch, .. } | Write::DeleteBranch { branch }) = op {
            assert_ne!(branch.as_ref(), SOURCE);
        }
    }
    for observation in world.observations() {
        if let skein_fake_forge_domain::Observation::Moved { branch, .. }
        | skein_fake_forge_domain::Observation::Deleted { branch, .. }
        | skein_fake_forge_domain::Observation::Rebased { branch, .. } = observation
        {
            assert_ne!(branch.as_ref(), SOURCE);
        }
    }
}

#[test]
fn a_taken_head_opens_is_ready_joins_the_next_landing_and_releases_only_its_own_branch() {
    let (mut world, head) = setup();
    assert_eq!(step(&mut world, 1), change::Decision::Effect(change::Effect::CreateBranch { head }));
    assert_eq!(world.change(TASK).kept, Some(head));
    assert_eq!(translate::commit(world.branch(OWN)), head);
    assert_eq!(step(&mut world, 2), change::Decision::Effect(change::Effect::Open));
    assert_eq!(step(&mut world, 3), change::Decision::Ready);
    assert!(matches!(world.change(TASK).change.state, change::State::Ready { head: got, .. } if got == head));
    assert!(world.change(TASK).delegate.is_none());
    let observations = world.observations();
    assert!(observations.iter().any(|event| matches!(event, skein_fake_forge_domain::Observation::Moved { branch, from: None, to, by: 1, .. } if branch.as_ref() == OWN && translate::commit(*to) == head)));
    assert!(!observations.iter().any(|event| matches!(event, skein_fake_forge_domain::Observation::Moved { branch, .. } | skein_fake_forge_domain::Observation::Deleted { branch, .. } if branch.as_ref() == SOURCE)));
    let calls = world.take_calls();
    assert_eq!(source_reads(&calls), 1);
    assert_eq!(calls.iter().filter(|op| matches!(op, Op::Write(Write::CreateBranch { branch, commit }) if branch.as_ref() == OWN && *commit == head)).count(), 1);
    assert!(calls.contains(&Op::Write(Write::OpenPull {
        title: b"Small fix".as_slice().into(),
        body: b"Fix the file".as_slice().into(),
        head: OWN.into(),
        base: b"main".as_slice().into()
    })));
    assert!(!calls.iter().any(|op| matches!(op, Op::Write(Write::CreateBranch { branch, .. } | Write::DeleteBranch { branch }) if branch.as_ref() == SOURCE)));
    let mut trains = landings::Trains::default();
    trains.judge(&mut world);
    assert_eq!(world.change(TASK).change.landing, Some(landings::LANDING));
    // This rule has no workflow gate for the landing either.
    trains.land(&mut world, landings::LANDING);
    assert!(matches!(world.change(TASK).change.state, change::State::Landed { .. }));
    release(&mut world, top::ReleaseEnding::Done);
    assert!(!world.has_branch(OWN));
    assert_eq!(translate::commit(world.branch(SOURCE)), head);
    untouched(&mut world);
}

#[test]
fn taking_refuses_a_mismatch_or_a_missing_source_before_any_make_and_releases_its_hold() {
    for missing in [false, true] {
        let mut world = World::new(87);
        world.adopt();
        let commit = [0xcc; 32];
        let expected = if missing {
            b"docs/x is not in org/repo".to_vec()
        } else {
            world.produce_on(SOURCE, b"main");
            let head = translate::commit(world.branch(SOURCE));
            [
                b"docs/x is at ".as_slice(),
                &top::commit_text(&head),
                b", not ",
                &top::commit_text(&commit),
                b": push it and open again",
            ]
            .concat()
        };
        register(&mut world, Some(commit));
        assert_eq!(step(&mut world, 1), change::Decision::Failed);
        assert_eq!(world.change(TASK).ending, Some(top::Ending::Failed { line: expected.into() }));
        assert_eq!(world.change(TASK).kept, None);
        assert!(!world.take_calls().iter().any(|op| matches!(op, Op::Write(Write::CreateBranch { .. }))));
        release(&mut world, top::ReleaseEnding::Failed);
        assert!(!world.has_branch(OWN));
        untouched(&mut world);
    }
}

#[test]
fn a_kept_head_survives_restart_before_the_make_even_when_the_source_moves() {
    let (mut world, head) = setup();
    ask(&mut world, 1);
    world.run_for(1);
    assert_eq!(world.change(TASK).kept, Some(head));
    assert!(!world.has_branch(OWN));
    assert_eq!(source_reads(&world.take_calls()), 1);
    world.push_as(SOURCE, b"new", b"newer", 2);
    let newer = world.branch(SOURCE);
    world.observations();
    world.restart();
    world.event(top::Event::Committed { entry: 1 });
    world.run_for(1);
    assert_eq!(translate::commit(world.branch(OWN)), head);
    assert_eq!(world.change(TASK).kept, Some(head));
    assert_eq!(step(&mut world, 2), change::Decision::Effect(change::Effect::Open));
    assert_eq!(source_reads(&world.take_calls()), 0);
    assert_eq!(world.branch(SOURCE), newer);
    untouched(&mut world);
}

#[test]
fn a_made_branch_with_its_answer_lost_is_found_after_restart_without_another_make() {
    let (mut world, head) = setup();
    ask(&mut world, 1);
    world.run_for(1);
    world.slow_calls(Duration::from_secs(5));
    world.event(top::Event::Committed { entry: 1 });
    for _ in 0..30 {
        world.run_for(1);
        if world.writes() == 1 {
            break;
        }
    }
    assert_eq!(world.writes(), 1);
    assert_eq!(world.change(TASK).effect, change::EffectResult::Pending);
    world.restart();
    world.run_for(20);
    assert_eq!(world.change(TASK).effect, change::EffectResult::Made);
    assert!(world.seen().iter().any(|event| matches!(event, top::Request::Outcome { entry: 1, outcome: skein_forge_client::Outcome::Made { made: skein_forge_client::Made::Branch(found), found: true }, .. } if *found == head)));
    assert_eq!(translate::commit(world.branch(OWN)), head);
    assert_eq!(world.writes(), 1);
    world.slow_calls(Duration::ZERO);
    assert_eq!(step(&mut world, 2), change::Decision::Effect(change::Effect::Open));
    assert_eq!(source_reads(&world.take_calls()), 1);
    untouched(&mut world);
}

#[test]
fn an_erroring_source_stalls_with_its_account_and_release_reads_again() {
    let (mut world, head) = setup();
    world.script_branch_error(SOURCE, Some(Error::Unavailable));
    ask(&mut world, 1);
    world.run_for(8);
    assert!(world.seen().iter().any(|event| matches!(
        event,
        top::Request::ChangeDecision { task: TASK, decision: change::Decision::Wait { until: Some(_) }, .. }
    )));
    let reads = source_reads(&world.take_calls());
    assert!(reads >= 3, "the client retried its read");
    world.run_for(23);
    assert!(matches!(world.change(TASK).change.state, change::State::Held { why: change::Hold::Stalled, .. }));
    assert_eq!(
        world.change(TASK).effect_account.as_ref().expect("taking read account").words.as_ref(),
        b"taking docs/x: forge unavailable"
    );
    assert_eq!(world.change(TASK).kept, None);
    world.script_branch_error(SOURCE, None);
    world.event(top::Event::ReleaseChange { task: TASK, release: None });
    assert_eq!(step(&mut world, 2), change::Decision::Effect(change::Effect::CreateBranch { head }));
    assert_eq!(world.change(TASK).kept, Some(head));
    assert_eq!(source_reads(&world.take_calls()), 1);
    untouched(&mut world);
}

#[test]
fn cancel_while_taking_settles_the_make_before_deleting_and_an_unmade_branch_only_loses_its_hold() {
    for phase in [0, 1, 2] {
        let (mut world, head) = setup();
        if phase > 0 {
            ask(&mut world, 1);
            world.run_for(1);
            assert_eq!(world.change(TASK).kept, Some(head));
        }
        if phase == 2 {
            world.slow_calls(Duration::from_secs(5));
            world.event(top::Event::Committed { entry: 1 });
            for _ in 0..30 {
                world.run_for(1);
                if world.writes() == 1 {
                    break;
                }
            }
            assert_eq!(world.change(TASK).effect, change::EffectResult::Pending);
        }
        release(&mut world, top::ReleaseEnding::Cancelled);
        assert!(!world.has_branch(OWN));
        let calls = world.take_calls();
        assert_eq!(
            calls
                .iter()
                .filter(|op| matches!(op, Op::Write(Write::DeleteBranch { branch }) if branch.as_ref() == OWN))
                .count(),
            usize::from(phase == 2)
        );
        assert_eq!(translate::commit(world.branch(SOURCE)), head);
        untouched(&mut world);
    }
}

#[test]
fn a_taken_head_already_in_the_landing_branch_ends_without_a_pull() {
    let mut world = World::turn();
    world.adopt();
    assert!(world.project_landing(5, change::Way::Turn, Box::new([])));
    let raw_head = world.branch(b"main");
    let head = translate::commit(raw_head);
    world.external(
        skein_fake_forge_domain::api::Op::Git(skein_fake_forge_domain::api::Git::Create {
            branch: SOURCE.into(),
            commit: raw_head,
        }),
        2,
    );
    register(&mut world, None);
    assert_eq!(step(&mut world, 1), change::Decision::Effect(change::Effect::CreateBranch { head }));
    assert_eq!(step(&mut world, 2), change::Decision::NoChange);
    assert_eq!(
        world.change(TASK).change.state,
        change::State::NoChange {
            words: [b"The landing branch already contains its head ".as_slice(), &top::commit_text(&head), b"."]
                .concat()
                .into()
        }
    );
    assert!(world.change(TASK).pull.is_none());
    assert!(!world.take_calls().iter().any(|op| matches!(op, Op::Write(Write::OpenPull { .. }))));
    release(&mut world, top::ReleaseEnding::Done);
    untouched(&mut world);
}
