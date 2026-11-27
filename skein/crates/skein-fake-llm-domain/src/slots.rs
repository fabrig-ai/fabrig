//! Menu body slots, borrowed from the query and sized before copying
//! (skein/docs/design/testing/fakes/drawn-agent.md, 2).

use alloc::boxed::Box;
use skein_lib::{List, Rng, Writer, bytes, usize_of};

use crate::api::{Bounds, Menu, Query, Slot};
use crate::take::{self, Span};

pub(crate) const SLOTS: usize = 8;
const CANDIDATES: u32 = 64;
pub(crate) const UNISSUED: [u8; SLOTS] = [0; SLOTS];

pub(crate) fn valid(slots: &[Slot]) -> bool {
    if slots.len() > SLOTS {
        return false;
    }
    for (index, slot) in slots.iter().enumerate() {
        if !take::valid_name(&slot.name)
            || !take::valid_marker(&slot.before)
            || slot.after.len() > 256
            || !take::valid_marker(&slot.unissued)
            || slot.unissued.contains(&b'\n')
            || slot.weight > 1000
            || slot.foreign > 1000
            || (slot.with.is_empty() && slot.foreign != 0)
        {
            return false;
        }
        if let Some(bounds) = slot.numeric
            && (bounds.low > bounds.high || !take::number(&slot.unissued))
        {
            return false;
        }
        let earlier = slots.get(..index).expect("preceding slots");
        let mut with_found = slot.with.is_empty();
        for previous in earlier {
            if previous.name == slot.name {
                return false;
            }
            if previous.name == slot.with {
                with_found = true;
            }
        }
        if !with_found {
            return false;
        }
    }
    true
}

pub(crate) fn valid_bodies(menu: &Menu) -> bool {
    for slot in &menu.slots {
        if slot.numeric.is_some() {
            continue;
        }
        for body in &menu.arguments {
            for (at, byte) in body.iter().enumerate() {
                if *byte == b'"'
                    && let Some((_, true)) = take::use_at(body.get(at..).expect("body suffix"), &slot.name, true)
                {
                    return false;
                }
            }
        }
    }
    true
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Text {
    Part { message: usize, part: usize },
    System,
}

impl Text {
    fn read(self, query: &Query) -> &[u8] {
        match self {
            Self::Part { message, part } => {
                let message = query.messages.get(message).expect("candidate message");
                let part = message.parts.get(part).expect("candidate part");
                take::text_in(part).expect("candidate in readable part")
            }
            Self::System => &query.system,
        }
    }
}

/// A range includes both delimiters for purposes of overlap.
struct Place {
    text: Text,
    start: usize,
    end: usize,
    value: Span,
}

impl Place {
    fn read<'q>(&self, query: &'q Query) -> &'q [u8] {
        self.value.value(self.text.read(query))
    }
}

/// Newest-first candidate places, one list per declaration.
pub(crate) struct Candidates {
    slots: List<List<Place>>,
}

impl Candidates {
    pub(crate) fn scan(query: &Query, slots: &[Slot]) -> Self {
        let mut sets = List::with_capacity(u32::try_from(slots.len()).expect("admitted slot count"));
        for slot in slots {
            let places = scan(query, slot, sets.as_slice());
            assert!(sets.push(places).is_ok(), "room for every slot's candidates");
        }
        Self { slots: sets }
    }

    /// For each used slot, in declaration order: roll the weight, then pick a
    /// candidate if that roll did not win. Zero denotes the never-issued value.
    pub(crate) fn draw(&self, rng: &mut Rng, query: &Query, slots: &[Slot], used: [bool; SLOTS]) -> [u8; SLOTS] {
        let mut picks = UNISSUED;
        for (index, slot) in slots.iter().enumerate() {
            if !used.get(index).copied().expect("admitted slot index") {
                continue;
            }
            let places = self.slots.as_slice().get(index).expect("scan for every slot");
            if !places.is_empty() && !rng.chance(slot.weight) {
                let rank = match self.with_value(query, slots, &slot.with, picks) {
                    Some((linked, value)) => paired_rank(rng, query, places, linked, value, slot.foreign),
                    None => rng.between(1, u64::from(places.len())),
                };
                *picks.get_mut(index).expect("admitted slot index") = u8::try_from(rank).expect("at most 64 picks");
            }
        }
        picks
    }

    fn with_value<'q>(
        &self,
        query: &'q Query,
        slots: &'q [Slot],
        name: &[u8],
        picks: [u8; SLOTS],
    ) -> Option<(&'q Slot, &'q [u8])> {
        if name.is_empty() {
            return None;
        }
        for (index, slot) in slots.iter().enumerate() {
            if slot.name.as_ref() == name {
                // Unused slots also have a zero pick.
                let rank = picks.get(index).copied().expect("admitted slot index").checked_sub(1)?;
                let places = self.slots.as_slice().get(index).expect("scanned linked slot");
                let picked = places.get(u32::from(rank)).expect("picked linked candidate");
                return Some((slot, picked.read(query)));
            }
        }
        unreachable!("admitted link names an earlier slot")
    }

    pub(crate) fn values<'q>(&self, query: &'q Query, slots: &'q [Slot], picks: [u8; SLOTS]) -> [&'q [u8]; SLOTS] {
        let mut values = unissued(slots);
        for (index, pick) in picks.iter().take(slots.len()).enumerate() {
            if let Some(rank) = pick.checked_sub(1) {
                let places = self.slots.as_slice().get(index).expect("picked slot");
                let place = places.get(u32::from(rank)).expect("picked candidate");
                *values.get_mut(index).expect("admitted slot index") = place.read(query);
            }
        }
        values
    }
}

fn paired_rank(rng: &mut Rng, query: &Query, places: &List<Place>, linked: &Slot, value: &[u8], foreign: u32) -> u64 {
    let mut own = 0_u32;
    // A bit per retained candidate, in newest-first order. The linked value
    // may occur in several parts even though its own scan kept it only once.
    let mut is_own = 0_u64;
    let mut part = None;
    let mut own_part = false;
    for (index, place) in places.as_slice().iter().enumerate() {
        if part != Some(place.text) {
            part = Some(place.text);
            own_part = holds(place.text.read(query), linked, value);
        }
        if own_part {
            is_own |= candidate_bit(index);
            own = own.checked_add(1).expect("bounded candidates");
        }
    }
    let foreign_count = places.len().checked_sub(own).expect("own candidates in the scan");
    let want_foreign = rng.chance(foreign);
    let take_own = foreign_count == 0 || (!want_foreign && own != 0);
    let count = if take_own { own } else { foreign_count };
    let mut remaining = rng.below(u64::from(count));
    for index in 0..places.as_slice().len() {
        if (is_own & candidate_bit(index) != 0) == take_own {
            if remaining == 0 {
                return skein_lib::len_u64(index).checked_add(1).expect("one-based candidate rank");
            }
            remaining = remaining.checked_sub(1).expect("nonzero remaining rank");
        }
    }
    unreachable!("the chosen pool contains its counted candidates")
}

fn candidate_bit(index: usize) -> u64 {
    1_u64.checked_shl(u32::try_from(index).expect("bounded candidate index")).expect("at most 64 candidates")
}

fn holds(text: &[u8], linked: &Slot, value: &[u8]) -> bool {
    let first = linked.before.first().expect("admitted nonempty delimiter");
    for (index, byte) in text.iter().enumerate() {
        if byte == first
            && let Some(span) = take::capture_at(text, index, &linked.before, &linked.after)
            && span.value(text) == value
        {
            return true;
        }
    }
    false
}

fn scan(query: &Query, slot: &Slot, earlier: &[List<Place>]) -> List<Place> {
    let mut places = List::with_capacity(CANDIDATES);
    for (message, entry) in query.messages.iter().enumerate().rev() {
        for (part, block) in entry.parts.iter().enumerate().rev() {
            if let Some(text) = take::text_in(block) {
                in_text(query, Text::Part { message, part }, text, slot, earlier, &mut places);
                if places.room() == 0 {
                    return places;
                }
            }
        }
    }
    in_text(query, Text::System, &query.system, slot, earlier, &mut places);
    places
}

fn in_text(query: &Query, source: Text, text: &[u8], slot: &Slot, earlier: &[List<Place>], places: &mut List<Place>) {
    let first = slot.before.first().expect("admitted nonempty delimiter");
    for (start, byte) in text.iter().enumerate().rev() {
        if byte != first {
            continue;
        }
        let Some(span) = take::capture_at(text, start, &slot.before, &slot.after) else { continue };
        let value = span.value(text);
        if let Some(bounds) = slot.numeric
            && !in_bounds(value, bounds)
        {
            continue;
        }
        let end = span.end.checked_add(slot.after.len()).expect("closing delimiter within text");
        let place = Place { text: source, start, end, value: span };
        if overlaps(&place, earlier) || contains(query, places, value) {
            continue;
        }
        assert!(places.push(place).is_ok(), "room for a candidate before the scan stops");
        if places.room() == 0 {
            return;
        }
    }
}

fn in_bounds(value: &[u8], bounds: Bounds) -> bool {
    if !take::number(value) {
        return false;
    }
    let Some(number) = bytes::decimal(value) else { return false };
    (bounds.low..=bounds.high).contains(&number)
}

fn overlaps(place: &Place, earlier: &[List<Place>]) -> bool {
    for places in earlier {
        for taken in places {
            if taken.text == place.text && taken.start < place.end && place.start < taken.end {
                return true;
            }
        }
    }
    false
}

fn contains(query: &Query, places: &List<Place>, value: &[u8]) -> bool {
    for place in places {
        if place.read(query) == value {
            return true;
        }
    }
    false
}

pub(crate) fn unissued(slots: &[Slot]) -> [&[u8]; SLOTS] {
    let mut values = [b"".as_slice(); SLOTS];
    for (value, slot) in values.iter_mut().zip(slots) {
        *value = &slot.unissued;
    }
    values
}

pub(crate) struct Measure {
    pub(crate) bytes: u64,
    pub(crate) used: [bool; SLOTS],
}

pub(crate) fn measure(body: &[u8], slots: &[Slot], values: &[&[u8]; SLOTS]) -> Option<Measure> {
    walk(body, slots, values, None)
}

pub(crate) fn measure_bytes(body: &[u8], slots: &[Slot], values: &[&[u8]; SLOTS]) -> Option<u64> {
    Some(measure(body, slots, values)?.bytes)
}

pub(crate) fn expand(body: &[u8], slots: &[Slot], values: &[&[u8]; SLOTS], size: u64) -> Box<[u8]> {
    let mut out = Writer::new(usize_of(u32::try_from(size).expect("admitted answer bytes")));
    walk(body, slots, values, Some(&mut out)).expect("same measured expansion");
    out.finish()
}

fn walk(body: &[u8], slots: &[Slot], values: &[&[u8]; SLOTS], mut out: Option<&mut Writer>) -> Option<Measure> {
    let mut used = [false; SLOTS];
    let mut literal = 0;
    let mut size = 0_u64;
    // Only an opening brace can start a use. Between uses, emit one literal slice.
    'body: for (marker, byte) in body.iter().enumerate() {
        if *byte != b'{' || marker < literal {
            continue;
        }
        let mut at = marker;
        if let Some(quote) = marker.checked_sub(1)
            && quote >= literal
            && body.get(quote..)?.starts_with(b"\"{{#")
        {
            at = quote;
        }
        let suffix = body.get(at..)?;
        for (index, slot) in slots.iter().enumerate() {
            if let Some((width, numeric)) = take::use_at(suffix, &slot.name, true) {
                size = size.checked_add(take::emit(&mut out, body.get(literal..at)?))?;
                let value = values.get(index).expect("value for each slot");
                let len = if numeric { take::emit(&mut out, value) } else { take::escaped(&mut out, value)? };
                *used.get_mut(index).expect("admitted slot index") = true;
                size = size.checked_add(len)?;
                literal = at.checked_add(width)?;
                continue 'body;
            }
        }
    }
    size = size.checked_add(take::emit(&mut out, body.get(literal..)?))?;
    Some(Measure { bytes: size, used })
}

#[cfg(test)]
mod tests;
