//! Random calls keep their tool, body and slot picks until the whole batch fits
//! (skein/docs/design/testing/fakes/drawn-agent.md, 2).

use alloc::boxed::Box;
use skein_lib::{List, Rng, len_u64, usize_of};

use crate::api::{Error, Menu, Part, Query, Source};
use crate::domain::Config;
use crate::draws::SlotCapture;
use crate::slots::Candidates;
use crate::{limits, slots};

use super::{allowed, call_id, tool_count};

pub(super) struct Batch {
    pub(super) count: u32,
    pub(super) outside: bool,
}

pub(super) fn batch(
    rng: &mut Rng,
    minted: &mut u64,
    config: &Config,
    menu: &Menu,
    query: &Query,
    Batch { count, outside }: Batch,
    capture: &mut SlotCapture,
) -> Result<List<Part>, Error> {
    let mut calls = List::with_capacity(count);
    if menu.slots.is_empty() {
        for _ in 0..count {
            let call = draw(rng, minted, config, menu, query, outside)?;
            let part = call.part(menu, query, call.body(menu).into());
            calls.push(part).expect("room for each call");
        }
        return Ok(calls);
    }

    let unissued = slots::unissued(&menu.slots);
    let mut candidates = None;
    let mut planned = List::with_capacity(count);
    let mut drawn_size = Some(0_u64);
    for _ in 0..count {
        let call = draw(rng, minted, config, menu, query, outside)?;
        let mut picks = slots::UNISSUED;
        let mut used = [false; slots::SLOTS];
        let body = call.body(menu);
        let (base_bytes, bytes) = match call.input {
            Input::Body(_) => {
                let measure = slots::measure(body, &menu.slots, &unissued).expect("admitted never-issued body");
                used = measure.used;
                if used.contains(&true) {
                    if candidates.is_none() {
                        candidates = Some(Candidates::scan(query, &menu.slots));
                    }
                    let scanned = candidates.as_ref().expect("scanned slots");
                    picks = scanned.draw(rng, &menu.slots, used);
                    let bytes = if picks == slots::UNISSUED {
                        Some(measure.bytes)
                    } else {
                        let values = scanned.values(query, &menu.slots, picks);
                        slots::measure_bytes(body, &menu.slots, &values)
                    };
                    (measure.bytes, bytes)
                } else {
                    (measure.bytes, Some(measure.bytes))
                }
            }
            Input::Invalid(_) => {
                let bytes = len_u64(body.len());
                (bytes, Some(bytes))
            }
        };
        drawn_size = batch_size(drawn_size, call.name(menu, query), bytes);
        let plan = Planned { call, picks, used, base_bytes, bytes };
        assert!(planned.push(plan).is_ok(), "room for each call plan");
    }

    let filled = limits::fits(drawn_size, config.answer_bytes);
    for planned in &planned {
        let picks = if filled { planned.picks } else { slots::UNISSUED };
        let values = match &candidates {
            Some(candidates) if picks != slots::UNISSUED => candidates.values(query, &menu.slots, picks),
            Some(_) | None => unissued,
        };
        let body = planned.call.body(menu);
        let arguments = if planned.used.contains(&true) {
            let bytes = if filled { planned.bytes.expect("each filled body was measured") } else { planned.base_bytes };
            slots::expand(body, &menu.slots, &values, bytes)
        } else {
            body.into()
        };
        let part = planned.call.part(menu, query, arguments);
        if capture.active() {
            let Part::ToolCall { id, .. } = &part else { unreachable!("a random batch holds only calls") };
            record(capture, calls.len(), id, planned.used, &values, picks);
        }
        calls.push(part).expect("room for each call");
    }
    Ok(calls)
}

fn record(
    capture: &mut SlotCapture,
    call: u32,
    id: &[u8],
    used: [bool; slots::SLOTS],
    values: &[&[u8]; slots::SLOTS],
    picks: [u8; slots::SLOTS],
) {
    for (slot, ((used, value), pick)) in used.iter().zip(values).zip(picks).enumerate() {
        if *used {
            let slot = u32::try_from(slot).expect("at most eight admitted slots");
            let source = if pick == 0 { Source::Unissued } else { Source::Conversation };
            capture.record(call, id, slot, value, source, b"");
        }
    }
}

fn batch_size(total: Option<u64>, name: &[u8], body: Option<u64>) -> Option<u64> {
    total?.checked_add(limits::random_call(len_u64(name.len()), body?)?)
}

struct Planned {
    call: Call,
    picks: [u8; slots::SLOTS],
    used: [bool; slots::SLOTS],
    base_bytes: u64,
    bytes: Option<u64>,
}

#[derive(Clone, Copy)]
enum Input {
    Body(usize),
    Invalid(usize),
}

struct Call {
    id: u64,
    tool: usize,
    input: Input,
}

impl Call {
    fn name<'q>(&self, menu: &'q Menu, query: &'q Query) -> &'q [u8] {
        match self.input {
            Input::Body(_) => {}
            Input::Invalid(index) => {
                let invalid = menu.invalid.get(index).expect("picked invalid input");
                if let Some(name) = &invalid.name {
                    return name;
                }
            }
        }
        &query.tools.get(self.tool).expect("picked tool").name
    }

    fn body<'m>(&self, menu: &'m Menu) -> &'m [u8] {
        match self.input {
            Input::Body(index) => menu.arguments.get(index).expect("picked menu body"),
            Input::Invalid(index) => &menu.invalid.get(index).expect("picked invalid input").arguments,
        }
    }

    fn part(&self, menu: &Menu, query: &Query, arguments: Box<[u8]>) -> Part {
        Part::ToolCall { id: call_id(self.id), name: self.name(menu, query).into(), arguments }
    }
}

/// Draws the tool, malformed roll and input index, in that order. Slot picks
/// for a regular body follow its input index and precede the next call.
fn draw(
    rng: &mut Rng,
    minted: &mut u64,
    config: &Config,
    menu: &Menu,
    query: &Query,
    outside: bool,
) -> Result<Call, Error> {
    let mut pick = rng.below(tool_count(query, outside));
    let mut selected = None;
    for (index, tool) in query.tools.iter().enumerate() {
        if allowed(&query.choice, &tool.name) != outside {
            if pick == 0 {
                selected = Some(index);
                break;
            }
            pick = pick.checked_sub(1).expect("a nonzero remaining tool index");
        }
    }
    let tool = selected.expect("picked below the eligible tool count");
    *minted = minted.checked_add(1).ok_or(Error::InvalidRequest)?;
    let input = if rng.chance(config.malformed) && !menu.invalid.is_empty() {
        Input::Invalid(input_index(rng, menu.invalid.len()))
    } else {
        Input::Body(input_index(rng, menu.arguments.len()))
    };
    Ok(Call { id: *minted, tool, input })
}

fn input_index(rng: &mut Rng, len: usize) -> usize {
    let pick = rng.below(len_u64(len));
    usize_of(u32::try_from(pick).expect("menu entries are bounded by script_bytes"))
}
