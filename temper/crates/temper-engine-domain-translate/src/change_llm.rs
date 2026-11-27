//! A change's producer and role choices in parameter 11
//! (`temper/docs/design/domain/forge/change/README.md`, 1).

use crate::append;
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::{
    List,
    bytes::{decimal, find},
};

#[must_use]
pub fn spell(producer: Option<u32>, llm: &crate::ChangeLlm) -> Box<[u8]> {
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
        append(&mut out, &crate::decimal(u64::from(producer)));
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
    append(out, &crate::decimal(u64::from(entry.model)));
    if let Some(effort) = entry.effort {
        out.push(b'.').expect("effort separator");
        append(out, &crate::decimal(u64::from(effort)));
    }
}

#[must_use]
pub fn read(text: &[u8]) -> Option<(Option<u32>, crate::ChangeLlm)> {
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
    Some(tasks::LlmChoice { later: None, account: None, main, subagents: subagents.into_boxed() })
}

fn read_entry(text: &[u8]) -> Option<tasks::ModelEffort> {
    let (model, effort) = match split(text, b'.') {
        Some((model, effort)) => (model, Some(u8::try_from(decimal(effort)?).ok()?)),
        None => (text, None),
    };
    Some(tasks::ModelEffort { model: u8::try_from(decimal(model)?).ok()?, effort, speed: None })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn choice(model: u8, effort: Option<u8>) -> tasks::LlmChoice {
        tasks::LlmChoice {
            later: None,
            account: None,
            main: tasks::ModelEffort { model, effort, speed: None },
            subagents: Box::new([]),
        }
    }
    #[test]
    fn change_llm_text_round_trips_each_role_and_optional_effort() {
        use super::*;
        for (text, producer, llm) in [
            (b";;".as_slice(), None, crate::ChangeLlm::default()),
            (b"6;;", Some(6), crate::ChangeLlm::default()),
            (b";3;", None, crate::ChangeLlm { producer: Some(choice(3, None)), review: None }),
            (b";;3.0", None, crate::ChangeLlm { producer: None, review: Some(choice(3, Some(0))) }),
            (
                b"6;3.1 0.0 2;3.0",
                Some(6),
                crate::ChangeLlm {
                    producer: Some(tasks::LlmChoice {
                        subagents: Box::new([
                            tasks::ModelEffort { model: 0, effort: Some(0), speed: None },
                            tasks::ModelEffort { model: 2, effort: None, speed: None },
                        ]),
                        ..choice(3, Some(1))
                    }),
                    review: Some(choice(3, Some(0))),
                },
            ),
            (
                b"4294967295;255.255;0",
                Some(u32::MAX),
                crate::ChangeLlm { producer: Some(choice(u8::MAX, Some(u8::MAX))), review: Some(choice(0, None)) },
            ),
        ] {
            assert_eq!(spell(producer, &llm).as_ref(), text);
            assert_eq!(read(text), Some((producer, llm)));
        }
    }

    #[test]
    fn change_llm_text_refuses_malformed_parts_and_too_many_entries() {
        for text in [
            b"".as_slice(),
            b";",
            b";;;",
            b"6;3",
            b"0;;",
            b"4294967296;;",
            b"-1;;",
            b"+1;;",
            b"x;;",
            b";256;",
            b";1.256;",
            b";1.;",
            b";.1;",
            b";1.2.3;",
            b"; 1;",
            b";1 ;",
            b";1  2;",
            b";1\t2;",
            b";x;",
            b";;1  ",
            b";;\xff",
            b" 6;;",
        ] {
            assert_eq!(read(text), None, "{text:?}");
        }
        let mut text = List::with_capacity(140);
        append(&mut text, b";0");
        for _ in 0..tasks::MODELS {
            append(&mut text, b" 1");
        }
        text.push(b';').unwrap();
        let accepted = read(text.as_slice()).expect("main plus MODELS sub-agents");
        assert_eq!(spell(accepted.0, &accepted.1).as_ref(), text.as_slice());
        text.truncate(text.len() - 1);
        append(&mut text, b" 2;");
        assert_eq!(read(text.as_slice()), None);
    }
}
