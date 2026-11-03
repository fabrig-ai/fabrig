//! A change's producer and role choices in parameter 11
//! (`temper/docs/design/domain/forge.md`, 8.1).

use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::{List, bytes::decimal};

pub(super) fn spell(producer: Option<u32>, llm: &crate::ChangeLlm) -> Box<[u8]> {
    // A charter has at most ten digits, two separators, and each choice entry
    // at most seven bytes including its preceding space.
    let mut size = 12_u32;
    for choice in [&llm.producer, &llm.review].into_iter().flatten() {
        let entries = u32::try_from(choice.subagents.len()).expect("bounded sub-agents");
        size = size.checked_add(entries.checked_add(1).expect("main entry").checked_mul(8).expect("choice bytes"))
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
    let text = core::str::from_utf8(text).ok()?;
    let mut parts = text.split(';');
    let charter = parts.next()?;
    let producer = if charter.is_empty() {
        None
    } else {
        let charter = u32::try_from(decimal(charter.as_bytes())?).ok()?;
        if charter == 0 {
            return None;
        }
        Some(charter)
    };
    let llm = crate::ChangeLlm { producer: read_choice(parts.next()?)?, review: read_choice(parts.next()?)? };
    if parts.next().is_some() {
        return None;
    }
    Some((producer, llm))
}

fn read_choice(text: &str) -> Option<Option<tasks::LlmChoice>> {
    if text.is_empty() {
        return Some(None);
    }
    let mut entries = text.split(' ');
    let main = read_entry(entries.next()?)?;
    let mut subagents = List::with_capacity(tasks::MODELS);
    for entry in entries {
        subagents.push(read_entry(entry)?).ok()?;
    }
    Some(Some(tasks::LlmChoice { account: None, main, subagents: subagents.into_boxed() }))
}

fn read_entry(text: &str) -> Option<tasks::ModelEffort> {
    let mut parts = text.split('.');
    let model = u8::try_from(decimal(parts.next()?.as_bytes())?).ok()?;
    let effort = match parts.next() {
        Some(effort) => Some(u8::try_from(decimal(effort.as_bytes())?).ok()?),
        None => None,
    };
    if parts.next().is_some() {
        return None;
    }
    Some(tasks::ModelEffort { model, effort })
}
