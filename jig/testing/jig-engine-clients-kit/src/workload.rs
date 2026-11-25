//! Sign-ins and their exchanges (`jig/docs/design/domain/testing-workloads.md`, 6).
//! Work is drawn apart from each party's acts; passes advance only the world's clock.

#[cfg(test)]
mod tests;

use crate::{Peer, fixtures, open::RESENDS_A_PENDING_REQUEST};
use jig_client_protocol_wire::{self as w, Record as _};
use jig_domain_core as core;
use jig_domain_people as people;
use skein_lib::{
    Duration, Reader, ReplyTo, Token, Writer,
    base64::{self, Alphabet},
};
use skein_world::{
    faults::{FaultLedger, Struck},
    streams::{Stream, Streams},
    sweep::Sweep,
};

/// Each party's part is also its stable name in the draw.
#[derive(Debug, PartialEq, Eq)]
pub struct Party {
    pub name: String,
    pub delays: Vec<u64>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Workload {
    pub parties: Vec<Party>,
}

/// The first request is still pending at this pass, including at its answer pass.
#[derive(Debug, PartialEq, Eq)]
pub struct Acts {
    pub resend_at: u64,
    pub park_first: bool,
}

#[must_use]
pub fn draw(streams: &Streams) -> Workload {
    let mut rng = streams.rng(Stream::Workload, "sign-ins");
    let parties = (0..2 + rng.below(2))
        .map(|party| Party {
            name: format!("party-{party}"),
            delays: (0..2 + rng.below(3)).map(|_| 1 + rng.below(4)).collect(),
        })
        .collect();
    Workload { parties }
}

#[must_use]
pub fn acts(streams: &Streams, party: &Party) -> Acts {
    let mut rng = streams.rng(Stream::Parties, &party.name);
    Acts {
        resend_at: 1 + rng.below(*party.delays.first().expect("a party sends requests")),
        park_first: rng.below(4) == 0,
    }
}

/// Run each sign-in's requests against one peer. A live original and its
/// resend use both exchange slots; a parked original leaves its slot free.
pub fn run(streams: Streams, sweep: &Sweep) -> Result<FaultLedger, String> {
    let workload = draw(&streams);
    let mut peer = Peer::started(&fixtures::settings());
    peer.start_workload(streams.seed());
    for (index, party) in workload.parties.iter().enumerate() {
        let sign_in = u64::try_from(index).expect("small party count") + 1;
        sign_in_party(&mut peer, sign_in);
        let acts = acts(&streams, party);
        for (request, delay) in party.delays.iter().enumerate() {
            exchange(&mut peer, sweep, party, sign_in, request, *delay, &acts)?;
        }
    }
    assert!(peer.inspection().tables().is_empty(), "all sign-ins released their exchanges");
    Ok(peer.workload_ledger.take().expect("run's ledger"))
}

fn sign_in_party(peer: &mut Peer, sign_in: u64) {
    // Spending the local link replaces it, so each party signs in through its own exchange.
    let socket = peer.post_link(&peer.local_link());
    peer.answer_signed_in(sign_in, fixtures::FIRST_PERSON + sign_in);
    peer.answer_kept(sign_in);
    let answer = peer.sign_in_answer(socket);
    assert_eq!(answer.party, fixtures::FIRST_PERSON + sign_in);
    let mut bearer = Writer::new(61);
    bearer.put(b"Bearer ").expect("prefix");
    base64::encode(Alphabet::Url, answer.token.as_ref().expect("native token").expose(), &mut bearer).expect("bearer");
    peer.use_bearer(bearer.finish());
}

fn exchange(
    peer: &mut Peer,
    sweep: &Sweep,
    party: &Party,
    sign_in: u64,
    request: usize,
    delay: u64,
    acts: &Acts,
) -> Result<(), String> {
    let number = u64::try_from(request).expect("small request count") + 1;
    let mut key = [0; 16];
    key[..8].copy_from_slice(&sign_in.to_le_bytes());
    key[8..].copy_from_slice(&number.to_le_bytes());
    let words = format!("{} request {number}", party.name).into_bytes().into_boxed_slice();
    let body = keyed(key, words.clone());
    let first = peer.post_signed(b"/jig/v1/request", body.clone());
    let first_reply = take_request(peer, sign_in, key, &words);
    assert!(peer.answered(first).is_none(), "first waits for its answer");
    let mut resend = None;
    let mut parked = false;
    for pass in 1..=delay {
        peer.advance_time(Duration::from_millis(1));
        if request == 0 && pass == acts.resend_at {
            if acts.park_first {
                peer.cut_exchange(first);
                assert!(peer.inspection().tables().is_empty(), "a parked original uses no sign-in slot");
                parked = true;
            }
            let holding = if parked { ["the first is parked"].as_slice() } else { &[] };
            let at = peer.inspection_moment().at;
            if let Some(exclusion) = sweep.leaves_out(RESENDS_A_PENDING_REQUEST, holding) {
                peer.workload_ledger.as_mut().expect("run's ledger").leave_out(exclusion.scenario, exclusion.fault, at);
            } else {
                let entry = peer.workload_ledger.as_mut().expect("run's ledger").inject(
                    RESENDS_A_PENDING_REQUEST,
                    at,
                    Struck { request: Some(format!("{} request {number}", party.name)), ..Struck::default() },
                );
                let socket = peer.post_signed(b"/jig/v1/request", body.clone());
                let reply = take_request(peer, sign_in, key, &words);
                assert!(peer.answered(socket).is_none(), "resend waits for the first's answer");
                peer.workload_ledger.as_mut().expect("run's ledger").stop(entry, at)?;
                resend = Some((socket, reply));
            }
        }
        assert!(peer.answered(first).is_none(), "request stays pending until its drawn answer pass");
    }
    let answer = people::Reply::Outcome(people::Outcome::Said { task: 8, message: number });
    answer_once(peer, first_reply, answer.clone());
    if parked {
        assert!(peer.answered(first).is_none(), "a cut caller takes no answer");
    } else {
        check_answer(peer, first, number);
    }
    if let Some((socket, reply)) = resend {
        answer_once(peer, reply, answer);
        check_answer(peer, socket, number);
        if !parked {
            assert_eq!(peer.answered(socket), peer.answered(first), "resend receives the first's answer");
        }
    }
    assert!(peer.inspection().tables().is_empty(), "answered requests leave no pending exchanges");
    Ok(())
}

fn take_request(peer: &mut Peer, expected_sign_in: u64, expected_key: [u8; 16], words: &[u8]) -> ReplyTo {
    let Some(core::Event::People(people::Event::Ask { reply_to, sign_in, key, ask, .. })) = peer.take_core() else {
        panic!("signed request reaches the core");
    };
    assert_eq!(sign_in, expected_sign_in);
    assert_eq!(key, expected_key);
    assert_eq!(ask, people::Ask::Say { project: 3, task: 8, words: words.into() });
    assert!(peer.take_core().is_none(), "one core request per exchange");
    reply_to
}

fn answer_once(peer: &mut Peer, reply: ReplyTo, answer: people::Reply) {
    let token = reply.into_token();
    let waiter = format!("exchange {}", token.raw());
    peer.answer_request(ReplyTo::new(token), answer);
    assert!(
        !peer.inspection().waits().iter().any(|wait| wait.waiter == waiter),
        "answered exchange is no longer pending"
    );
}

fn check_answer(peer: &Peer, socket: Token, number: u64) {
    let bytes = peer.answered(socket).unwrap_or_else(|| {
        panic!(
            "request {number} on {socket:?} missed its drawn bound at {:?}: response {:?}, inspection {:?}",
            peer.inspection_moment(),
            peer.response(socket),
            peer.inspection()
        )
    });
    let w::KeyedAnswer::FinalAnswer(answer) =
        w::KeyedAnswer::decode(&fixtures::limits().wire, &mut Reader::new(bytes)).expect("wire answer")
    else {
        panic!("final answer");
    };
    assert_eq!(answer.owner, w::Owner::Jig);
    assert_eq!(answer.version, jig_client_protocol_wire::VERSION);
    assert_eq!(
        w::Final::decode(&fixtures::limits().wire, &mut Reader::new(&answer.record)).expect("owner answer"),
        w::Final::Done(w::Done {
            outcome: w::Outcome::Document(w::OutcomeDocument {
                value: w::DocumentOutcome::Said(w::OutcomeSaid { task: 8, message: number })
            })
        }),
    );
}

fn keyed(key: [u8; 16], words: Box<[u8]>) -> Box<[u8]> {
    let ask = w::Ask::Message(w::Message { project: 3, task: 8, words });
    let mut out = Writer::new(skein_lib::length::usize_of(ask.measure(&w::CEILINGS).expect("bounded ask")));
    ask.write(&w::CEILINGS, &mut out).expect("ask bytes");
    let record = w::Keyed { key, owner: w::Owner::Jig, version: w::VERSION, request: out.finish(), value: None };
    let mut out = Writer::new(skein_lib::length::usize_of(record.measure(&w::CEILINGS).expect("bounded request")));
    record.write(&w::CEILINGS, &mut out).expect("request bytes");
    out.finish()
}
