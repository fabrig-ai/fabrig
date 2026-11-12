//! Turn-local conversation values, measured before copying (testing/fakes/llm.md, 2.6).
use alloc::boxed::Box;
use skein_lib::{Writer, bytes, len_u64};

use crate::api::{Error, Failure, Line, Part, Query, Script, Turn};
use crate::{limits, respond};

pub(crate) fn valid(scripts: &[Script]) -> bool {
    for script in scripts {
        for turn in &script.turns {
            let mut takes = 0_u32;
            let mut pieces = false;
            for (index, line) in turn.lines.iter().enumerate() {
                match line {
                    Line::Take { name, before, after } => {
                        takes = takes.checked_add(1).expect("bounded line count");
                        if takes > 8
                            || name.is_empty()
                            || name.len() > 32
                            || before.is_empty()
                            || before.len() > 256
                            || after.is_empty()
                            || after.len() > 256
                        {
                            return false;
                        }
                        for byte in name {
                            if !byte.is_ascii_alphanumeric() && *byte != b'_' {
                                return false;
                            }
                        }
                        for earlier in turn.lines.get(..index).expect("line prefix") {
                            match earlier {
                                Line::Take { name: previous, .. } => {
                                    if previous == name {
                                        return false;
                                    }
                                }
                                Line::Pieces { .. }
                                | Line::Text { .. }
                                | Line::Call { .. }
                                | Line::Fail { .. }
                                | Line::Opaque { .. } => {}
                            }
                        }
                    }
                    Line::Pieces { .. } => pieces = true,
                    Line::Text { .. } | Line::Call { .. } | Line::Fail { .. } | Line::Opaque { .. } => {}
                }
            }
            if pieces && takes != 0 {
                return false;
            }
        }
    }
    true
}

fn in_part<'q>(text: &'q [u8], before: &[u8], after: &[u8]) -> Option<&'q [u8]> {
    for index in (0..text.len()).rev() {
        let suffix = text.get(index..).expect("position within part");
        if let Some(value) = suffix.strip_prefix(before)
            && let Some(end) = bytes::find(value, after)
            && (1..=256).contains(&end)
        {
            let value = value.get(..end).expect("found suffix within part");
            if !value.contains(&b'\n') {
                return Some(value);
            }
        }
    }
    None
}

fn taken<'q>(query: &'q Query, before: &[u8], after: &[u8]) -> Option<&'q [u8]> {
    for message in query.messages.iter().rev() {
        for part in message.parts.iter().rev() {
            let text = match part {
                Part::Text { text } => text,
                Part::ToolOutput { output, .. } => output,
                Part::Opaque { .. } | Part::ToolCall { .. } => continue,
            };
            if let Some(value) = in_part(text, before, after) {
                return Some(value);
            }
        }
    }
    in_part(&query.system, before, after)
}

fn value<'q>(query: &'q Query, name: &[u8], before: &[u8], after: &[u8]) -> Result<&'q [u8], Failure> {
    match taken(query, before, after) {
        Some(value) => Ok(value),
        None => {
            let reason = bytes::concat(&[
                b"no value taken for ",
                name,
                b": \"",
                before,
                "\" … \"".as_bytes(),
                after,
                b"\" is nowhere in the conversation",
            ]);
            Err(respond::invalid_script(b"no_take", &reason, query))
        }
    }
}

fn number(value: &[u8]) -> bool {
    if value.is_empty() || value.len() > 20 || (value.len() > 1 && value.first() == Some(&b'0')) {
        return false;
    }
    for byte in value {
        if !byte.is_ascii_digit() {
            return false;
        }
    }
    true
}

pub(crate) fn check(turn: &Turn, query: &Query, cap: u32) -> Result<(), Failure> {
    let size = measure(turn, query)?;
    if !limits::fits(Some(size), cap) {
        return Err(Error::ContextTooLong.into());
    }
    Ok(())
}

/// Whole answer accounting and all take failures precede any reply allocation.
pub(crate) fn measure(turn: &Turn, query: &Query) -> Result<u64, Failure> {
    let mut total = limits::scripted_answer(turn).ok_or(Error::ContextTooLong)?;
    for (index, line) in turn.lines.iter().enumerate() {
        let earlier = turn.lines.get(..index).expect("line prefix");
        let (source, arguments) = match line {
            Line::Take { name, before, after } => {
                value(query, name, before, after)?;
                continue;
            }
            Line::Text { text } => (text, false),
            Line::Call { arguments, .. } => (arguments, true),
            Line::Pieces { .. } | Line::Fail { .. } | Line::Opaque { .. } => continue,
        };
        let expanded = walk(source, earlier, query, arguments, None)?;
        total = total.checked_sub(len_u64(source.len())).ok_or(Error::ContextTooLong)?;
        total = total.checked_add(expanded).ok_or(Error::ContextTooLong)?;
    }
    Ok(total)
}

pub(crate) fn expand(source: &[u8], earlier: &[Line], query: &Query, arguments: bool) -> Box<[u8]> {
    let len = walk(source, earlier, query, arguments, None).expect("expansion measured before copying");
    let mut out = Writer::new(skein_lib::usize_of(u32::try_from(len).expect("admitted answer bytes")));
    walk(source, earlier, query, arguments, Some(&mut out)).expect("same measured expansion");
    out.finish()
}

fn use_at(source: &[u8], name: &[u8], arguments: bool) -> Option<(usize, bool)> {
    if arguments
        && let Some(rest) = source.strip_prefix(b"\"{{#")
        && let Some(rest) = rest.strip_prefix(name)
        && rest.starts_with(b"}}\"")
    {
        return Some((name.len().checked_add(7).expect("bounded name"), true));
    }
    if let Some(rest) = source.strip_prefix(b"{{")
        && let Some(rest) = rest.strip_prefix(name)
        && rest.starts_with(b"}}")
    {
        return Some((name.len().checked_add(4).expect("bounded name"), false));
    }
    None
}

fn emit(out: &mut Option<&mut Writer>, value: &[u8]) -> u64 {
    if let Some(out) = out {
        out.put(value).expect("measured output room");
    }
    len_u64(value.len())
}

fn escaped(out: &mut Option<&mut Writer>, value: &[u8]) -> Result<u64, Failure> {
    let mut len = 0_u64;
    for byte in value {
        let size = match *byte {
            b'"' | b'\\' => emit(out, &[b'\\', *byte]),
            0..=0x1f => {
                let high = bytes::hex_digit(byte.checked_div(16).expect("nonzero divisor"));
                let low = bytes::hex_digit(byte.checked_rem(16).expect("nonzero divisor"));
                emit(out, &[b'\\', b'u', b'0', b'0', high, low])
            }
            0x20..=0x21 | 0x23..=0x5b | 0x5d..=0xff => emit(out, &[*byte]),
        };
        len = len.checked_add(size).ok_or(Error::ContextTooLong)?;
    }
    Ok(len)
}

fn walk(
    source: &[u8],
    earlier: &[Line],
    query: &Query,
    arguments: bool,
    mut out: Option<&mut Writer>,
) -> Result<u64, Failure> {
    let mut end = 0_usize;
    let mut len = 0_u64;
    for (index, byte) in source.iter().enumerate() {
        if index < end {
            continue;
        }
        if *byte != b'{' && (!arguments || *byte != b'"') {
            len = len.checked_add(emit(&mut out, &[*byte])).ok_or(Error::ContextTooLong)?;
            continue;
        }
        let suffix = source.get(index..).expect("source position");
        let mut replaced = false;
        for line in earlier {
            match line {
                Line::Take { name, before, after } => {
                    if let Some((used, numeric)) = use_at(suffix, name, arguments) {
                        let value = value(query, name, before, after)?;
                        if numeric && !number(value) {
                            let reason =
                                bytes::concat(&[b"the value taken for ", name, b", \"", value, b"\", is no number"]);
                            return Err(respond::invalid_script(b"no_take", &reason, query));
                        }
                        let size =
                            if arguments && !numeric { escaped(&mut out, value)? } else { emit(&mut out, value) };
                        len = len.checked_add(size).ok_or(Error::ContextTooLong)?;
                        end = index.checked_add(used).expect("placeholder within source");
                        replaced = true;
                        break;
                    }
                }
                Line::Text { .. }
                | Line::Call { .. }
                | Line::Pieces { .. }
                | Line::Opaque { .. }
                | Line::Fail { .. } => {}
            }
        }
        if !replaced {
            len = len.checked_add(emit(&mut out, &[*byte])).ok_or(Error::ContextTooLong)?;
        }
    }
    Ok(len)
}
