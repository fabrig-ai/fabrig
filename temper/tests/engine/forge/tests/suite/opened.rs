//! Opened changes follow the queue and drift rules (forge/change/opened.md, 5).
use skein_fake_forge_domain::api as raw;
use skein_forge::api::{Op, Read};
use skein_lib::Duration;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::landings::{self, LANDING, PROJECT};
use temper_engine_forge_world::{REPO, World, changes, translate};

const SOURCE: &[u8] = b"docs/x";
const OWN: &[u8] = b"temper/51/c51";

fn step(world: &mut World, entry: u64, gates: Box<[change::GateReport]>) -> change::Decision {
    changes::change_step(world, entry, change::Status::Unknown, gates)
}
fn taken(mut world: World) -> World {
    world.adopt();
    world.produce_on(SOURCE, b"main");
    changes::open_change(&mut world, 51, SOURCE, None);
    let head = translate::commit(world.branch(SOURCE));
    assert_eq!(step(&mut world, 1, Box::new([])), change::Decision::Effect(change::Effect::CreateBranch { head }));
    assert_eq!(step(&mut world, 2, Box::new([])), change::Decision::Effect(change::Effect::Open));
    world
}

#[test]
fn an_opened_queue_change_checks_gates_updates_checks_again_and_merges() {
    let mut world = taken(World::new(419));
    let head = world.change(51).kept.expect("taken head");
    let mut row = world.change(51).clone();
    row.change.gates = Box::new([change::Gate {
        number: 9,
        kind: change::GateKind::Agent,
        blocking: true,
        freshness: change::Freshness::Clean,
        eager: false,
    }]);
    world.event(top::Event::Restore { record: top::Stored::Change(Box::new(row)) });
    world.status(OWN, raw::Check::Pending);
    assert!(matches!(step(&mut world, 3, Box::new([])), change::Decision::Wait { .. }));
    assert_eq!(world.change(51).change.state, change::State::Checking { head });
    world.status(OWN, raw::Check::Passed);
    assert_eq!(
        step(&mut world, 4, Box::new([])),
        change::Decision::Delegate(change::Delegate::Gate { number: 9, head })
    );
    let gates = Box::new([change::GateReport { revision: 0, number: 9, head, status: change::Status::Passed }]);
    assert_eq!(step(&mut world, 5, gates.clone()), change::Decision::Ready);
    world.push_file(b"main", b"other", b"moved base");
    let base = translate::commit(world.branch(b"main"));
    assert_eq!(step(&mut world, 6, gates.clone()), change::Decision::Effect(change::Effect::Update { base, head }));
    let updated = translate::commit(world.branch(OWN));
    assert_ne!(updated, head);
    world.status(OWN, raw::Check::Pending);
    assert!(matches!(step(&mut world, 7, gates.clone()), change::Decision::Wait { .. }));
    assert_eq!(world.change(51).change.state, change::State::Checking { head: updated });
    assert_ne!(translate::commit(world.branch(b"main")), updated);
    world.status(OWN, raw::Check::Passed);
    let mut merged = false;
    for entry in 8..20 {
        if matches!(step(&mut world, entry, gates.clone()), change::Decision::Finish { .. }) {
            merged = true;
            break;
        }
    }
    assert!(merged, "{:?}", world.change(51));
    assert_eq!(world.change(51).kept, Some(head));
    assert!(world.change(51).delegate.is_none(), "taking never delegates a producer");
    assert_eq!(
        world.inspect(&raw::Read::Tree { commit: world.branch(b"main") }),
        world.inspect(&raw::Read::Tree { commit: world.branch(OWN) }),
        "main has the updated head's tree"
    );
    let calls = world.take_calls();
    for commit in [head, updated] {
        assert!(calls.contains(&Op::Read(Read::Checks { commit })), "CI at each head");
    }
    assert!(
        calls
            .iter()
            .any(|call| matches!(call, Op::Write(skein_forge::api::Write::Merge { head, .. }) if *head == updated))
    );
}

#[test]
fn an_opened_changes_landing_repair_pins_the_rounds_newer_base_not_its_source() {
    let world = World::public_turn(
        31,
        raw::Workflow {
            requirements: Box::new([]),
            file: b"gate.yml".as_slice().into(),
            jobs: Box::new([b"test".as_slice().into()]),
            fails: Box::new([]),
            flaky: Box::new([]),
            report: None,
            select: None,
            latency_min: Duration::from_secs(2),
            latency_max: Duration::from_secs(2),
            silent: 0,
            passes: 1000,
            cue: Some(raw::Cue::File { path: b"file".as_slice().into(), green: b"fixed".as_slice().into() }),
        },
    );
    let mut world = taken(world);
    assert!(world.project_landing(
        PROJECT,
        change::Way::Turn,
        Box::new([top::RepositoryGates {
            repository: REPO,
            gates: Box::new([landing::Gate {
                name: b"gate".as_slice().into(),
                workflow: b"gate.yml".as_slice().into(),
                ignored: Box::new([]),
                blocking: true,
                base: false,
                report: None,
            }]),
            tiers: Box::new([]),
        }])
    ));
    let source = world.change(51).kept.expect("taken source");
    world.push_file(b"main", b"base", b"main moved after taking");
    let base = translate::commit(world.branch(b"main"));
    assert_eq!(step(&mut world, 3, Box::new([])), change::Decision::Ready);
    let row = landings::landing_row(world.change(51));
    landings::register(&mut world, row);
    let mut failure = None;
    for entry in 100..180 {
        if let landing::Decision::Leave { why: landing::Leave::Failed { gate, .. }, .. } =
            landings::step_landing(&mut world, LANDING, entry)
        {
            failure = Some(gate);
            break;
        }
    }
    let gate = failure.expect("the candidate's gate fails");
    let candidate = world.landing(LANDING).landing.members[0].candidate.expect("candidate");
    let why = change::Repair::LandingAt { gate, candidate, below: None };
    assert_eq!(step(&mut world, 300, Box::new([])), change::Decision::Delegate(change::Delegate::Repair(why)));
    world.event(top::Event::Delegated { task: 51, child: 71, kind: change::Delegate::Repair(why) });
    let receipt = world
        .stored()
        .iter()
        .find_map(|(_, row)| match row {
            top::Stored::Receipt { receipt, .. } if !receipt.passed && receipt.candidate == candidate => Some(receipt),
            _ => None,
        })
        .expect("failed receipt");
    assert_eq!(receipt.base, Some(base));
    assert_ne!(base, source);
    assert_ne!(Some(base), world.change(51).change.last_head);
    assert_eq!(world.change(51).kept, Some(source));
    let item = world
        .workspace_item(top::items::Seed {
            project: PROJECT,
            modify: false,
            task: 71,
            parent: Some(51),
            repository: REPO,
            run_branch: Box::new([]),
            saved_branch: None,
        })
        .expect("repair workspace");
    assert_eq!(item.landing, None);
    assert_eq!(item.pin, Some(base));
}

#[test]
fn pushing_the_source_again_neither_reads_it_nor_moves_the_taken_change() {
    let mut world = taken(World::turn());
    assert!(world.project_landing(5, change::Way::Turn, Box::new([])));
    assert_eq!(step(&mut world, 3, Box::new([])), change::Decision::Ready);
    let before = world.change(51).clone();
    let own = world.branch(OWN);
    world.take_calls();
    world.push_as(SOURCE, b"new", b"new source", 2);
    world.event(top::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: REPO,
            change: skein_forge::api::Change::Branch(SOURCE.into()),
            key: None,
        },
    });
    world.run_for(3);
    assert_eq!(step(&mut world, 4, Box::new([])), change::Decision::Wait { until: None });
    assert_eq!(world.change(51), &before);
    assert_eq!(world.branch(OWN), own);
    assert!(!world.take_calls().contains(&Op::Read(Read::Branch { branch: SOURCE.into() })));
}

#[test]
fn another_identity_pushing_the_taken_branch_holds_it_as_drift() {
    let mut world = taken(World::turn());
    assert!(world.project_landing(5, change::Way::Turn, Box::new([])));
    assert_eq!(step(&mut world, 3, Box::new([])), change::Decision::Ready);
    let kept = world.change(51).kept;
    world.push_as(OWN, b"other", b"outside", 2);
    let found = translate::commit(world.branch(OWN));
    assert_eq!(step(&mut world, 4, Box::new([])), change::Decision::Hold(change::Hold::BranchMoved));
    assert_eq!(world.change(51).kept, kept);
    assert_ne!(Some(found), kept);
    assert!(matches!(world.change(51).change.state, change::State::Held { why: change::Hold::BranchMoved, .. }));
}
