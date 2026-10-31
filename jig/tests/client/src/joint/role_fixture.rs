//! Role fixtures at the engine boundary (jig/docs/design/client/domain/testing.md, 5).
use super::{Joint, engine::Driver, fixture::clock, session::Session};
use jig_client_domain::contract as client;
use jig_core_world::peers::Peers;
use jig_domain_people as people;
use jig_fake_parties::{Act, Party, Role as PartyRole};
use jig_fake_workers::{Script, Worker};
use skein_lib::{Duration, Time};

pub(super) fn front(
    seed: u64,
    scripts: Vec<Box<[Script]>>,
    slots: u32,
    configure: fn(&mut jig_test_domain::Config, &jig_test_domain::Limits),
    receiving: &jig_client_test_domain::Limits,
) -> Joint {
    let (mut configuration, mut owner) = jig_core_world::peers::fixture(seed, 0);
    configuration.core.owners = Box::new([]);
    owner.core.fleet.slots = slots;
    // Four adoption identities and their four later public client sign-ins.
    owner.core.people.sign_ins = 8;
    // Four adoption peer watches plus eight public watches per client.
    owner.core.views.watchers = 36;
    owner.core.documents.rows = 2;
    owner.core.documents.lines = 2;
    owner.core.documents.chips = 2;
    owner.core.documents.text_bytes = receiving.jig.text;
    // This scenario offers its agent and human executor, not a procedure.
    let kinds: Box<[jig_domain_authority::Executor]> =
        Box::new([jig_domain_authority::Executor::Charter(1), jig_domain_authority::Executor::Role(1)]);

    let mut authority_rules = configuration.core.defaults.rules.clone();
    authority_rules.ceiling.delegation.kinds.clone_from(&kinds);
    let mut policy = configuration.core.seed_projects[0].policy.policy.clone();
    policy.ceiling.delegation.kinds.clone_from(&kinds);
    for role in &mut policy.roles {
        role.authority.delegation.kinds.clone_from(&kinds);
    }
    policy.roles[0].authority.budget.spend = 500;
    configuration.core.defaults.rules = (jig_domain_authority::Domain::new(authority_rules, owner.core.authority)
        .expect("role scenario bounds"))
    .rules()
    .clone();
    configuration.core.seed_projects[0].policy.policy = policy;
    configure(&mut configuration, &owner);
    jig_core_world::peers::journal(&mut owner);
    let roles = [PartyRole::Owner, PartyRole::Maintainer, PartyRole::Member, PartyRole::Observer];
    let parties = roles
        .iter()
        .enumerate()
        .map(|(i, role)| {
            Party::new(
                1,
                *role,
                10_000 * (i as u64 + 1),
                Box::new([Act::SignIn { provider: 0, subject: (7 + i as u64).to_be_bytes().into() }]),
            )
        })
        .collect();
    let mut driver = Driver::new(
        configuration,
        owner,
        Peers::new(vec![Worker::new(7, slots, scripts)], parties),
        Time::ZERO,
        clock(Time::ZERO),
    );
    let holdings = driver
        .world
        .peers
        .as_ref()
        .expect("setup peers")
        .parties
        .iter()
        .enumerate()
        .map(|(i, party)| people::Holding {
            person: party.person.expect("setup sign-in"),
            role: match i {
                0 => people::Role::Owner,
                1 => people::Role::Maintainer,
                2 => people::Role::Member,
                3 => people::Role::Observer,
                _ => unreachable!("four setup roles"),
            },
        })
        .collect();
    driver.send(
        Time::ZERO,
        clock(Time::ZERO),
        jig_test_domain::Event::Core(jig_domain_core::Event::People(people::Event::Roles { project: 1, holdings })),
    );
    let codec = jig_protocol_transcript::Limits {
        smith: smith_transcript::v6::CEILINGS,
        bytes: 1_048_576,
        turn: client::TurnLimits { messages: 8, blocks: 8, text: 128 },
    };
    let identities = (1..=4)
        .map(|id| {
            (
                id,
                people::Identity {
                    key: people::IdentityKey { provider: 0, subject: (id + 6_u64).to_be_bytes().into() },
                    login: b"login".as_slice().into(),
                    name: b"Person".as_slice().into(),
                },
            )
        })
        .collect();
    let mut front = Joint::new(Session::new(driver, *receiving, codec), identities, clock);
    front.unsigned_on_bind();
    front.session.capture_task_offers();
    front.session.stream_heartbeat(Duration::from_millis(50));
    front
}

pub(super) fn seeded(story: crate::harness::seed::RoleStory) -> Joint {
    use crate::harness::{
        Seed,
        seed::{RoleStory, seed_core},
    };
    use jig_fake_workers::Call;
    let scripts: Vec<Box<[Script]>> = match story {
        RoleStory::Proposal => vec![
            Box::new([
                Script::Call(Call::ProposeGoal {
                    words: b"decide this goal".as_slice().into(),
                    budget: 150,
                    priority: 7,
                }),
                Script::Park,
            ]),
            Box::new([Script::Finish { report: b"one accepted goal".as_slice().into() }]),
        ],
        RoleStory::Choice => vec![Box::new([
            Script::Call(Call::Choose {
                role: 1,
                first: b"first report".as_slice().into(),
                second: b"second report".as_slice().into(),
            }),
            Script::Wait,
            Script::Finish { report: b"choice received".as_slice().into() },
        ])],
        RoleStory::Goal | RoleStory::Observer => vec![],
    };
    let mut value = front(8510, scripts, 1, |_, _| {}, &crate::stories::shared::a_roles::limits());
    if matches!(story, RoleStory::Proposal | RoleStory::Choice) {
        seed_core(
            &mut value.session.driver,
            Time::ZERO,
            clock(Time::ZERO),
            Seed::Conversation { party: 3, project: 1, words: b"Hello".as_slice().into() },
        );
    }
    value
}
