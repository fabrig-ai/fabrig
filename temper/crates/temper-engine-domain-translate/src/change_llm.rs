//! A change's producer and role choices in parameter 11
//! (`temper/docs/design/domain/forge/change/README.md`, 1).

use crate::append;
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::List;
use temper_engine_domain_forge::change_llm_text;

#[must_use]
pub fn spell(producer: Option<u32>, llm: &crate::ChangeLlm) -> Box<[u8]> {
    // A charter has at most ten digits, two separators, and each choice entry
    // at most 8 bytes for the indices, dot and preceding space, plus `@` and the name.
    let entry_bytes = u32::try_from(change_llm_text::SPEED_BYTES).expect("speed bound fits u32").saturating_add(9);
    let mut size = 12_u32;
    for choice in [&llm.producer, &llm.review].into_iter().flatten() {
        let entries = u32::try_from(choice.subagents.len()).expect("bounded sub-agents");
        size = size
            .checked_add(entries.checked_add(1).expect("main entry").checked_mul(entry_bytes).expect("choice bytes"))
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
    if let Some(speed) = entry.speed {
        out.push(b'@').expect("speed separator");
        append(out, speed.as_bytes());
    }
}

#[must_use]
pub fn read(text: &[u8]) -> Option<(Option<u32>, crate::ChangeLlm)> {
    let (producer, producing, review) = change_llm_text::parts(text)?;
    let producer_choice = if producing.is_empty() { None } else { Some(read_choice(producing)?) };
    let review = if review.is_empty() { None } else { Some(read_choice(review)?) };
    Some((producer, crate::ChangeLlm { producer: producer_choice, review }))
}

fn read_choice(text: &[u8]) -> Option<tasks::LlmChoice> {
    let mut subagents = List::with_capacity(tasks::MODELS);
    let main = change_llm_text::choice(text, Some(&mut subagents))?;
    Some(tasks::LlmChoice { later: None, account: None, main, subagents: subagents.into_boxed() })
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
    fn choice_at_speed(model: u8, effort: Option<u8>, speed: &[u8]) -> tasks::LlmChoice {
        let mut llm = choice(model, effort);
        llm.main.speed = Some(skein_lib::Name::from_checked(speed).expect("speed fixture fits"));
        llm
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
                b"6;3.1@ultrafast 0.0 2;3.0@standard",
                Some(6),
                crate::ChangeLlm {
                    producer: Some(tasks::LlmChoice {
                        subagents: Box::new([
                            tasks::ModelEffort { model: 0, effort: Some(0), speed: None },
                            tasks::ModelEffort { model: 2, effort: None, speed: None },
                        ]),
                        ..choice_at_speed(3, Some(1), b"ultrafast")
                    }),
                    review: Some(choice_at_speed(3, Some(0), b"standard")),
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
    fn change_llm_text_fits_both_roles_at_their_longest() {
        const SUBAGENTS: usize = 64; // jig domain/tasks/README.md, section 3.
        const MAIN: &[u8] = b"255.255@abcdefghijklmnopqrstuvwxyz0123-_"; // 32 speed bytes.
        const TEXT_BYTES: u32 = 5340; // 12 charter/separator bytes, plus both roles' 40 + 64 * 41 bytes.
        assert_eq!(SUBAGENTS, skein_lib::usize_of(tasks::MODELS));
        let mut text = List::with_capacity(TEXT_BYTES);
        append(&mut text, b"4294967295");
        for _ in 0..2_u32 {
            append(&mut text, b";");
            append(&mut text, MAIN);
            for _ in 0..SUBAGENTS {
                append(&mut text, b" ");
                append(&mut text, MAIN);
            }
        }
        assert_eq!(text.len(), TEXT_BYTES);
        let entry = choice_at_speed(u8::MAX, Some(u8::MAX), b"abcdefghijklmnopqrstuvwxyz0123-_").main;
        let role = tasks::LlmChoice { main: entry, subagents: Box::new([entry; SUBAGENTS]), ..choice(0, None) };
        let llm = crate::ChangeLlm { producer: Some(role.clone()), review: Some(role) };
        assert_eq!(spell(Some(u32::MAX), &llm).as_ref(), text.as_slice());
        assert_eq!(read(text.as_slice()), Some((Some(u32::MAX), llm)));
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
            b";1@;",
            b";1@abcdefghijklmnopqrstuvwxyz0123456;",
            b";1@Fast;",
            b";1@fast@standard;",
            b";;1@fast.speed",
            b";1@\xff;",
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
