//! The grammar shared by parameter 11 and follows' producers
//! (temper/docs/design/domain/forge/change/README.md, 1; temper/docs/design/domain/forge/follows.md, 2).

use jig_domain_tasks as tasks;
use skein_lib::{
    List, Name,
    bytes::{decimal, find},
};

/// A speed name's byte bound.
pub const SPEED_BYTES: usize = 32;

/// Check the spelling alone; the tasks child checks a name against the chosen model.
#[must_use]
pub fn speed_name(text: &[u8]) -> Option<Name<SPEED_BYTES>> {
    if text.is_empty() || text.len() > SPEED_BYTES {
        return None;
    }
    for byte in text {
        match byte {
            b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' => {}
            _ => return None,
        }
    }
    Name::from_checked(text)
}

/// Split the three parts and read the optional charter.
#[must_use]
pub fn parts(text: &[u8]) -> Option<(Option<u32>, &[u8], &[u8])> {
    let (charter, choices) = split(text, b';')?;
    let (producing, review) = split(choices, b';')?;
    if find(review, b";").is_some() {
        return None;
    }
    let charter = if charter.is_empty() {
        None
    } else {
        let charter = u32::try_from(decimal(charter)?).ok()?;
        if charter == 0 {
            return None;
        }
        Some(charter)
    };
    Some((charter, producing, review))
}

/// Read the main entry and check all sub-agents, collecting them if requested.
#[must_use]
pub fn choice(text: &[u8], mut subagents: Option<&mut List<tasks::ModelEffort>>) -> Option<tasks::ModelEffort> {
    let end = find(text, b" ").unwrap_or(text.len());
    let main = entry(text.get(..end)?)?;
    let mut at = end;
    for _ in 0..tasks::MODELS {
        if at == text.len() {
            break;
        }
        let start = at.checked_add(1)?;
        let remaining = text.get(start..)?;
        at = start.checked_add(find(remaining, b" ").unwrap_or(remaining.len()))?;
        let model = entry(text.get(start..at)?)?;
        if let Some(out) = &mut subagents {
            out.push(model).ok()?;
        }
    }
    if at != text.len() {
        return None;
    }
    Some(main)
}

fn entry(text: &[u8]) -> Option<tasks::ModelEffort> {
    let (text, speed) = match split(text, b'@') {
        Some((text, speed)) => (text, Some(speed_name(speed)?)),
        None => (text, None),
    };
    let (model, effort) = match split(text, b'.') {
        Some((model, effort)) => (model, Some(u8::try_from(decimal(effort)?).ok()?)),
        None => (text, None),
    };
    Some(tasks::ModelEffort { model: u8::try_from(decimal(model)?).ok()?, effort, speed })
}

fn split(text: &[u8], separator: u8) -> Option<(&[u8], &[u8])> {
    let at = find(text, &[separator])?;
    Some((text.get(..at)?, text.get(at.checked_add(1)?..)?))
}
