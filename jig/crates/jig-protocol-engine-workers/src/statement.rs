//! The statement handed up whole and its checks (protocol/channel/wires.md, 3 and 5).
use alloc::boxed::Box;
use jig_protocol_worker_wire::{self as wire, Agreed, Credential, Spoken};
use skein_channel::{Role, Schema, Term};
use skein_lib::{Decimal, List, bytes};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stated {
    pub name: Box<str>,
    pub lowest: u16,
    pub layouts: List<[u8; 32]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Statement {
    pub wires: List<Stated>,
    pub output: u32,
    pub terms: List<Term>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unfit {
    Wires(Box<[u8]>),
    Limits(Box<[u8]>),
}

/// Judge a kept opening as the engine would, after authentication.
pub fn judge(spoken: &List<Spoken>, room: u32, schema: &Schema, statement: &Statement) -> Result<List<Agreed>, Unfit> {
    let agreed = match crate::wires::agree(spoken, &statement.wires) {
        Ok(agreed) => agreed,
        Err(text) => return Err(Unfit::Wires(text)),
    };
    if let Err(text) = output(room, statement.output) {
        return Err(Unfit::Limits(text));
    }
    for wire in &agreed {
        if wire.name == wire::wire::FAMILY
            && let Err(text) = terms(schema, wire.version, &statement.terms)
        {
            return Err(Unfit::Limits(text));
        }
    }
    Ok(agreed)
}

impl Statement {
    #[expect(clippy::disallowed_methods, reason = "the credential codec validates wire names as UTF-8")]
    pub(crate) fn credential(credential: &Credential) -> Self {
        let mut wires = List::with_capacity(credential.wires().len());
        for wire in credential.wires() {
            wires
                .push(Stated {
                    name: core::str::from_utf8(wire.name()).expect("credential text is UTF-8").into(),
                    lowest: wire.lowest(),
                    layouts: wire.layouts().clone(),
                })
                .expect("one slot per stated wire");
        }
        Self { wires, output: credential.output(), terms: List::with_capacity(0) }
    }
}

pub(crate) fn output(room: u32, stated: u32) -> Result<(), Box<[u8]>> {
    if stated == room { Ok(()) } else { Err(crate::opening::output_room_words(room, stated)) }
}

pub(crate) fn terms(schema: &Schema, version: u16, terms: &List<Term>) -> Result<(), Box<[u8]>> {
    match wire::terms_gap(schema, version, Role::Responder, terms) {
        Some(kind) => Err(bytes::copy_of(Decimal::of(u64::from(kind)).as_bytes())),
        None => Ok(()),
    }
}
