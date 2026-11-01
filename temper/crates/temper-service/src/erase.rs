//! Durable erasure of a forge's secrets (temper/docs/design/draft/playground-reset.md, 4).
use crate::engine::Engine;
use alloc::boxed::Box;
use jig_protocol_store::{SecretOwner, Secrets};
use skein_lib::{Map, Token};

pub(crate) struct Layer {
    pending: Map<Token, u16>,
    forges: Map<u16, Flight>,
}
#[derive(Debug)]
struct Flight {
    remaining: u32,
    answers: u32,
}
impl Layer {
    pub(crate) fn new(forges: u32) -> Self {
        Self {
            pending: Map::with_capacity(forges.checked_mul(10).expect("forge erase owners")),
            forges: Map::with_capacity(forges),
        }
    }
}
pub(crate) fn erase(engine: &mut Engine, forge: u16) {
    if let Some(flight) = engine.erasing.forges.get_mut(&forge) {
        flight.answers = flight.answers.checked_add(1).expect("erase callers");
        return;
    }
    engine.erasing.forges.insert(forge, Flight { remaining: 10, answers: 1 }).expect("admitted forges");
    for item in temper_protocol_records::ITEMS {
        let owner = engine.setup.next_run();
        engine.erasing.pending.insert(owner, forge).expect("ten erase owners per forge");
        let answer = engine.store.secrets(
            owner,
            Secrets::Erase { owner: SecretOwner::Application, number: temper_protocol_records::number(forge, item) },
        );
        if let Some(answer) = answer {
            crate::setup::answer(engine, answer);
        }
        if engine.end.is_some() {
            return;
        }
    }
}
/// Returns whether this durable terminal belongs to a forge removal.
pub(crate) fn answered(engine: &mut Engine, owner: Token) -> bool {
    let Some(forge) = engine.erasing.pending.remove(&owner) else { return false };
    let flight = engine.erasing.forges.get_mut(&forge).expect("erase flight");
    flight.remaining = flight.remaining.checked_sub(1).expect("one terminal per erase");
    if flight.remaining == 0 {
        let flight = engine.erasing.forges.remove(&forge).expect("completed flight");
        if engine.end.is_none() && engine.drain.is_none() {
            for _ in 0..flight.answers {
                engine.events.push(temper_engine_domain::Event::Forge(Box::new(
                    temper_engine_domain_forge::Event::SecretsErased { forge },
                )));
            }
        }
    }
    true
}
