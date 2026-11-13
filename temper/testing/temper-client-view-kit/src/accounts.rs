use super::{self as fixtures, connect_model::text};
use jig_client::contract as c;
use skein_lib::{List, Token};
use temper_client_domain as d;

pub const PROVIDER: u32 = 17;

pub const SOURCE: u32 = 43;

pub fn card_document(provider: &[u8], display: &[u8], kind: c::AccountKind) -> c::Accounts {
    let mut doc = document(1);
    let row = doc.accounts.get_mut(0).unwrap();
    row.provider_name = Some(text(provider));
    row.display = text(display);
    row.plan = kind != c::AccountKind::Key;
    let source = row.sources.get_mut(0).unwrap();
    source.kind = kind;
    if kind == c::AccountKind::Program {
        source.name = Some(Box::from(display));
    }
    doc
}

pub fn email_facts(email: Option<&[u8]>) -> List<c::AccountFact> {
    match email {
        Some(email) => List::from_boxed(Box::new([c::AccountFact {
            kind: c::FACT_EMAIL,
            label: Box::new([]),
            value: c::AccountFactValue::Text(email.into()),
            seen: fixtures::NOW,
            source: Some(SOURCE),
            origin: 1,
        }])),
        None => List::with_capacity(0),
    }
}

pub fn source(number: u32) -> c::AccountSource {
    c::AccountSource {
        name: None,
        number,
        place: number,
        kind: c::AccountKind::Key,
        state: c::AccountState::Granted,
        usable: true,
        expiring: false,
        attention: false,
        spent_until: None,
    }
}

pub fn document(count: u32) -> c::Accounts {
    let mut accounts = List::with_capacity(count);
    let mut signing = List::with_capacity(count);
    for number in 1..=count {
        let mut sources = List::with_capacity(count);
        for n in 1..=count {
            sources.push(source(n)).unwrap();
        }
        accounts
            .push(c::AccountLine {
                list: None,
                facts: List::with_capacity(0),
                number,
                place: number,
                uses: None,
                plan: true,
                provider: Some(PROVIDER),
                provider_name: Some(c::DocumentText::complete(
                    c::Text::new(Box::from(b"Scripted".as_slice()), 32).unwrap(),
                )),
                manage_usage: None,
                display: c::DocumentText::complete(c::Text::new(Box::from(b"Pat".as_slice()), 32).unwrap()),
                holder: c::AccountHolder::Party(73),
                sources,
                cut: true,
            })
            .unwrap();
        signing
            .push(c::AccountSignIn { source: number, provider: PROVIDER, holder: c::AccountHolder::Party(73) })
            .unwrap();
    }
    c::Accounts {
        program_hosts: List::with_capacity(0),
        readiness: List::with_capacity(0),
        readiness_cut: false,
        accounts,
        signing,
        lendable: false,
        lending: None,
        lent_end: None,
        last_end: None,
        usable: true,
        settled: false,
        cut: true,
    }
}

pub fn deliver(root: &mut d::Domain, watch: Token, doc: c::Accounts) {
    fixtures::root_event(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Accounts(Box::new(doc))),
                },
            },
        },
    );
}

#[expect(
    clippy::disallowed_methods,
    reason = "the shared test fixture keeps its semantic inspection and setup unchanged"
)]
pub fn sign_in_address(address: &[u8]) -> d::Domain {
    let mut root = super::connect_model::offered_root();
    let limits = fixtures::limits();
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
    let mut out = skein_lib::Queue::uncapped();
    d::step(
        &mut root,
        &env,
        d::Event::Act {
            action: d::Action::Pages(d::pages::Act::ConnectModel(d::pages::connect_model::Act::SignIn {
                name: [97; 16],
                way: d::pages::accounts::Way::SignIn { provider: PROVIDER, plan: true },
            })),
        },
        &mut out,
    );
    fixtures::root_event(
        &mut root,
        d::Event::Went {
            place: d::Place::Address(d::Address::ConnectModel {
                way: Some(d::ModelWay { source: None, provider: PROVIDER, kind: d::WayKind::SignIn }),
            }),
        },
    );
    d::step(&mut root, &env, d::Event::Client { event: jig_client::Event::Stored }, &mut out);
    let attempt = out
        .iter()
        .find_map(|request| match request {
            d::Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, .. } },
            } => Some(*attempt),
            d::Request::Client { .. }
            | d::Request::Unlinked { .. }
            | d::Request::Show { .. }
            | d::Request::Leave { .. }
            | d::Request::SignOut
            | d::Request::Dropped { .. }
            | d::Request::Screen { .. }
            | d::Request::KeepLook { .. }
            | d::Request::KeepSignIn { .. }
            | d::Request::Tab { .. }
            | d::Request::Copy { .. }
            | d::Request::Push(_)
            | d::Request::Fact { .. } => None,
        })
        .expect("sign-in attempt");
    fixtures::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                        c::DocumentOutcome::AccountSignInStarted { source: SOURCE, address: Box::from(address) },
                    )))),
                    why: None,
                },
            },
        },
    );
    root
}
