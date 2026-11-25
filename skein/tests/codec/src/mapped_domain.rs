//! A domain and owner-written translation for the maps fixture (data/codec/maps.md, 8).

use alloc::boxed::Box;
use skein_codec::{Reason, Record};
use skein_lib::{Duration, List, Reader, Wall};

use crate::{mapped_records as r, mapped_translation as maps};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub number: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Empty {}

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
    UnboxedItem(Item),
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
    pub counts: List<u32>,
    pub padding: Box<[Empty]>,
    pub primary: Box<Item>,
    pub optional_items: Option<Box<[Item]>>,
    pub optional_primary: Option<Box<Item>>,
    pub events: List<Event>,
    pub checked: Checked,
    pub choice: HandChoice,
    pub enabled: bool,
    pub nonce: [u8; 2],
    pub age: Duration,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LongChoice {
    Slot00,
    Slot01,
    Slot02,
    Slot03,
    Slot04,
    Slot05,
    Slot06,
    Slot07,
    Slot08,
    Slot09,
    Slot10,
    Slot11,
    Slot12,
    Slot13,
    Slot14,
    Slot15,
    Slot16,
    Slot17,
    Slot18,
    Slot19,
    Slot20,
    Slot21,
    Slot22,
    Slot23,
    Slot24,
    Slot25,
    Slot26,
    Slot27,
    Slot28,
    Slot29,
    Slot30,
    Slot31,
    Slot32,
    Slot33,
    Slot34,
    Slot35,
    Slot36,
    Slot37,
    Slot38,
    Slot39,
    Slot40,
    Slot41,
    Slot42,
    Slot43,
    Slot44,
    Slot45,
    Slot46,
    Slot47,
    Slot48,
    Slot49,
    Slot50,
    Slot51,
    Slot52,
    Slot53,
    Slot54,
    Slot55,
    Slot56,
    Slot57,
    Slot58,
    Slot59,
    Slot60,
    Slot61,
    Slot62,
    Slot63,
    Slot64,
    Slot65,
    Slot66,
    Slot67,
    Slot68,
    Slot69,
    Slot70,
    Slot71,
    Slot72,
    Slot73,
    Slot74,
    Slot75,
    Slot76,
    Slot77,
    Slot78,
    Slot79,
    Slot80,
    Slot81,
    Slot82,
    Slot83,
    Slot84,
    Slot85,
    Slot86,
    Slot87,
    Slot88,
    Slot89,
    Slot90,
    Slot91,
    Slot92,
    Slot93,
    Slot94,
    Slot95,
    Slot96,
    Slot97,
    Slot98,
}

#[derive(Debug)]
pub struct SentRows {
    pub rows: Box<[u32]>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ReceivedRows {
    pub received: Box<[u32]>,
}

#[derive(Debug)]
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

#[must_use]
pub fn hand_choice_to_record(value: &HandChoice) -> r::HandChoice {
    match value {
        HandChoice::None => r::HandChoice::None,
        HandChoice::Some => r::HandChoice::Some,
    }
}

#[must_use]
pub fn hand_choice_to_domain(record: r::HandChoice) -> HandChoice {
    match record {
        r::HandChoice::None => HandChoice::None,
        r::HandChoice::Some => HandChoice::Some,
    }
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
