//! Activity summaries and whole turns (temper/docs/design/domain/agent/ask-temper.md, 1 and 4).
use super::{count, join, words};
use alloc::boxed::Box;
use jig_domain_core::reads::{Answer, EndedValue, PageRow, PageTurn, Why};
use jig_domain_tasks as t;
use jig_protocol_transcript as c;
use skein_lib::{List, bytes};
use temper_engine_domain::SeeNames;

fn name(attempt: u64, turn: u32) -> Box<[u8]> {
    join(&[&words::number(attempt), b".", &words::number(u64::from(turn))])
}

// Use the client's whole-turn contract and its lossless receiving shape. The
// wire's carried-turn ceiling bounds input; the decoded text can include both
// a call's outcome and its result block (jig/docs/design/client/domain/client/conversations.md).
fn decode(
    record: &jig_domain_core::TurnRecord,
    inspect: bool,
    withheld: bool,
) -> Result<c::Turn, jig_protocol_transcript::Undecodable> {
    let bound = jig_client_protocol_wire::v19::CEILINGS.turn_carried_bytes;
    let mut limits = jig_protocol_transcript::Limits::new(
        bound,
        c::TurnLimits {
            messages: 0,
            blocks: 0,
            text: bound.checked_mul(2).expect("carried turn ceiling"),
            outcome: bound,
        },
    );
    limits.turn.messages = limits.smith.turn_messages.checked_add(1).expect("system message and transcript");
    limits.turn.blocks = limits.smith.message_blocks;
    let projected;
    let transcript = if withheld {
        projected = c::withhold(&limits, &record.transcript, &crate::private())?;
        projected.as_ref()
    } else {
        record.transcript.as_ref()
    };
    if inspect {
        jig_protocol_transcript::decode(&limits, transcript)
    } else {
        jig_protocol_transcript::conversation(&limits, transcript)
    }
}

pub(super) fn activity(task: u64, answer: &Answer, names: &SeeNames, lines: &mut List<Box<[u8]>>) {
    let Answer::Page(page) = answer else {
        lines.push(b"activity: not read\n".as_slice().into()).expect("unread activity");
        return;
    };
    let more = page.before.is_some() || page.next.is_some() || page.cut;
    lines
        .push(join(&[
            b"activity of task ",
            &words::number(task),
            b": ",
            &count(page.rows.len(), more),
            b" events, oldest first\n",
        ]))
        .expect("activity heading");
    // The core supplies its end-page in descending commit position, not wall time.
    for row in page.rows.iter().rev() {
        let PageRow::Activity { line, turn } = row else { continue };
        let (who, what) = match &line.line {
            t::Line::Words { by, words: text, .. } => (party(*by, names), words::short(text, false)),
            t::Line::Heard(word) | t::Line::Notified { word, .. } => {
                (party(word.from, names), words::short(&word.words, false))
            }
            t::Line::RunBegan { attempt, .. } => {
                (b"its run".as_slice().into(), join(&[b"run ", &words::number(*attempt), b" began"]))
            }
            t::Line::RunEnded { attempt, .. } => {
                (b"its run".as_slice().into(), join(&[b"run ", &words::number(*attempt), b" ended"]))
            }
            t::Line::Turn { attempt, turn: number, .. } => {
                activity_turn(task, *attempt, *number, line.at, turn.as_ref(), lines);
                continue;
            }
            t::Line::Decided { revision, .. } => {
                (b"its run".as_slice().into(), join(&[b"decision ", &words::number(*revision)]))
            }
            t::Line::Milestone { .. } => (b"its run".as_slice().into(), b"phase changed".as_slice().into()),
            t::Line::Stood { .. } | t::Line::SessionEnded { .. } | t::Line::Kept { .. } => {
                (b"its run".as_slice().into(), b"parked".as_slice().into())
            }
        };
        lines
            .push(join(&[&words::time(line.at), " · ".as_bytes(), &who, " · ".as_bytes(), &what, b"\n"]))
            .expect("activity line");
    }
    if more {
        lines.push(b"earlier events not read\n".as_slice().into()).expect("earlier activity");
    }
}

fn party(value: t::Party, names: &SeeNames) -> Box<[u8]> {
    match value {
        t::Party::Person(number) => words::person(number, names),
        t::Party::Task(number) => join(&[b"task ", &words::number(number)]),
        t::Party::Deployment { .. } => b"the deployment".as_slice().into(),
    }
}

fn activity_turn(
    task: u64,
    attempt: u64,
    turn: u32,
    at: skein_lib::Wall,
    source: Option<&PageTurn>,
    lines: &mut List<Box<[u8]>>,
) {
    let heading = join(&[&words::time(at), " · its run · turn ".as_bytes(), &name(attempt, turn)]);
    let withheld = match source {
        Some(PageTurn::Withheld(_)) => true,
        Some(PageTurn::Carried(_) | PageTurn::TooLarge { .. } | PageTurn::Unread) | None => false,
    };
    match source {
        Some(PageTurn::Carried(record) | PageTurn::Withheld(record)) => match decode(record, false, withheld) {
            Ok(decoded) => {
                let mut calls = 0_u64;
                for message in &decoded.messages {
                    for block in &message.blocks {
                        match block {
                            c::Block::Call(_) => calls = calls.checked_add(1).expect("bounded calls"),
                            c::Block::Text(_) | c::Block::Result { .. } | c::Block::Opaque { .. } => {}
                        }
                    }
                }
                lines
                    .push(join(&[
                        &heading,
                        " · ".as_bytes(),
                        &words::number(calls),
                        if calls == 1 { b" call\n" } else { b" calls\n" },
                    ]))
                    .expect("turn summary");
                for message in &decoded.messages {
                    for block in &message.blocks {
                        match block {
                            c::Block::Call(call) => lines
                                .push(join(&[b"  ", call.tool.bytes(), b": ", &outcome(call), b"\n"]))
                                .expect("call summary"),
                            c::Block::Text(text) => lines
                                .push(join(&[b"  ", &words::short(text.bytes(), false), b"\n"]))
                                .expect("turn words"),
                            c::Block::Result { .. } | c::Block::Opaque { .. } => {}
                        }
                    }
                }
            }
            Err(why) => unread_activity(task, attempt, turn, &heading, &problem(why, record.transcript.len()), lines),
        },
        Some(PageTurn::TooLarge { bytes, .. }) => unread_activity(
            task,
            attempt,
            turn,
            &heading,
            &join(&[&words::number(*bytes), b" bytes, too large for this page"]),
            lines,
        ),
        Some(PageTurn::Unread) | None => unread_activity(task, attempt, turn, &heading, b"not read", lines),
    }
}

fn outcome(call: &c::Call) -> Box<[u8]> {
    match &call.outcome {
        Some(text) => {
            // A see reply begins with the address's heading; summarize that read
            // by its address instead of reproducing its page's contents.
            if !call.failed
                && call.tool.bytes() == b"see"
                && let Ok(value) = crate::json::parse(call.input.bytes())
                && let Ok(address) = value.required(b"address")
                && let Ok(address) = address.text(b"address")
            {
                return join(&[b"read ", &words::short(&address, false)]);
            }
            words::short(text.bytes(), false)
        }
        None => b"result not read".as_slice().into(),
    }
}

fn unread_activity(task: u64, attempt: u64, turn: u32, heading: &[u8], why: &[u8], lines: &mut List<Box<[u8]>>) {
    lines
        .push(join(&[
            heading,
            " · ".as_bytes(),
            why,
            b"\n  see: task ",
            &words::number(task),
            b" turn ",
            &name(attempt, turn),
            b"\n",
        ]))
        .expect("unread turn");
}

pub(super) fn turn(task: u64, attempt: u64, turn: u32, answer: &Answer, lines: &mut List<Box<[u8]>>) {
    let heading = join(&[b"turn ", &name(attempt, turn), b" of task ", &words::number(task)]);
    if let Answer::None(Why::TooLarge { bytes }) = answer {
        lines
            .push(join(&[&heading, b": ", &words::number(*bytes), b" bytes, past what one read holds\n"]))
            .expect("oversize turn");
        return;
    }
    let record = match answer {
        Answer::Ended(ended) => match &ended.value {
            EndedValue::Turn(record) => Some((record, false)),
            EndedValue::WithheldTurn(record) => Some((record, true)),
            EndedValue::Task { .. }
            | EndedValue::Proposal(_)
            | EndedValue::PersonProposal { .. }
            | EndedValue::Escalation { .. }
            | EndedValue::Question { .. }
            | EndedValue::PersonTask { .. }
            | EndedValue::Reply(_) => None,
        },
        Answer::None(_)
        | Answer::Document(_)
        | Answer::TaskText(_)
        | Answer::Setting(_)
        | Answer::Revision(_)
        | Answer::Revisions(_)
        | Answer::Tasks(_)
        | Answer::Page(_) => None,
    };
    let Some((record, withheld)) = record else {
        lines.push(join(&[&heading, b": not read\n"])).expect("unread turn");
        return;
    };
    let decoded = match decode(record, true, withheld) {
        Ok(decoded) => decoded,
        Err(why) => {
            lines
                .push(join(&[&heading, b": ", &problem(why, record.transcript.len()), b"\n"]))
                .expect("unreadable turn");
            return;
        }
    };
    lines.push(join(&[&heading, " · ".as_bytes(), &words::money(decoded.spent), b"\n"])).expect("turn heading");
    for message in &decoded.messages {
        for block in &message.blocks {
            match block {
                c::Block::Text(text) => {
                    if message.speaker == c::Speaker::System {
                        brief(text.bytes(), lines);
                    } else {
                        lines.push(join(&[text.bytes(), b"\n"])).expect("turn words");
                    }
                }
                c::Block::Call(call) => {
                    let prefix = bytes::utf8_prefix(call.input.bytes(), 512);
                    lines
                        .push(join(&[
                            call.tool.bytes(),
                            b": ",
                            prefix,
                            if prefix.len() < call.input.bytes().len() { "…\n".as_bytes() } else { b"\n" },
                        ]))
                        .expect("call input");
                }
                c::Block::Result { summary, .. } => lines.push(join(&[summary.bytes(), b"\n"])).expect("call result"),
                c::Block::Opaque { dialect, size } => lines
                    .push(join(&[b"opaque ", dialect.bytes(), b": ", &words::number(*size), b" bytes\n"]))
                    .expect("opaque block"),
            }
        }
    }
    if decoded.cut {
        lines.push(b"turn text not read whole\n".as_slice().into()).expect("cut turn");
    }
}

fn problem(why: jig_protocol_transcript::Undecodable, length: usize) -> Box<[u8]> {
    match why {
        jig_protocol_transcript::Undecodable::TooLarge => {
            join(&[&words::number(skein_lib::len_u64(length)), b" bytes, past what one read holds"])
        }
        jig_protocol_transcript::Undecodable::Version { .. }
        | jig_protocol_transcript::Undecodable::Malformed
        | jig_protocol_transcript::Undecodable::Unknown => b"unreadable turn".as_slice().into(),
    }
}

fn brief(text: &[u8], lines: &mut List<Box<[u8]>>) {
    // Brief parts are text, with Markdown headings, in the client's supplied
    // system block. Keep each heading and its first nonempty line.
    let mut first = true;
    let mut start = 0;
    for (at, byte) in text.iter().chain(b"\n").enumerate() {
        if *byte != b'\n' {
            continue;
        }
        let line = text.get(start..at).expect("brief line").trim_ascii();
        start = at.checked_add(1).expect("brief offset");
        if line.starts_with(b"#") {
            lines.push(join(&[line, b"\n"])).expect("brief heading");
            first = true;
        } else if first && !line.is_empty() {
            lines.push(join(&[&words::short(line, false), b"\n"])).expect("brief first line");
            first = false;
        }
    }
}
