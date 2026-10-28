//! M5 QA-B: use the existing browser world; do not add a world to force a move.
use jig_client_protocol_wire::v19 as w;
use skein_fake_person::{Find, Step as PersonStep};
use skein_fake_person_browser::{Harness, Outcome, Script, Step, harness::Lent};
use skein_lib::Writer;
use skein_ui::Role;
use temper_client_browser_world::{Tier, WITHIN};

fn request(written: bool) -> Box<[u8]> {
    let change = if written {
        w::AccountChange::Written(
            w::AccountChangeWritten::new(&w::CEILINGS, w::AccountChangeWrittenParts { number: 1 }).expect("first move"),
        )
    } else {
        w::AccountChange::MoveOut(
            w::AccountChangeMoveOut::new(&w::CEILINGS, w::AccountChangeMoveOutParts { account: 1 })
                .expect("first account"),
        )
    };
    let ask = w::Ask::AccountChange(
        w::AskAccountChange::new(&w::CEILINGS, w::AskAccountChangeParts { holder: w::AccountHolder::Own, change })
            .expect("move request"),
    );
    let mut out = Writer::new(usize::try_from(ask.measure()).expect("ask fits"));
    ask.encode(&mut out).expect("measured ask");
    let keyed = w::Keyed::new(
        &w::CEILINGS,
        w::KeyedParts {
            key: [if written { 0x92 } else { 0x91 }; 16],
            owner: w::Owner::Jig,
            version: jig_client_protocol_wire::VERSION,
            request: out.finish(),
            value: None,
        },
    )
    .expect("keyed request");
    let mut out = Writer::new(usize::try_from(keyed.measure()).expect("keyed fits"));
    keyed.encode(&mut out).expect("measured request");
    out.finish()
}
// jig domain/accounts.md 6 requires Moving followed by removal. The existing
// browser world reaches a granted source, but Step::Request is a raw HTTP peer:
// it has no bearer or the tab's HttpOnly cookie. Its 200 is not proof of a move.
// The page stays Ready at step 8. There is no split-move driver in this world;
// reaching a moving source here needs one (backlog).
#[test]
#[ignore = "backlog: the browser world's raw requests carry no tab sign-in cookie, and it has no split-move driver to hold a source moving"]
fn the_open_web_accounts_page_sees_moving_then_gone() {
    let (tier, mut issuer) = Tier::accounts_sign_in("qa-b-account-moving");
    let setup = tier.setup();
    let config = setup.args.windows(2).find(|pair| pair[0] == "--config").expect("tier config");
    let value: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::path::Path::new(&config[1]).join("temper.json")).expect("tier configuration"),
    )
    .expect("JSON");
    let host = value["application"]["clients"]["listener"]["socket"].as_str().expect("reserved listener").as_bytes();
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let go = |address: &[u8]| Step::Person(PersonStep::Go { address: address.into() });
    let moving = || Find::text(b"Moving to another deployment");
    let card = || Find::named(Role::Disclosure, b"Test provider");
    let send = |written| Step::Request {
        method: skein_http::Method::Post,
        path: b"/jig/v1/request".as_slice().into(),
        host: host.into(),
        headers: vec![(b"Content-Type".as_slice().into(), b"application/octet-stream".as_slice().into())],
        body: request(written),
        status: 200,
        within: WITHIN,
    };
    let mut script = Script::new(Box::new([
        see(Find::exactly_named(Role::Link, b"Set up temper")),
        go(b"/accounts/connect"),
        see(Find::exactly_named(Role::Button, b"Sign in to Test provider")),
        Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Button, b"Sign in to Test provider") }),
        see(Find::text(b"Signed in to Test provider")),
        go(b"/accounts"),
        see(card()),
        send(false),
        see(moving()),
        send(true),
        Step::Person(PersonStep::Gone { find: moving(), within: WITHIN }),
        Step::Person(PersonStep::Gone { find: card(), within: WITHIN }),
    ]));
    assert_eq!(
        Harness::new(setup).run_lending(&mut [&mut script], &mut [Lent { name: b"issuer", peer: &mut issuer }]),
        Outcome::Done
    );
}
