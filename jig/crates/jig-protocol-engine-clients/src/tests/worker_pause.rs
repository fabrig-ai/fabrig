//! The independent client pause vocabulary on both ends of its unbridged wire.
use super::fixtures as f;
use crate::translate;
use jig_client_domain::contract as c;
use jig_client_protocol_link::translate as client;
use jig_client_protocol_wire::{self as w, Record as _};
use jig_domain_link::{PauseMode, PauseState};
use jig_domain_people as p;
use skein_lib::{List, Reader};

#[test]
fn worker_pause_contract_crosses_the_wire_both_ways() {
    for (mode, expected) in [
        (c::WorkerPauseMode::Resume, PauseMode::Resume),
        (c::WorkerPauseMode::Pause, PauseMode::Pause),
        (c::WorkerPauseMode::Drain, PauseMode::Drain),
    ] {
        let ask = client::asks::ask(&c::Ask::PauseWorker { worker: 17, mode }, &w::CEILINGS).unwrap();
        let bytes = ask.encode(&w::CEILINGS).unwrap();
        let ask = w::Ask::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap();
        assert_eq!(
            translate::asks::ask(ask, &[], &w::CEILINGS),
            Ok(p::Ask::PauseWorker { worker: 17, mode: expected })
        );
    }
    for (reply, expected) in [
        (
            p::Reply::Outcome(p::Outcome::WorkerPauseAsked { worker: 17 }),
            c::Answer::Done(c::Outcome::WorkerPauseAsked { worker: 17 }),
        ),
        (
            p::Reply::Refused(p::Refusal::Unlinked { worker: 17 }),
            c::Answer::Refused { refusal: c::Refusal::Unlinked { worker: 17 }, at: None },
        ),
        (
            p::Reply::Refused(p::Refusal::Role { held: Some(p::Role::Member) }),
            c::Answer::Refused { refusal: c::Refusal::Role { held: Some(c::HeldRole::Member) }, at: None },
        ),
    ] {
        let wire = translate::answers::answer(&reply, &[], &w::CEILINGS).unwrap();
        assert_eq!(client::answers::answer(wire, &f::client_limits()).unwrap(), expected);
    }
    for (pause, expected) in [
        (PauseState::Taking, c::WorkerPause::Taking),
        (PauseState::Paused, c::WorkerPause::Paused),
        (PauseState::Draining, c::WorkerPause::Draining),
        (PauseState::Drained, c::WorkerPause::Drained),
    ] {
        let mut document = f::deployment_document();
        document.worker_settings = List::from_boxed(Box::new([jig_domain_core_vocabulary::documents::WorkerState {
            number: 17,
            name: b"worker".as_slice().into(),
            places: 1,
            digest: None,
            withdrawn: false,
            linked: true,
            since: None,
            pause,
        }]));
        let wire = translate::deployment::document(&document, &w::CEILINGS).unwrap();
        let bytes = wire.encode(&w::CEILINGS).unwrap();
        let wire = w::Deployment::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap();
        let document = client::documents::deployment(wire, &f::client_limits()).unwrap();
        assert_eq!(document.worker_settings.get(0).unwrap().pause, expected);
    }
}
