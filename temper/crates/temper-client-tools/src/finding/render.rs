//! Finding lines, whole records and refusals (temper/docs/design/domain/areas.md, 7).
use super::{Act, Limits, Pending, jig, w};
use alloc::boxed::Box;
use jig::render::Text;
use skein_json::writer::Encoder;

fn holder(value: &w::Holder, out: &mut Text) {
    match value {
        w::Holder::Task(task) => {
            out.put(b"task ");
            out.number(task.number());
        }
        w::Holder::Party(party) => {
            out.put(b"party ");
            out.number(party.number());
        }
        w::Holder::Role(role) => {
            out.put(b"role ");
            out.number(u64::from(role.number()));
        }
    }
}
fn owner(value: &w::AreaOwner, out: &mut Text) {
    match value {
        w::AreaOwner::Task(task) => {
            out.put(b"task ");
            out.number(task.number());
        }
        w::AreaOwner::Party(party) => {
            out.put(b"party ");
            out.number(party.number());
        }
        w::AreaOwner::Role(role) => {
            out.put(b"role ");
            out.number(u64::from(role.number()));
        }
        w::AreaOwner::Charter(charter) => {
            out.put(b"charter ");
            out.number(u64::from(charter.number()));
        }
    }
}
fn reference(project: u32, number: u32, out: &mut Text) {
    out.number(u64::from(project));
    out.put(b"/");
    out.number(u64::from(number));
}
fn state(value: &w::FindingState) -> &'static [u8] {
    match value {
        w::FindingState::New => b"new",
        w::FindingState::Routed => b"routed",
        w::FindingState::Backlog => b"backlog",
        w::FindingState::Dropped => b"dropped",
    }
}
fn kind(value: &w::FindingKind) -> &'static [u8] {
    match value {
        w::FindingKind::Qa => b"qa",
        w::FindingKind::Flake => b"flake",
        w::FindingKind::Red => b"red",
        w::FindingKind::Review => b"review",
        w::FindingKind::Feedback => b"feedback",
    }
}
fn severity(value: &w::Severity) -> &'static [u8] {
    match value {
        w::Severity::Low => b"low",
        w::Severity::Medium => b"medium",
        w::Severity::High => b"high",
    }
}
fn row_text(project: u32, row: &w::FindingRow, out: &mut Text) {
    reference(project, row.number(), out);
    for word in [state(row.state()), kind(row.kind()), severity(row.severity()), row.area().as_deref().unwrap_or(b"-")]
    {
        out.put(b" \xc2\xb7 ");
        out.put(word);
    }
    out.put(b" \xc2\xb7 ");
    holder(row.holder(), out);
    out.put(b" \xc2\xb7 ");
    out.put(row.title());
    out.put(b"\n");
}
fn list_text(findings: &w::Findings, out: &mut Text) {
    for area in findings.areas() {
        out.put(area.name());
        out.put(b" owner ");
        owner(area.owner(), out);
        out.put(b", held by ");
        match area.holder() {
            Some(value) => holder(value, out),
            None => out.put(b"-"),
        }
        out.put(b": ");
        out.put(area.words());
        out.put(b"\n");
    }
    out.put(b"triager owner ");
    match findings.triager() {
        Some(value) => owner(value, out),
        None => out.put(b"role 0"),
    }
    out.put(b", held by ");
    holder(findings.holder(), out);
    out.put(b"\n");
    for row in findings.rows() {
        row_text(findings.project(), row, out);
    }
    if findings.cut() {
        out.put(b"[cut]\n");
    }
}
fn show_text(project: u32, detail: &w::FindingDetail, out: &mut Text) {
    row_text(project, detail.row(), out);
    out.put(b"filer: ");
    match detail.filer() {
        w::Filer::Task(task) => {
            out.put(b"task ");
            out.number(task.number());
        }
        w::Filer::Party(party) => {
            out.put(b"party ");
            out.number(party.number());
        }
    }
    out.put(b"\nfiled: ");
    out.number(detail.row().at());
    out.put(b"\n");
    if let Some(why) = detail.row().why() {
        out.put(b"handed on: ");
        out.put(why);
        out.put(b"\n");
    }
    if let Some(goal) = detail.goal() {
        out.put(b"goal: task ");
        out.number(*goal);
        out.put(b"\n");
    }
    if let Some(reason) = detail.reason() {
        out.put(b"dropped: ");
        out.put(reason);
        out.put(b"\n");
    }
    out.put(b"\n");
    out.put(detail.body());
    if !detail.body().ends_with(b"\n") {
        out.put(b"\n");
    }
}
fn success(bytes: Box<[u8]>) -> jig::Printed {
    jig::Printed { file: jig::FileEnd::Keep, status: 0, stdout: bytes, stderr: Box::default() }
}
pub(super) fn list(findings: &w::Findings, json: bool, limits: &Limits) -> jig::Printed {
    if json {
        let bounds = skein_json::writer::Limits {
            depth: 16,
            length: u32::try_from(limits.tools.result_bytes).expect("output bound"),
        };
        let mut measure = Encoder::measure(&bounds);
        list_json(findings, &mut measure);
        let Ok(size) = measure.measured() else { return jig::render::failure(b"result limit", true, 1) };
        let mut out = Encoder::write(size, &bounds);
        list_json(findings, &mut out);
        success(out.finish())
    } else {
        let mut measure = Text::measure();
        list_text(findings, &mut measure);
        let mut out = Text::writing_exact(measure);
        list_text(findings, &mut out);
        success(out.finish())
    }
}
pub(super) fn show(project: u32, detail: &w::FindingDetail, json: bool, limits: &Limits) -> jig::Printed {
    if json {
        let bounds = skein_json::writer::Limits {
            depth: 16,
            length: u32::try_from(limits.tools.result_bytes).expect("output bound"),
        };
        let mut measure = Encoder::measure(&bounds);
        show_json(project, detail, &mut measure);
        let Ok(size) = measure.measured() else { return jig::render::failure(b"result limit", true, 1) };
        let mut out = Encoder::write(size, &bounds);
        show_json(project, detail, &mut out);
        success(out.finish())
    } else {
        let mut measure = Text::measure();
        show_text(project, detail, &mut measure);
        let mut out = Text::writing_exact(measure);
        show_text(project, detail, &mut out);
        success(out.finish())
    }
}
fn act_word(act: Act) -> &'static [u8] {
    match act {
        Act::File => b"filed",
        Act::Hand => b"handed",
        Act::Route => b"routed",
        Act::Backlog => b"backlog",
        Act::Drop => b"dropped",
        Act::List | Act::Show => unreachable!("act result"),
    }
}
fn acted_text(pending: &Pending, held: Option<&w::Holder>, out: &mut Text) {
    out.put(act_word(pending.act));
    out.put(b": ");
    reference(pending.project, pending.number, out);
    if let Some(held) = held {
        out.put(b", held by ");
        holder(held, out);
    }
    out.put(b"\n");
}
pub(super) fn acted(pending: &Pending, held: Option<&w::Holder>, limits: &Limits) -> jig::Printed {
    if pending.json {
        let bounds = skein_json::writer::Limits {
            depth: 4,
            length: u32::try_from(limits.tools.result_bytes).expect("output bound"),
        };
        let mut measure = Encoder::measure(&bounds);
        acted_json(pending, held, &mut measure);
        let Ok(size) = measure.measured() else { return jig::render::failure(b"result limit", true, 1) };
        let mut out = Encoder::write(size, &bounds);
        acted_json(pending, held, &mut out);
        success(out.finish())
    } else {
        let mut measure = Text::measure();
        acted_text(pending, held, &mut measure);
        let mut out = Text::writing_exact(measure);
        acted_text(pending, held, &mut out);
        success(out.finish())
    }
}
pub(super) fn refused(pending: &Pending, why: &w::FindingRefusal) -> jig::Printed {
    let mut measure = Text::measure();
    refusal_text(pending, why, &mut measure);
    let mut out = Text::writing_exact(measure);
    refusal_text(pending, why, &mut out);
    jig::render::failure(&out.finish(), pending.json, 2)
}
fn refusal_text(pending: &Pending, why: &w::FindingRefusal, out: &mut Text) {
    match why {
        w::FindingRefusal::NoArea(areas) => {
            out.put(b"no area ");
            out.put(pending.area.as_deref().unwrap_or(b"-"));
            out.put(b" in project ");
            out.number(u64::from(pending.project));
            out.put(b"; its areas: ");
            for (index, area) in areas.areas().iter().enumerate() {
                if index != 0 {
                    out.put(b", ");
                }
                out.put(area);
            }
        }
        w::FindingRefusal::NotYours(value) => {
            out.put(b"finding ");
            reference(pending.project, pending.number, out);
            out.put(b" is not yours: its owner is ");
            holder(value.holder(), out);
        }
        w::FindingRefusal::Full(value) => {
            out.put(b"the project's findings are full: ");
            out.number(u64::from(value.count()));
            out.put(b" new");
        }
        w::FindingRefusal::NoFinding => out.put(b"no finding"),
        w::FindingRefusal::NoGoal => out.put(b"no live goal of this project"),
        w::FindingRefusal::Ended => out.put(b"finding already routed or dropped"),
        w::FindingRefusal::Words(value) => {
            out.put(b"finding words past their bounds: ");
            out.put(value.field());
        }
    }
}
fn held_json(value: &w::Holder, out: &mut Encoder) {
    out.object_start();
    match value {
        w::Holder::Task(value) => {
            out.key(b"task");
            out.unsigned(value.number());
        }
        w::Holder::Party(value) => {
            out.key(b"party");
            out.unsigned(value.number());
        }
        w::Holder::Role(value) => {
            out.key(b"role");
            out.unsigned(u64::from(value.number()));
        }
    }
    out.object_end();
}
fn owner_json(value: &w::AreaOwner, out: &mut Encoder) {
    out.object_start();
    match value {
        w::AreaOwner::Task(value) => {
            out.key(b"task");
            out.unsigned(value.number());
        }
        w::AreaOwner::Party(value) => {
            out.key(b"party");
            out.unsigned(value.number());
        }
        w::AreaOwner::Role(value) => {
            out.key(b"role");
            out.unsigned(u64::from(value.number()));
        }
        w::AreaOwner::Charter(value) => {
            out.key(b"charter");
            out.unsigned(u64::from(value.number()));
        }
    }
    out.object_end();
}
fn row_fields(project: u32, row: &w::FindingRow, out: &mut Encoder) {
    out.key(b"project");
    out.unsigned(u64::from(project));
    out.key(b"number");
    out.unsigned(u64::from(row.number()));
    for (key, value) in [
        (b"state".as_slice(), state(row.state())),
        (b"kind", kind(row.kind())),
        (b"severity", severity(row.severity())),
        (b"title", row.title()),
    ] {
        out.key(key);
        out.string(value);
    }
    out.key(b"area");
    optional(row.area().as_deref(), out);
    out.key(b"holder");
    held_json(row.holder(), out);
    out.key(b"at");
    out.unsigned(row.at());
    out.key(b"why");
    optional(row.why().as_deref(), out);
}
fn optional(value: Option<&[u8]>, out: &mut Encoder) {
    match value {
        Some(value) => out.string(value),
        None => out.null(),
    }
}
fn list_json(findings: &w::Findings, out: &mut Encoder) {
    out.object_start();
    out.key(b"project");
    out.unsigned(u64::from(findings.project()));
    out.key(b"areas");
    out.array_start();
    for area in findings.areas() {
        out.object_start();
        out.key(b"name");
        out.string(area.name());
        out.key(b"words");
        out.string(area.words());
        out.key(b"owner");
        owner_json(area.owner(), out);
        out.key(b"holder");
        match area.holder() {
            Some(value) => held_json(value, out),
            None => out.null(),
        }
        out.object_end();
    }
    out.array_end();
    out.key(b"triager");
    match findings.triager() {
        Some(value) => owner_json(value, out),
        None => out.null(),
    }
    out.key(b"holder");
    held_json(findings.holder(), out);
    out.key(b"rows");
    out.array_start();
    for row in findings.rows() {
        out.object_start();
        row_fields(findings.project(), row, out);
        out.object_end();
    }
    out.array_end();
    out.key(b"cut");
    out.boolean(findings.cut());
    out.object_end();
}
fn show_json(project: u32, detail: &w::FindingDetail, out: &mut Encoder) {
    out.object_start();
    row_fields(project, detail.row(), out);
    out.key(b"body");
    out.string(detail.body());
    out.key(b"filer");
    out.object_start();
    match detail.filer() {
        w::Filer::Task(task) => {
            out.key(b"task");
            out.unsigned(task.number());
        }
        w::Filer::Party(party) => {
            out.key(b"party");
            out.unsigned(party.number());
        }
    }
    out.object_end();
    out.key(b"goal");
    match detail.goal() {
        Some(goal) => out.unsigned(*goal),
        None => out.null(),
    }
    out.key(b"reason");
    optional(detail.reason().as_deref(), out);
    out.object_end();
}
fn acted_json(pending: &Pending, held: Option<&w::Holder>, out: &mut Encoder) {
    out.object_start();
    out.key(b"act");
    out.string(act_word(pending.act));
    out.key(b"project");
    out.unsigned(u64::from(pending.project));
    out.key(b"number");
    out.unsigned(u64::from(pending.number));
    if let Some(held) = held {
        out.key(b"holder");
        held_json(held, out);
    }
    out.object_end();
}
