//! Accounts history, field re-entry, and both sign-in stories (temper/docs/design/client/browser.md, 4.2).
use skein_fake_person::{Find, Step as PersonStep};
use skein_fake_person_browser::{Harness, Outcome, Script, Step};
use skein_ui::Role;
use temper_client_browser_world::{Stories, Tier, WITHIN};
#[test]
fn the_accounts_page_returns_after_a_key_send() {
    let tier = Tier::accounts("accounts-history");
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let press = |name: &[u8], role| Step::Person(PersonStep::Press { find: Find::exactly_named(role, name) });
    let input = |name: &[u8], value: &[u8]| {
        Step::Person(PersonStep::Type { find: Find::exactly_named(Role::TextField, name), words: Box::from(value) })
    };
    let accounts = || Find::exactly_named(Role::Heading { level: 1 }, b"Connect a model");
    let chats = || Find::named(Role::Heading { level: 1 }, b"Talk to an agent");
    let mut script = Script::new(Box::new([
        see(Find::exactly_named(Role::Link, b"Set up temper")),
        Step::Person(PersonStep::Go { address: Box::from(b"/accounts/connect".as_slice()) }),
        see(accounts()),
        Step::Address(Box::from(b"/accounts/connect".as_slice())),
        see(Find::exactly_named(Role::Link, b"Add key")),
        press(b"Add key", Role::Link),
        input(b"Key (pasted once)", b"private-browser-key"),
        press(b"Add key", Role::Button),
        see(Find::text(b"Added your Test provider API key")),
        Step::Address(b"/accounts".as_slice().into()),
        press(b"Chats", Role::Link),
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
        press(b"Chats", Role::Link),
        see(chats()),
        Step::Back,
        see(Find::exactly_named(Role::Heading { level: 1 }, b"A Test provider API key")),
        Step::Address(Box::from(b"/accounts/connect?provider=17&way=key".as_slice())),
        Step::Forward,
        see(chats()),
        Step::Address(Box::from(b"/chats".as_slice())),
        Step::Back,
        see(Find::exactly_named(Role::Heading { level: 1 }, b"A Test provider API key")),
    ]));
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut script]), Outcome::Done);
}

/// Read the returned Accounts page, check its replaced history entry, then reload it.
struct RoundTrip {
    script: Script,
    before: u32,
    reading: bool,
    reloaded: bool,
}
impl skein_fake_person_browser::harness::Play for RoundTrip {
    fn next(
        &mut self,
        face: &mut skein_fake_person_browser::BrowserFace<'_>,
        now: skein_lib::Time,
    ) -> skein_fake_person::Next<Box<[u8]>> {
        use skein_fake_person::Next;
        if self.reading {
            let address = face.take_address().expect("completed address read");
            let address = std::str::from_utf8(&address).expect("ASCII return address");
            let (_, path) =
                address.strip_prefix("http://").expect("clients HTTP origin").split_once('/').expect("page path");
            assert_eq!(path, "accounts", "the account replaces the sign-in way's history entry");
            self.before = skein_fake_person_browser::harness::Play::at(&self.script);
            self.script = Script::new(Box::new([
                Step::Address(format!("/{path}").into_bytes().into_boxed_slice()),
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
                Step::Person(PersonStep::See { find: Find::named(Role::Heading { level: 2 }, "Your accounts · ".as_bytes()), within: WITHIN }),
                Step::Address(format!("/{path}").into_bytes().into_boxed_slice()),
                Step::Person(PersonStep::Gone { find: Find::text(b"Added your"), within: WITHIN }),
                Step::Person(PersonStep::Reload),
                Step::Person(PersonStep::See { find: Find::named(Role::Heading { level: 2 }, "Your accounts · ".as_bytes()), within: WITHIN }),
                Step::Person(PersonStep::Gone { find: Find::text(b"Added your"), within: WITHIN }),
                Step::Address(format!("/{path}").into_bytes().into_boxed_slice()),
                sign_in_tabs(2),
            ]));
            self.reading = false;
            self.reloaded = true;
        }
        let next = self.script.next(face, now);
        if matches!(next, Next::Done) && !self.reloaded {
            face.address();
            self.reading = true;
            Next::Wait { until: None }
        } else {
            match next {
                Next::Failed { step, why } => {
                    Next::Failed { step: self.before.checked_add(step).expect("script step fits"), why }
                }
                Next::Do(_) | Next::Wait { .. } | Next::Done => next,
            }
        }
    }
    fn at(&self) -> u32 {
        self.before.checked_add(skein_fake_person_browser::harness::Play::at(&self.script)).expect("script step fits")
    }
}

#[test]
fn the_sign_in_opens_a_tab_and_lands_on_accounts() {
    use skein_fake_peers::oauth::Observation;
    use skein_fake_person_browser::harness::Lent;
    let (tier, mut issuer) = Tier::accounts_sign_in("accounts-round-trip");
    let mut stories = Stories::of_person(0);
    let mut steps = vec![];
    stories.begin("welcome", &steps);
    steps.extend(welcome_steps());
    stories.begin("connect model", &steps);
    steps.extend([
        Step::Person(PersonStep::Go { address: b"/chats".as_slice().into() }),
        Step::Person(PersonStep::See {
            find: Find::named(Role::Heading { level: 1 }, b"Talk to an agent"),
            within: WITHIN,
        }),
        Step::Address(b"/chats".as_slice().into()),
        Step::Person(PersonStep::Go { address: Box::from(b"/accounts/connect".as_slice()) }),
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
    ]);
    let mut story =
        RoundTrip { script: Script::new(steps.into_boxed_slice()), before: 0, reading: false, reloaded: false };
    let outcome =
        Harness::new(tier.setup()).run_lending(&mut [&mut story], &mut [Lent { name: b"issuer", peer: &mut issuer }]);
    stories.assert_done(outcome);
    assert_eq!(
        issuer.observations().iter().filter(|event| matches!(event, Observation::Authorize { .. })).count(),
        2,
        "one authorization for each sign-in: a reload makes no second departure"
    );
    assert_eq!(
        issuer.observations().iter().filter(|event| matches!(event, Observation::Post { .. })).count(),
        2,
        "one code exchange for each sign-in"
    );
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
        Step::Person(PersonStep::Go { address: b"/welcome?at=model".as_slice().into() }),
        see(Find::exactly_named(Role::Button, b"Sign in to Welcome provider")),
        Step::Address(b"/welcome?at=model".as_slice().into()),
        Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Button, b"Sign in to Welcome provider") }),
        sign_in_tabs(1),
        see(Find::exactly_named(Role::Heading { level: 1 }, b"Almost ready")),
        Step::Address(b"/welcome?at=ready".as_slice().into()),
    ]
}
