//! The authenticated roles are setup; every business act is a person on the view
//! (domain/people.md, 11; client/domain/testing.md, 5–6).
use super::*;

fn receiving_limits() -> jig_client_test_domain::Limits {
    let mut limits = crate::test::limits();
    limits.client.domain.turn_text = 128;
    limits.client.domain.turn_bytes = 128;
    limits.client.domain.text = 128;
    limits.client.domain.rows = 2;
    limits.client.domain.projects = 2;
    limits.jig.roles = 4;
    limits.client.domain.delegates = 2;
    limits.first.text = 128;
    limits
}

fn client(seed: u64) -> (jig_client_test_domain::Domain, jig_client_test_view::View<'static>) {
    crate::test::new(&receiving_limits(), seed)
}

fn world(seed: u64, scripts: Vec<Box<[Script]>>) -> World<Test, Joint> {
    configured(seed, scripts, |_, _| {})
}

fn configured(
    seed: u64,
    scripts: Vec<Box<[Script]>>,
    configure: fn(&mut jig_test_domain::Config, &jig_test_domain::Limits),
) -> World<Test, Joint> {
    configured_with_slots(seed, scripts, 1, configure)
}

fn configured_with_slots(
    seed: u64,
    scripts: Vec<Box<[Script]>>,
    slots: u32,
    configure: fn(&mut jig_test_domain::Config, &jig_test_domain::Limits),
) -> World<Test, Joint> {
    configured_with_bounds(seed, scripts, slots, configure, &receiving_limits())
}

fn configured_with_bounds(
    seed: u64,
    scripts: Vec<Box<[Script]>>,
    slots: u32,
    configure: fn(&mut jig_test_domain::Config, &jig_test_domain::Limits),
    receiving: &jig_client_test_domain::Limits,
) -> World<Test, Joint> {
    World::new(
        Settings { seed, ..Settings::default() },
        super::super::role_fixture::front(seed, scripts, slots, configure, receiving),
    )
}

fn start(words: &[u8]) -> Vec<Step<jig_client_test_domain::Address>> {
    let within = Duration::from_secs(1);
    vec![
        Step::See { find: Find::named(Role::Button, b"Sign in with Test"), within },
        Step::Press { find: Find::named(Role::Button, b"Sign in with Test") },
        Step::See { find: Find::named(Role::TextArea, b"First words"), within },
        Step::Type { find: Find::named(Role::TextArea, b"First words"), words: words.into() },
        Step::Press { find: Find::named(Role::Button, b"Start a conversation") },
        Step::See { find: Find::text(words), within },
    ]
}

#[test]
fn a_chat_proposing_a_goal_its_member_cannot_fund_is_accepted_by_a_maintainer() {
    let mut world = super::world(super::turn());
    crate::stories::shared::a_roles::a_maintainer_accepts_a_members_proposal(&mut world);
    let actual = world.front().session.driver.world.peers.as_ref().expect("actual peers").parties[1]
        .person
        .expect("actual maintainer");
    assert!(world.front().session.driver.world.store.rows.values().any(|row| matches!(row,
        jig_test_domain::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Ended(task)))
        if task.tracked == Some(7) && task.requester == jig_domain_tasks::Party::Person(actual)
        && task.numbers.budget == 150)));
}

#[test]
fn a_person_answers_a_choice_between_two_reports() {
    let mut world = super::world(super::turn());
    let maintainer = crate::stories::shared::a_roles::prepare_choice(&mut world);
    let keys: Vec<_> = world
        .front()
        .session
        .card_offer_keys(maintainer)
        .into_iter()
        .filter(|key| matches!(key, client::ObjectKey::PersonTask(_)))
        .collect();
    assert_eq!(keys.len(), 1);
    super::offered_acts::card(&mut world, maintainer, keys[0], b"Person task");
    crate::stories::shared::a_roles::complete_choice(&mut world, maintainer);
    let owner = &world.front().session.driver.world;
    assert!(owner.store.rows.values().any(|row| matches!(row,
        jig_test_domain::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Ended(task)))
        if matches!(&task.phase, jig_domain_tasks::Phase::Ended(jig_domain_tasks::Ending::Done(
            jig_domain_tasks::TaskResult::Verdict { code: 2, words, .. })) if words.as_ref() == b"second report"))));
    assert!(
        owner.peers.as_ref().expect("actual worker").workers[0]
            .messages
            .iter()
            .any(|(_, _, _, words)| words.as_ref() == b"second report")
    );
}

#[test]
fn two_parties_decide_one_proposal_and_the_second_is_told_who() {
    let mut world = super::world(super::turn());
    let (owner, maintainer) = crate::stories::shared::a_roles::prepare_two_parties(&mut world);
    for party in [owner, maintainer] {
        let keys: Vec<_> = world
            .front()
            .session
            .card_offer_keys(party)
            .into_iter()
            .filter(|key| matches!(key, client::ObjectKey::Proposal(_)))
            .collect();
        assert_eq!(keys.len(), 1);
        super::offered_acts::card(&mut world, party, keys[0], b"Proposal");
    }
    crate::stories::shared::a_roles::confirm_two_parties(&mut world, owner, maintainer);
    let first_person = 1;
    let engine = &world.front().session.driver.world;
    assert_eq!(
        engine
            .store
            .rows
            .values()
            .filter(|row| matches!(row,
        jig_test_domain::Record::Core(jig_domain_core::Record::People(people::Stored::Answer { outcome, .. }))
        if matches!(outcome, people::Outcome::ProposalDecided { by, choice: people::ProposalChoice::Accepted, .. }
            if *by == first_person)))
            .count(),
        2
    );
    assert_eq!(
        engine
            .store
            .rows
            .values()
            .filter(|row| matches!(row,
        jig_test_domain::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Ended(task)))
        if task.tracked == Some(7) && task.numbers.budget == 150))
            .count(),
        1
    );
    let assignments = &engine.peers.as_ref().expect("actual worker").workers[0].seen;
    assert_eq!(assignments.len(), 3, "the proposing chat resumes after its one accepted goal");
    let tasks: std::collections::BTreeSet<_> = assignments.iter().map(|row| row.task).collect();
    assert_eq!(tasks.len(), 2, "only the proposing chat and its accepted goal");
    let parent = assignments[0].task;
    let parent_attempts: std::collections::BTreeSet<_> =
        assignments.iter().filter(|row| row.task == parent).map(|row| row.attempt).collect();
    assert_eq!(parent_attempts.len(), 2, "the original chat parks and then resumes");
    assert_eq!(assignments.iter().filter(|row| row.task != parent).count(), 1, "one goal assignment");
}

#[test]
fn an_observer_is_refused_a_request_and_told_why() {
    let mut world = super::world(super::turn());
    crate::stories::shared::a_roles::an_observer_sees_why_creation_is_unavailable(&mut world);
    let engine = &world.front().session.driver.world;
    assert!(engine.peers.as_ref().expect("actual worker").workers[0].seen.is_empty());
    assert!(!engine.store.rows.values().any(|row| matches!(
        row,
        jig_test_domain::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Live(_)))
    )));
    assert_eq!(
        engine
            .store
            .rows
            .values()
            .filter(|row| matches!(row,
        jig_test_domain::Record::Core(jig_domain_core::Record::People(people::Stored::Answer { outcome, .. }))
        if matches!(outcome, people::Outcome::Refused(people::Refusal::Role { held: Some(people::Role::Observer) }))))
            .count(),
        0
    );
}

#[path = "engine_stories.rs"]
mod engine_stories;

#[path = "role_stories/proposed_story.rs"]
mod proposed_story;

#[path = "role_stories/card_stories.rs"]
mod card_stories;

#[path = "role_stories/fault_stories.rs"]
mod fault_stories;

#[path = "role_stories/family_stories.rs"]
mod family_stories;
