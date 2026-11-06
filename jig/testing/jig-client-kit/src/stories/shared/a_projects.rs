//! Project identities shared by all seeding fronts (jig/docs/design/client/domain/testing.md, 4–5).
use crate::{Front, Test, World, harness::Seed};
use skein_fake_person::{Find, Step};
use skein_lib::Duration;
use skein_ui::Role;

/// The same owner holds two projects, one explicitly named and one with its seed name.
#[must_use]
pub fn setup(party: u64) -> (jig_test_domain::Config, jig_test_domain::Limits) {
    assert_eq!(party, 1, "the configured owner");
    let (mut configuration, mut limits) = jig_core_kit::fixtures::two_projects(8510);
    configuration.core.seed_projects[0].name = Box::from("Tally");
    let receiving = crate::test::limits().client.domain;
    limits.core.documents.rows = receiving.rows;
    limits.core.documents.lines = receiving.rows;
    limits.core.documents.chips = receiving.rows;
    limits.core.documents.text_bytes = receiving.text;
    limits.core.views.watchers = receiving.watches;
    (configuration, limits)
}

/// Both frame names are visible, and choosing the second reads its own project page.
pub fn moving_between_named_projects<F: Front<Test>>(world: &mut World<Test, F>) {
    const WITHIN: Duration = Duration::from_secs(1);
    world.seed(Seed::TwoProjects { party: 1 });
    let client = world.client(1, crate::stories::conversation());
    world.person(
        client,
        vec![
            Step::See { find: Find::named(Role::Button, b"Sign in with Test"), within: WITHIN },
            Step::Press { find: Find::named(Role::Button, b"Sign in with Test") },
            Step::See { find: Find::exactly_named(Role::Button, b"Project: Tally"), within: WITHIN },
            Step::See { find: Find::exactly_named(Role::Button, b"Project: Project 2"), within: WITHIN },
            Step::Press { find: Find::exactly_named(Role::Button, b"Project: Tally") },
            Step::See { find: Find::exactly_named(Role::Tabs, b"Tally"), within: WITHIN },
            Step::Press { find: Find::exactly_named(Role::Button, b"Project: Project 2") },
            Step::See { find: Find::exactly_named(Role::Tabs, b"Project 2"), within: WITHIN },
        ],
    );
    super::a_person::finish(world);
}
