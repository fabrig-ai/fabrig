//! Supplied activity and run text; the host is the sole source of provisional words.
use super::{Text, lists, turn};
use crate::jig::{Limits, call::Call};
use alloc::boxed::Box;
use jig_client::{
    contract as c,
    thread::{Line, Thread},
};
pub(super) fn page(page: &c::ActivityPage, call: &Call, limits: &Limits, out: &mut Text) {
    let task = crate::jig::call::number(call, 0).expect("activity task");
    if call.tool == crate::jig::rows::ACTIVITY_AT {
        out.put(b"brief: ");
        super::task_source::call(task, c::TaskTextField::Specification, 0, out);
        out.put(b"\ninstructions: ");
        super::task_source::call(task, c::TaskTextField::Instructions, 0, out);
        out.put(b"\n");
    }
    let rows = lists::activity_limit(call, limits.rows);
    let first = page.rows.len().saturating_sub(rows);
    for row in page.rows.as_slice().iter().skip(usize::try_from(first).expect("rows fit")) {
        line(row, task, call, limits, out);
    }
    match read_on(page, call, limits.rows) {
        Some(next) => {
            out.put(if call.tool == 3 { b"read with: " } else { b"read on with: " });
            out.put(&next);
            out.put(b"\n");
        }
        None if call.tool == 5 => out.put(b"no more\n"),
        None => {}
    }
}
pub(super) fn read_on(page: &c::ActivityPage, call: &Call, rows: u32) -> Option<Box<[u8]>> {
    if page.rows.len() > lists::activity_limit(call, rows) {
        let mut measure = Text::measure();
        replay(call, &mut measure);
        let mut out = Text::writing_exact(measure);
        replay(call, &mut out);
        return Some(out.finish());
    }
    if call.tool == 5
        && let Some(cursor) = &page.before
    {
        return Some(lists::read_on(call, 1, cursor));
    }
    None
}
fn replay(call: &Call, out: &mut Text) {
    if call.tool == 3 {
        out.put(b"task activity ");
        out.number(crate::jig::call::number(call, 0).expect("task number"));
    } else {
        let mut call = call.clone();
        *call.values.get_mut(2).expect("activity limit") = None;
        lists::write_call(&call, out);
    }
}

fn line(row: &c::DecodedActivityLine, task: u64, call: &Call, limits: &Limits, out: &mut Text) {
    match &row.decoded {
        Some(value) => turn::text(task, value.name, &value.turn, call, limits, out),
        None => owner(&row.source, out),
    }
}
fn owner(row: &c::ActivityLine, out: &mut Text) {
    out.number(row.position);
    out.put(b" \xc2\xb7 ");
    match &row.line {
        c::Activity::Words { words, .. } => out.words(words.bytes()),
        c::Activity::Heard(c::Word { kind: c::MessageKind::QuestionAnswered { question }, .. }) => {
            out.put(b"You answered question ");
            out.number(*question);
            out.put(b" in your inbox");
        }
        c::Activity::Heard(word) | c::Activity::Notified { word, .. } => message_words(&word.kind, &word.words, out),
        c::Activity::RunBegan { attempt, .. } => {
            out.put(b"run ");
            out.number(*attempt);
            out.put(b" began");
        }
        c::Activity::RunEnded { attempt, end, .. } => run_ended(*attempt, *end, out),
        c::Activity::Turn { attempt, turn: number, .. } => {
            out.put(b"turn ");
            turn::name(c::TurnName { attempt: *attempt, number: *number }, out);
            out.put(b" unreadable");
        }
        c::Activity::Decided { revision, .. } => {
            out.put(b"decision ");
            out.number(*revision);
        }
        c::Activity::NowWritten { line, .. } => {
            out.put(b"Wrote its Now: ");
            out.put(line.bytes());
        }
        c::Activity::Stood { .. } | c::Activity::Kept { .. } => out.put(b"parked"),
        c::Activity::TalkOpened { party } => {
            out.put(b"Talk opened by party ");
            out.number(*party);
        }
        c::Activity::TalkEnded { lapsed, .. } => out.put(if *lapsed { b"Talk lapsed" } else { b"Talk ended" }),
        c::Activity::SessionEnded { end, .. } => {
            out.put(b"session ended");
            match end {
                c::SessionEnd::Talk => out.put(b" by a talk"),
                c::SessionEnd::Left => out.put(b" as its person left"),
                c::SessionEnd::Quiet
                | c::SessionEnd::Bound
                | c::SessionEnd::Unoffered
                | c::SessionEnd::Fresh
                | c::SessionEnd::Closed
                | c::SessionEnd::Rules => {}
            }
        }
        c::Activity::Milestone { .. } => out.put(b"phase changed"),
    }
    out.put(b"\n");
}
fn run_ended(attempt: u64, end: c::RunEnd, out: &mut Text) {
    out.put(b"run ");
    out.number(attempt);
    out.put(b" ended");
    match end {
        c::RunEnd::Failed { invalid_reason: Some(reason), .. } => out.put(match reason {
            c::InvalidReason::Form => b": its result broke its contract (form)",
            c::InvalidReason::Verdict => b": its result broke its contract (verdict)",
            c::InvalidReason::Words => b": its result broke its contract (words)",
            c::InvalidReason::Change => b": its result broke its contract (change)",
            c::InvalidReason::Followups => b": its result broke its contract (followups)",
            c::InvalidReason::UnreadableResult => b": its result couldn't be read",
        }),
        c::RunEnd::Parked
        | c::RunEnd::Finished
        | c::RunEnd::Cancelled
        | c::RunEnd::Refused
        | c::RunEnd::Failed { invalid_reason: None, .. } => {}
    }
}
fn message_words(kind: &c::MessageKind, words: &c::Text, out: &mut Text) {
    if let c::MessageKind::Result(c::WordResult::Verdict { code, .. }) = kind {
        out.put(b"verdict ");
        out.number(u64::from(*code));
        out.put(b" \xc2\xb7 ");
    }
    out.words(words.bytes());
}

pub(super) fn chat(thread: &Thread, call: &Call, limits: &Limits, out: &mut Text) {
    for entry in thread.lines().into_boxed() {
        match entry {
            Line::Turn { name, turn: value, .. } => {
                turn::text(thread.task, name, &c::TurnRead::Decoded(value.clone()), call, limits, out);
            }
            Line::Undecoded { name, why, .. } => {
                turn::text(thread.task, name, &c::TurnRead::Undecodable { why }, call, limits, out);
            }
            Line::Words { by, words, .. } => {
                if let Some(by) = by {
                    out.words(by.name.bytes());
                    out.put(b": ");
                }
                out.words(words);
                out.put(b"\n");
            }
            Line::Provisional { name, text, .. } => {
                out.put(b"provisional ");
                turn::name(name.turn(), out);
                out.put(b": ");
                out.words(text);
                out.put(b"\n");
            }

            Line::Activity { line: row } => line(row, thread.task, call, limits, out),
        }
    }
    for part in thread.parts(limits.rows, thread.live.as_slice()).into_boxed() {
        turn::reference_text(&part, out);
    }
    match &thread.lines.edges.after {
        jig_client::thread::window::Edge::More(cursor) => lists::next(call, 1, Some(cursor), out),
        jig_client::thread::window::Edge::End | jig_client::thread::window::Edge::Dropped => {
            lists::next(call, 1, None, out);
        }
    }
}
pub(super) fn run(delivery: &c::Delivery, text: Option<&c::RunText>, out: &mut Text) {
    out.put(b"run: ");
    match delivery {
        c::Delivery::Run(run) => {
            out.number(run.attempt);
            out.put(b"\n");
        }
        c::Delivery::RunState(run) => {
            match run.attempt {
                Some(attempt) => out.number(attempt),
                None => out.put(b"none"),
            }
            out.put(b"\n");
            if let Some(call) = &run.call {
                out.put(b"call: ");
                out.put(call.tool.bytes());
                out.put(b"\n");
                out.words(call.input.bytes());
                out.put(b"\n");
            }
        }

        c::Delivery::Project(_)
        | c::Delivery::Me(_)
        | c::Delivery::Task(_)
        | c::Delivery::Proposal(_)
        | c::Delivery::Tree(_)
        | c::Delivery::Chips(_)
        | c::Delivery::Inbox(_)
        | c::Delivery::Tasks(_)
        | c::Delivery::Settings(_)
        | c::Delivery::Deployment(_)
        | c::Delivery::Terminal(_)
        | c::Delivery::Accounts(_) => unreachable!("run watch delivers a run"),
    }
    if let Some(text) = text {
        out.put(b"provisional: ");
        out.words(text.text.bytes());
        out.put(b"\n");
    }
}
