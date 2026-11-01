use jig_domain_people::{Ask, Holding, InitialOwner, Key, Outcome, Reply, RequestKey, Role};
use jig_people_world::{Settings, World, identity};

const PROJECT: u32 = 1;
const USER: u64 = 1;
const SIGN_IN: u64 = 10;
const KEY: [u8; 16] = [7; 16];

#[test]
fn a_dropped_project_leaves_no_people_row_but_its_answers() {
    for selected in [false, true] {
        let mut world = World::with_owners(
            Settings::calm(96),
            Box::new([InitialOwner { project: PROJECT, identity: identity(USER).key }]),
        );
        world.roles(PROJECT, Box::new([]));
        let signed = world.signin(SIGN_IN, USER);
        world.commit_all();
        let Reply::SignedIn { person, .. } = world.reply(signed).expect("durable sign-in") else {
            panic!("sign-in answer")
        };
        let ask = Ask::SetRoles { project: PROJECT, holdings: Box::new([Holding { person, role: Role::Owner }]) };
        let call = world.request(SIGN_IN, KEY, ask.clone());
        world.commit_all();
        let answer = Reply::Outcome(Outcome::RolesSet { project: PROJECT });
        assert_eq!(world.reply(call), Some(answer.clone()));
        world.drop_project(PROJECT);
        world.commit_all();
        let retained = world.records().clone();
        assert!(!retained.contains_key(&Key::Roles(PROJECT)));
        assert!(retained.contains_key(&Key::Answer(RequestKey { person, key: KEY })));
        assert!(retained.contains_key(&Key::Person(person)));
        assert!(retained.contains_key(&Key::SignIn(SIGN_IN)));
        if selected {
            world.restart_start();
        } else {
            world.restart();
        }
        let replay = world.request(SIGN_IN, KEY, ask);
        world.commit_all();
        assert_eq!(world.reply(replay), Some(answer));
        assert_eq!(world.records(), &retained);
        assert_eq!(world.stats().routes, 1);
        world.assert_settled();
    }
}
