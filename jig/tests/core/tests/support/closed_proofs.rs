use jig_core_world::world::Setup;
use jig_core_world::world::World;
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use jig_protocol_store::{KeptFamily, KeptMap, Owner, OwnerSet, OwnerSets};
use jig_test_domain as root;
use jig_test_patches_smith as fixture;

const TURN: KeptFamily = KeptFamily {
    retired: &[],
    current: fixture::patches::kept::fixture_turn::TEXTS[0],
    types: &[],
    name: "fixture-turn",
    map: KeptMap::Store,
    keys: &[],
    frames: Some(jig_protocol_store::Frames {
        root: fixture::ROOT,
        texts: &[fixture::patches::kept::fixture_turn::TEXTS[0]],
        patches: &[],
    }),
};
const FIXTURE: OwnerSet =
    OwnerSet { jobs: &[], owner: Owner::Smith, name: fixture::NAME, families: &[TURN], level: Some(0) };
pub const SETS: OwnerSets = OwnerSets { sets: &[FIXTURE] };
// The fixture binary renders the said field of each loaded v1 turn.
pub const TAIL: &[u8] = b"closed windowwindow continuation";

fn turn_text(bytes: &[u8]) -> Option<Box<[u8]>> {
    let schema = skein_codec::schema::parse(TURN.current.as_bytes()).ok()?;
    let root = schema.find(fixture::ROOT.as_bytes())?;
    let mut value = skein_codec::value::Value::with_room(3);
    let turn = value.decode(&schema, root, bytes.into()).ok()?;
    let said = value.field(turn, fixture::patches::kept::fixture_turn::v1::turn::SAID).ok()?;
    Some(value.content(said).ok()?.into())
}

pub fn turn(said: &[u8]) -> Box<[u8]> {
    // Independent v1 encoding: version, said's text and replay's bytes.
    let mut bytes = 1_u16.to_be_bytes().to_vec();
    for field in [said, b"fixture replay".as_slice()] {
        bytes.extend(u32::try_from(field.len()).expect("fixture field size").to_be_bytes());
        bytes.extend(field);
    }
    let schema = skein_codec::schema::parse(TURN.current.as_bytes()).expect("kept v1 schema");
    let root = schema.find(fixture::ROOT.as_bytes()).expect("fixture root");
    // One record and its two fields: validate the independent bytes against
    // the kept text, then take the value's canonical encoding.
    let mut value = skein_codec::value::Value::with_room(3);
    value.decode(&schema, root, bytes.into_boxed_slice()).expect("fixture v1 turn");
    value.bytes().into()
}

pub fn parked() -> World {
    let mut world = World::new(Setup { seed: SEED, owner_sets: SETS, ..Setup::default() });
    world.store.transcript_text(turn_text);
    world.turn(1, 1, None, &turn(b"earlier window"));
    world.park_for_message(1);
    let read = world.say(86);
    world.window_turn(1, 1, Some(read), &turn(b"closed window"));
    world.turn(2, 2, Some(read), &turn(b"window continuation"));
    world.park_for_message(2);
    world
}

// Commit the fixture's closed proofs before the restart, leaving every other field alone.
pub fn close(world: &mut World) {
    let rows = world
        .store
        .rows
        .values()
        .filter_map(|row| {
            if let root::Record::Core(core::Record::Core(core::CoreRecord::RunProof(proof))) = row {
                let mut proof = proof.clone();
                proof.crossed = true;
                if let Some(window) = &mut proof.window {
                    window.closed = true;
                }
                Some(root::Record::Core(core::Record::Core(core::CoreRecord::RunProof(proof))))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    world.seed(rows.into_boxed_slice());
}

pub fn restart(world: &mut World) {
    world.restart(SEED, false);
    hello(world);
    world.grant_account(1);
}
pub fn proof(world: &World) -> &core::RunProof {
    let task = world.assigned.as_ref().expect("chat assignment").task;
    let key = root::Key::Core(core::Key::Core(core::CoreKey::RunProof(task)));
    let root::Record::Core(core::Record::Core(core::CoreRecord::RunProof(proof))) = &world.store.rows[&key] else {
        panic!("durable run proof")
    };
    proof
}
pub fn endpoint(world: &mut World, task: u64, endpoint: Option<u32>) {
    let mut proof = world.domain.core().proofs.get(&task).expect("saved run proof").clone();
    proof.endpoint = endpoint;
    world.seed(Box::new([root::Record::Core(core::Record::Core(core::CoreRecord::RunProof(proof)))]));
}

pub fn tries(world: &World) -> tasks::Tries {
    let task = proof(world).task;
    let key = root::Key::Core(core::Key::Tasks(tasks::Key::Live(task)));
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) = &world.store.rows[&key] else {
        panic!("live chat")
    };
    row.tries
}
pub fn activity(world: &World) -> Vec<tasks::Line> {
    world
        .store
        .rows
        .values()
        .filter_map(|row| {
            if let root::Record::Core(core::Record::Tasks(tasks::Stored::Activity(row))) = row {
                Some(row.line.clone())
            } else {
                None
            }
        })
        .collect()
}
pub fn fresh(world: &World) {
    let assignment = world.assigned.as_ref().expect("fresh assignment");
    assert!(!assignment.run.policy.resume);
    assert!(assignment.transcript.is_empty());
    assert_eq!(assignment.conversation, assignment.attempt);
    assert_eq!(proof(world).conversation, assignment.attempt);
    assert_eq!(proof(world).window, None);
    assert!(!proof(world).crossed, "every new claim writes its own uncrossed proof");
    assert!(matches!(activity(world).last(), Some(tasks::Line::RunBegan { resumed: false, .. })));
}

pub use crate::run_helpers::{SEED, hello, hello_hosting, tail};

#[test]
fn a_window_resumes_only_on_its_claimed_endpoint() {
    use crate::closed_proofs as f;
    for (endpoint, resume) in [(2, false), (1, true)] {
        let mut world = f::parked();
        let task = f::proof(&world).task;
        f::endpoint(&mut world, task, Some(endpoint));
        f::restart(&mut world);
        let previous = world.domain.core().proofs.get(&task).expect("restored proof").clone();
        assert!(!previous.window.expect("saved open window").closed);
        let tries = f::tries(&world);
        let refusals = jig_core_world::fixtures::live_row(&world, task).refusals;
        world.say(87);
        let assigned = world.assigned.as_ref().expect("prepared run");
        assert_eq!(assigned.run.policy.resume, resume);
        assert_eq!(!assigned.transcript.is_empty(), resume);
        assert_eq!(f::tail(&world), if resume { None } else { Some(f::TAIL) });
        assert_eq!(f::proof(&world).endpoint, Some(assigned.run.policy.model.endpoint));
        assert_eq!(f::proof(&world).endpoint, Some(1));
        assert_eq!(f::tries(&world), tries);
        assert_eq!(jig_core_world::fixtures::live_row(&world, task).refusals, refusals);
        assert_eq!(world.domain.take_brief_failure(), None);
        assert!(
            matches!(f::activity(&world).last(), Some(tasks::Line::RunBegan { resumed, .. }) if *resumed == resume)
        );
        if !resume {
            f::fresh(&world);
        }
        world.close();
    }
}
