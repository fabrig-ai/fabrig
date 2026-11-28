//! Attempts and sources from a page of task history (client/ctl/tools-reads.md, 1).
use super::{Limits, Printed, Text, document, lists};
use crate::jig::call::{Call, Value};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_json::writer::{Encoder, Limits as JsonLimits};
use skein_lib::Wall;

pub(super) fn print(page: &c::HistoryPage, call: &Call, json: bool, limits: &Limits) -> Printed {
    let stdout = if json {
        let bounds = JsonLimits { depth: 8, length: u32::try_from(limits.result_bytes).expect("output bound fits") };
        let mut measure = Encoder::measure(&bounds);
        encode(page, &mut measure);
        let Ok(size) = measure.measured() else { return super::failure(b"result limit", true, 1) };
        let mut out = Encoder::write(size, &bounds);
        encode(page, &mut out);
        out.finish()
    } else {
        let mut measure = Text::measure();
        text(page, call, &mut measure);
        if skein_lib::len_u64(measure.structural()) > limits.result_bytes {
            return super::failure(b"result limit", false, 1);
        }
        let mut out = Text::writing_exact(measure);
        text(page, call, &mut out);
        out.finish()
    };
    Printed { file: crate::jig::FileEnd::Keep, status: 0, stdout, stderr: Box::default() }
}

fn text(page: &c::HistoryPage, call: &Call, out: &mut Text) {
    let mut found = false;
    for row in &page.rows {
        if let c::HistoryRow::Attempt(attempt) = row {
            found = true;
            attempt_text(attempt, out);
        }
    }
    if !found {
        out.put(b"no attempts on this page\n");
    }
    if page.cut {
        out.put(b"history cut\n");
    }
    lists::next(call, 1, page.after.as_ref(), out);
}

fn attempt_text(attempt: &c::AttemptLine, out: &mut Text) {
    out.put(b"attempt ");
    out.number(attempt.attempt);
    out.put(b": claimed ");
    time(attempt.claimed, out);
    out.put(b", started ");
    time(attempt.started, out);
    out.put(b", ended ");
    time(attempt.ended, out);
    if let Some(end) = attempt.end {
        out.put(b", ");
        out.put(match end {
            c::RunEnd::Parked => b"parked",
            c::RunEnd::Finished => b"finished",
            c::RunEnd::Cancelled => b"cancelled",
            c::RunEnd::Failed { .. } => b"failed",
            c::RunEnd::Refused => b"refused",
            c::RunEnd::Suspended { .. } => b"suspended",
        });
    }
    out.put(b"\n");
    let Some(sources) = &attempt.sources else {
        out.put(b"  sources: unavailable\n");
        return;
    };
    out.put(b"  build ");
    out.put(sources.build.as_bytes());
    out.put(b", settings revision ");
    out.number(u64::from(sources.settings));
    out.put(b"\n  charters: ");
    if sources.charters == 0 {
        out.put(b"none\n");
    } else {
        out.put(b"project revision ");
        out.number(sources.charters);
        out.put(b", ");
        out.put(&sources.source);
        out.put(b"\n");
    }
    let Some(guides) = &sources.guides else {
        out.put(b"  guides: unreported\n");
        return;
    };
    if guides.is_empty() {
        out.put(b"  guides: none\n");
    }
    for guide in guides {
        guide_text(guide, out);
    }
}

fn time(at: Option<Wall>, out: &mut Text) {
    match at {
        Some(at) => document::clock::minute(at, out),
        None => out.put(b"not recorded"),
    }
}

fn guide_text(guide: &c::Guide, out: &mut Text) {
    out.put(b"  guide ");
    out.put(guide.workspace.as_bytes());
    out.put(b" at ");
    out.put(guide.commit.as_bytes());
    out.put(b": ");
    out.put(guide.path.as_bytes());
    out.put(b", ");
    out.number(u64::from(guide.bytes));
    out.put(if guide.whole { b" bytes, whole, sha256 " } else { b" bytes, partial, sha256 " });
    let digest = crate::jig::argv::value::encode(&[], crate::jig::table::Kind::Sha256, &Value::Sha256(guide.digest))
        .expect("bounded SHA-256 hexadecimal");
    out.put(&digest);
    out.put(b"\n");
}

fn encode(page: &c::HistoryPage, out: &mut Encoder) {
    out.object_start();
    out.key(b"runs");
    out.array_start();
    for row in &page.rows {
        if let c::HistoryRow::Attempt(attempt) = row {
            encode_attempt(attempt, out);
        }
    }
    out.array_end();
    out.key(b"cut");
    out.boolean(page.cut);
    out.key(b"after");
    match &page.after {
        Some(cursor) => document::hex(cursor.value.bytes(), out),
        None => out.null(),
    }
    out.object_end();
}

fn encode_attempt(attempt: &c::AttemptLine, out: &mut Encoder) {
    out.object_start();
    out.key(b"attempt");
    out.unsigned(attempt.attempt);
    for (name, at) in
        [(b"claimed".as_slice(), attempt.claimed), (b"started", attempt.started), (b"ended", attempt.ended)]
    {
        out.key(name);
        match at {
            Some(at) => out.unsigned(at.as_nanos()),
            None => out.null(),
        }
    }
    out.key(b"end");
    match attempt.end {
        Some(end) => document::run_end(end, out),
        None => out.null(),
    }
    out.key(b"sources");
    match &attempt.sources {
        Some(sources) => encode_sources(sources, out),
        None => out.null(),
    }
    out.object_end();
}

fn encode_sources(sources: &c::RunSources, out: &mut Encoder) {
    out.object_start();
    out.key(b"build");
    out.string(sources.build.as_bytes());
    out.key(b"settings");
    out.unsigned(u64::from(sources.settings));
    out.key(b"charters");
    out.unsigned(sources.charters);
    out.key(b"source");
    out.string(&sources.source);
    out.key(b"guides");
    match &sources.guides {
        Some(guides) => {
            out.array_start();
            for guide in guides {
                out.object_start();
                out.key(b"repository");
                out.string(guide.workspace.as_bytes());
                out.key(b"commit");
                out.string(guide.commit.as_bytes());
                out.key(b"path");
                out.string(guide.path.as_bytes());
                out.key(b"bytes");
                out.unsigned(u64::from(guide.bytes));
                out.key(b"whole");
                out.boolean(guide.whole);
                out.key(b"digest");
                document::hex(&guide.digest, out);
                out.object_end();
            }
            out.array_end();
        }
        None => out.null(),
    }
    out.object_end();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jig::call::Values;
    use skein_lib::List;

    #[test]
    fn task_runs_prints_attempt_sources_in_order_with_history_cursor() {
        const TASK: u64 = 17;
        let guide = c::Guide {
            workspace: "acme/shop".into(),
            commit: "abc".into(),
            path: "AGENTS.md".into(),
            digest: [0xab; 32],
            bytes: 3_201,
            whole: true,
        };
        let source = c::RunSources {
            build: "build-name".into(),
            settings: 12,
            charters: 4,
            source: b"acme/shop at abc".as_slice().into(),
            guides: Some(Box::new([
                guide.clone(),
                c::Guide {
                    workspace: "acme/lib".into(),
                    path: "docs/AGENTS.md".into(),
                    bytes: 99,
                    whole: false,
                    ..guide
                },
            ])),
        };
        let first = c::AttemptLine {
            task: TASK,
            attempt: 1,
            claimed: Some(Wall::EPOCH),
            started: Some(Wall::EPOCH),
            ended: Some(Wall::from_nanos(60_000_000_000)),
            end: Some(c::RunEnd::Finished),
            phases: None,
            sources: Some(source.clone()),
        };
        let second = c::AttemptLine {
            attempt: 2,
            end: Some(c::RunEnd::Parked),
            sources: Some(c::RunSources { guides: None, ..source.clone() }),
            ..first.clone()
        };
        let third = c::AttemptLine {
            attempt: 3,
            started: None,
            ended: None,
            end: None,
            sources: Some(c::RunSources { charters: 0, source: Box::new([]), guides: Some(Box::new([])), ..source }),
            ..first.clone()
        };
        let hold = c::HoldLine {
            task: TASK,
            position: 7,
            at: Wall::EPOCH,
            before: c::PriorPhase::Waiting,
            why: c::HoldCategory::Deadline,
            source: None,
            ending: None,
        };
        let page = c::HistoryPage {
            cut: false,
            before: None,
            after: Some(c::Cursor::new(Box::new([0xab]), 1).unwrap()),
            rows: List::from_boxed(Box::new([
                c::HistoryRow::Attempt(Box::new(first)),
                c::HistoryRow::Attempt(Box::new(second)),
                c::HistoryRow::Attempt(Box::new(third)),
                c::HistoryRow::Hold(Box::new(hold)),
            ])),
        };
        let call = Call {
            tool: crate::jig::rows::TASK_RUNS_AT,
            form: 0,
            key: None,
            values: Box::new([Some(Values::One(Value::Number(TASK))), None]),
        };
        let mut measure = Text::measure();
        text(&page, &call, &mut measure);
        let mut out = Text::writing_exact(measure);
        text(&page, &call, &mut out);
        assert_eq!(out.finish().as_ref(), b"\
attempt 1: claimed 1970-01-01T00:00Z, started 1970-01-01T00:00Z, ended 1970-01-01T00:01Z, finished\n\
\x20\x20build build-name, settings revision 12\n\
\x20\x20charters: project revision 4, acme/shop at abc\n\
\x20\x20guide acme/shop at abc: AGENTS.md, 3201 bytes, whole, sha256 abababababababababababababababababababababababababababababababab\n\
\x20\x20guide acme/lib at abc: docs/AGENTS.md, 99 bytes, partial, sha256 abababababababababababababababababababababababababababababababab\n\
attempt 2: claimed 1970-01-01T00:00Z, started 1970-01-01T00:00Z, ended 1970-01-01T00:01Z, parked\n\
\x20\x20build build-name, settings revision 12\n\
\x20\x20charters: project revision 4, acme/shop at abc\n\
\x20\x20guides: unreported\n\
attempt 3: claimed 1970-01-01T00:00Z, started not recorded, ended not recorded\n\
\x20\x20build build-name, settings revision 12\n\
\x20\x20charters: none\n\
\x20\x20guides: none\n\
read on with: task runs 17 --after ab\n");
    }
}
