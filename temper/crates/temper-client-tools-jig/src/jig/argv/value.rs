//! Typed values shared by the table parser and printer.
use super::{Unwritten, Usage};
use crate::jig::subject::{self, Condition};
use crate::jig::{
    Limits,
    call::{Project, Value},
    table::{Argument, Bound, Kind, Place, Tool, bound},
};
use alloc::boxed::Box;
use jig_client::contract::{InboxGroup, ProposalKey, Proposer, QueryOrder, TurnName};
use skein_lib::{Decimal, Duration, Overflow, Writer};
pub(super) fn valid(
    table: &[&Tool],
    spec: &Argument,
    value: &Value,
    argument: u8,
    limits: &Limits,
) -> Result<(), Usage> {
    if spec.place == Place::Switch {
        return if *value == Value::Switch { Ok(()) } else { Err(Usage::Kind { tool: 0, form: 0, argument }) };
    }
    let correct = match spec.kind {
        Kind::Amount | Kind::Number | Kind::Count | Kind::Places | Kind::NumberRange { .. } => {
            return valid_number(spec, value, argument, limits);
        }
        Kind::NameOrNumber { .. } => {
            if let Value::Text(name) = value {
                !decimal_digits(name)
            } else {
                return valid_number(spec, value, argument, limits);
            }
        }
        Kind::NamedNumber { words, prefixes, .. } => {
            let Value::Text(bytes) = value else {
                return Err(Usage::Kind { tool: 0, form: 0, argument });
            };
            let Bound::Numbers { least, most } = bound(spec, limits) else {
                return Err(Usage::Kind { tool: 0, form: 0, argument });
            };
            named_number(bytes, words, prefixes, least, most)
        }
        Kind::Word { set } => {
            let Value::Word(index) = value else {
                return Err(Usage::Kind { tool: 0, form: 0, argument });
            };
            set.get(usize::from(*index)).is_some()
        }
        Kind::Worker => {
            let Value::Text(name) = value else { return Err(Usage::Kind { tool: 0, form: 0, argument }) };
            if !worker_name(name) {
                return Err(Usage::Kind { tool: 0, form: 0, argument });
            }
            return valid_text(spec, value, argument, limits);
        }
        Kind::Literal if *value == Value::Stdin => false,
        Kind::Literal | Kind::Text => return valid_text(spec, value, argument, limits),
        Kind::File => return valid_file(spec, value, argument, limits),
        Kind::Project { empty } => return valid_project(value, argument, empty),
        Kind::Finding => {
            let Value::Text(text) = value else { return Err(Usage::Kind { tool: 0, form: 0, argument }) };
            finding(text).is_some()
        }
        Kind::Turn => {
            let Value::Turn(_) = value else {
                return Err(Usage::Kind { tool: 0, form: 0, argument });
            };
            true
        }
        Kind::Group => {
            let Value::Group(_) = value else {
                return Err(Usage::Kind { tool: 0, form: 0, argument });
            };
            true
        }
        Kind::Order => {
            let Value::Order(_) = value else {
                return Err(Usage::Kind { tool: 0, form: 0, argument });
            };
            true
        }
        Kind::Cursor => {
            let Value::Cursor(bytes) = value else {
                return Err(Usage::Kind { tool: 0, form: 0, argument });
            };
            !bytes.is_empty()
        }
        Kind::Duration => return valid_duration(spec, value, argument, limits),
        Kind::Proposal => {
            let Value::Proposal(_) = value else {
                return Err(Usage::Kind { tool: 0, form: 0, argument });
            };
            true
        }
        Kind::Digest => {
            let Value::Digest(_) = value else {
                return Err(Usage::Kind { tool: 0, form: 0, argument });
            };
            true
        }
        Kind::Sha256 => {
            let Value::Sha256(_) = value else { return Err(Usage::Kind { tool: 0, form: 0, argument }) };
            true
        }
        Kind::Condition => {
            let Value::Condition(_) = value else {
                return Err(Usage::Kind { tool: 0, form: 0, argument });
            };
            true
        }

        Kind::Tool => {
            let Value::Tool(id) = value else {
                return Err(Usage::Kind { tool: 0, form: 0, argument });
            };
            table.get(usize::from(*id)).is_some()
        }
    };
    if correct { Ok(()) } else { Err(Usage::Kind { tool: 0, form: 0, argument }) }
}
#[must_use]
pub fn decode(table: &[&Tool], kind: Kind, bounds: Bound, word: &[u8]) -> Option<Value> {
    Some(match kind {
        Kind::Number | Kind::Count => Value::Number(number(word)?),
        Kind::Amount => Value::Number(amount(word)?),
        Kind::NumberRange { .. } | Kind::Places => {
            let Bound::Numbers { least, most } = bounds else { return None };
            Value::Number(bounded_number(word, least, most)?)
        }
        Kind::NameOrNumber { .. } => name_or_number(bounds, word)?,
        Kind::NamedNumber { words, prefixes, .. } => {
            let Bound::Numbers { least, most } = bounds else { return None };
            Value::Text(named_text(word, words, prefixes, least, most)?)
        }
        Kind::Word { set } => {
            let mut found = None;
            for (index, name) in set.iter().enumerate() {
                if *name == word {
                    found = Some(u8::try_from(index).ok()?);
                    break;
                }
            }
            Value::Word(found?)
        }
        Kind::Text | Kind::File if word == b"-" => Value::Stdin,
        Kind::Text | Kind::Literal => Value::Text(Box::from(word)),
        Kind::Worker => {
            if !worker_name(word) {
                return None;
            }
            Value::Text(Box::from(word))
        }
        Kind::File => Value::File(Box::from(word)),
        Kind::Project { empty } => {
            if word.is_empty() {
                return if empty { Some(Value::Project(Project::Name(Box::default()))) } else { None };
            }
            Value::Project(if decimal_digits(word) {
                Project::Number(u32::try_from(number(word)?).ok()?)
            } else {
                Project::Name(Box::from(word))
            })
        }
        Kind::Finding => {
            finding(word)?;
            Value::Text(Box::from(word))
        }
        Kind::Turn => Value::Turn(turn(word)?),
        Kind::Cursor => Value::Cursor(hex(word)?),
        Kind::Group => Value::Group(match word {
            b"questions" => InboxGroup::Questions,
            b"proposals" => InboxGroup::Proposals,
            b"escalations" => InboxGroup::Escalations,
            b"person-tasks" => InboxGroup::PersonTasks,
            b"results" => InboxGroup::Results,
            b"replies" => InboxGroup::Replies,
            _ => return None,
        }),
        Kind::Order => Value::Order(match word {
            b"activity" => QueryOrder::Activity,
            b"priority" => QueryOrder::Priority,
            _ => return None,
        }),
        Kind::Duration => Value::Duration(duration(word)?),
        Kind::Proposal => {
            let (tag, rest) = word.split_first()?;
            let mut slash = None;
            for (index, byte) in rest.iter().enumerate() {
                if *byte == b'/' {
                    slash = Some(index);
                    break;
                }
            }
            let slash = slash?;
            let owner = number(rest.get(..slash)?)?;
            let proposer = match tag {
                b't' => Proposer::Task(owner),
                b'p' => Proposer::Party(owner),
                _ => return None,
            };
            Value::Proposal(ProposalKey { proposer, number: number(rest.get(slash.checked_add(1)?..)?)? })
        }
        Kind::Digest => {
            if word.len() != 16 {
                return None;
            }
            let bytes = hex(word)?;
            let mut digest = [0; 8];
            for (slot, byte) in digest.iter_mut().zip(bytes.iter()) {
                *slot = *byte;
            }
            Value::Digest(digest)
        }
        Kind::Sha256 => Value::Sha256(sha256(word)?),
        Kind::Condition => Value::Condition(subject::parse(word)?),
        Kind::Tool => Value::Tool(super::tool_index(table, word)?),
    })
}
fn name_or_number(bounds: Bound, word: &[u8]) -> Option<Value> {
    if decimal_digits(word) {
        let Bound::Numbers { least, most } = bounds else { return None };
        Some(Value::Number(bounded_number(word, least, most)?))
    } else {
        Some(Value::Text(Box::from(word)))
    }
}
fn decimal_digits(word: &[u8]) -> bool {
    if word.is_empty() {
        return false;
    }
    for byte in word {
        if !byte.is_ascii_digit() {
            return false;
        }
    }
    true
}
pub(super) fn number(word: &[u8]) -> Option<u64> {
    if word.is_empty() {
        return None;
    }
    let mut value = 0_u64;
    for byte in word {
        if !byte.is_ascii_digit() {
            return None;
        }
        value = value.checked_mul(10)?.checked_add(u64::from(byte.checked_sub(b'0')?))?;
    }
    Some(value)
}
fn amount(word: &[u8]) -> Option<u64> {
    for (at, byte) in word.iter().enumerate() {
        if *byte == b'.' {
            let whole = number(word.get(..at)?)?.checked_mul(100)?;
            let digits = word.get(at.checked_add(1)?..)?;
            let fraction = if digits.len() == 1 {
                number(digits)?.checked_mul(10)?
            } else if digits.len() == 2 {
                number(digits)?
            } else {
                return None;
            };
            return whole.checked_add(fraction);
        }
    }
    number(word)?.checked_mul(100)
}
pub(crate) fn worker_name(name: &[u8]) -> bool {
    let Some(first) = name.first() else { return false };
    if !first.is_ascii_alphanumeric() {
        return false;
    }
    for byte in name {
        if !(byte.is_ascii_alphanumeric() || *byte == b'-' || *byte == b'_' || *byte == b'.') {
            return false;
        }
    }
    true
}
fn sha256(word: &[u8]) -> Option<[u8; 32]> {
    if word.len() != 64 {
        return None;
    }
    let bytes = hex(word)?;
    <[u8; 32]>::try_from(bytes.as_ref()).ok()
}
fn hex(word: &[u8]) -> Option<Box<[u8]>> {
    if word.is_empty() || word.len().checked_rem(2)? != 0 {
        return None;
    }
    let mut out = Writer::new(word.len().checked_div(2)?);
    for pair in word.as_chunks::<2>().0 {
        let mut value = 0_u8;
        for byte in pair {
            let nibble = match byte {
                b'0'..=b'9' => byte.checked_sub(b'0')?,
                b'a'..=b'f' => byte.checked_sub(b'a')?.checked_add(10)?,
                _ => return None,
            };
            value = value.checked_mul(16)?.checked_add(nibble)?;
        }
        out.put(&[value]).ok()?;
    }
    Some(out.finish())
}
#[expect(clippy::too_many_lines, reason = "value dispatch keeps named worker conditions beside the other conditions")]
pub(crate) fn encode(table: &[&Tool], kind: Kind, value: &Value) -> Result<Box<[u8]>, Unwritten> {
    Ok(match value {
        Value::Number(n) if kind == Kind::Amount => {
            let mut measured = crate::jig::render::Text::measure();
            crate::jig::render::amount(*n, &mut measured);
            let mut out = crate::jig::render::Text::writing_exact(measured);
            crate::jig::render::amount(*n, &mut out);
            out.finish()
        }
        Value::Number(n) => Box::from(Decimal::of(*n).as_bytes()),
        Value::Word(index) => {
            let Kind::Word { set } = kind else {
                return Err(Unwritten::NoForm);
            };
            Box::from(*set.get(usize::from(*index)).ok_or(Unwritten::NoForm)?)
        }
        Value::Text(text) | Value::File(text) => text.clone(),
        Value::Stdin => Box::from(b"-".as_slice()),
        Value::Project(Project::Number(n)) => Box::from(Decimal::of(u64::from(*n)).as_bytes()),
        Value::Project(Project::Name(name)) => name.clone(),
        Value::Turn(turn) => {
            let attempt = Decimal::of(turn.attempt);
            let number = Decimal::of(u64::from(turn.number));
            let prefix = attempt.as_bytes().len().checked_add(1).ok_or(Unwritten::Overflow(Overflow))?;
            let size = prefix.checked_add(number.as_bytes().len()).ok_or(Unwritten::Overflow(Overflow))?;
            let mut out = Writer::new(size);
            written(out.put(attempt.as_bytes()))?;
            written(out.put(b"."))?;
            written(out.put(number.as_bytes()))?;
            out.finish()
        }
        Value::Cursor(bytes) => hexadecimal(bytes)?,
        Value::Group(group) => Box::from(match group {
            InboxGroup::Questions => b"questions".as_slice(),
            InboxGroup::Proposals => b"proposals",
            InboxGroup::Escalations => b"escalations",
            InboxGroup::PersonTasks => b"person-tasks",
            InboxGroup::Results => b"results",
            InboxGroup::Replies => b"replies",
        }),
        Value::Order(order) => Box::from(match order {
            QueryOrder::Number | QueryOrder::Changed | QueryOrder::Ended => return Err(Unwritten::NoForm),
            QueryOrder::Activity => b"activity".as_slice(),
            QueryOrder::Priority => b"priority",
        }),
        Value::Duration(duration) => {
            if duration.as_nanos().checked_rem(1_000_000_000) != Some(0) {
                return Err(Unwritten::NoForm);
            }
            Box::from(Decimal::of(duration.as_nanos().checked_div(1_000_000_000).ok_or(Unwritten::NoForm)?).as_bytes())
        }
        Value::Proposal(proposal) => {
            let (tag, owner) = match proposal.proposer {
                Proposer::Task(n) => (b"t", n),
                Proposer::Party(n) => (b"p", n),
            };
            let owner = Decimal::of(owner);
            let number = Decimal::of(proposal.number);
            let size =
                owner.as_bytes().len().checked_add(number.as_bytes().len()).ok_or(Unwritten::Overflow(Overflow))?;
            let size = size.checked_add(2).ok_or(Unwritten::Overflow(Overflow))?;
            let mut out = Writer::new(size);
            written(out.put(tag))?;
            written(out.put(owner.as_bytes()))?;
            written(out.put(b"/"))?;
            written(out.put(number.as_bytes()))?;
            out.finish()
        }
        Value::Digest(digest) => hexadecimal(digest)?,
        Value::Sha256(digest) => hexadecimal(digest)?,
        Value::Condition(condition) => {
            let bytes: &[u8] = match condition {
                Condition::Changed => b"changed",
                Condition::Waiting => b"waiting",
                Condition::Usable => b"usable",
                Condition::Settled => b"settled",
                Condition::Updates => b"updates",
                Condition::Ended => b"ended",
                Condition::WaitingOnMe => b"waiting-on-me",
                Condition::Held => b"held",
                Condition::Replied => b"replied",
                Condition::Drained(name) | Condition::Paused(name) => {
                    let prefix: &[u8] = if let Condition::Drained(_) = condition { b"drained:" } else { b"paused:" };
                    let mut out =
                        Writer::new(name.len().checked_add(prefix.len()).ok_or(Unwritten::Overflow(Overflow))?);
                    written(out.put(prefix))?;
                    written(out.put(name))?;
                    return Ok(out.finish());
                }
                Condition::Phase(phase) => {
                    let word = subject::phase_word(*phase);
                    let mut out = Writer::new(word.len().checked_add(6).ok_or(Unwritten::Overflow(Overflow))?);
                    written(out.put(b"phase:"))?;
                    written(out.put(word))?;
                    return Ok(out.finish());
                }
            };
            Box::from(bytes)
        }
        Value::SettingsFile(_) | Value::Switch => return Err(Unwritten::NoForm),
        Value::Tool(id) => Box::from(table.get(usize::from(*id)).ok_or(Unwritten::NoForm)?.name),
    })
}

fn written(result: Result<(), Overflow>) -> Result<(), Unwritten> {
    match result {
        Ok(()) => Ok(()),
        Err(error) => Err(Unwritten::Overflow(error)),
    }
}

fn hexadecimal(bytes: &[u8]) -> Result<Box<[u8]>, Unwritten> {
    let size = bytes.len().checked_mul(2).ok_or(Unwritten::Overflow(Overflow))?;
    let mut hex = Writer::new(size);
    for byte in bytes {
        for nibble in [byte.checked_div(16).unwrap_or(0), byte.checked_rem(16).unwrap_or(0)] {
            let digit = b"0123456789abcdef".get(usize::from(nibble)).ok_or(Unwritten::NoForm)?;
            written(hex.put(&[*digit]))?;
        }
    }
    Ok(hex.finish())
}

fn valid_text(spec: &Argument, value: &Value, argument: u8, limits: &Limits) -> Result<(), Usage> {
    if *value == Value::Stdin {
        return Ok(());
    }
    let Value::Text(text) = value else {
        return Err(Usage::Kind { tool: 0, form: 0, argument });
    };
    if let Bound::Bytes { most } = bound(spec, limits)
        && skein_lib::len_u64(text.len()) > most
    {
        return Err(Usage::Long { tool: 0, form: 0, argument });
    }
    Ok(())
}

fn duration(word: &[u8]) -> Option<Duration> {
    let (digits, unit) = match word.last()? {
        b's' => (word.get(..word.len().checked_sub(1)?)?, 1_u64),
        b'm' => (word.get(..word.len().checked_sub(1)?)?, 60),
        b'h' => (word.get(..word.len().checked_sub(1)?)?, 3600),
        _ => (word, 1),
    };
    Some(Duration::from_nanos(number(digits)?.checked_mul(unit)?.checked_mul(1_000_000_000)?))
}

fn valid_project(value: &Value, argument: u8, empty: bool) -> Result<(), Usage> {
    let Value::Project(project) = value else {
        return Err(Usage::Kind { tool: 0, form: 0, argument });
    };
    match project {
        Project::Number(_) => Ok(()),
        Project::Name(name) => {
            if name.is_empty() && empty {
                return Ok(());
            }
            if name.is_empty() || decimal_digits(name) {
                Err(Usage::Kind { tool: 0, form: 0, argument })
            } else {
                Ok(())
            }
        }
    }
}

fn valid_number(spec: &Argument, value: &Value, argument: u8, limits: &Limits) -> Result<(), Usage> {
    let Value::Number(number) = value else {
        return Err(Usage::Kind { tool: 0, form: 0, argument });
    };
    if let Bound::Numbers { least, most } = bound(spec, limits)
        && (*number < least || *number > most)
    {
        return Err(if spec.kind == Kind::Count {
            Usage::Rows { tool: 0, form: 0, argument }
        } else {
            Usage::Kind { tool: 0, form: 0, argument }
        });
    }
    Ok(())
}

fn named_number(word: &[u8], words: &[&[u8]], prefixes: &[&[u8]], min: u64, max: u64) -> bool {
    if words.contains(&word) {
        return true;
    }
    for prefix in prefixes {
        if let Some(digits) = word.strip_prefix(*prefix)
            && bounded_number(digits, min, max).is_some()
        {
            return true;
        }
    }
    false
}

fn valid_duration(spec: &Argument, value: &Value, argument: u8, limits: &Limits) -> Result<(), Usage> {
    let Value::Duration(duration) = value else {
        return Err(Usage::Kind { tool: 0, form: 0, argument });
    };
    if let Bound::Seconds { least, most } = bound(spec, limits)
        && (*duration < Duration::from_secs(least) || *duration > Duration::from_secs(most))
    {
        return Err(Usage::Rows { tool: 0, form: 0, argument });
    }
    if duration.as_nanos().checked_rem(1_000_000_000) == Some(0) {
        Ok(())
    } else {
        Err(Usage::Kind { tool: 0, form: 0, argument })
    }
}
fn bounded_number(word: &[u8], min: u64, max: u64) -> Option<u64> {
    let n = number(word)?;
    if n < min || n > max { None } else { Some(n) }
}
fn named_text(word: &[u8], words: &[&[u8]], prefixes: &[&[u8]], min: u64, max: u64) -> Option<Box<[u8]>> {
    if named_number(word, words, prefixes, min, max) { Some(Box::from(word)) } else { None }
}

fn valid_file(spec: &Argument, value: &Value, argument: u8, limits: &Limits) -> Result<(), Usage> {
    if let Value::Text(_) = value
        && let Bound::Bytes { .. } = bound(spec, limits)
    {
        return valid_text(spec, value, argument, limits);
    }
    if *value == Value::Stdin {
        return Ok(());
    }
    let Value::File(_) = value else {
        return Err(Usage::Kind { tool: 0, form: 0, argument });
    };
    Ok(())
}

fn turn(word: &[u8]) -> Option<TurnName> {
    let mut dot = None;
    for (index, byte) in word.iter().enumerate() {
        if *byte == b'.' {
            dot = Some(index);
            break;
        }
    }
    let split = dot?;
    let end = split.checked_add(1)?;
    Some(TurnName { attempt: number(word.get(..split)?)?, number: u32::try_from(number(word.get(end..)?)?).ok()? })
}

/// FINDING is P/N, P a nonempty project name or a u32, N a positive u32.
#[must_use]
pub fn finding(word: &[u8]) -> Option<(&[u8], u32)> {
    let slash = skein_lib::bytes::find(word, b"/")?;
    let project = word.get(..slash)?;
    let number = u32::try_from(number(word.get(slash.checked_add(1)?..)?)?).ok()?;
    if project.is_empty() || number == 0 {
        return None;
    }
    let mut digits = true;
    for byte in project {
        digits &= byte.is_ascii_digit();
    }
    if digits {
        u32::try_from(self::number(project)?).ok()?;
    }
    Some((project, number))
}

#[cfg(test)]
#[test]
fn only_decimal_digits_make_a_setting_reference_numeric() {
    let kind = Kind::NameOrNumber { min: 0, max: u64::MAX };
    let bounds = Bound::Numbers { least: 0, most: u64::MAX };
    for (argument, expected) in [
        (b"0004".as_slice(), Some(Value::Number(4))),
        (b"18446744073709551615", Some(Value::Number(u64::MAX))),
        (b"18446744073709551616", None),
        (b"draft-4", Some(Value::Text(Box::from(b"draft-4".as_slice())))),
        (b"4draft", Some(Value::Text(Box::from(b"4draft".as_slice())))),
        (b"+4", Some(Value::Text(Box::from(b"+4".as_slice())))),
    ] {
        assert_eq!(decode(&[], kind, bounds, argument), expected, "{argument:?}");
    }
}
