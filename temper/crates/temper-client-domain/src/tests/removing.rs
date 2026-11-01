use super::{
    connect::{event, opened, reply, sent},
    fixtures as f,
};
use crate::{
    Event,
    forge::{
        self,
        removing::{self, RemoveAnswer},
    },
};
use jig_client::{Up, contract as c};
use skein_lib::{List, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};
const HERE: u16 = 3;
const OTHER: u16 = 4;
fn document(root: &mut crate::Domain, watch: Token, numbers: &[u16]) {
    let mut rows = List::with_capacity(2);
    for number in numbers {
        rows.push(
            w::ForgeRow::new(
                &w::CEILINGS,
                w::ForgeRowParts {
                    number: *number,
                    kind: w::Kind::Forgejo,
                    mode: w::Mode::Owned,
                    connection: w::Connection::NotSetUp,
                    offer: w::Offer::Offered,
                    first: None,
                    installations: List::with_capacity(0),
                    ended: None,
                    app: None,
                    web: None,
                    repositories: List::with_capacity(0),
                    owned: None,
                    admin: false,
                    read_only: false,
                    ended_stop: None,
                },
            )
            .unwrap(),
        )
        .unwrap();
    }
    event(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: Up::FamilyDelivered {
                    watch,
                    record: jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::forges(&w::Forges::new(&w::CEILINGS, w::ForgesParts { forges: rows }).unwrap()),
                    },
                },
            },
        },
    );
}
fn root() -> (crate::Domain, Token) {
    let (mut root, _, watch) = opened(true);
    document(&mut root, watch, &[HERE, OTHER]);
    (root, watch)
}
fn confirm(root: &mut crate::Domain) {
    f::form_act(root, forge::Act::AskRemove { forge: HERE });
}
fn remove(root: &mut crate::Domain) -> Token {
    confirm(root);
    sent(f::form_act(root, forge::Act::RemoveForge { forge: HERE })).0
}
#[test]
fn remove_asks_first_then_sends_one_deployment_request() {
    let (mut root, _) = root();
    assert!(f::form_act(&mut root, forge::Act::RemoveForge { forge: HERE }).is_empty());
    assert!(f::form_act(&mut root, forge::Act::AskRemove { forge: HERE }).is_empty());
    let removal = root.forge().removal().unwrap();
    assert!(removal.confirming);
    assert!(removal.request.is_none());
    let (_, _, w::Request::RemoveForge(request)) =
        sent(f::form_act(&mut root, forge::Act::RemoveForge { forge: HERE }))
    else {
        panic!("remove forge")
    };
    assert_eq!(request.forge(), HERE);
    assert!(!root.forge().removal().unwrap().confirming);
    assert!(f::form_act(&mut root, forge::Act::RemoveForge { forge: HERE }).is_empty());
    let jig_client::Ask::Family { scope, .. } = removing::ask(HERE, &f::limits().forge).unwrap() else {
        panic!("family")
    };
    assert_eq!(scope, c::Scope::Deployment);
    assert!(removing::ask(0, &f::limits().forge).is_none());
}
#[test]
fn keep_it_drops_the_confirmation_and_sends_nothing() {
    let (mut root, _) = root();
    confirm(&mut root);
    assert!(f::form_act(&mut root, forge::Act::KeepForge).is_empty());
    assert!(root.forge().removal().is_none());
    assert!(f::form_act(&mut root, forge::Act::RemoveForge { forge: HERE }).is_empty());
}
#[test]
fn removing_is_kept_until_the_row_leaves_the_document() {
    let (mut root, watch) = root();
    let attempt = remove(&mut root);
    reply(&mut root, attempt, w::Outcome::Removing);
    event(&mut root, Event::Went { place: crate::Place::Address(crate::Address::Accounts) });
    document(&mut root, watch, &[HERE, OTHER]);
    assert_eq!(root.forge().removal().unwrap().answer, Some(RemoveAnswer::Removing));
    document(&mut root, watch, &[OTHER]);
    assert!(root.forge().removal().is_none());
}
#[test]
fn no_such_forge_and_playground_refusals_are_kept_for_the_row() {
    for (outcome, expected) in [
        (
            w::Outcome::ConnectRefused(
                w::ConnectRefused::new(&w::CEILINGS, w::ConnectRefusedParts { why: w::ForgeRefusal::NoSuchForge })
                    .unwrap(),
            ),
            RemoveAnswer::NoSuchForge,
        ),
        (w::Outcome::Playground, RemoveAnswer::Playground),
        (w::Outcome::Unknown, RemoveAnswer::Unknown),
    ] {
        let (mut root, watch) = root();
        let attempt = remove(&mut root);
        reply(&mut root, attempt, outcome);
        document(&mut root, watch, &[HERE, OTHER]);
        assert_eq!(root.forge().removal().unwrap().answer, Some(expected));
        assert!(root.forge().removal().unwrap().progress.is_none());
        event(&mut root, Event::SignedOut);
        assert!(root.forge().removal().is_none());
    }
}
#[test]
fn one_removal_at_a_time() {
    let (mut root, watch) = root();
    let attempt = remove(&mut root);
    for act in [forge::Act::AskRemove { forge: OTHER }, forge::Act::RemoveForge { forge: OTHER }, forge::Act::KeepForge]
    {
        assert!(f::form_act(&mut root, act).is_empty());
        assert_eq!(root.forge().removal().unwrap().forge, HERE);
    }
    reply(&mut root, attempt, w::Outcome::Removing);
    f::form_act(&mut root, forge::Act::AskRemove { forge: OTHER });
    assert_eq!(root.forge().removal().unwrap().forge, HERE);
    document(&mut root, watch, &[OTHER]);
    f::form_act(&mut root, forge::Act::AskRemove { forge: HERE });
    assert!(root.forge().removal().is_none());
    f::form_act(&mut root, forge::Act::AskRemove { forge: OTHER });
    assert_eq!(root.forge().removal().unwrap().forge, OTHER);
}
#[test]
fn removing_parks_out_of_date_and_cannot_send_signed_out_or_stale() {
    let (mut root, _) = root();
    remove(&mut root);
    event(&mut root, Event::Forge(forge::Event::Served { served: forge::Served::OutOfDate }));
    assert_eq!(root.forge().removal().unwrap().progress, Some(skein_lib::RequestProgress::Parked));
    event(&mut root, Event::SignedOut);
    assert!(f::form_act(&mut root, forge::Act::RemoveForge { forge: HERE }).is_empty());
}

#[test]
fn a_row_leaving_before_its_answer_keeps_the_in_flight_removal() {
    let (mut root, watch) = root();
    let attempt = remove(&mut root);
    document(&mut root, watch, &[OTHER]);
    f::form_act(&mut root, forge::Act::AskRemove { forge: OTHER });
    assert_eq!(root.forge().removal().unwrap().forge, HERE);
    reply(&mut root, attempt, w::Outcome::Removing);
    assert!(root.forge().removal().is_none());
}

#[test]
fn removal_decodes_only_its_answers_for_this_family_and_version() {
    let limits = f::limits().forge;
    let mut record = jig_client::Family {
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        bytes: wire::answer(&w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome: w::Outcome::Removing }).unwrap()),
    };
    assert_eq!(removing::answer(&jig_client::Answered::Family(record.clone()), &limits), Some(RemoveAnswer::Removing));
    record.family = 2;
    assert_eq!(removing::answer(&jig_client::Answered::Family(record.clone()), &limits), None);
    record.family = wire::FORGE;
    record.version = record.version.checked_add(1).unwrap();
    assert_eq!(removing::answer(&jig_client::Answered::Family(record.clone()), &limits), None);
    record.version = wire::CLIENT_VERSION;
    for outcome in [
        w::Outcome::SettingUp,
        w::Outcome::Reset,
        w::Outcome::ConnectRefused(
            w::ConnectRefused::new(&w::CEILINGS, w::ConnectRefusedParts { why: w::ForgeRefusal::Busy }).unwrap(),
        ),
    ] {
        record.bytes = wire::answer(&w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).unwrap());
        assert_eq!(removing::answer(&jig_client::Answered::Family(record.clone()), &limits), None);
    }
    let refused = jig_client::Answered::Jig(c::Answer::Refused { refusal: c::Refusal::NotOffered, at: None });
    assert_eq!(
        removing::answer(&refused, &limits),
        Some(RemoveAnswer::EntranceRefused(c::EntranceRefusal::new(c::Refusal::NotOffered).unwrap()))
    );
}
