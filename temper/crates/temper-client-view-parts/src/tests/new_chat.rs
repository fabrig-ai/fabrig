//! The new chat's session layout and start (temper/docs/design/client/pages/README.md).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "ordinary tests inspect semantic subtrees and terminal text"
)]
use super::fixtures as f;

use jig_client::contract as c;

use skein_lib::{Env, List, Queue, Time};

use temper_client_domain::{self as d, Address, pages};

const DRAFT: &[u8] = b"Make the change";

fn input(root: &mut d::Domain, act: pages::Act) {
    f::root_event(root, d::Event::Act { action: d::Action::Pages(act) });
}

fn start(root: &mut d::Domain) -> skein_lib::Token {
    input(root, pages::Act::ChatWords { words: DRAFT.into() });
    let env = Env { now: Time::ZERO, wall: f::NOW, limits: f::domain_limits() };
    let mut out = Queue::uncapped();
    for event in [
        d::Event::Act { action: d::Action::Pages(pages::Act::StartChat) },
        d::Event::Client { event: jig_client::Event::Stored },
    ] {
        d::step(root, &env, event, &mut out);
    }
    for request in &out {
        if let d::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, .. } },
        } = request
        {
            return *attempt;
        }
    }
    panic!("start sent");
}

fn answer(root: &mut d::Domain, attempt: skein_lib::Token, answer: c::Answer) -> Queue<d::Request> {
    let mut out = Queue::uncapped();
    d::step(
        root,
        &Env { now: Time::ZERO, wall: f::NOW, limits: f::domain_limits() },
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    why: None,
                    answer: Some(jig_client::Answered::Jig(answer)),
                },
            },
        },
        &mut out,
    );
    out
}

#[test]
fn made_chat_replaces_only_the_new_chat_address() {
    for from in [
        Address::NewChat { draft: false },
        Address::NewChat { draft: true },
        Address::Ask { project: f::PROJECT },
        Address::Chats { filter: d::ChatsFilter::All },
    ] {
        for document in [false, true] {
            let mut root = f::playground_root(from.clone(), f::PROJECT, true);
            let attempt = start(&mut root);
            let value = if document {
                c::Outcome::Document(c::DocumentOutcome::Created { first: 42, last: 42, given: None })
            } else {
                let mut tasks = List::with_capacity(1);
                tasks.push(42).unwrap();
                c::Outcome::Made { what: c::Made::Tasks { tasks } }
            };
            let out = answer(&mut root, attempt, c::Answer::Done(value));
            let shows: Vec<_> = out
                .iter()
                .filter_map(|request| {
                    if let d::Request::Show { address, replace } = request {
                        Some((address.clone(), *replace))
                    } else {
                        None
                    }
                })
                .collect();
            let replaces = from == (Address::NewChat { draft: false }) || from == (Address::NewChat { draft: true });
            assert_eq!(shows, [(Address::Chat { task: 42 }, replaces)]);
        }
    }
}
