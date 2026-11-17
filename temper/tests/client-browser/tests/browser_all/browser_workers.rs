//! The Workers tab's fields survive a settings answer and a reload (client/pages/workers.md, 2).
use skein_browser::boundary::Key;
use skein_fake_person::{Find, Step as PersonStep};
use skein_fake_person_browser::{Script, Step};
use skein_ui::Role;
use temper_client_browser_world::{Stories, Tier, WITHIN};

#[test]
fn an_owner_adds_changes_and_removes_a_worker_without_restarting() {
    let tier = Tier::new("workers-settings");
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let field = |name: &[u8]| Find::exactly_named(Role::TextField, name);
    let type_in = |name: &[u8], value: &[u8]| Step::Type { find: field(name), words: value.into() };
    let open = || {
        Step::Person(PersonStep::Go {
            address: temper_client_browser_world::page_address(b"/projects/1/settings/builders"),
        })
    };
    let mut stories = Stories::of_person(0);
    let mut steps = vec![];
    stories.begin("workers read and add", &steps);
    steps.extend([
        open(),
        see(Find::exactly_named(Role::Heading { level: 1 }, b"Where agents work.")),
        see(Find::exactly_named(Role::Heading { level: 2 }, "Workers · 1".as_bytes())),
        see(field(b"Places for local")),
        see(Find::exactly_named(Role::Heading { level: 2 }, b"Add a worker")),
        type_in("The worker’s name".as_bytes(), b"w2"),
        type_in(b"Places", b"1"),
        type_in("Its secret’s digest".as_bytes(), b"abababababababababababababababababababababababababababababababab"),
        Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Button, b"Add") }),
        see(Find::exactly_named(Role::Heading { level: 2 }, "Workers · 2".as_bytes())),
        see(field(b"Places for w2")),
    ]);
    stories.begin("workers places and reload", &steps);
    steps.extend([
        type_in(b"Places for w2", b"2"),
        Step::Key(Key::Enter),
        see(Find::text("2 places · no runs · digest abababab…".as_bytes())),
        open(),
        see(Find::text("2 places · no runs · digest abababab…".as_bytes())),
    ]);
    stories.begin("workers removal", &steps);
    steps.extend([
        Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Button, b"Remove") }),
        see(Find::exactly_named(Role::Heading { level: 2 }, "Workers · 1".as_bytes())),
        Step::Person(PersonStep::Gone { find: field(b"Places for w2"), within: WITHIN }),
    ]);
    let mut script = Script::new(steps.into_boxed_slice());
    stories.assert_done(temper_client_browser_world::harness(tier.setup()).run(&mut [&mut script]));
}
