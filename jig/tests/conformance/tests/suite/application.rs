use jig_core_world::world::Setup;
use jig_core_world::world::{Cut, World};
use jig_domain_core as core;
use jig_domain_people as people;
use jig_test_connector as connector;
use jig_test_domain as root;
use sha2::{Digest, Sha256};
use skein_lib::{ReplyTo, Token};

fn ask(sign_in: u64, key: u8, value: connector::ApplicationRequest) -> root::Event {
    let mut bytes = Vec::new();
    bytes.extend(value.object.to_be_bytes());
    bytes.extend(value.revision.to_be_bytes());
    bytes.extend(value.value.to_be_bytes());
    match value.release {
        Some(release) => {
            bytes.push(1);
            bytes.extend(release.task.to_be_bytes());
            bytes.extend(release.revision.to_be_bytes());
            bytes.push(match release.again {
                None => 0,
                Some(false) => 1,
                Some(true) => 2,
            });
        }
        None => bytes.push(0),
    }
    root::Event::Application {
        reply_to: ReplyTo::new(Token::new(600 + u64::from(key))),
        sign_in,
        key: [key; 16],
        scope: jig_domain_people::ApplicationScope::Project(1),
        number: 2,
        digest: Sha256::digest(bytes).into(),
        value,
        carried: None,
    }
}

#[test]
fn admitted_application_routes_and_its_core_part_share_the_commit_barrier() {
    let mut world = World::new(Setup { seed: 711, ..Setup::default() });
    let sign_in = world.sign_in.expect("signed in owner");
    let value = connector::ApplicationRequest { object: 1, revision: 0, value: 18, release: None };
    let commit = world.store.applied + 1;
    world.cut = Some(Cut::Submitted(commit));
    world.send(ask(sign_in, 1, value));
    assert_eq!(world.reached_cut, Some(Cut::Submitted(commit)));
    assert!(world.application_answers.is_empty());
    world.cut = None;
    world.release_commits();
    let changed = connector::ApplicationOutcome::Changed {
        object: 1,
        now: connector::ApplicationObject { revision: 1, value: 18 },
    };
    assert_eq!(world.application_answers[0].1, 2);
    assert_eq!(world.application_answers[0].2, changed);
    let writes = world.store.applied;
    world.send(ask(sign_in, 1, value));
    assert_eq!(world.application_answers.len(), 2);
    assert_eq!(world.application_answers[1].2, changed);
    assert_eq!(world.store.applied, writes, "replay decides nothing");

    let refused = connector::ApplicationRequest {
        object: 1,
        revision: 1,
        value: 25,
        release: Some(connector::ApplicationRelease { task: u64::MAX, revision: 1, again: None }),
    };
    world.send(ask(sign_in, 2, refused));
    assert_eq!(
        world.application_answers[2].2,
        connector::ApplicationOutcome::CoreRefused { why: connector::application_core::Refusal::Unknown }
    );
    assert!(world.store.rows.values().any(|row| matches!(row, root::Record::Connector { number: 2, record: connector::Record::ApplicationObject { object: 1, now } } if *now == connector::ApplicationObject { revision: 1, value: 18 })));
    assert!(world.store.rows.values().any(|row| matches!(
        row,
        root::Record::Core(core::Record::People(people::Stored::Answer {
            outcome: people::Outcome::Applied { connector: 2, .. },
            ..
        }))
    )));
}

#[test]
fn application_replay_after_a_durable_restart_does_not_repeat_a_decision() {
    let mut configured = jig_test_profile::config(715);
    let mut limits = jig_test_profile::limits();
    limits.core.authority.roles = 2;
    let policy = &mut configured.core.seed_projects[0].policy.policy;
    let mut member = policy.roles[0].clone();
    member.number = people::Role::Member.number();
    member.period_spend = 0;
    member.requests = jig_domain_authority::Requests(0);
    member.decides = jig_domain_authority::Proposals(0);
    policy.roles = Box::new([policy.roles[0].clone(), member]);
    let mut world = World::new(Setup { seed: 715, configuration: Some(configured), limits, ..Setup::default() });
    let sign_in = world.sign_in.expect("signed in owner");
    let value = connector::ApplicationRequest { object: 1, revision: 0, value: 14, release: None };
    let commit = world.store.applied + 1;
    world.cut = Some(Cut::Durable(commit));
    world.send(ask(sign_in, 11, value));
    assert_eq!(world.reached_cut, Some(Cut::Durable(commit)));
    assert!(world.application_answers.is_empty(), "durability alone has not delivered the completion");
    world.restart(715, false);
    let writes = world.store.applied;
    world.send(ask(sign_in, 11, value));
    let now = connector::ApplicationObject { revision: 1, value: 14 };
    assert_eq!(
        world.application_answers.last().expect("retained terminal").2,
        connector::ApplicationOutcome::Changed { object: 1, now }
    );
    assert_eq!(world.store.applied, writes, "restored replay decides nothing");

    let task = world.assigned.as_ref().expect("first person's task").task;
    let other_sign_in = jig_core_world::fixtures::second_party(&mut world, b"other", people::Role::Member);
    assert_ne!(other_sign_in, sign_in);
    let answers = world.application_answers.len();
    world.send(ask(
        other_sign_in,
        12,
        connector::ApplicationRequest {
            object: 1,
            revision: 1,
            value: 90,
            release: Some(connector::ApplicationRelease { task, revision: 1, again: None }),
        },
    ));
    assert_eq!(world.application_answers.len(), answers + 1, "the second party reaches the connector");
    assert_eq!(
        world.application_answers.last().expect("whole refusal").2,
        connector::ApplicationOutcome::CoreRefused { why: connector::application_core::Refusal::Unknown }
    );
    assert!(world.store.rows.values().any(|row| matches!(row, root::Record::Connector { number: 2, record: connector::Record::ApplicationObject { object: 1, now: saved } } if *saved == now)), "an unauthorized core part changes no own object");
}
