use jig_domain_people::{
    Act, ActGrant, ActKey, ActKind, Ask, Entry, EntryKind, Holding, InitialOwner, Key, Outcome, PolicyValue, Refusal,
    Reply, RequestKey, Role, Seen, Whom,
};
use jig_people_world::{Settings, World, fixtures::people_member, identity};
use skein_lib::Wall;
use std::collections::BTreeSet;

const PROJECT: u32 = 1;
const OWNER: u64 = 1;
const SIGN_IN: u64 = 10;
const SERVICE_SIGN_IN: u64 = 20;
const CONNECTOR: u16 = 1;
const KEY: [u8; 16] = [7; 16];
const SERVICE_KEY: [u8; 16] = [8; 16];
const ACT_KEY: ActKey = ActKey { connector: CONNECTOR, key: KEY };

fn world(explicit: bool) -> World {
    let settings = Settings::calm(960);
    let owners = Box::new([InitialOwner { project: PROJECT, identity: identity(OWNER).key }]);
    let grants = Box::new([ActGrant { connector: CONNECTOR, may: Box::new([ActKind::Close]) }]);
    if explicit {
        World::configured_start(settings, owners, grants)
    } else {
        World::configured(settings, owners, grants)
    }
}

#[test]
fn a_project_closed_then_gone_survives_a_restart_as_gone() {
    for explicit in [false, true] {
        let mut world = world(explicit);
        world.roles(PROJECT, Box::new([]));
        let signed = world.signin(SIGN_IN, OWNER);
        world.commit_all();
        let Some(Reply::SignedIn { person: owner, .. }) = world.reply(signed) else { panic!("signed in") };
        world.roles(PROJECT, Box::new([Holding { person: owner, role: Role::Owner }]));
        let policy = PolicyValue {
            ceiling: people_member(b"policy", OWNER).authority,
            escalation_role: None,
            period_spend: 0,
            projections: Box::new([]),
            roles: Box::new([]),
            requirements: Box::new([]),
            permissions: Box::new([]),
            templates: Box::new([]),
        };
        world.project_policy(PROJECT, policy);
        world.commit_all();
        let call = world.request(
            SIGN_IN,
            SERVICE_KEY,
            Ask::MakeService { project: PROJECT, name: b"service".as_slice().into(), role: Role::Member },
        );
        world.commit_all();
        let Some(Reply::Outcome(Outcome::ServiceMade { person: service })) = world.reply(call) else {
            panic!("made service")
        };
        world.signin_service(SERVICE_SIGN_IN, service);
        world.commit_all();
        world.request(
            SERVICE_SIGN_IN,
            KEY,
            Ask::AccountSignIn { holder: jig_domain_people::AccountHolder::Own, provider: 1 },
        );
        world.request(SERVICE_SIGN_IN, [9; 16], Ask::MarkSeen { seen: Seen::Results { through: 1 } });
        world.commit_all();
        let create = Ask::Create { project: PROJECT, batch: Box::new([people_member(b"work", OWNER)]) };
        let call = world.request(SIGN_IN, KEY, create.clone());
        world.commit_all();
        let Some(Reply::Outcome(Outcome::Created { first: task, .. })) = world.reply(call) else { panic!("made task") };
        world.waiting(
            task,
            Box::new([Entry {
                task,
                project: PROJECT,
                whom: Whom::Person(owner),
                kind: EntryKind::ChatReply { message: 3 },
                at: Wall::EPOCH,
            }]),
        );
        world.request(SIGN_IN, [10; 16], Ask::MarkSeen { seen: Seen::Conversation { task, through: 1 } });
        world.act(CONNECTOR, KEY, Act::Close { project: PROJECT, task, grace: None, reason: Box::new([]) });
        world.decide_acts(&Outcome::Compound { project: PROJECT, operations: 1, first_message: 1 });
        world.commit_all();
        world.end_conversation(owner, task, 3);
        world.commit_all();
        world.close_project(PROJECT);
        world.commit_all();
        let survivor = Key::Answer(RequestKey { person: owner, key: KEY });
        let conversation = Key::EndedRead { party: owner, task };
        let kept =
            BTreeSet::from([Key::Person(owner), Key::SignIn(SIGN_IN), survivor, conversation, Key::Acted(ACT_KEY)]);
        assert_eq!(world.records().keys().copied().collect::<BTreeSet<_>>(), kept);
        world.restart();
        assert_eq!(world.records().keys().copied().collect::<BTreeSet<_>>(), kept);
        world.erase_task(task, PROJECT);
        world.project_gone(PROJECT);
        world.commit_all();
        let kept = BTreeSet::from([Key::Person(owner), Key::SignIn(SIGN_IN)]);
        assert_eq!(world.records().keys().copied().collect::<BTreeSet<_>>(), kept);
        world.restart();
        assert_eq!(world.records().keys().copied().collect::<BTreeSet<_>>(), kept);
        let retry = world.request(SIGN_IN, KEY, create);
        world.commit_all();
        assert_eq!(world.reply(retry), Some(Reply::Outcome(Outcome::Refused(Refusal::Unknown))));
        world.assert_settled();
    }
}
