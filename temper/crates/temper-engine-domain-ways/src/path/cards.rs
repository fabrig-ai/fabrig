//! A stop's words and card (temper/docs/design/domain/forge.md, 8.13).
use super::{ASKED, Actor, Card, Fact, Inputs, LABEL, NAME, Opens, Step, Stop, VALUE, WHO, WORDS};
use crate::cut_on_word;
use alloc::boxed::Box;
use skein_lib::{List, bytes};

pub(super) fn text(value: &[u8], limit: usize) -> Box<[u8]> {
    if !bytes::is_utf8(value) {
        return Box::default();
    }
    bytes::utf8_prefix(value, limit).into()
}

pub(super) fn number(value: u32) -> Box<[u8]> {
    let mut digits = List::uncapped();
    let mut started = false;
    for place in [1_000_000_000, 100_000_000, 10_000_000, 1_000_000, 100_000, 10_000, 1_000, 100, 10, 1] {
        let digit = value.checked_div(place).expect("nonzero decimal place").checked_rem(10).expect("ten is nonzero");
        if digit != 0 || started || place == 1 {
            started = true;
            let digit = u8::try_from(digit).expect("decimal digit");
            digits.push(b'0'.checked_add(digit).expect("ASCII digit")).expect("ten digits");
        }
    }
    digits.into_boxed()
}

pub(super) fn fact(label: &[u8], value: &[u8]) -> Fact {
    Fact { label: text(label, LABEL), value: cut_on_word(value, VALUE) }
}

pub(super) fn card(who: &[u8], name: &[u8], facts: Box<[Fact]>) -> Card {
    Card { who: text(who, WHO), name: text(name, NAME), facts }
}

pub(super) fn stop(actor: Actor, step: Step, words: &[u8], opens: Opens, card: Card) -> Stop {
    Stop { actor, step, words: text(words, WORDS), opens, card, needed: false, over: Box::default() }
}

pub(super) fn agent_card(inputs: &Inputs, charter: u32, who: &[u8], name: &[u8]) -> Card {
    for agent in &inputs.agents {
        if agent.charter != charter {
            continue;
        }
        return role_card(agent, who, name);
    }
    let value = bytes::concat(&[b"charter ", &number(charter)]);
    card(who, name, Box::new([fact(b"Charter", &value)]))
}

/// An agent’s role facts (temper/docs/design/domain/forge.md, 8.13).
#[must_use]
pub fn role_card(agent: &super::Agent, who: &[u8], name: &[u8]) -> Card {
    let mut model = List::uncapped();
    if let Some(value) = &agent.model {
        append(&mut model, value);
    }
    if let Some(value) = &agent.effort {
        if !model.is_empty() {
            append(&mut model, " · ".as_bytes());
        }
        append(&mut model, value);
    }
    if agent.model.is_none() && agent.effort.is_none() {
        append(&mut model, b"no model");
    }
    if !agent.usable {
        append(&mut model, " · no account runs it now".as_bytes());
    }
    let asked = bytes::concat(&[
        "“".as_bytes(),
        &cut_on_word(&agent.asked, ASKED.checked_sub("“”".len()).expect("quote room")),
        "”".as_bytes(),
    ]);
    card(
        who,
        name,
        Box::new([
            fact(b"Model", model.as_slice()),
            fact(b"Can", &agent.can),
            Fact { label: text(b"Asked to", LABEL), value: asked },
        ]),
    )
}

pub(super) fn append(out: &mut List<u8>, value: &[u8]) {
    let room = VALUE.saturating_sub(out.as_slice().len());
    let value = cut_on_word(value, room);
    for &byte in &value {
        out.push(byte).expect("card value bound");
    }
}
