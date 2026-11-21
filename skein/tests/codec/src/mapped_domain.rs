//! A domain and owner-written translation for the maps fixture (data/codec/maps.md, 8).

use alloc::boxed::Box;
use skein_codec::{Reason, Record};
use skein_lib::{Duration, List, Reader, Wall};

use crate::{mapped_records as r, mapped_translation as maps};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub number: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Positive(pub u32);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Checked(pub u32);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HandChoice {
    None,
    Some,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Idle,
    Inline { total: Positive },
    Alternate { other: Positive },
    Item(Box<Item>),
    Label(Box<str>),
    Renamed,
    Checked(Checked),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Packet {
    pub changed: Wall,
    pub caption: Box<str>,
    pub maybe: Option<Wall>,
    pub times: Box<[Wall]>,
    pub references: List<u64>,
    pub labels: Box<[Box<str>]>,
    pub items: Box<[Item]>,
    pub primary: Box<Item>,
    pub events: List<Event>,
    pub checked: Checked,
    pub choice: HandChoice,
    pub enabled: bool,
    pub nonce: [u8; 2],
    pub age: Duration,
}

pub struct Context {
    pub permitted: u32,
}

pub fn positive_to_record(value: &Positive) -> Result<u32, Reason> {
    if value.0 == 0 { Err(Reason::Bound) } else { Ok(value.0) }
}

pub fn positive_to_domain(value: u32) -> Result<Positive, Reason> {
    if value == 0 { Err(Reason::Bound) } else { Ok(Positive(value)) }
}

pub fn checked_to_record(value: &Checked, context: &Context) -> Result<r::Checked, r::Problem> {
    if value.0 != context.permitted {
        return Err(r::paths::CHECKED_CODE.problem(Reason::Tag));
    }
    Ok(r::Checked { code: value.0 })
}

pub fn checked_to_domain(record: r::Checked, context: &Context) -> Result<Checked, r::Problem> {
    if record.code != context.permitted {
        return Err(r::paths::CHECKED_CODE.problem(Reason::Tag));
    }
    Ok(Checked(record.code))
}

pub fn hand_choice_to_record(value: &HandChoice, _context: &Context) -> Result<r::HandChoice, r::Problem> {
    Ok(match value {
        HandChoice::None => r::HandChoice::None,
        HandChoice::Some => r::HandChoice::Some,
    })
}

pub fn hand_choice_to_domain(record: r::HandChoice, _context: &Context) -> Result<HandChoice, r::Problem> {
    Ok(match record {
        r::HandChoice::None => HandChoice::None,
        r::HandChoice::Some => HandChoice::Some,
    })
}

/// The owner's root calls the generated pair, then the family's codec.
pub fn encode(value: &Packet, context: &Context) -> Result<Box<[u8]>, r::Problem> {
    let packet = maps::packet_to_record(value, context)?;
    r::Root { packet }.encode(&r::CEILINGS)
}

/// The owner's root decodes and delegates the mirrored part to the generator.
pub fn decode(bytes: &[u8], context: &Context) -> Result<Packet, r::Problem> {
    let record = r::Root::decode(&r::CEILINGS, &mut Reader::new(bytes))?;
    maps::packet_to_domain(record.packet, context)
}
