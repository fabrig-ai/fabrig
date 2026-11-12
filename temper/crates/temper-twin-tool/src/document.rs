//! The fake document's strict validation remains temper-fake's.
use crate::{Refusal, Replace, Take, ValueTake, takes};
use serde_json::value::RawValue;
use smith_transcript::{Block, Role};
use std::{collections::BTreeMap, fmt::Write};

type Base = BTreeMap<String, Box<RawValue>>;

pub(crate) fn base(bytes: &[u8]) -> Result<Base, Refusal> {
    if temper_fake::llm::Document::decode(bytes).is_err() {
        return Err(Refusal::Base);
    }
    match serde_json::from_slice(bytes) {
        Ok(base) => Ok(base),
        Err(_) => Err(Refusal::Base),
    }
}

fn quoted(bytes: &[u8]) -> Result<String, Refusal> {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return Err(Refusal::Bounds { what: "UTF-8 text" });
    };
    Ok(serde_json::to_string(text).expect("text JSON"))
}

/// Smith's fresh Open brief is the opening request (smith/docs/design/domain/run.md, 3.4).
/// The transcript records messages, not the system text or the run mode.
pub(crate) fn asked(take: &Take, turns: &[smith_transcript::Turn]) -> bool {
    // frames::read admits at least one turn.
    for message in turns[0].messages() {
        match message.role() {
            Role::Assistant => break,
            Role::User => {
                for block in message.blocks() {
                    if let Block::Text(text) = block {
                        // Finite main and delegate openings carry the begin instruction,
                        // even when an offered message also quotes the cue.
                        if text.text().starts_with(b"Begin the work your brief describes.")
                            || text.text().starts_with(b"Begin the work your instructions describe.")
                        {
                            return false;
                        }
                        if skein_lib::bytes::find(text.text(), &take.cue).is_some() {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

pub(crate) fn script(
    take: &Take,
    turns: &[smith_transcript::Turn],
    replaces: &[Replace],
    counts: &mut [u32],
    dropped: &mut u32,
) -> Result<String, Refusal> {
    let mut script = format!("{{\"cue\":{},\"turns\":[", quoted(&take.cue)?);
    for (i, turn) in turns.iter().enumerate() {
        if i != 0 {
            script.push(',');
        }
        script.push_str("{\"lines\":[");
        let mut lines = 0;
        let mut calls = false;
        for message in turn.messages() {
            for block in message.blocks() {
                if matches!(block, Block::Opaque(_)) {
                    *dropped = dropped.checked_add(1).ok_or(Refusal::Bounds { what: "opaque count" })?;
                    continue;
                }
                if *message.role() != Role::Assistant {
                    continue;
                }
                if lines == temper_fake::ENTRIES {
                    return Err(Refusal::Bounds { what: "lines" });
                }
                if lines != 0 {
                    script.push(',');
                }
                match block {
                    Block::Text(text) | Block::Refusal(text) => {
                        let text = replace(text.text(), replaces, counts)?;
                        write!(script, "{{\"text\":{}}}", quoted(&text)?).expect("writing to String");
                    }
                    Block::Call(call) => {
                        calls = true;
                        let arguments = replace(call.input(), replaces, counts)?;
                        if !matches!(
                            serde_json::from_slice::<serde_json::Value>(&arguments),
                            Ok(serde_json::Value::Object(_))
                        ) {
                            return Err(Refusal::Bounds { what: "replacement arguments" });
                        }
                        // Raw arguments are embedded without a JSON parse/serialize
                        // round trip, preserving their recorded order and spacing.
                        let Ok(text) = std::str::from_utf8(&arguments) else {
                            return Err(Refusal::Bounds { what: "arguments" });
                        };
                        write!(script, "{{\"call\":{},\"arguments\":{text}}}", quoted(call.name())?)
                            .expect("writing to String");
                    }
                    Block::Opaque(_) | Block::Result(_) => return Err(Refusal::Bounds { what: "assistant block" }),
                }
                lines += 1;
            }
        }
        let finish = if calls { "tool_calls" } else { "stop" };
        let tokens = turn.usage().output().unwrap_or(1);
        write!(script, "],\"finish\":\"{finish}\",\"tokens\":{tokens}}}").expect("writing to String");
        if script.len() > temper_fake::DOCUMENT_BYTES as usize {
            return Err(Refusal::Bounds { what: "document bytes" });
        }
    }
    script.push_str("]}");
    Ok(script)
}

pub(crate) fn replace(input: &[u8], replaces: &[Replace], counts: &mut [u32]) -> Result<Vec<u8>, Refusal> {
    let mut bytes = input.to_vec();
    for (i, replace) in replaces.iter().enumerate() {
        let mut out = Vec::new();
        let mut at = 0;
        while at < bytes.len() {
            if bytes[at..].starts_with(&replace.old) {
                out.extend_from_slice(&replace.new);
                at += replace.old.len();
                counts[i] = counts[i].checked_add(1).ok_or(Refusal::Bounds { what: "replace count" })?;
            } else {
                out.push(bytes[at]);
                at += 1;
            }
            if out.len() > temper_fake::DOCUMENT_BYTES as usize {
                return Err(Refusal::Bounds { what: "arguments" });
            }
        }
        bytes = out;
    }
    Ok(bytes)
}

pub(crate) fn finish(base: &mut Base, cued: &[String], asked: &[String]) -> Result<Box<[u8]>, Refusal> {
    for (name, scripts) in [("cued", cued), ("asked", asked)] {
        let scripts = format!("[{}]", scripts.join(","));
        let Ok(scripts) = RawValue::from_string(scripts) else {
            return Err(Refusal::Bounds { what: "scripts" });
        };
        base.insert(String::from(name), scripts);
    }
    checked(base)
}

pub(crate) fn checked(base: &Base) -> Result<Box<[u8]>, Refusal> {
    let bytes = serde_json::to_vec(base).expect("raw JSON members");
    if temper_fake::llm::Document::decode(&bytes).is_err() {
        return Err(Refusal::Bounds { what: "fake document" });
    }
    Ok(bytes.into_boxed_slice())
}

pub(crate) fn transform(
    base: &mut Base, groups: &[&str], replaces: &[Replace], counts: &mut [u32],
    values: &[ValueTake], taken: &mut [u32],
) -> Result<(), Refusal> {
    for group in groups {
        let Some(raw) = base.get_mut(*group) else { continue };
        let mut scripts: Vec<Box<RawValue>> = serde_json::from_str(raw.get()).expect("checked scripts");
        for raw in &mut scripts {
            if *group == "otherwise" {
                *raw = turn(raw, replaces, counts, values, taken)?;
            } else {
                let mut script: Base = serde_json::from_str(raw.get()).expect("checked script");
                let turns = script.get_mut("turns").expect("script turns");
                let mut list: Vec<Box<RawValue>> = serde_json::from_str(turns.get()).expect("checked turns");
                for raw in &mut list {
                    *raw = turn(raw, replaces, counts, values, taken)?;
                }
                *turns = json(&list);
                *raw = json(&script);
            }
        }
        *raw = json(&scripts);
    }
    for (take, count) in values.iter().zip(taken) {
        if *count == 0 {
            return Err(Refusal::UnusedTake { name: take.name.clone() });
        }
    }
    Ok(())
}
fn json<T: serde::Serialize>(value: &T) -> Box<RawValue> {
    serde_json::value::to_raw_value(value).expect("JSON value")
}

fn turn(raw: &RawValue, replaces: &[Replace], counts: &mut [u32], values: &[ValueTake], taken: &mut [u32]) -> Result<Box<RawValue>, Refusal> {
    if replaces.is_empty() && values.is_empty() {
        return Ok(raw.to_owned());
    }
    let mut turn: Base = if raw.get().starts_with('"') {
        let text: String = serde_json::from_str(raw.get()).expect("checked shorthand turn");
        BTreeMap::from([
            ("lines".into(), json(&[BTreeMap::from([("text", text.clone())])])),
            ("tokens".into(), json(&text.len().div_ceil(4))),
            ("finish".into(), json(&"stop")),
        ])
    } else {
        serde_json::from_str(raw.get()).expect("checked turn")
    };
    let mut lines: Vec<Base> = serde_json::from_str(turn.get("lines").expect("turn lines").get()).expect("checked lines");
    let before = taken.to_vec();
    for line in &mut lines {
        if let Some(text) = line.get_mut("text") {
            let original: String = serde_json::from_str(text.get()).expect("checked text");
            let mut words = String::from_utf8(replace(original.as_bytes(), replaces, counts)?)
                .map_err(|_| Refusal::Bounds { what: "UTF-8 text" })?;
            for (take, count) in values.iter().zip(taken.iter_mut()) {
                words = takes::text(&words, take, None, "", count)?;
            }
            *text = json(&words);
        } else if let Some(call) = line.get("call") {
            let name: String = serde_json::from_str(call.get()).expect("checked call name");
            let arguments = line.get_mut("arguments").expect("call arguments");
            let bytes = replace(arguments.get().as_bytes(), replaces, counts)?;
            let mut raw = RawValue::from_string(String::from_utf8(bytes)
                .map_err(|_| Refusal::Bounds { what: "replacement arguments" })?)
                .map_err(|_| Refusal::Bounds { what: "replacement arguments" })?;
            if !raw.get().starts_with('{') {
                return Err(Refusal::Bounds { what: "replacement arguments" });
            }
            for (take, count) in values.iter().zip(taken.iter_mut()) {
                raw = takes::arguments(&raw, take, &name, "", count)?;
            }
            *arguments = raw;
        }
    }
    let mut prefix = Vec::new();
    for ((take, count), before) in values.iter().zip(taken.iter()).zip(&before) {
        if count != before {
            prefix.push(BTreeMap::from([
                ("take".into(), json(&take.name)), ("before".into(), json(&take.before)), ("after".into(), json(&take.after)),
            ]));
        }
    }
    prefix.extend(lines);
    turn.insert("lines".into(), json(&prefix));
    Ok(json(&turn))
}
