//! QA1: one Chromium check of a task address across new loads and history.
use skein_fake_person::{Find, Step as PersonStep};
use skein_fake_person_browser::{Harness, Outcome, Script, Step};
use skein_ui::Role;
use temper_client_browser_world::{Tier, WITHIN, start_playground_chat};

fn see(find: Find) -> Step {
    Step::Person(PersonStep::See { find, within: WITHIN })
}
fn go(path: &[u8]) -> Step {
    Step::Person(PersonStep::Go { address: path.into() })
}
fn agent() -> Find {
    Find::exactly_named(Role::Region, b"Task brief")
}

// client/README.md 6.10 and client/browser.md 2.3: "/tasks/N typed or opened"
// and "a reload ... is a new client at that address"; lineage links (5.9)
// make history entries, and Back returns to the task page at its own address.
#[test]
fn typed_agent_address_survives_reload_and_a_lineage_round_trip() {
    let tier = Tier::demo("qa-task-page-load-history");
    let mut steps = start_playground_chat(b"Add CSV export to widget");
    steps.extend([
        see(Find::exactly_named(Role::Region, b"Task 2")),
        go(b"/tasks/3"),
        see(agent()),
        see(Find::named(Role::Heading { level: 1 }, "T3 · ".as_bytes())),
        Step::Address(b"/tasks/3".as_slice().into()),
        Step::Person(PersonStep::Reload),
        see(agent()),
        Step::Address(b"/tasks/3".as_slice().into()),
        Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Link, b"Task 2") }),
        see(Find::exactly_named(Role::Region, b"Its tasks")),
        Step::Address(b"/tasks/2".as_slice().into()),
        Step::Back,
        see(agent()),
        Step::Address(b"/tasks/3".as_slice().into()),
        Step::Person(PersonStep::Reload),
        see(agent()),
        Step::Address(b"/tasks/3".as_slice().into()),
    ]);
    let mut script = Script::new(steps.into_boxed_slice());
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut script]), Outcome::Done);
}
