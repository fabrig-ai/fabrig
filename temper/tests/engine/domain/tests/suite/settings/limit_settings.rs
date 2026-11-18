//! Only settings revisions naming limits send the service a durable snapshot.
use jig_domain_core::settings as s;
use jig_domain_people as people;
use skein_lib::{ReplyTo, Token};
use temper_engine_domain::{self as root, Event, PartyInput, Request};
use temper_engine_domain_forge::settings as f;
use temper_engine_domain_world::{direct::Driver, kinds as k};

fn overrides(name: &[u8], value: u64) -> Box<[s::Override]> {
    Box::new([s::Override { name: name.into(), value, reason: b"room for work".as_slice().into() }])
}
fn jig() -> s::Item {
    s::Item { setting: s::Setting::Limits, edit: s::Edit::Set(s::Value::Limits(overrides(b"tasks.tasks", 1024))) }
}
fn temper() -> s::Item {
    k::item(21, 0, 1, &f::Value::Limits(overrides(b"forge.entries", 16)))
}
fn word(driver: &mut Driver, revision: u32, jig: &[s::Override], temper: &[s::Override]) {
    let words: Vec<_> = driver
        .immediate
        .drain(..)
        .filter_map(|request| {
            if let Request::LimitSettingsCommitted { revision, jig, temper } = request {
                Some((revision, jig, temper))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        words,
        vec![(revision, Box::from(jig), Box::from(temper))],
        "one whole snapshot at the revision's barrier"
    );
}
fn no_word(driver: &Driver) {
    assert!(!driver.immediate.iter().any(|request| matches!(request, Request::LimitSettingsCommitted { .. })));
}

#[test]
fn limit_setting_words_wait_for_durability_and_snapshot_both_kinds_once() {
    let mut driver = k::driver();
    no_word(&driver);
    driver.send(k::change(driver.session(), 1, 71, Box::new([jig()])));
    for _ in 0..16 {
        driver.advance(false);
    }
    assert!(!driver.store.pending.is_empty(), "the settings write is held by the store");
    no_word(&driver);
    driver.settle();
    word(&mut driver, 2, &overrides(b"tasks.tasks", 1024), &[]);
    driver.send(k::change(driver.session(), 2, 72, Box::new([temper()])));
    driver.settle();
    word(&mut driver, 3, &overrides(b"tasks.tasks", 1024), &overrides(b"forge.entries", 16));
    // A proposal has no new revision; its acceptance, not its creation, sends the word.
    driver.send(k::proposal(
        driver.session(),
        3,
        73,
        Box::new([s::Item { setting: s::Setting::Limits, edit: s::Edit::Reset }]),
    ));
    driver.settle();
    no_word(&driver);
    let proposal = driver
        .delivered
        .iter()
        .rev()
        .find_map(|delivery| {
            if let root::Delivery::WebReply {
                reply: people::Reply::Outcome(people::Outcome::Proposed { proposal, .. }),
                ..
            } = delivery
            {
                Some(*proposal)
            } else {
                None
            }
        })
        .expect("owner's proposal");
    driver.send(Event::Party(PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(74)),
        sign_in: driver.session(),
        key: [74; 16],
        models: Box::new([]),
        ask: people::Ask::DecideProposal {
            project: 0,
            proposer: people::Proposer::Party(1),
            proposal,
            decision: people::ProposalDecision::Accept,
            running: Some(people::Running::Stays),
        },
    }));
    driver.settle();
    word(&mut driver, 4, &[], &overrides(b"forge.entries", 16));
}

#[test]
fn a_revision_resetting_both_kinds_sends_one_empty_word_and_unrelated_revisions_send_none() {
    let mut driver = k::driver();
    driver.send(k::change(driver.session(), 1, 71, Box::new([jig(), temper()])));
    driver.settle();
    word(&mut driver, 2, &overrides(b"tasks.tasks", 1024), &overrides(b"forge.entries", 16));
    driver.send(k::change(
        driver.session(),
        2,
        72,
        Box::new([
            s::Item { setting: s::Setting::Limits, edit: s::Edit::Reset },
            s::Item { setting: s::Setting::Application { kind: 21, scope: 0, number: 1 }, edit: s::Edit::Reset },
        ]),
    ));
    driver.settle();
    assert_eq!(driver.root.settings_revision(), 3);
    word(&mut driver, 3, &[], &[]);
    driver.send(k::change(driver.session(), 3, 73, Box::new([super::rules(2)])));
    driver.settle();
    assert_eq!(driver.root.settings_revision(), 4);
    no_word(&driver);
}
