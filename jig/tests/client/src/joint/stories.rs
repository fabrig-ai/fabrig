//! People act on the public tree over actual core, worker and applied store doors
//! (client/domain/testing.md, sections 5–6; domain/people.md, section 11).
use super::{Joint, engine::Driver, session::Session};
use crate::{Settings, Test, Until, World};
use jig_client_domain::contract as client;
use jig_core_world::peers::Peers;
use jig_domain_people as people;
use jig_fake_workers::{Script, Worker};
use skein_fake_person::{Find, Step};
use skein_lib::{Duration, Time, Token};
use skein_ui::Role;
use std::collections::BTreeMap;

use super::fixture::{bounded_world, clock, configured_world, turn, world};

fn report(
    world: &mut World<Test, Joint>,
    assignment: &jig_fake_workers::Assignment,
    activity: jig_domain_fleet::Activity,
) {
    let now = world.now();
    world.front().session.driver.send(
        now,
        clock(now),
        jig_test_domain::Event::Core(jig_domain_core::Event::Fleet(jig_domain_fleet::Event::Observe {
            channel: Token::new(1),
            run: Token::new(assignment.task),
            attempt: Token::new(assignment.attempt),
            report: jig_domain_fleet::Report::Activity(activity),
        })),
    );
}

#[test]
fn signing_in_and_starting_a_chat_on_the_engine() {
    let body = turn();
    let mut world = world(body.clone());
    let client = world.client(1, crate::stories::conversation());
    let within = Duration::from_secs(1);
    world.person(
        client,
        vec![
            Step::See { find: Find::named(Role::Button, b"Sign in with Test"), within },
            Step::Press { find: Find::named(Role::Button, b"Sign in with Test") },
            Step::See { find: Find::named(Role::TextArea, b"First words"), within },
            Step::Type { find: Find::named(Role::TextArea, b"First words"), words: b"Hello".as_slice().into() },
            Step::Press { find: Find::named(Role::Button, b"Start a conversation") },
            Step::See { find: Find::text(b"Reported work starting"), within },
            Step::See { find: Find::text(b"Reported work writing"), within },
            Step::See { find: Find::text(b"abc"), within },
            Step::Pause { span: Duration::from_millis(350) },
            Step::See { find: Find::text(b"abc"), within },
        ],
    );
    world.run(Until::Time(Time::from_nanos(30_000_000)));
    let assignment = world.front().session.driver.world.peers.as_ref().expect("ordinary peers").workers[0]
        .seen
        .first()
        .expect("worker received actual assignment")
        .clone();
    report(&mut world, &assignment, jig_domain_fleet::Activity::Starting);
    world.run(Until::Time(Time::from_nanos(80_000_000)));
    report(&mut world, &assignment, completion());
    world.run(Until::Time(Time::from_nanos(180_000_000)));
    assert!(world.tree(client).nodes().iter().any(|node| node.name.as_deref() == Some(b"Provisional".as_slice())));
    assert!(!world.front().session.driver.world.store.rows.values().any(|value| matches!(
        value,
        jig_test_domain::Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Turn(_)))
    )));
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    let rows = &world.front().session.driver.world.store.rows;
    assert_eq!(
        rows.values()
            .filter(|value| matches!(value,
                jig_test_domain::Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Turn(value)))
                if value.task == assignment.task && value.attempt == assignment.attempt && value.turn == 1
                    && value.transcript == body
            ))
            .count(),
        1
    );
    assert_eq!(
        rows.values()
            .filter(|value| matches!(
                value,
                jig_test_domain::Record::Core(jig_domain_core::Record::People(people::Stored::Answer { .. }))
            ))
            .count(),
        1
    );
    assert!(!world.tree(client).nodes().iter().any(|node| node.name.as_deref() == Some(b"Provisional".as_slice())));
}

#[test]
fn a_person_reads_actual_project_settings_and_returns_to_its_overview() {
    let mut world = world(turn());
    let client = world.client(1, crate::stories::conversation());
    let within = Duration::from_secs(1);
    world.person(
        client,
        vec![
            Step::See { find: Find::named(Role::Button, b"Sign in with Test"), within },
            Step::Press { find: Find::named(Role::Button, b"Sign in with Test") },
            Step::See { find: Find::named(Role::Button, b"Project 1"), within },
            Step::Press { find: Find::named(Role::Button, b"Project 1") },
            Step::See { find: Find::named(Role::Tab, b"Settings"), within },
            Step::Press { find: Find::named(Role::Tab, b"Settings") },
            Step::See { find: Find::text(b"Period spend: 1000"), within },
            Step::See { find: Find::text(b"Period spend: 500"), within },
            Step::See { find: Find::text(b"Escalation role: 0"), within },
            Step::See { find: Find::text(b"Configured pool period: 1"), within },
            Step::See { find: Find::named(Role::Tab, b"Overview"), within },
            Step::Press { find: Find::named(Role::Tab, b"Overview") },
            Step::See { find: Find::named(Role::Tab, b"Settings"), within },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    let rows = &world.front().session.driver.world.store.rows;
    assert_eq!(
        rows.values()
            .filter(|value| matches!(
                value,
                jig_test_domain::Record::Core(jig_domain_core::Record::People(people::Stored::Answer { .. }))
            ))
            .count(),
        0
    );
    assert_eq!(
        rows.values()
            .filter(|value| matches!(
                value,
                jig_test_domain::Record::Core(jig_domain_core::Record::People(people::Stored::SignIn { .. }))
            ))
            .count(),
        1
    );
    assert!(!rows.values().any(|value| matches!(
        value,
        jig_test_domain::Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Turn(_)))
    )));
}

#[path = "deployment_story.rs"]
mod deployment;

#[path = "historical_story.rs"]
mod historical;

#[path = "people_stories.rs"]
mod person_stories;

#[path = "role_stories.rs"]
mod role_stories;

#[path = "offered_acts.rs"]
mod offered_acts;

mod fault_stories;

mod job_stories;
mod referee_stories;

fn completion() -> jig_domain_fleet::Activity {
    jig_domain_fleet::Activity::Long {
        operation: jig_domain_fleet::Operation::Completion,
        span: Duration::from_secs(1),
    }
}
