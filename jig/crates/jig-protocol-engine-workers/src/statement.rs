//! The statement handed up whole and its checks (jig/docs/design/protocol/channel/wires.md, 2 and 3).
use crate::generated::worker_wire as mapped;
use alloc::boxed::Box;
use jig_protocol_worker_wire::{
    self as wire, Credential, Root, Spoken,
    layouts::{Gap, Peer, Plans, Theirs},
};
use skein_channel::{Kind, Role, Term};
use skein_lib::{Decimal, List, bytes};

/// A wire as the worker stated it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stated {
    pub name: Box<str>,
    pub layout: [u8; 32],
    pub text: Box<[u8]>,
}

/// The credential's wires and output, the worker's terms and the hello's job kinds, in their order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Statement {
    pub wires: List<Stated>,
    pub output: u32,
    pub terms: List<Term>,
    pub jobs: List<jig_domain_link::JobBound>,
}

/// The opening's reason and text, after authentication.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unfit {
    /// A wire did not fit.
    Wires(Box<[u8]>),
    /// The output room or a term did not fit.
    Limits(Box<[u8]>),
}

/// Judge a kept opening as the engine would, after authentication.
pub fn judge(
    spoken: &List<Spoken>,
    roots: &List<Root>,
    room: u32,
    limits: &wire::v22::Limits,
    statement: &Statement,
) -> Result<List<Box<str>>, Unfit> {
    let mut plans = match plans(spoken, roots) {
        Ok(plans) => plans,
        Err(text) => return Err(Unfit::Wires(text)),
    };
    let fitted = match wires(&mut plans, spoken, limits, &statement.wires) {
        Ok(fitted) => fitted,
        Err(text) => return Err(Unfit::Wires(text)),
    };
    if let Err(text) = output(room, statement.output) {
        return Err(Unfit::Limits(text));
    }
    if let Err(text) = terms(&fitted.table, &statement.terms) {
        return Err(Unfit::Limits(text));
    }
    Ok(fitted.converted)
}

/// Lists what a worker's layouts cannot keep after the opening's wire checks.
pub fn gaps(
    spoken: &List<Spoken>,
    roots: &List<Root>,
    limits: &wire::v22::Limits,
    stated: &List<Stated>,
) -> Result<List<Gap>, Unfit> {
    let mut plans = match plans(spoken, roots) {
        Ok(plans) => plans,
        Err(text) => return Err(Unfit::Wires(text)),
    };
    match wires(&mut plans, spoken, limits, stated) {
        Ok(fitted) => Ok(fitted.peer.gaps()),
        Err(text) => Err(Unfit::Wires(text)),
    }
}

pub(crate) fn plans(spoken: &List<Spoken>, roots: &List<Root>) -> Result<Plans, Box<[u8]>> {
    let mut texts = List::with_capacity(spoken.len());
    for wire in spoken {
        texts.push(wire.text).expect("one text per wire");
    }
    Plans::new(texts.as_slice(), roots)
}

pub(crate) struct Fitted {
    pub peer: Peer,
    pub table: List<Kind>,
    pub converted: List<Box<str>>,
}

pub(crate) fn wires(
    plans: &mut Plans,
    spoken: &List<Spoken>,
    limits: &wire::v22::Limits,
    stated: &List<Stated>,
) -> Result<Fitted, Box<[u8]>> {
    let mut theirs = List::with_capacity(stated.len());
    for wire in stated {
        theirs.push(Theirs { name: &wire.name, layout: wire.layout, text: &wire.text }).expect("one slot per wire");
    }
    let peer = plans.judge(theirs.as_slice())?;
    let table = plans.fit_table(&peer, limits)?;
    let mut converted = List::uncapped();
    for ours in spoken {
        for theirs in stated {
            if theirs.name.as_ref() == ours.name && theirs.layout != ours.layout {
                converted.push(Box::from(ours.name)).expect("uncapped wires");
            }
        }
    }
    Ok(Fitted { peer, table, converted })
}

impl Statement {
    pub(crate) fn credential(credential: Credential) -> Self {
        let mut wires = List::with_capacity(credential.wires.len());
        for wire in credential.wires.into_boxed() {
            let stated = mapped::wire_to_domain(wire).expect("credential text is UTF-8");
            wires.push(stated).expect("one slot per stated wire");
        }
        Self { wires, output: credential.output, terms: List::with_capacity(0), jobs: List::with_capacity(0) }
    }
}

pub(crate) fn output(room: u32, stated: u32) -> Result<(), Box<[u8]>> {
    if stated == room { Ok(()) } else { Err(crate::opening::output_room_words(room, stated)) }
}

pub(crate) fn terms(table: &List<Kind>, terms: &List<Term>) -> Result<(), Box<[u8]>> {
    match wire::terms_gap(table, Role::Responder, terms) {
        Some(kind) => Err(kind_words(kind)),
        None => Ok(()),
    }
}

pub(crate) fn kind_words(kind: u16) -> Box<[u8]> {
    bytes::copy_of(Decimal::of(u64::from(kind)).as_bytes())
}
