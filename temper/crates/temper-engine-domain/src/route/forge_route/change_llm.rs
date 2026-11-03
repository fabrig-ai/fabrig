//! A change's producer and role choices in parameter 11
//! (`temper/docs/design/domain/forge.md`, 8.1).

use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::{
    List,
    bytes::{decimal, find},
};

pub(super) fn spell(producer: Option<u32>, llm: &crate::ChangeLlm) -> Box<[u8]> {
    // A charter has at most ten digits, two separators, and each choice entry
    // at most eight bytes including its preceding space.
    let mut size = 12_u32;
    for choice in [&llm.producer, &llm.review].into_iter().flatten() {
        let entries = u32::try_from(choice.subagents.len()).expect("bounded sub-agents");
        size = size
            .checked_add(entries.checked_add(1).expect("main entry").checked_mul(8).expect("choice bytes"))
            .expect("parameter bytes");
    }
    let mut out = List::with_capacity(size);
    if let Some(producer) = producer {
        super::append(&mut out, &super::decimal(u64::from(producer)));
    }
    for choice in [&llm.producer, &llm.review] {
        out.push(b';').expect("two separators");
        if let Some(choice) = choice {
            spell_entry(&mut out, choice.main);
            for entry in &choice.subagents {
                out.push(b' ').expect("entry separator");
                spell_entry(&mut out, *entry);
            }
        }
    }
    out.into_boxed()
}

fn spell_entry(out: &mut List<u8>, entry: tasks::ModelEffort) {
    super::append(out, &super::decimal(u64::from(entry.model)));
    if let Some(effort) = entry.effort {
        out.push(b'.').expect("effort separator");
        super::append(out, &super::decimal(u64::from(effort)));
    }
}

pub(super) fn read(text: &[u8]) -> Option<(Option<u32>, crate::ChangeLlm)> {
    let (charter, choices) = split(text, b';')?;
    let (producer_choice, review_choice) = split(choices, b';')?;
    if find(review_choice, b";").is_some() {
        return None;
    }
    let producer = if charter.is_empty() {
        None
    } else {
        let charter = u32::try_from(decimal(charter)?).ok()?;
        if charter == 0 {
            return None;
        }
        Some(charter)
    };
    let producing = if producer_choice.is_empty() { None } else { Some(read_choice(producer_choice)?) };
    let review = if review_choice.is_empty() { None } else { Some(read_choice(review_choice)?) };
    Some((producer, crate::ChangeLlm { producer: producing, review }))
}

fn split(text: &[u8], separator: u8) -> Option<(&[u8], &[u8])> {
    let at = find(text, &[separator])?;
    Some((text.get(..at)?, text.get(at.checked_add(1)?..)?))
}

fn read_choice(text: &[u8]) -> Option<tasks::LlmChoice> {
    let end = find(text, b" ").unwrap_or(text.len());
    let main = read_entry(text.get(..end)?)?;
    let mut subagents = List::with_capacity(tasks::MODELS);
    let mut at = end;
    for _ in 0..tasks::MODELS {
        if at == text.len() {
            break;
        }
        let start = at.checked_add(1)?;
        let remaining = text.get(start..)?;
        at = start.checked_add(find(remaining, b" ").unwrap_or(remaining.len()))?;
        subagents.push(read_entry(text.get(start..at)?)?).ok()?;
    }
    if at != text.len() {
        return None;
    }
    Some(tasks::LlmChoice { account: None, main, subagents: subagents.into_boxed() })
}

fn read_entry(text: &[u8]) -> Option<tasks::ModelEffort> {
    let (model, effort) = match split(text, b'.') {
        Some((model, effort)) => (model, Some(u8::try_from(decimal(effort)?).ok()?)),
        None => (text, None),
    };
    Some(tasks::ModelEffort { model: u8::try_from(decimal(model)?).ok()?, effort })
}
