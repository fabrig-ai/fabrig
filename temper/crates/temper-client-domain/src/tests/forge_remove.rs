//! Removal's confirmation, request and watch lifecycle (temper/docs/design/draft/playground-reset.md, 4).
use super::{connect, fixtures as f};
use crate::{
    Domain, Event, Request,
    forge::{
        self,
        remove::{self, RemoveRefusal, State},
    },
};
use jig_client::{Down, Outcome, Told, Up, contract as c};
use skein_lib::{List, Queue, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};

fn act(root: &mut Domain, act: forge::Act) -> Queue<Request> {
    let mut out = Queue::uncapped();
    crate::step(root, &f::env(&f::limits()), Event::Forge(forge::Event::Act { act }), &mut out);
    out
}
fn deliver(root: &mut Domain, watch: Token, present: bool) {
    let mut rows = List::with_capacity(1);
    if present {
        rows.push(
            w::ForgeRow::new(
                &w::CEILINGS,
                w::ForgeRowParts {
                    skein_bools: [false; 4],
                    number: 3,
                    kind: w::Kind::Forgejo,
                    mode: w::Mode::Owned,
                    connection: w::Connection::Connecting,
                    offer: w::Offer::Offered,
                    first: None,
                    installations: List::with_capacity(0),
                    ended: None,
                    app: None,
                    web: None,
                    repositories: List::with_capacity(0),
                    owned: None,
                    ended_stop: None,
                },
            )
            .unwrap(),
        )
        .unwrap();
    }
    let record = jig_client::Family {
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        bytes: wire::forges(
            &w::Forges::new(&w::CEILINGS, w::ForgesParts { forges: rows, owned: w::Offer::NotOffered }).unwrap(),
        ),
    };
    crate::step(
        root,
        &f::env(&f::limits()),
        Event::Client { event: jig_client::Event::Up { up: Up::FamilyDelivered { watch, record } } },
        &mut Queue::uncapped(),
    );
    crate::take(root, &f::env(&f::limits()), &mut Queue::uncapped());
}
fn opened() -> (Domain, Token) {
    let (mut root, _, watch) = connect::opened(true);
    deliver(&mut root, watch, true);
    (root, watch)
}
fn removing(root: &mut Domain) -> Token {
    act(root, forge::Act::Remove { forge: 3 });
    act(root, forge::Act::ConfirmRemove);
    let Some(State::Removing { request, .. }) = root.forge().removal(3) else { panic!("removing") };
    *request
}
fn outcome(root: &mut Domain, outcome: Outcome) {
    let Some(State::Removing { request, .. }) = root.forge().removal(3) else { panic!("removing") };
    let key = jig_client::Key { asker: crate::owner::Owner::Forge.asker(), number: request.raw() };
    crate::route::told(root, &f::env(&f::limits()), Told::Answered { key, outcome }, &mut Queue::uncapped());
}
fn answered(root: &mut Domain, value: w::Outcome) {
    outcome(
        root,
        Outcome::Answered(jig_client::Answered::Family(jig_client::Family {
            family: wire::FORGE,
            version: wire::CLIENT_VERSION,
            bytes: wire::answer(&w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome: value }).unwrap()),
        })),
    );
}
fn refused(why: w::RemoveRefusal) -> w::Outcome {
    w::Outcome::RemoveRefused(w::RemoveRefused::new(&w::CEILINGS, w::RemoveRefusedParts { why }).unwrap())
}
#[test]
fn remove_asks_for_its_confirmation_and_sends_nothing() {
    let (mut root, _) = opened();
    assert!(act(&mut root, forge::Act::Remove { forge: 3 }).is_empty());
    assert_eq!(root.forge().removal(3), Some(&State::Confirming(3)));
    assert!(root.forge().removal(4).is_none());
}
#[test]
fn confirm_sends_one_remove_forge_at_the_deployments_scope() {
    let (mut root, _) = opened();
    assert!(act(&mut root, forge::Act::ConfirmRemove).is_empty());
    act(&mut root, forge::Act::Remove { forge: 3 });
    let out = act(&mut root, forge::Act::ConfirmRemove);
    let mut count = 0_u32;
    for request in &out {
        if let Request::Client {
            request:
                jig_client::Request::Down { down: Down::Send { request: jig_client::Ask::Family { scope, record }, .. } },
        } = request
        {
            count += 1;
            assert_eq!(*scope, c::Scope::Deployment);
            assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
            let w::Request::RemoveForge(value) =
                wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().into_parts().request
            else {
                panic!("remove forge")
            };
            assert_eq!(value.forge(), 3);
        }
    }
    assert_eq!(count, 1);
    assert!(act(&mut root, forge::Act::ConfirmRemove).is_empty());
    assert!(act(&mut root, forge::Act::Remove { forge: 4 }).is_empty());
    assert!(act(&mut root, forge::Act::CancelRemove).is_empty());
    assert!(match root.forge().removal(3) {
        Some(State::Removing { .. }) => true,
        Some(State::Confirming(_) | State::Refused { .. } | State::Removed(_)) | None => false,
    });
    assert!(root.forge().removal(4).is_none());
}
#[test]
fn removing_is_kept_until_the_row_goes() {
    let (mut root, watch) = opened();
    removing(&mut root);
    answered(&mut root, w::Outcome::Removing);
    assert_eq!(root.forge().removal(3), Some(&State::Removed(3)));
    deliver(&mut root, watch, true);
    assert_eq!(root.forge().removal(3), Some(&State::Removed(3)));
    assert!(act(&mut root, forge::Act::Remove { forge: 4 }).is_empty());
    assert_eq!(root.forge().removal(3), Some(&State::Removed(3)));
    deliver(&mut root, watch, false);
    assert!(root.forge().removal(3).is_none());
    act(&mut root, forge::Act::Remove { forge: 4 });
    assert_eq!(root.forge().removal(4), Some(&State::Confirming(4)));
}
#[test]
fn refused_brings_the_button_back_and_no_such_forge_reads_removed() {
    let (mut root, _) = opened();
    for (wire, why) in
        [(w::RemoveRefusal::NotOffered, RemoveRefusal::NotOffered), (w::RemoveRefusal::Role, RemoveRefusal::Role)]
    {
        removing(&mut root);
        answered(&mut root, refused(wire));
        assert_eq!(root.forge().removal(3), Some(&State::Refused { forge: 3, why }));
    }
    removing(&mut root);
    answered(&mut root, refused(w::RemoveRefusal::NoSuchForge));
    assert_eq!(root.forge().removal(3), Some(&State::Removed(3)));
}
#[test]
fn cancel_drops_the_confirmation() {
    let (mut root, _) = opened();
    act(&mut root, forge::Act::Remove { forge: 3 });
    assert!(act(&mut root, forge::Act::CancelRemove).is_empty());
    assert!(root.forge().removal(3).is_none());
    assert!(act(&mut root, forge::Act::ConfirmRemove).is_empty());
}
#[test]
fn unknown_lost_and_unrelated_answers_restore_the_control() {
    for value in [Outcome::Unknown, Outcome::Lost, Outcome::Unsent, Outcome::Unconfirmed] {
        let (mut root, _) = opened();
        removing(&mut root);
        outcome(&mut root, value);
        assert!(root.forge().removal(3).is_none());
    }
    let (mut root, _) = opened();
    removing(&mut root);
    answered(&mut root, w::Outcome::Running);
    assert!(root.forge().removal(3).is_none());
}
#[test]
fn the_shared_sender_guards_sign_in_version_and_a_second_request() {
    let limits = f::limits().forge;
    let mut side = forge::Side::new(&limits, f::SEED);
    let mut out = Queue::uncapped();
    remove::request(&mut side, 3, limits, &mut out);
    assert!(out.is_empty());
    let (mut root, _) = opened();
    let mut out = Queue::uncapped();
    remove::request(&mut root.forge, 3, limits, &mut out);
    assert_eq!(out.len(), 1);
    remove::request(&mut root.forge, 4, limits, &mut out);
    assert_eq!(out.len(), 1);
    let (mut root, _) = opened();
    crate::step(
        &mut root,
        &f::env(&f::limits()),
        Event::Forge(forge::Event::Served { served: forge::Served::OutOfDate }),
        &mut Queue::uncapped(),
    );
    let mut out = Queue::uncapped();
    remove::request(&mut root.forge, 3, limits, &mut out);
    assert!(out.is_empty());
}
