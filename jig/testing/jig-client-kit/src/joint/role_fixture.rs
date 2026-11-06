//! Role fixtures at the engine boundary (jig/docs/design/client/domain/testing.md, 5).
use super::{Joint, engine::Driver, fixture::clock, session::Session};
use crate::harness::seed::RoleStory as Story;
use jig_client_domain::contract as client;
use jig_core_kit::peers::Peers;
use jig_domain_people as people;
use jig_fake_parties::{Act, Party, Role as PartyRole};
use jig_fake_workers::{Script, Worker};
use jig_test_domain::{Config, Limits};
use skein_lib::{Duration, Time, Wall};

#[cfg(test)]
pub(super) fn front(
    seed: u64,
    scripts: Vec<Box<[Script]>>,
    slots: u32,
    configure: fn(&mut jig_test_domain::Config, &jig_test_domain::Limits),
    receiving: &jig_client_test_domain::Limits,
) -> Joint {
    let (configuration, owner, peers) = configured(seed, scripts, slots, configure, receiving);
    let mut driver = Driver::new(configuration, owner, peers, Time::ZERO, clock(Time::ZERO));
    roles(&mut driver, Time::ZERO, clock(Time::ZERO));
    with_driver(driver, receiving)
}

fn configured(
    seed: u64,
    scripts: Vec<Box<[Script]>>,
    slots: u32,
    configure: fn(&mut Config, &Limits),
    receiving: &jig_client_test_domain::Limits,
) -> (Config, Limits, Peers) {
    let (mut configuration, mut owner) = jig_core_kit::peers::fixture(seed, 0);
    configuration.core.owners = Box::new([]);
    owner.core.fleet.slots = slots;
    // Four adoption identities and their four later public client sign-ins.
    owner.core.people.sign_ins = 8;
    // Four adoption peer watches plus eight public watches per client.
    owner.core.views.watchers = 36;
    owner.core.documents.rows = 2;
    owner.core.documents.lines = 2;
    owner.core.documents.chips = 2;
    owner.core.documents.text_bytes = receiving.client.domain.text;
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
    jig_core_kit::peers::journal(&mut owner);
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
    (configuration, owner, Peers::new(vec![Worker::new(7, slots, scripts)], parties))
}

fn roles(driver: &mut Driver, now: Time, wall: Wall) {
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
        now,
        wall,
        jig_test_domain::Event::Core(jig_domain_core::Event::People(people::Event::Roles { project: 1, holdings })),
    );
}

fn with_driver(driver: Driver, receiving: &jig_client_test_domain::Limits) -> Joint {
    let codec = jig_protocol_transcript::Limits {
        smith: smith_transcript::v6::CEILINGS,
        bytes: 1_048_576,
        turn: client::TurnLimits { messages: 8, blocks: 8, text: 128, outcome: 128 },
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

/// Configuration, bounds and peers shared by both transports' role stories.
#[must_use]
pub fn setup(story: Story) -> (Config, Limits, Peers) {
    use jig_fake_workers::Call;
    let scripts: Vec<Box<[Script]>> = match story {
        Story::Proposal => vec![
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
        Story::DelegateHold => vec![Box::new([
            Script::Call(Call::Delegate { words: b"Held delegate".as_slice().into(), budget: 1 }),
            Script::Park,
        ])],
        Story::Choice => vec![Box::new([
            Script::Call(Call::Choose {
                role: 1,
                first: b"first report".as_slice().into(),
                second: b"second report".as_slice().into(),
            }),
            Script::Wait,
            Script::Finish { report: b"choice received".as_slice().into() },
        ])],
        Story::Goal | Story::Observer => vec![],
    };
    let configure: fn(&mut jig_test_domain::Config, &jig_test_domain::Limits) = if matches!(story, Story::DelegateHold)
    {
        |configuration, limits| {
            let mut rules = configuration.core.defaults.rules.clone();
            rules.maximum_run_spend = 5;
            configuration.core.defaults.rules = jig_domain_authority::Domain::new(rules, limits.core.authority)
                .expect("delegate run allowance")
                .rules()
                .clone();
        }
    } else {
        |_, _| {}
    };
    configured(8510, scripts, 1, configure, &crate::stories::shared::a_roles::limits())
}

/// Install project holdings and the story's first work through the core.
pub fn install(driver: &mut Driver, story: Story, now: Time, wall: Wall) {
    use crate::harness::{Seed, seed::seed_core};
    roles(driver, now, wall);
    if matches!(story, Story::DelegateHold) {
        let world = &mut driver.world;
        let creating = world.peers.as_ref().expect("role peers").creating.as_ref().expect("role authority");
        let people::Ask::Create { project, mut batch } =
            jig_test_profile::fixtures::create_chat(creating, 1, 2, b"Hello".as_slice().into())
        else {
            unreachable!("conversation recipe")
        };
        batch[0].flags.ended_by = people::EndedBy::Requester;
        batch[0].authority.spend = 10;
        batch[0].budget = people::Budget::Named;
        let sign_in = world.peers.as_ref().expect("role peers").parties[2].sign_in.expect("member sign-in");
        jig_core_kit::fixtures::ask_as(world, sign_in, 80, people::Ask::Create { project, batch });
    }
    if matches!(story, Story::Proposal | Story::Choice) {
        seed_core(driver, now, wall, Seed::Conversation { party: 3, project: 1, words: b"Hello".as_slice().into() });
    }
}

pub(super) fn seeded(story: Story) -> Joint {
    let (configuration, owner, peers) = setup(story);
    let mut driver = Driver::new(configuration, owner, peers, Time::ZERO, clock(Time::ZERO));
    install(&mut driver, story, Time::ZERO, clock(Time::ZERO));
    with_driver(driver, &crate::stories::shared::a_roles::limits())
}
