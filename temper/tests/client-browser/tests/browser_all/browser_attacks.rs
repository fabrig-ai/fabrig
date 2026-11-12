//! Origin and frame policies share a signed-in browser
//! (temper/docs/design/client/web/testing.md, 6, item 6).
use jig_client_fake_person_browser::pages;
use skein_browser::boundary::Key;
use skein_fake_person::{Find, Step as PersonStep};
use skein_fake_person_browser::{Outcome, Script, Step, origins::Origin};
use skein_ui::Role;
use temper_client_browser_world::{ABSENT, Tier, WITHIN};

#[test]
fn cross_origin_requests_and_framing_cannot_act_as_the_person() {
    let tier = Tier::new("origin-and-frame-policies");
    let target = tier.target();
    let mut setup = tier.setup();
    setup.expected.push(b"Failed to load resource".as_slice().into());
    setup.expected.push(b"blocked by CORS policy".as_slice().into());
    // One port on the service's host and one other host name exercise both origin boundaries.
    setup.origins = [b"127.0.0.1".as_slice(), b"localhost"]
        .into_iter()
        .map(|host| Origin { host: host.into(), pages: vec![pages::forging(&target), pages::framing(&target)] })
        .collect();
    let row = Find::exactly_named(Role::Button, b"Owner");
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let mut steps = vec![
        // The printed link signs in at the welcome, whose frame holds its steps and the top bar's end.
        see(Find::exactly_named(Role::List, b"Steps")),
        Step::Visit { origin: 1, path: b"/framing".as_slice().into() },
        Step::Trouble { words: b"frame-ancestors".as_slice().into(), within: WITHIN },
        Step::Person(PersonStep::Gone { find: row.clone(), within: ABSENT }),
        Step::Person(PersonStep::Go { address: temper_client_browser_world::page_address(target.page) }),
        see(Find::named(Role::Heading { level: 1 }, b"Talk to an agent")),
        see(tier.states().live),
        temper_client_browser_world::connected(),
        Step::Person(PersonStep::Press { find: row.clone() }),
        see(Find::exactly_named(Role::MenuItem, b"Sign out")),
        Step::Key(Key::Escape),
    ];
    for origin in [0, 1] {
        steps.extend([
            Step::Visit { origin, path: b"/forging".as_slice().into() },
            see(Find::exactly_named(Role::Heading { level: 1 }, b"Forged requests completed")),
            Step::Person(PersonStep::Go { address: temper_client_browser_world::page_address(target.page) }),
            see(Find::named(Role::Heading { level: 1 }, b"Talk to an agent")),
            see(tier.states().live),
            temper_client_browser_world::connected(),
            Step::Person(PersonStep::Gone { find: target.done.clone(), within: ABSENT }),
            Step::Person(PersonStep::Press { find: row.clone() }),
            see(Find::exactly_named(Role::MenuItem, b"Sign out")),
            Step::Key(Key::Escape),
        ]);
    }
    let mut script = Script::new(steps.into_boxed_slice());
    assert_eq!(temper_client_browser_world::harness(setup).run(&mut [&mut script]), Outcome::Done);
}
