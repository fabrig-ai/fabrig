//! People's actions and visible results (jig/docs/design/client/domain/testing.md, 4–6).
use crate::{ClientId, Front, Test, Until, World, harness::Seed};
use jig_client_test_domain::Address as RootAddress;
use jig_client_view::address::{Address, TaskSection};
use skein_fake_person::{Find, Step};
use skein_lib::Duration;
use skein_ui::Role;
use skein_world::domain::Verdict;
const WITHIN: Duration = Duration::from_secs(1);

pub(super) fn finish<F: Front<Test>>(world: &mut World<Test, F>) {
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert!(outcome.judged.0 > 0);
}
fn start<F: Front<Test>>(world: &mut World<Test, F>) -> ClientId {
    world.seed(Seed::ChatProject { party: 1, project: 1 });
    let client = world.client(1, crate::stories::conversation());
    let mut steps = vec![
        Step::See { find: Find::named(Role::Button, b"Sign in with Test"), within: WITHIN },
        Step::Press { find: Find::named(Role::Button, b"Sign in with Test") },
        Step::See { find: Find::named(Role::TextArea, b"First words"), within: WITHIN },
    ];
    steps.extend(crate::stories::start_chat_steps());
    steps.push(Step::See { find: Find::text(b"Hello"), within: WITHIN });
    world.person(client, steps);
    finish(world);
    client
}

/// A person's words are shown in their running conversation.
pub fn words_in_a_running_conversation<F: Front<Test>>(world: &mut World<Test, F>) -> ClientId {
    let client = start(world);
    world.person(
        client,
        vec![
            Step::See { find: Find::exactly_named(Role::Button, b"Write"), within: WITHIN },
            Step::Press { find: Find::exactly_named(Role::Button, b"Write") },
            Step::Type {
                find: Find::named(Role::TextArea, b"Your answer"),
                words: b"Please continue".as_slice().into(),
            },
            Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
            Step::See { find: Find::text(b"Please continue"), within: WITHIN },
        ],
    );
    finish(world);
    client
}

/// Stop and release show the hold and its subsequent release.
pub fn stopping_and_releasing_a_conversation<F: Front<Test>>(world: &mut World<Test, F>) -> ClientId {
    let client = start(world);
    world.person(
        client,
        vec![
            Step::See { find: Find::exactly_named(Role::Button, b"Stop"), within: WITHIN },
            Step::Press { find: Find::exactly_named(Role::Button, b"Stop") },
            Step::See { find: Find::text(b"Stop the live run and hold its task"), within: WITHIN },
            Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
            Step::See { find: Find::text(b"Held by a placed hold"), within: WITHIN },
            Step::See { find: Find::exactly_named(Role::Button, b"Release"), within: WITHIN },
            Step::Press { find: Find::exactly_named(Role::Button, b"Release") },
            Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
            Step::See { find: Find::text(b"Released"), within: WITHIN },
        ],
    );
    finish(world);
    client
}

/// Reload restores the unsent words in the person's editor.
pub fn a_draft_survives_reload<F: Front<Test>>(world: &mut World<Test, F>) -> ClientId {
    let client = start(world);
    world.person(
        client,
        vec![
            Step::See { find: Find::exactly_named(Role::Button, b"Write"), within: WITHIN },
            Step::Press { find: Find::exactly_named(Role::Button, b"Write") },
            Step::Type { find: Find::named(Role::TextArea, b"Your answer"), words: b"Unsent draft".as_slice().into() },
            Step::Pause { span: Duration::from_millis(30) },
            Step::Reload,
            Step::See { find: Find::exactly_named(Role::Button, b"Write"), within: WITHIN },
            Step::Press { find: Find::exactly_named(Role::Button, b"Write") },
        ],
    );
    finish(world);
    assert!(
        world
            .tree(client)
            .nodes()
            .iter()
            .any(|node| node.value.as_ref().is_some_and(|value| value.text.as_ref() == b"Unsent draft"))
    );
    client
}

/// An owner reads settings and returns to the project overview.
pub fn reading_project_settings<F: Front<Test>>(world: &mut World<Test, F>) {
    world.seed(Seed::ChatProject { party: 1, project: 1 });
    world.seed(Seed::Profile);
    let client = world.client(1, crate::stories::conversation());
    world.person(
        client,
        vec![
            Step::See { find: Find::named(Role::Button, b"Sign in with Test"), within: WITHIN },
            Step::Press { find: Find::named(Role::Button, b"Sign in with Test") },
            Step::See { find: Find::named(Role::Button, b"Project 1"), within: WITHIN },
            Step::Press { find: Find::named(Role::Button, b"Project 1") },
            Step::See { find: Find::named(Role::Tab, b"Settings"), within: WITHIN },
            Step::Press { find: Find::named(Role::Tab, b"Settings") },
            Step::See { find: Find::text(b"Period spend: 1000"), within: WITHIN },
            Step::See { find: Find::text(b"Period spend: 500"), within: WITHIN },
            Step::See { find: Find::text(b"Escalation role: 0"), within: WITHIN },
            Step::See { find: Find::text(b"Configured pool period: 1"), within: WITHIN },
            Step::Press { find: Find::named(Role::Tab, b"Overview") },
            Step::See { find: Find::named(Role::Tab, b"Settings"), within: WITHIN },
        ],
    );
    finish(world);
}

/// An owner sees deployment capacity, accounts and configured connectors.
pub fn reading_deployment<F: Front<Test>>(world: &mut World<Test, F>) {
    world.seed(Seed::ChatProject { party: 1, project: 1 });
    world.seed(Seed::Profile);
    let client = world.client(1, RootAddress::Jig(Address::Deployment));
    world.person(
        client,
        vec![
            Step::See { find: Find::named(Role::Button, b"Sign in with Test"), within: WITHIN },
            Step::Press { find: Find::named(Role::Button, b"Sign in with Test") },
            Step::See { find: Find::named(Role::Heading { level: 1 }, b"Deployment"), within: WITHIN },
            Step::See { find: Find::named(Role::Region, b"Tasks Capacity"), within: WITHIN },
            Step::See { find: Find::text(b"Account: 0"), within: WITHIN },
            Step::See { find: Find::text(b"Account generation: 1"), within: WITHIN },
            Step::See { find: Find::text(b"Hosts forgotten: 0"), within: WITHIN },
            Step::See { find: Find::named(Role::Region, b"Configured connectors"), within: WITHIN },
        ],
    );
    finish(world);
    let tree = world.tree(client);
    let (start, region) = tree
        .nodes()
        .iter()
        .enumerate()
        .find(|(_, node)| node.role == Role::Region && node.name.as_deref() == Some(b"Configured connectors"))
        .expect("configured connector catalogue");
    let connectors: Vec<_> = tree
        .nodes()
        .iter()
        .skip(start.checked_add(1).expect("catalogue start"))
        .take(usize::try_from(region.size.saturating_sub(1)).expect("catalogue size"))
        .filter_map(|node| node.slot)
        .map(|slot| match jig_client_view::Owner::from_opaque(slot.owner) {
            Some(jig_client_view::Owner::Application { number }) => number,
            Some(jig_client_view::Owner::Jig) | None => panic!("connector slot belongs to its application family"),
        })
        .collect();
    assert_eq!(connectors, [1, 2]);
    world.person(
        client,
        vec![
            Step::Press { find: Find::named(Role::Button, b"Project 1") },
            Step::See { find: Find::named(Role::Tab, b"Settings"), within: WITHIN },
        ],
    );
    finish(world);
}

/// A completed conversation retains its report and offers no cancellation.
pub fn reading_a_completed_conversation<F: Front<Test>>(world: &mut World<Test, F>) {
    completed(world, false);
}
/// A fresh public task address still shows the completed report.
pub fn revisiting_a_completed_conversation<F: Front<Test>>(world: &mut World<Test, F>) {
    completed(world, true);
}
fn completed<F: Front<Test>>(world: &mut World<Test, F>, revisit: bool) {
    world.seed(Seed::ChatResult { report: b"finished".as_slice().into() });
    let client = start(world);
    if revisit {
        world.go(client, RootAddress::Jig(Address::Task { task: 1, section: TaskSection::Activity }));
    }
    world.person(
        client,
        vec![
            Step::See { find: Find::named(Role::Region, b"Ended object"), within: WITHIN },
            Step::See { find: Find::text(b"finished"), within: WITHIN },
            Step::Gone { find: Find::exactly_named(Role::Button, b"Cancel"), within: WITHIN },
        ],
    );
    finish(world);
}

pub(super) fn sign_in() -> Vec<Step<RootAddress>> {
    vec![
        Step::See { find: Find::named(Role::Button, b"Sign in with Test"), within: WITHIN },
        Step::Press { find: Find::named(Role::Button, b"Sign in with Test") },
    ]
}

pub(super) fn sign_in_if_needed<F: Front<Test>>(world: &mut World<Test, F>, client: ClientId) {
    use skein_fake_person::{Face, Found, TreeFace};
    world.person(client, vec![Step::Pause { span: Duration::from_millis(50) }]);
    finish(world);
    if matches!(
        TreeFace::new(world.tree(client)).find(&Find::named(Role::Button, b"Sign in with Test")),
        Found::Node(_)
    ) {
        world.person(client, sign_in());
        finish(world);
    }
}

/// Signing out returns the person to the sign-in prompt.
pub fn signing_out<F: Front<Test>>(world: &mut World<Test, F>) -> ClientId {
    world.seed(Seed::ChatProject { party: 1, project: 1 });
    let client = world.client(1, crate::stories::inbox());
    let mut steps = sign_in();
    steps.extend([
        Step::See { find: Find::named(Role::Button, b"Sign out"), within: WITHIN },
        Step::Press { find: Find::named(Role::Button, b"Sign out") },
        Step::See { find: Find::named(Role::Button, b"Sign in with Test"), within: WITHIN },
    ]);
    world.person(client, steps);
    finish(world);
    client
}
