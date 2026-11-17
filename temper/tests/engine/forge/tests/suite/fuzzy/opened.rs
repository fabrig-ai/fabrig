//! Taking, cancellation and source moves before batch turns (forge/change/opened.md, 4–5).
use super::super::landing_batches::{heard, workflow};
use skein_fake_forge_domain::{Observation, api as raw};
use skein_forge::api;
use skein_lib::Rng;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{
    REPO, World, changes,
    landings::{self, LANDING, Trains},
    translate,
};

fn name(branch: &[u8]) -> top::Name {
    top::Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: top::What::Branch(branch.split(|byte| *byte == b'/').map(Into::into).collect()),
    }
}
fn restart(world: &mut World) {
    world.restart();
    world.run_for(2);
}
fn step(world: &mut World, task: u64, entry: u64) -> change::Decision {
    changes::change_step_task(world, task, entry, change::Status::Unknown, Box::new([]), false, false)
}
fn ask(world: &mut World, task: u64, entry: u64) {
    world.event(top::Event::StepChange {
        task,
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
    world.run_for(1);
}
fn release(world: &mut World, task: u64, ending: top::ReleaseEnding) {
    let branch = world.change(task).branch.clone();
    world.event(top::Event::SettleEffects { task, root: task, ending });
    for entry in task * 1000..task * 1000 + 50 {
        world.run_for(1);
        for request in world.take_seen() {
            match request {
                top::Request::ContinueRelease { task: due } if due == task => {
                    world.event(top::Event::ContinueRelease { task, entry });
                }
                top::Request::EffectsSettled { task: due } if due == task => {
                    world.event(top::Event::Release { task, root: task, ending, reason: None, entry });
                }
                _ => {}
            }
        }
        if world.hold(&name(&branch)).is_none() {
            assert!(!world.has_branch(&branch), "own branch cleaned up");
            return;
        }
    }
    panic!("change release did not settle")
}
fn tree(world: &World, commit: u64) -> std::collections::BTreeMap<Box<[u8]>, Box<[u8]>> {
    let raw::Answer::Tree(files) = world.inspect(&raw::Read::Tree { commit }) else { panic!("tree") };
    files.into_iter().map(|file| (file.path, file.content)).collect()
}
fn number(commit: api::Commit) -> u64 {
    skein_fake_forge_domain::commit_count(&commit).expect("fake commit")
}
fn contains(world: &World, tip: u64, commit: api::Commit) -> bool {
    let raw::Answer::Commits { commits } = world.inspect(&raw::Read::Commits { before: tip, after: number(commit) })
    else {
        panic!("ancestry")
    };
    commits.is_empty()
}
#[derive(Debug, PartialEq)]
struct Member {
    task: u64,
    source: Box<[u8]>,
    branch: Box<[u8]>,
    kept: Option<api::Commit>,
    first_push: Option<api::Commit>,
    cancelled: bool,
}
#[derive(Default)]
struct Oracle {
    members: Vec<Member>,
    calls: Vec<api::Op>,
}
impl Oracle {
    fn check(&mut self, world: &mut World) {
        let calls = world.take_calls();
        for member in &self.members {
            assert!(world.hold(&name(&member.source)).is_none(), "source never held");
            assert!(world.has_branch(&member.source), "source retained");
            assert_eq!(world.change(member.task).kept, member.kept, "the selected head stays kept");
            for call in &calls {
                match call {
                    api::Op::Write(
                        api::Write::CreateBranch { branch, .. }
                        | api::Write::DeleteBranch { branch }
                        | api::Write::Dispatch { branch, .. },
                    ) => assert_ne!(branch, &member.source, "no source write"),
                    api::Op::Write(api::Write::Merge { number, .. } | api::Write::Update { number, .. }) => {
                        let raw::Answer::Pull(pull) = world.inspect(&raw::Read::Pull { number: *number }) else {
                            panic!("written pull")
                        };
                        assert_ne!(pull.head, member.source, "no source pull write");
                        assert_ne!(pull.base, member.source, "no source base write");
                    }
                    _ => {}
                }
            }
        }
        self.calls.extend(calls);
        for event in world.observations() {
            if let Observation::Moved { branch, by: 1, .. }
            | Observation::Deleted { branch, by: 1, .. }
            | Observation::Rebased { branch, by: 1, .. } = &event
            {
                assert!(self.members.iter().all(|member| *branch != member.source), "source never mutated");
            }
            if let Observation::Moved { branch, from: None, to, by: 1, .. } = event
                && let Some(member) = self.members.iter_mut().find(|member| member.branch == branch)
            {
                assert_eq!(Some(translate::commit(to)), member.kept, "first push is the kept head");
                assert!(member.first_push.replace(translate::commit(to)).is_none(), "only one first push");
            }
        }
    }
}
#[derive(Debug, PartialEq)]
struct Replay {
    exercised: u16,
    landing: landing::Landing,
    main: u64,
    calls: Vec<api::Op>,
    members: Vec<Member>,
    histories: Vec<Vec<top::history::Event>>,
}

#[expect(clippy::too_many_lines, reason = "one replay generates taking boundaries, then proves and cleans its batch")]
fn run(seed: u64) -> Replay {
    let mut rng = Rng::new(seed);
    let count = 2 + rng.below(2);
    let squash = rng.below(2) == 0;
    let mut exercised = if squash { 1 } else { 2 };
    let mut world = landings::turn_in(World::batch_turn(seed, Some(workflow(false))));
    world.batch(u16::try_from(count).expect("small batch"));
    let mut oracle = Oracle::default();
    for index in 0..count {
        let task = 51 + index;
        let source = if index == 0 { "docs/x".to_owned() } else { format!("docs/x{index}") };
        let path = format!("member{index}");
        world.external(
            raw::Op::Git(raw::Git::Create { branch: source.as_bytes().into(), commit: world.branch(b"main") }),
            2,
        );
        world.push_as(source.as_bytes(), path.as_bytes(), b"first", 2);
        let at = (rng.below(2) == 0).then(|| translate::commit(world.branch(source.as_bytes())));
        exercised |= if at.is_some() { 4 } else { 8 };
        changes::open_change(&mut world, task, source.as_bytes(), at);
        oracle.members.push(Member {
            task,
            source: source.as_bytes().into(),
            branch: world.change(task).branch.clone(),
            kept: None,
            first_push: None,
            cancelled: false,
        });
        oracle.check(&mut world);
        // A cancellation can meet the unread source, the committed read, the made
        // branch or the ready change. The first member always supplies a landing.
        let cancel = if index == 0 { 4 } else { rng.below(5) };
        if cancel < 4 {
            exercised |= 16 << cancel;
        }
        if rng.below(2) == 0 {
            restart(&mut world);
            exercised |= 256;
        }
        if cancel != 0 {
            ask(&mut world, task, task * 100);
            let kept = world.change(task).kept.expect("read kept its head before make");
            oracle.members.last_mut().expect("member").kept = Some(kept);
            assert!(!world.has_branch(&world.change(task).branch), "seed {seed}, task {task}: make waits for commit");
            if rng.below(2) == 0 {
                world.push_as(source.as_bytes(), path.as_bytes(), b"newer after read", 2);
                exercised |= 1024;
            }
            if rng.below(2) == 0 {
                restart(&mut world);
                exercised |= 512;
            }
            oracle.check(&mut world);
            if cancel != 1 {
                world.event(top::Event::Committed { entry: task * 100 });
                world.run_for(1);
                oracle.check(&mut world);
                assert_eq!(oracle.members.last().expect("member").first_push, Some(kept));
                if cancel != 2 {
                    assert_eq!(step(&mut world, task, task * 100 + 1), change::Decision::Effect(change::Effect::Open));
                    assert_eq!(step(&mut world, task, task * 100 + 2), change::Decision::Ready);
                }
            }
        }
        if cancel < 4 {
            release(&mut world, task, top::ReleaseEnding::Cancelled);
            oracle.members.last_mut().expect("member").cancelled = true;
        } else {
            assert!(oracle.members.last().expect("member").first_push.is_some());
        }
        for _ in 0..rng.below(3) {
            world.push_as(source.as_bytes(), path.as_bytes(), b"later source", 2);
            if rng.below(2) == 0 {
                restart(&mut world);
            }
            oracle.check(&mut world);
        }
        oracle.check(&mut world);
    }
    Trains::default().judge(&mut world);
    assert_eq!(world.landing(LANDING).landing.members.len(), oracle.members.iter().filter(|m| !m.cancelled).count());
    let mut finished = false;
    for entry in 10_000..10_120 {
        if rng.below(7) == 0 {
            let before: Vec<_> = oracle.members.iter().map(|m| world.history(m.task)).collect();
            restart(&mut world);
            for (member, prior) in oracle.members.iter().zip(before) {
                assert!(
                    world.history(member.task).starts_with(&prior),
                    "restart preserves history; gates may finish while it recovers"
                );
            }
        }
        if rng.below(3) == 0 {
            let member = &oracle.members[usize::try_from(rng.below(count)).expect("member")];
            let index = member.task - 51;
            world.push_as(&member.source, format!("member{index}").as_bytes(), b"source during landing", 2);
            exercised |= 2048;
        }
        let before = world.branch(b"main");
        let decision = landings::step_landing_heard(&mut world, LANDING, entry, heard(landing::Preparation::Idle, b""));
        if matches!(decision, landing::Decision::Prepare { .. }) {
            world.prepare_landing(LANDING, squash);
        }
        if world.branch(b"main") != before {
            let landing::Decision::Merge { member, candidate } = decision else { panic!("only the turn moves main") };
            assert_eq!(translate::commit(world.branch(b"main")), candidate, "only the proved prefix lands");
            assert!(contains(&world, world.branch(b"main"), translate::commit(before)), "fast forward");
            assert!(
                world.landing(LANDING).landing.members[usize::from(member)]
                    .gates
                    .iter()
                    .all(|gate| matches!(gate.proof, landing::Proof::Passed { .. })),
                "proof before turn"
            );
        }
        oracle.check(&mut world);
        if decision == landing::Decision::Finish {
            finished = true;
            break;
        }
    }
    assert!(finished, "seed {seed}: {:?}", world.landing(LANDING));
    let row = world.landing(LANDING).landing.clone();
    let main = world.branch(b"main");
    let mut below = row.members[0].base.expect("round base");
    let mut merged_tree = tree(&world, number(below));
    for member in &row.members {
        let opened = oracle.members.iter().find(|m| m.task == member.change).expect("opened member");
        let kept = opened.kept.expect("taken");
        assert_eq!(member.head, Some(kept));
        assert_eq!(opened.first_push, Some(kept));
        let prefix = member.landed.expect("every selected green member lands");
        step(&mut world, member.change, 20_000 + member.change);
        assert!(
            matches!(world.change(member.change).change.state, change::State::Landed { merge, .. } if merge == prefix),
            "opened change lands at its own prefix"
        );
        assert_eq!(member.candidate, Some(prefix), "member lands at its proved prefix");
        assert!(member.left.is_none());
        assert!(contains(&world, main, prefix), "landed prefix in main");
        assert!(contains(&world, number(prefix), below), "prefixes form a fast-forward stack");
        below = prefix;
        merged_tree.extend(tree(&world, number(kept)));
        assert_eq!(tree(&world, number(prefix)), merged_tree, "prefix has the merge's tree, including under squash");
        if !squash {
            assert!(contains(&world, number(prefix), kept), "member's own merge prefix contains its kept head");
        }
        assert_eq!(member.combined, Some(if squash { landing::Combine::Squash } else { landing::Combine::Merge }));
    }
    landings::close_landing(&mut world, LANDING);
    for member in &row.members {
        assert!(!world.has_branch(&member.branch), "prefix released");
    }
    let mut histories = Vec::new();
    for index in 0..oracle.members.len() {
        let member = &oracle.members[index];
        if !member.cancelled {
            release(&mut world, member.task, top::ReleaseEnding::Done);
        }
        let events = world.history(member.task);
        assert!(!events.is_empty(), "history is exercised for cancelled and landed members");
        super::history::check(seed, member.task, &events);
        histories.push(events);
        oracle.check(&mut world);
    }
    Replay { exercised, landing: row, main, calls: oracle.calls, members: oracle.members, histories }
}
#[test]
fn seeded_opens_keep_sources_untouched_and_land_only_proved_prefixes() {
    let mut exercised = 0;
    for seed in 1..33 {
        let first = run(seed);
        exercised |= first.exercised;
        assert_eq!(first, run(seed), "seed {seed} replays");
    }
    assert_eq!(
        exercised, 4095,
        "both combines, both at forms, each cancellation boundary, restarts before and after the read, and source pushes before and during landing"
    );
}
