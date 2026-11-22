//! Service reissue and its durable decision (domain/people/acts.md, 1).
use jig_domain_people::{
    Ask, DEPLOYMENT, Entry, EntryKind, Holding, InitialOwner, Key, Outcome, Reply, Role, Stored, Whom,
};
use jig_people_world::{
    Settings, World,
    fixtures::{OWNER, PROJECT, SIGN_IN, create},
    identity,
};
use skein_lib::Wall;

const SERVICE_NAME: &[u8] = b"build";
const FIRST_SIGN_IN: u64 = 20;
const SECOND_SIGN_IN: u64 = 21;
const FRESH_SIGN_IN: u64 = 22;
const MAKE: u8 = 1;
const CREATE: u8 = 2;
const REISSUE: u8 = 3;
const PROBE: u8 = 4;
const QUESTION: u64 = 1;
const BEFORE_CREATE: usize = 1; // the service's first task
const AFTER_CREATE: usize = 2; // the service's request on its fresh sign-in adds one task

fn with_service(deployment_owner: bool) -> (World, u64) {
    let owners = Box::new([InitialOwner { project: PROJECT, identity: identity(OWNER).key }]);
    let mut world = World::with_owners(Settings::calm(0), owners);
    world.roles(PROJECT, Box::new([]));
    let sign_in = world.sign_in_owner();
    world.commit_all();
    let Some(Reply::SignedIn { person: owner, .. }) = world.reply(sign_in) else { panic!("owner signed in") };
    if deployment_owner {
        world.roles(DEPLOYMENT, Box::new([Holding { person: owner, role: Role::Owner }]));
    }
    let made = world.request(
        SIGN_IN,
        [MAKE; 16],
        Ask::MakeService { project: PROJECT, name: SERVICE_NAME.into(), role: Role::Member },
    );
    world.commit_all();
    let Some(Reply::Outcome(Outcome::ServiceMade { person: service })) = world.reply(made) else {
        panic!("service made")
    };
    if deployment_owner {
        world.roles(PROJECT, Box::new([Holding { person: service, role: Role::Member }]));
    }
    world.signin_service(FIRST_SIGN_IN, service);
    world.commit_all();
    (world, service)
}

fn reissue(world: &mut World, party: u64) -> u64 {
    world.request(SIGN_IN, [REISSUE; 16], Ask::ReissueService { party })
}

fn request_work(world: &mut World, sign_in: u64, key: u8) -> u64 {
    world.request(sign_in, [key; 16], create(b"goal", Some(1)))
}

#[test]
fn either_owner_scope_reissues_without_changing_service_history() {
    for deployment_owner in [true, false] {
        let (mut world, party) = with_service(deployment_owner);
        world.signin_service(SECOND_SIGN_IN, party);
        let created = request_work(&mut world, FIRST_SIGN_IN, CREATE);
        world.commit_all();
        let Some(Reply::Outcome(Outcome::Created { first: task, .. })) = world.reply(created) else {
            panic!("task made")
        };
        let entry = Entry {
            task,
            project: PROJECT,
            whom: Whom::Person(party),
            kind: EntryKind::Question { message: QUESTION },
            at: Wall::EPOCH,
        };
        world.waiting(task, Box::new([entry]));
        let mut remaining = world.records().clone();
        let ended = [FIRST_SIGN_IN, SECOND_SIGN_IN];
        for sign_in in ended {
            remaining.remove(&Key::SignIn(sign_in));
        }

        let call = reissue(&mut world, party);
        assert_eq!(world.reply(call), None, "the reissue reply waits for the erases to commit");
        world.commit_all();
        assert_eq!(world.reply(call), Some(Reply::Outcome(Outcome::ServiceReissued { party })));
        assert_eq!(world.records(), &remaining, "only the two sign-in rows were erased");
        assert_eq!(world.tasks(), BEFORE_CREATE);
        assert_eq!(&*world.inbox(party), &[entry]);

        let calls = ended.map(|sign_in| request_work(&mut world, sign_in, PROBE));
        world.commit_all();
        assert_eq!(calls.map(|call| world.reply(call)), [Some(Reply::SignedOut), Some(Reply::SignedOut)]);

        let fresh = world.signin_service(FRESH_SIGN_IN, party);
        world.commit_all();
        let admitted = request_work(&mut world, FRESH_SIGN_IN, PROBE);
        world.commit_all();
        assert!(
            matches!(world.reply(fresh), Some(Reply::SignedIn { person, sign_in: FRESH_SIGN_IN, .. }) if person == party)
        );
        assert!(matches!(world.reply(admitted), Some(Reply::Outcome(Outcome::Created { .. }))));
        assert_eq!(world.tasks(), AFTER_CREATE);
        world.assert_settled();
    }
}

#[test]
fn the_same_key_reissues_again_and_accepts_other_work() {
    let (mut world, party) = with_service(false);
    let first = reissue(&mut world, party);
    world.commit_all();
    let fresh = world.signin_service(FRESH_SIGN_IN, party);
    world.commit_all();
    let second = reissue(&mut world, party);
    world.commit_all();
    let ended = request_work(&mut world, FRESH_SIGN_IN, PROBE);
    let other = request_work(&mut world, SIGN_IN, REISSUE);
    world.commit_all();
    let reissued = Some(Reply::Outcome(Outcome::ServiceReissued { party }));
    assert_eq!(world.reply(first), reissued);
    assert_eq!(world.reply(second), reissued);
    assert!(matches!(world.reply(fresh), Some(Reply::SignedIn { person, .. }) if person == party));
    assert_eq!(world.reply(ended), Some(Reply::SignedOut));
    assert!(!world.records().contains_key(&Key::SignIn(FRESH_SIGN_IN)));
    assert!(matches!(world.reply(other), Some(Reply::Outcome(Outcome::Created { .. }))));
    world.assert_settled();
}

#[test]
fn restart_keeps_the_service_but_not_its_ended_sign_ins() {
    let (mut world, party) = with_service(false);
    world.signin_service(SECOND_SIGN_IN, party);
    world.commit_all();
    let person = world.records().get(&Key::Person(party)).cloned();
    let roles = world.records().get(&Key::Roles(PROJECT)).cloned();
    reissue(&mut world, party);
    world.commit_all();
    world.restart();

    let calls = [FIRST_SIGN_IN, SECOND_SIGN_IN].map(|sign_in| request_work(&mut world, sign_in, PROBE));
    world.commit_all();
    assert_eq!(calls.map(|call| world.reply(call)), [Some(Reply::SignedOut), Some(Reply::SignedOut)]);
    assert_eq!(world.records().get(&Key::Person(party)), person.as_ref());
    assert_eq!(world.records().get(&Key::Roles(PROJECT)), roles.as_ref());
    let Some(Stored::Person { identity, .. }) = person else { panic!("stored service") };
    assert_eq!(world.role_for_identity(identity.key, PROJECT), Some((party, Role::Member)));
    world.assert_settled();
}
