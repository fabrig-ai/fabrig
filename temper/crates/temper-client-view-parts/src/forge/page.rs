//! Adoption outcomes shared by their callers (temper/docs/design/client/forge/family.md, 6).
use super::words;
use crate::cards::CardWords as _;
use crate::words::Words;
use alloc::boxed::Box;
use skein_lib::Writer;
use skein_ui::{Builder, Role, phrases::Hole};
use temper_client_domain::forge::{self, words::Word};
fn status(builder: &mut Builder, word: Word, holes: &[Hole], words: &Words) {
    builder.open(Role::Status);
    builder.name(&words::say(words, word, holes));
    words::text(words, builder, word, holes);
    builder.close();
}

fn kinds(kinds: forge::Kinds, words: &Words) -> Box<[u8]> {
    let rows = forge::words::kinds(kinds);
    let mut bytes = 0_usize;
    for (on, word) in rows {
        if on {
            if bytes != 0 {
                bytes = bytes.checked_add(2).expect("separator fits");
            }
            bytes = bytes.checked_add(words::say(words, word, &[]).len()).expect("nine kinds fit");
        }
    }
    let mut writer = Writer::new(bytes);
    let mut first = true;
    for (on, word) in rows {
        if on {
            if !first {
                writer.put(b"; ").expect("nine kinds fit");
            }
            writer.put(&words::say(words, word, &[])).expect("nine kinds fit");
            first = false;
        }
    }
    writer.finish()
}
pub fn outcome(
    builder: &mut Builder,
    fields: (&[u8], &[u8], &[u8], &[u8]),
    answer: &forge::Answer,
    words: &Words,
    text_limit: u32,
) {
    let (owner, name, prefix, forge) = fields;
    match answer {
        forge::Answer::Adopted(adopted) => {
            let role = forge::words::role(adopted.role);
            status(
                builder,
                Word::Adopted,
                &[
                    Hole::Name(forge::words::full_name(&adopted.owner, &adopted.name)),
                    Hole::Number(u64::from(adopted.forge)),
                    Hole::Name(words::say(words, role, &[])),
                ],
                words,
            );
            if adopted.home {
                status(builder, Word::IsHome, &[], words);
            }
            status(builder, Word::May, &[Hole::Name(kinds(adopted.kinds, words))], words);
            match adopted.protection {
                forge::Protection::Unknown => status(builder, Word::Protection, &[], words),
                forge::Protection::Absent | forge::Protection::Rule(_) => {}
            }
        }
        forge::Answer::CoreRefused(refusal) => {
            builder.open(Role::Alert);
            words.refusal(builder, refusal, text_limit);
            builder.close();
        }
        forge::Answer::EntranceRefused(refusal) => {
            builder.open(Role::Alert);
            words.refusal(builder, refusal.refusal(), text_limit);
            builder.close();
        }
        forge::Answer::ForgeRefused(reason) => {
            builder.open(Role::Alert);
            let name = Hole::Name(forge::words::full_name(owner, name));
            let number = Hole::Name(forge.into());
            match reason {
                forge::Reason::NoSuchForge => words::text(words, builder, Word::NoSuchForge, &[number]),
                forge::Reason::NotFound => words::text(words, builder, Word::NotFound, &[name, number]),
                forge::Reason::NoAccess => words::text(words, builder, Word::NoAccess, &[name, number]),
                forge::Reason::Refused => {
                    words::text(words, builder, Word::Refused, &[name, Hole::Name(prefix.into())]);
                }
                forge::Reason::AnotherProject => words::text(words, builder, Word::AnotherProject, &[name]),
                forge::Reason::HomeHeld => words::text(words, builder, Word::HomeHeld, &[]),
                forge::Reason::PrefixOccupied => {
                    words::text(words, builder, Word::PrefixOccupied, &[Hole::Name(prefix.into())]);
                }
                forge::Reason::InFlight => words::text(words, builder, Word::InFlight, &[name]),
                forge::Reason::TooLarge => words::text(words, builder, Word::TooLarge, &[]),
                forge::Reason::Unreachable => words::text(words, builder, Word::Unreachable, &[number]),
                forge::Reason::InvalidAnswer => words::text(words, builder, Word::InvalidAnswer, &[number]),
            }
            builder.close();
        }
        forge::Answer::Unknown => {
            status(builder, Word::Unknown, &[Hole::Name(forge::words::full_name(owner, name))], words);
        }
    }
}
