//! A goal's task document and reds after landing (client/ctl/findings.md, 4).
use crate::{
    finding,
    jig::{
        self,
        call::number,
        render::{self, Text},
        rows,
        runner::reads::Reading,
    },
};
use jig_client::{Client, Handle, Readout, Watchable, contract as c};
use skein_json::writer::Encoder;
use temper_client_domain_forge::forge::findings::{self, Stands};
use temper_client_protocol_wire::{self as wire, v1 as w};

pub(crate) fn next(reading: &Reading) -> Option<(u32, Watchable)> {
    if reading.called.call.tool != rows::GOAL_SHOW_AT || reading.pages.contains_key(&2) {
        return None;
    }
    let Readout::Jig(c::ReadResult::Value(value)) = reading.pages.get(&1)? else { return None };
    let (task, _) = render::read_task(value)?;
    let goal = number(&reading.called.call, 0)?;
    Some((2, finding::filtered_subject(task.project, w::FindingsFilter::Goal(w::FindingsGoal { goal }))))
}
pub(crate) fn print(
    reading: &Reading,
    me: Option<&c::Me>,
    client: &Client,
    handle: Handle,
    limits: &jig::Limits,
    parts: &dyn render::Parts,
) -> jig::Printed {
    let json = reading.called.json;
    let Some(page) = reading.pages.get(&1) else { return render::failure(b"missing task document", json, 1) };
    let Readout::Jig(c::ReadResult::Value(value)) = page else {
        return render::read(page, client, handle, json, limits, parts);
    };
    let Some((task, _)) = render::read_task(value) else {
        return render::failure(b"unexpected task document", json, 1);
    };
    if task.chip.task != number(&reading.called.call, 0).expect("task argument") {
        return render::failure(b"unexpected task", json, 1);
    }
    let findings = match reading.pages.get(&2) {
        Some(Readout::Family { family: wire::FORGE, record: Some(bytes) }) => {
            wire::decode_findings(bytes, &w::CEILINGS)
        }
        Some(Readout::Limit | Readout::NotServed { .. } | Readout::Jig(_) | Readout::Family { .. }) | None => None,
    };
    let Some(findings) = findings else { return render::failure(b"unexpected findings document", json, 1) };
    if findings.project != task.project {
        return render::failure(b"unexpected findings project", json, 1);
    }
    let document = render::document::print(render::document::Document::TaskRead(reading, me), json, limits, parts);
    if document.status != 0 {
        return document;
    }
    let rows = findings::uncleared_first(findings.rows.as_slice());
    let suffix = if json {
        let bounds =
            skein_json::writer::Limits { depth: 16, length: u32::try_from(limits.result_bytes).expect("result bound") };
        let mut measure = Encoder::measure(&bounds);
        encode(findings.project, &rows, &mut measure);
        let Ok(size) = measure.measured() else { return render::failure(b"result limit", json, 1) };
        let mut out = Encoder::write(size, &bounds);
        encode(findings.project, &rows, &mut out);
        out.finish()
    } else {
        let mut measure = Text::measure();
        text(findings.project, &rows, findings.cut, &mut measure);
        let mut out = Text::writing_exact(measure);
        text(findings.project, &rows, findings.cut, &mut out);
        out.finish()
    };
    let chunks: &[&[u8]] = if json {
        &[
            b"{\"task\":",
            &document.stdout,
            b",\"reds\":",
            &suffix,
            if findings.cut { b",\"cut\":true}" } else { b",\"cut\":false}" },
        ]
    } else {
        &[&document.stdout, &suffix]
    };
    let bytes = skein_lib::bytes::concat(chunks);
    if skein_lib::len_u64(bytes.len()) > limits.result_bytes {
        return render::failure(b"result limit", json, 1);
    }
    jig::Printed { stdout: bytes, ..document }
}
fn text(project: u32, rows: &[w::FindingRow], cut: bool, out: &mut Text) {
    out.put(b"reds after landing:");
    if rows.is_empty() {
        out.put(b" none");
    }
    out.put(b"\n");
    for row in rows {
        out.put(b"  ");
        out.number(u64::from(project));
        out.put(b"/");
        out.number(u64::from(row.number));
        out.put(b" ");
        out.put(&row.title);
        out.put(b" \xc2\xb7 ");
        match findings::stands(row) {
            Stands::Cleared { tier, run } => {
                out.put(b"cleared by ");
                out.put(&tier);
                out.put(b" run ");
                out.number(run);
            }
            Stands::Waiting => out.put(b"waiting for triage"),
            Stands::Repairing(goal) => {
                out.put(b"being repaired");
                if let Some(goal) = goal {
                    out.put(b" by goal ");
                    out.number(goal);
                }
            }
            Stands::Backlog => out.put(b"in the backlog"),
            Stands::Dropped(reason) => {
                out.put(b"dropped: ");
                out.put(&reason);
            }
        }
        out.put(b"\n");
    }
    if cut {
        out.put(b"[cut]\n");
    }
}
fn encode(project: u32, rows: &[w::FindingRow], out: &mut Encoder) {
    out.array_start();
    for row in rows {
        out.object_start();
        finding::render::row_fields(project, row, out);
        out.key(b"stands");
        out.object_start();
        out.key(b"kind");
        match findings::stands(row) {
            Stands::Cleared { tier, run } => {
                out.string(b"cleared");
                out.key(b"tier");
                out.string(&tier);
                out.key(b"run");
                out.unsigned(run);
            }
            Stands::Waiting => out.string(b"waiting"),
            Stands::Repairing(goal) => {
                out.string(b"repairing");
                if let Some(goal) = goal {
                    out.key(b"goal");
                    out.unsigned(goal);
                }
            }
            Stands::Backlog => out.string(b"backlog"),
            Stands::Dropped(reason) => {
                out.string(b"dropped");
                out.key(b"reason");
                out.string(&reason);
            }
        }
        out.object_end();
        out.object_end();
    }
    out.array_end();
}
