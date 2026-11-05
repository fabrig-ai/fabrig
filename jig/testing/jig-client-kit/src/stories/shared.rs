//! Person stories shared by all fronts (jig/docs/design/client/domain/testing.md, 4–5).
use crate::{Front, PROJECT, SIGNING_PARTY, Test, Until, World, harness::Seed};
use skein_fake_person::{Find, Step};
use skein_lib::Duration;
use skein_ui::Role;
use skein_world::domain::Verdict;

/// A person signs in, sees an existing conversation, and starts another one.
pub fn signing_in_and_starting_a_chat<F: Front<Test>>(world: &mut World<Test, F>) {
    const WITHIN: Duration = Duration::from_secs(1);
    world.seed(Seed::ChatProject { party: SIGNING_PARTY, project: PROJECT });
    world.seed(Seed::Conversation { party: SIGNING_PARTY, project: PROJECT, words: b"Earlier".as_slice().into() });
    let client = world.client(SIGNING_PARTY, super::conversation());
    world.person(
        client,
        vec![
            Step::See { find: Find::named(Role::Button, b"Sign in with Test"), within: WITHIN },
            Step::Press { find: Find::named(Role::Button, b"Sign in with Test") },
            Step::See { find: Find::named(Role::TextArea, b"First words"), within: WITHIN },
            Step::See { find: Find::named(Role::Link, b"Earlier"), within: WITHIN },
            Step::Type { find: Find::named(Role::TextArea, b"First words"), words: b"Hello".as_slice().into() },
            Step::Press { find: Find::named(Role::Button, b"Start a conversation") },
            Step::See { find: Find::named(Role::Link, b"Hello"), within: WITHIN },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert!(outcome.judged.0 > 0);
}

pub mod a_person;
pub mod a_projects;

#[cfg(test)]
mod a_tests;

pub mod a_proposals;

pub mod a_roles;
pub mod b_config;
pub mod b_conversations;
pub mod b_effects;
pub mod b_family;
pub mod b_goals;
pub mod b_held;
pub mod b_inbox;
pub mod b_proposals;
mod b_runner;
pub mod b_scenarios;
pub mod b_scripted;
pub mod b_seed;
