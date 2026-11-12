//! Accounts history, field re-entry, and both sign-in stories (temper/docs/design/client/web/testing.md, 6).
use skein_fake_person::{Find, Step as PersonStep};
use skein_fake_person_browser::{Script, Step};
use skein_ui::Role;
use temper_client_browser_world::{Stories, Tier, WITHIN};

#[test]
fn the_sign_in_opens_a_tab_and_lands_on_accounts() {
    use skein_fake_peers::oauth::Observation;
    use skein_fake_person_browser::harness::Lent;
    let (tier, mut issuer) = Tier::accounts_sign_in("accounts-round-trip");
    let mut stories = Stories::of_person(0);
    let mut steps = vec![];
    stories.begin("welcome sign-in", &steps);
    steps.extend(welcome_steps());
    stories.begin("connect model sign-in", &steps);
    steps.extend(sign_in_steps());
    stories.begin("returned account history and reload", &steps);
    steps.extend(returned_history_steps());
    stories.begin("key return row and field history", &steps);
    steps.extend(key_history_steps());
    let mut script = Script::new(steps.into_boxed_slice());
    let outcome = temper_client_browser_world::harness(tier.setup())
        .run_lending(&mut [&mut script], &mut [Lent { name: b"issuer", peer: &mut issuer }]);
    stories.assert_done(outcome);
    assert_eq!(
        issuer.observations().iter().filter(|event| matches!(event, Observation::Authorize { .. })).count(),
        2,
        "sign-in: one authorization for each sign-in; a reload makes no second departure"
    );
    assert_eq!(
        issuer.observations().iter().filter(|event| matches!(event, Observation::Post { .. })).count(),
        2,
        "sign-in: one code exchange for each sign-in"
    );
}

fn sign_in_steps() -> Vec<Step> {
    vec![
        Step::Person(PersonStep::Go { address: temper_client_browser_world::page_address(b"/chats".as_slice()) }),
        Step::Person(PersonStep::See {
            find: Find::named(Role::Heading { level: 1 }, b"Talk to an agent"),
            within: WITHIN,
        }),
        Step::Address(b"/chats".as_slice().into()),
        Step::Person(PersonStep::Go {
            address: temper_client_browser_world::page_address(b"/accounts/connect".as_slice()),
        }),
        Step::Person(PersonStep::See {
            find: Find::exactly_named(Role::Heading { level: 1 }, b"Connect a model"),
            within: WITHIN,
        }),
        Step::Person(PersonStep::See {
            find: Find::exactly_named(Role::Button, b"Sign in to Test provider"),
            within: WITHIN,
        }),
        Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Button, b"Sign in to Test provider") }),
        Step::Person(PersonStep::See { find: Find::text(b"Added your Test provider account"), within: WITHIN }),
        sign_in_tabs(2),
    ]
}

fn returned_history_steps() -> Vec<Step> {
    vec![
        Step::Address(b"/accounts".as_slice().into()),
        Step::Back,
        Step::Person(PersonStep::See {
            find: Find::exactly_named(Role::Heading { level: 1 }, b"Connect a model"),
            within: WITHIN,
        }),
        Step::Address(b"/accounts/connect".as_slice().into()),
        Step::Back,
        Step::Person(PersonStep::See {
            find: Find::named(Role::Heading { level: 1 }, b"Talk to an agent"),
            within: WITHIN,
        }),
        Step::Address(b"/chats".as_slice().into()),
        Step::Forward,
        Step::Forward,
        Step::Person(PersonStep::See {
            find: Find::named(Role::Heading { level: 2 }, "Your accounts · ".as_bytes()),
            within: WITHIN,
        }),
        Step::Address(b"/accounts".as_slice().into()),
        Step::Person(PersonStep::Gone { find: Find::text(b"Added your"), within: WITHIN }),
        Step::Person(PersonStep::Go { address: temper_client_browser_world::page_address(b"/accounts") }),
        Step::Person(PersonStep::See {
            find: Find::named(Role::Heading { level: 2 }, "Your accounts · ".as_bytes()),
            within: WITHIN,
        }),
        Step::Person(PersonStep::Gone { find: Find::text(b"Added your"), within: WITHIN }),
        Step::Address(b"/accounts".as_slice().into()),
        sign_in_tabs(2),
    ]
}

fn key_history_steps() -> Vec<Step> {
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let press = |name: &[u8], role| Step::Person(PersonStep::Press { find: Find::exactly_named(role, name) });
    let input = |name: &[u8], value: &[u8]| {
        Step::Person(PersonStep::Type { find: Find::exactly_named(Role::TextField, name), words: Box::from(value) })
    };
    let accounts = || Find::exactly_named(Role::Heading { level: 1 }, b"Connect a model");
    let chats = || Find::named(Role::Heading { level: 1 }, b"Talk to an agent");
    let mut steps = key_link_steps();
    steps.extend([
        input(b"Key (pasted once)", b"private-browser-key"),
        press(b"Add key", Role::Button),
        see(Find::text(b"Added your Test provider API key")),
        Step::Address(b"/accounts".as_slice().into()),
        see(Find::named(Role::Disclosure, b"Test provider API key")),
        Step::Person(PersonStep::Gone {
            find: Find::exactly_named(Role::Heading { level: 1 }, b"A Test provider API key"),
            within: WITHIN,
        }),
        // No chat has started work, so Work has no Chats filter.
        Step::Person(PersonStep::Go { address: temper_client_browser_world::page_address(b"/chats".as_slice()) }),
        see(chats()),
        Step::Back,
        see(Find::named(Role::Heading { level: 2 }, "Your accounts · ".as_bytes())),
        Step::Address(b"/accounts".as_slice().into()),
        Step::Person(PersonStep::Gone { find: Find::text(b"Added your"), within: WITHIN }),
        press(b"+ Connect a model", Role::Link),
        see(accounts()),
        press(b"Add key", Role::Link),
        see(Find::exactly_named(Role::Heading { level: 1 }, b"A Test provider API key")),
        // The browser kit refuses typing into a nonempty field: this proves re-entry after clearing.
        input(b"Key (pasted once)", b"another-private-key"),
        // No chat has started work, so Work has no Chats filter.
        Step::Person(PersonStep::Go { address: temper_client_browser_world::page_address(b"/chats".as_slice()) }),
        see(chats()),
        Step::Back,
        see(Find::exactly_named(Role::Heading { level: 1 }, b"A Test provider API key")),
        Step::Address(Box::from(b"/accounts/connect?provider=1&way=key".as_slice())),
        Step::Forward,
        see(chats()),
        Step::Address(Box::from(b"/chats".as_slice())),
        Step::Back,
        see(Find::exactly_named(Role::Heading { level: 1 }, b"A Test provider API key")),
    ]);
    steps
}

fn key_link_steps() -> Vec<Step> {
    let see = |role, name: &[u8]| Step::Person(PersonStep::See { find: Find::named(role, name), within: WITHIN });
    let key = |nth| Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Link, b"Add key").nth(nth) });
    vec![
        Step::Person(PersonStep::Go { address: temper_client_browser_world::page_address(b"/accounts".as_slice()) }),
        see(Role::Heading { level: 2 }, "Your accounts · ".as_bytes()),
        Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Link, b"+ Connect a model") }),
        see(Role::Heading { level: 1 }, b"Connect a model"),
        Step::Address(b"/accounts/connect".as_slice().into()),
        // Key links follow provider number: Test is 1, Welcome is 2.
        key(1),
        see(Role::Heading { level: 1 }, b"A Welcome provider API key"),
        Step::Address(b"/accounts/connect?provider=2&way=key".as_slice().into()),
        Step::Back,
        see(Role::Heading { level: 1 }, b"Connect a model"),
        key(0),
        see(Role::Heading { level: 1 }, b"A Test provider API key"),
        Step::Address(b"/accounts/connect?provider=1&way=key".as_slice().into()),
    ]
}

fn sign_in_tabs(count: usize) -> Step {
    use skein_fake_person_browser::harness::TabSeen;
    let tabs = (0..count)
        .map(|_| TabSeen { title: None, address: Some(b"/callback?".as_slice().into()), opener: Some(false) })
        .collect();
    Step::Tabs { tabs, within: WITHIN }
}

fn welcome_steps() -> Vec<Step> {
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    vec![
        see(Find::exactly_named(Role::Link, b"Set up temper")),
        Step::Person(PersonStep::Go {
            address: temper_client_browser_world::page_address(b"/welcome?at=model".as_slice()),
        }),
        see(Find::exactly_named(Role::Button, b"Sign in to Welcome provider")),
        Step::Address(b"/welcome?at=model".as_slice().into()),
        Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Button, b"Sign in to Welcome provider") }),
        sign_in_tabs(1),
        see(Find::exactly_named(Role::Heading { level: 1 }, b"Almost ready")),
        Step::Address(b"/welcome?at=ready".as_slice().into()),
    ]
}
