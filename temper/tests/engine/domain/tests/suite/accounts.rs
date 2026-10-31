//! The root forwards account envelopes and signing work; jig owns account policy.
use jig_domain_accounts as accounts;
use jig_domain_core as core;
use jig_domain_people as people;
use skein_lib::{ReplyTo, Token};
use temper_engine_domain::{self as engine, Delivery, PartyInput};
use temper_engine_domain_world::{direct::Driver, store::Store, walking};

fn driver() -> Driver {
    let config = std::array::from_fn(|_| {
        let mut config = walking::config(91);
        config.core.settings.llm_providers = accounts::Providers {
            providers: Box::new([accounts::Provider {
                number: 3,
                name: Box::from(b"Issuer".as_slice()),
                signed_in_plan: false,
                manage_usage: None,
                models: Box::new([]),
                offers: accounts::Offers { sign_in: true, key: true, programs: Box::new([]) },
            }]),
        };
        config
    });
    let mut driver = Driver::configured(Store::new(), config, &walking::limits());
    driver.settle();
    driver.sign_in();
    driver.settle();
    driver
}
#[test]
fn a_refused_key_envelope_returns_its_value_before_its_answer() {
    let mut driver = driver();
    let carried = core::Carried { number: 1_u64 << 63, digest: [7; 32] };
    let sign_in = driver.session();
    driver.delivered.clear();
    driver.send(engine::Event::Party(PartyInput::Key {
        reply_to: ReplyTo::new(Token::new(95)),
        sign_in,
        key: [95; 16],
        ask: Box::new(people::Ask::AccountChange {
            holder: people::AccountHolder::Own,
            change: people::AccountChange::AddKey { provider: 3, join: None, digest: [8; 32] },
        }),
        carried,
    }));
    driver.settle();
    assert!(matches!(&driver.delivered[..], [Delivery::CarriedUnused { number }, Delivery::WebReply { reply:
        people::Reply::Refused(people::Refusal::NotOffered), .. }] if *number == carried.number));
}
#[test]
fn signing_and_the_accounts_watch_cross_the_root_without_losing_their_facts() {
    let mut driver = driver();
    driver.send(engine::Event::Party(PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(95)),
        sign_in: driver.session(),
        key: [95; 16],
        ask: people::Ask::AccountSignIn { holder: people::AccountHolder::Own, provider: 3 },
    }));
    driver.settle();
    let source = driver
        .protocol
        .accounts
        .iter()
        .find_map(|request| {
            if let accounts::Request::Signing(accounts::Signing::SignIn { source, provider: 3 }) = request {
                Some(*source)
            } else {
                None
            }
        })
        .expect("Signing forwarded wholesale");
    driver.send(engine::Event::Account(accounts::Event::Visited {
        source,
        address: b"http://issuer/visit".as_slice().into(),
    }));
    driver.settle();
    assert!(driver.delivered.iter().any(|delivery| matches!(delivery, Delivery::WebReply { reply:
        people::Reply::Outcome(people::Outcome::AccountSignInStarted { source: got, address }), .. }
        if *got == source && &**address == b"http://issuer/visit")));
    driver.send(engine::Event::Party(PartyInput::Watch {
        watcher: Token::new(96),
        sign_in: driver.session(),
        subject: core::documents::Watched::Accounts,
        window: core::documents::Window { rows: 4, after: None },
    }));
    driver.settle();
    assert!(driver.delivered.iter().any(|delivery| {
        if let Delivery::Document { watcher, document } = delivery {
            *watcher == Token::new(96)
                && matches!(&**document,
                core::documents::Document::Accounts(value) if value.signing.len() == 1 && !value.settled)
        } else {
            false
        }
    }));
    assert!(
        driver
            .protocol
            .accounts
            .iter()
            .any(|request| matches!(request, accounts::Request::Signing(accounts::Signing::Restored)))
    );
}
