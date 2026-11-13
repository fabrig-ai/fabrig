use super as fixtures;
use jig_client::contract as c;
use skein_lib::List;
use temper_client_domain::{self as d, pages::connect_model::Act as M};

pub const PROVIDER: u32 = 17;

pub fn terminal_root(welcome: bool, doc: c::Accounts) -> (d::Domain, skein_lib::Token) {
    let address = if welcome {
        d::Address::Welcome { step: d::Step::Model { way: None } }
    } else {
        d::Address::ConnectModel { way: None }
    };
    let (mut root, me_watch, _, _, watch) = fixtures::root_at_full(address, &fixtures::limits(), b"Shop");
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    me.providers = List::from_boxed(Box::new([
        offer(PROVIDER, b"openai", true, false),
        offer(PROVIDER + 1, b"anthropic", false, true),
    ]));
    fixtures::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me_watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                },
            },
        },
    );
    super::accounts::deliver(&mut root, watch.unwrap(), doc);
    (root, watch.unwrap())
}

pub fn terminal_document(lendable: bool) -> c::Accounts {
    let mut doc = super::accounts::document(0);
    doc.lendable = lendable;
    doc.usable = false;
    doc.settled = true;
    doc.cut = false;
    doc
}

pub fn offer(provider: u32, name: &[u8], sign_in: bool, program: bool) -> c::ProviderOffer {
    let mut programs = List::with_capacity(1);
    if program {
        programs.push(c::Text::new(Box::from(b"claude_code".as_slice()), 32).unwrap()).unwrap();
    }
    c::ProviderOffer {
        provider,
        name: text(name),
        key_bytes: 4096,
        sign_in,
        signed_in_plan: true,
        key: true,
        programs,
        programs_cut: false,
        none: false,
        manage_usage: None,
    }
}

pub fn text(value: &[u8]) -> c::DocumentText {
    c::DocumentText::complete(c::Text::new(Box::from(value), 64).unwrap())
}

pub fn ways_root(offers: List<c::ProviderOffer>, playground: bool) -> d::Domain {
    let limits = fixtures::limits();
    let (mut root, watch, _) = fixtures::root_at(d::Address::ConnectModel { way: None }, &limits);
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    me.providers = offers;
    me.providers_cut = true;
    me.deployment_role = Some(c::HeldRole::Owner);
    if playground {
        return fixtures::playground_frame_root(
            d::Address::ConnectModel { way: None },
            fixtures::PROJECT,
            true,
            None,
            Some(me),
        );
    }
    fixtures::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))) },
            },
        },
    );
    root
}

/// `openai`'s sign-in and `anthropic`'s program, each provider taking keys too.
pub fn plans_root() -> d::Domain {
    let mut providers = List::with_capacity(2);
    providers.push(offer(PROVIDER, b"openai", true, false)).unwrap();
    providers.push(offer(PROVIDER + 1, b"anthropic", false, true)).unwrap();
    ways_root(providers, false)
}

pub fn offered_root() -> d::Domain {
    let mut providers = List::with_capacity(1);
    providers.push(offer(PROVIDER, b"openai", true, false)).unwrap();
    ways_root(providers, false)
}

pub fn flow_fixture(kind: d::WayKind) -> (d::Domain, skein_lib::Token) {
    flow_fixture_at(
        kind,
        d::Address::ConnectModel { way: Some(d::ModelWay { source: None, provider: PROVIDER, kind }) },
    )
}

pub fn flow_fixture_at(kind: d::WayKind, address: d::Address) -> (d::Domain, skein_lib::Token) {
    let (mut root, me_watch, _, _, accounts_watch) = fixtures::root_at_full(address, &fixtures::limits(), b"Shop");
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    me.providers = List::with_capacity(1);
    me.providers
        .push(offer(PROVIDER, if kind == d::WayKind::Program { b"anthropic" } else { b"openai" }, true, true))
        .unwrap();
    fixtures::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me_watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                },
            },
        },
    );
    (root, accounts_watch.unwrap())
}

#[expect(
    clippy::disallowed_methods,
    reason = "the shared test fixture keeps its semantic inspection and setup unchanged"
)]
pub fn sent(root: &mut d::Domain, act: M) -> skein_lib::Token {
    let limits = fixtures::limits();
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: fixtures::NOW, limits };
    let mut out = skein_lib::Queue::uncapped();
    d::step(root, &env, d::Event::Act { action: d::Action::Pages(d::pages::Act::ConnectModel(act)) }, &mut out);
    d::step(root, &env, d::Event::Client { event: jig_client::Event::Stored }, &mut out);
    out.iter()
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
        .expect("flow sends one request")
}

pub fn answer(root: &mut d::Domain, attempt: skein_lib::Token, outcome: c::DocumentOutcome) {
    fixtures::root_event(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(outcome)))),
                    why: None,
                },
            },
        },
    );
}

pub fn ending(end: c::AccountEnd, display: &[u8]) -> d::Domain {
    let (mut root, watch) = flow_fixture(d::WayKind::SignIn);
    let attempt = sent(
        &mut root,
        M::SignIn { name: [1; 16], way: d::pages::accounts::Way::SignIn { provider: PROVIDER, plan: true } },
    );
    answer(
        &mut root,
        attempt,
        c::DocumentOutcome::AccountSignInStarted {
            source: 43,
            address: Box::from(b"https://issuer.example/visit".as_slice()),
        },
    );
    let mut doc = super::accounts::card_document(b"openai", display, c::AccountKind::SignIn);
    doc.usable = match &end {
        c::AccountEnd::Made { declined, .. } | c::AccountEnd::Renewed { declined, .. } => !declined,
        c::AccountEnd::Refused { .. } | c::AccountEnd::Failed { .. } => false,
    };
    doc.accounts.get_mut(0).unwrap().manage_usage = Some(Box::from(b"https://provider.example/usage".as_slice()));
    doc.signing.clear();
    doc.last_end = Some(c::AccountLastEnd { source: 43, end });
    super::accounts::deliver(&mut root, watch, doc);
    root
}

pub fn pasting() -> (d::Domain, skein_lib::Token) {
    let (mut root, watch) = flow_fixture(d::WayKind::SignIn);
    let way = d::pages::accounts::Way::SignIn { provider: PROVIDER, plan: true };
    let action = M::SignIn { name: [1; 16], way };
    let attempt = sent(&mut root, action);
    answer(
        &mut root,
        attempt,
        c::DocumentOutcome::AccountSignInStarted {
            source: 43,
            address: Box::from(b"https://issuer.example/visit".as_slice()),
        },
    );
    (root, watch)
}

pub const ENVIRONMENT: &[u8] = b"TEMPER_CONFIG='/config space' TEMPER_DATA='/data' TEMPER_CACHE='/cache' temper";

#[expect(
    clippy::disallowed_methods,
    reason = "the shared test fixture keeps its semantic inspection and setup unchanged"
)]
pub fn answer_environment(root: &mut d::Domain, words: &[u8]) {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let address = root.pages().address().clone();
    fixtures::root_event(root, d::Event::Went { place: d::Place::Address(d::Address::Accounts) });
    let limits = fixtures::limits();
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: fixtures::NOW, limits };
    let mut out = skein_lib::Queue::uncapped();
    d::step(root, &env, d::Event::Went { place: d::Place::Address(address) }, &mut out);
    let read = out
        .iter()
        .find_map(|request| {
            if let d::Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Read { read, what: jig_client::Readable::Family { record, .. } },
                    },
            } = request
                && wire::decode_reading(&record.bytes, &w::CEILINGS)
                    .is_some_and(|reading| *reading.read() == w::Read::ProgramEnvironment)
            {
                Some(*read)
            } else {
                None
            }
        })
        .expect("program page reads its environment");
    let answer =
        w::ProgramEnvironmentAnswer::new(&w::CEILINGS, w::ProgramEnvironmentAnswerParts { environment: words.into() })
            .unwrap();
    fixtures::root_event(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone {
                    read,
                    result: jig_client::Readout::Family {
                        family: wire::FORGE,
                        record: Some(wire::program_environment_answer(&answer)),
                    },
                },
            },
        },
    );
}

#[expect(
    clippy::disallowed_methods,
    reason = "the shared test fixture keeps its semantic inspection and setup unchanged"
)]
pub fn opened(out: &skein_lib::Queue<d::Request>, subject: &c::Subject) -> skein_lib::Token {
    out.iter()
        .find_map(|request| match request {
            d::Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(found) },
                    },
            } if found == subject => Some(*watch),
            d::Request::Screen { .. }
            | d::Request::KeepLook { .. }
            | d::Request::KeepSignIn { .. }
            | d::Request::Tab { .. }
            | d::Request::Client { .. }
            | d::Request::Unlinked { .. }
            | d::Request::Show { .. }
            | d::Request::Leave { .. }
            | d::Request::SignOut
            | d::Request::Dropped { .. }
            | d::Request::Copy { .. }
            | d::Request::Push(_)
            | d::Request::Fact { .. } => None,
        })
        .unwrap()
}

#[expect(
    clippy::disallowed_methods,
    reason = "the shared test fixture keeps its semantic inspection and setup unchanged"
)]
pub fn program_root_with_line(address: d::Address, line: Option<&[u8]>) -> (d::Domain, skein_lib::Token) {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let limits = fixtures::limits();
    let (offered, _) = flow_fixture(d::WayKind::Program);
    let me = d::pages::accounts::current_me(offered.client()).unwrap().clone();
    let accounts = super::accounts::document(0);
    let mut root = d::unlinked(&limits, fixtures::SEED);
    let mut out = skein_lib::Queue::uncapped();
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: fixtures::NOW, limits };
    for event in [
        d::Event::Client { event: jig_client::Event::StartUnlinked { kept: List::with_capacity(0) } },
        d::Event::Start { address, kept: List::with_capacity(0), with: Some(jig_client::With::Transport) },
        d::Event::Client {
            event: jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } },
        },
    ] {
        d::step(&mut root, &env, event, &mut out);
    }
    let me_watch = opened(&out, &c::Subject::Me);
    d::step(
        &mut root,
        &env,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me_watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                },
            },
        },
        &mut out,
    );
    let accounts_watch = opened(&out, &c::Subject::Accounts);
    fixtures::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: accounts_watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Accounts(Box::new(accounts))),
                },
            },
        },
    );
    let read = out
        .iter()
        .find_map(|request| match request {
            d::Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Read { read, what: jig_client::Readable::Family { record, .. } },
                    },
            } if wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read() == &w::Read::Deployment => {
                Some(*read)
            }
            d::Request::Screen { .. }
            | d::Request::KeepLook { .. }
            | d::Request::KeepSignIn { .. }
            | d::Request::Tab { .. }
            | d::Request::Client { .. }
            | d::Request::Unlinked { .. }
            | d::Request::Show { .. }
            | d::Request::Leave { .. }
            | d::Request::SignOut
            | d::Request::Dropped { .. }
            | d::Request::Copy { .. }
            | d::Request::Push(_)
            | d::Request::Fact { .. } => None,
        })
        .expect("live link reads deployment");
    let answer = w::DeploymentAnswer::new(
        &w::CEILINGS,
        w::DeploymentAnswerParts { push: None, welcome: true, owned: true, program_login: line.map(Into::into) },
    )
    .unwrap();
    fixtures::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone {
                    read,
                    result: jig_client::Readout::Family {
                        family: wire::FORGE,
                        record: Some(wire::deployment_answer(&answer)),
                    },
                },
            },
        },
    );
    (root, accounts_watch)
}
