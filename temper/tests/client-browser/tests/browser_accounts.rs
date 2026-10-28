//! The browser owns accounts history, field re-entry, and the sign-in tab's round trip.
use skein_fake_person::{Find, Step as PersonStep};
use skein_fake_person_browser::{Harness, Outcome, Script, Step};
use skein_ui::Role;
use temper_client_browser_world::{Tier, WITHIN};
#[test]
fn the_accounts_page_returns_after_a_key_send() {
    let tier = Tier::accounts("accounts-history");
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let press = |name: &[u8], role| Step::Person(PersonStep::Press { find: Find::exactly_named(role, name) });
    let input = |name: &[u8], value: &[u8]| {
        Step::Person(PersonStep::Type { find: Find::exactly_named(Role::TextField, name), words: Box::from(value) })
    };
    let accounts = || Find::exactly_named(Role::Heading { level: 1 }, b"Connect a model");
    let chats = || Find::exactly_named(Role::Heading { level: 1 }, b"Chats");
    let mut script = Script::new(Box::new([
        see(Find::exactly_named(Role::Link, b"Set up temper")),
        Step::Person(PersonStep::Go { address: Box::from(b"/accounts/connect".as_slice()) }),
        see(accounts()),
        Step::Address(Box::from(b"/accounts/connect".as_slice())),
        see(Find::exactly_named(Role::Link, b"Use an API key")),
        press(b"Use an API key", Role::Link),
        input(b"Key (pasted once)", b"private-browser-key"),
        press(b"Add key", Role::Button),
        see(Find::text(b"Added your Test provider API key")),
        press(b"Chats", Role::Link),
        see(chats()),
        Step::Back,
        see(accounts()),
        // The browser kit refuses typing into a nonempty field: this proves re-entry after clearing.
        input(b"Key (pasted once)", b"another-private-key"),
        press(b"Chats", Role::Link),
        see(chats()),
        Step::Back,
        see(accounts()),
        Step::Address(Box::from(b"/accounts/connect?provider=17&way=key".as_slice())),
        Step::Forward,
        see(chats()),
        Step::Address(Box::from(b"/chats".as_slice())),
        Step::Back,
        see(accounts()),
        // The accounts watch retains the key added before leaving the flow.
        press(b"Accounts", Role::Link),
        see(Find::exactly_named(Role::Heading { level: 2 }, b"Your accounts")),
        see(Find::named(Role::Disclosure, b"Test provider API key")),
        Step::Person(PersonStep::Press { find: Find::named(Role::Disclosure, b"Test provider API key") }),
        see(Find::exactly_named(Role::Button, b"Replace")),
        see(Find::exactly_named(Role::Button, b"Remove account")),
    ]));
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut script]), Outcome::Done);
}

/// Read the source the returned tab names, then reload that same page.
struct RoundTrip {
    script: Script,
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
            let source = path
                .strip_prefix("accounts/connect?provider=17&way=sign-in&source=")
                .expect("returned ConnectModel source");
            assert!(!source.is_empty() && source.bytes().all(|byte| byte.is_ascii_digit()), "numeric source");
            self.script = Script::new(Box::new([
                Step::Person(PersonStep::See { find: Find::text(b"Signed in to Test provider"), within: WITHIN }),
                Step::Address(format!("/{path}").into_bytes().into_boxed_slice()),
            ]));
            self.reading = false;
            self.reloaded = true;
            face.history(skein_browser::boundary::Go::Reload);
            return Next::Wait { until: None };
        }
        let next = self.script.next(face, now);
        if matches!(next, Next::Done) && !self.reloaded {
            face.address();
            self.reading = true;
            Next::Wait { until: None }
        } else {
            next
        }
    }
    fn at(&self) -> u32 {
        skein_fake_person_browser::harness::Play::at(&self.script)
    }
}

#[test]
fn the_sign_in_leaves_the_page_and_comes_back_to_ready() {
    use skein_fake_peers::oauth::Observation;
    use skein_fake_person_browser::harness::Lent;
    let (tier, mut issuer) = Tier::accounts_sign_in("accounts-round-trip");
    let mut story = RoundTrip {
        script: Script::new(Box::new([
            Step::Person(PersonStep::See { find: Find::exactly_named(Role::Link, b"Set up temper"), within: WITHIN }),
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
            Step::Person(PersonStep::See { find: Find::text(b"Signed in to Test provider"), within: WITHIN }),
        ])),
        reading: false,
        reloaded: false,
    };
    let outcome =
        Harness::new(tier.setup()).run_lending(&mut [&mut story], &mut [Lent { name: b"issuer", peer: &mut issuer }]);
    assert_eq!(outcome, Outcome::Done);
    assert_eq!(
        issuer.observations().iter().filter(|event| matches!(event, Observation::Authorize { .. })).count(),
        1,
        "reload makes no second departure"
    );
    assert_eq!(
        issuer.observations().iter().filter(|event| matches!(event, Observation::Post { .. })).count(),
        1,
        "one code exchange"
    );
}
