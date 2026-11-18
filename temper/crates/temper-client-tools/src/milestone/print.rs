//! Milestone text and records (client/ctl/milestones.md, 4).
use crate::jig::{
    Limits, Printed,
    render::{self, Text},
};
use crate::jig::{
    call::{Call, Project, Value, Values, value},
    rows,
    runner::reads::Reading,
};
use alloc::{boxed::Box, collections::BTreeMap};
use jig_client::{Client, Handle, Readout, contract as c};
use skein_json::writer::{Encoder, Limits as JsonLimits};
use skein_lib::Wall;
use temper_client_domain_forge::{forge, milestones};
use temper_client_protocol_wire::v1 as w;

pub(crate) fn print(
    reading: &Reading,
    me: Option<&c::Me>,
    client: &Client,
    handle: Handle,
    limits: &Limits,
    parts: &dyn render::Parts,
) -> Printed {
    for (local, page) in &reading.pages {
        let valid = match (reading.called.call.tool, *local, page) {
            (rows::MILESTONE_LIST_AT, 1, Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(_))))
            | (rows::MILESTONE_SHOW_AT, 2 | 6, Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(_))))
            | (rows::MILESTONE_SHOW_AT, 4, Readout::Jig(c::ReadResult::Value(c::ReadValue::Project(_)))) => true,
            (rows::MILESTONE_SHOW_AT, 1 | 5, Readout::Jig(c::ReadResult::Value(value))) => {
                render::read_task(value).is_some()
            }
            (rows::MILESTONE_SHOW_AT, 3, page) => forge::milestones::answer(page, &super::TOTALS).is_some(),
            _ => false,
        };
        if !valid {
            if let Readout::Jig(c::ReadResult::Value(_)) = page {
                return render::failure(
                    b"tools' bug; unexpected document; report it as a finding",
                    reading.called.json,
                    1,
                );
            }
            return render::read(page, client, handle, reading.called.json, limits, parts);
        }
    }
    let stdout = if reading.called.json {
        let bounds = JsonLimits { depth: 32, length: u32::try_from(limits.result_bytes).expect("result bound") };
        let mut measure = Encoder::measure(&bounds);
        encode(reading, &mut measure);
        let Ok(size) = measure.measured() else {
            return render::failure(b"result limit", true, 1);
        };
        let mut out = Encoder::write(size, &bounds);
        encode(reading, &mut out);
        out.finish()
    } else {
        let mut measure = Text::measure();
        text(reading, me, &mut measure);
        if skein_lib::len_u64(measure.structural()) > limits.result_bytes {
            return render::failure(b"result limit", false, 1);
        }
        let mut out = Text::writing_exact(measure);
        text(reading, me, &mut out);
        out.finish()
    };
    Printed { file: crate::jig::FileEnd::Keep, status: 0, stdout, stderr: Box::default() }
}
fn page(reading: &Reading, local: u32) -> &c::TaskPage {
    let Some(Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(page)))) = reading.pages.get(&local) else {
        unreachable!("validated tasks")
    };
    page
}
fn document(reading: &Reading) -> (&c::Task, Option<&c::Result>) {
    let Some(Readout::Jig(c::ReadResult::Value(value))) = reading.pages.get(&1) else { unreachable!("validated task") };
    render::read_task(value).expect("validated task")
}
fn selected<'a>(page: &'a c::TaskPage, call: &Call) -> (&'a [c::TaskLine], Option<&'a c::Cursor>) {
    if value(call, 1).is_some() {
        match &page.ended {
            c::EndedGroup::Rows { rows, after, .. } => (rows.as_slice(), after.as_ref()),
            c::EndedGroup::None => (&[], None),
        }
    } else {
        (page.live.as_slice(), page.live_after.as_ref())
    }
}
fn goals<'a>(page: &'a c::TaskPage, ended: &'a c::TaskPage) -> BTreeMap<u64, &'a c::TaskLine> {
    let mut goals = BTreeMap::new();
    if let c::EndedGroup::Rows { rows, .. } = &ended.ended {
        for row in rows {
            goals.insert(row.chip.task, row);
        }
    }
    for row in &page.live {
        goals.insert(row.chip.task, row);
    }
    goals
}
fn goal_state(chip: &c::Chip) -> &'static [u8] {
    if let Some(ending) = &chip.details.details.ending {
        return match ending {
            c::ChipEnding::Done(_) => b"done",
            c::ChipEnding::Failed(_) => b"failed",
            c::ChipEnding::Cancelled { .. } => b"closed",
        };
    }
    match chip.phase {
        c::Phase::Done => b"done",
        c::Phase::Ended | c::Phase::Failed => b"failed",
        c::Phase::Cancelled => b"closed",
        c::Phase::Claimed
        | c::Phase::Active
        | c::Phase::Waiting
        | c::Phase::Idle
        | c::Phase::Due
        | c::Phase::Preparing
        | c::Phase::Running
        | c::Phase::BackingOff
        | c::Phase::Held
        | c::Phase::Closing => b"live",
    }
}
fn state(chip: &c::Chip) -> &'static [u8] {
    if let Some(ending) = &chip.details.details.ending {
        match ending {
            c::ChipEnding::Cancelled { .. } => return b"closed",
            c::ChipEnding::Failed(_) => return b"ended",
            c::ChipEnding::Done(value) => {
                if let c::ResultValue::Report(report) = value
                    && failed(report.bytes())
                {
                    return b"ended";
                }
                return b"complete";
            }
        }
    }
    match chip.phase {
        c::Phase::Cancelled => b"closed",
        c::Phase::Failed => b"ended",
        c::Phase::Done | c::Phase::Ended => b"complete",
        c::Phase::Claimed
        | c::Phase::Active
        | c::Phase::Waiting
        | c::Phase::Idle
        | c::Phase::Due
        | c::Phase::Preparing
        | c::Phase::Running
        | c::Phase::BackingOff
        | c::Phase::Held
        | c::Phase::Closing => b"live",
    }
}
fn failed(report: &[u8]) -> bool {
    if let Some(rest) = report.strip_prefix(b"Concluded: ")
        && let Some(at) = skein_lib::bytes::find(rest, b" done, ")
        && digits(rest.get(..at).expect("done count")).is_some()
        && let Some(rest) = rest.get(at.checked_add(7).expect("separator")..)
        && let Some(at) = skein_lib::bytes::find(rest, b" failed, ")
        && let Some(count) = digits(rest.get(..at).expect("failed count"))
        && let Some(rest) = rest.get(at.checked_add(9).expect("separator")..)
        && let Some(cancelled) = rest.strip_suffix(b" cancelled.")
        && digits(cancelled).is_some()
    {
        return count > 0;
    }
    skein_lib::bytes::find(report, b"failed delegates").is_some()
        || skein_lib::bytes::find(report, b"delegates failed").is_some()
}
fn digits(bytes: &[u8]) -> Option<u64> {
    if bytes.is_empty() {
        return None;
    }
    let mut number = 0u64;
    for byte in bytes {
        if !byte.is_ascii_digit() {
            return None;
        }
        number = number.checked_mul(10)?.checked_add(u64::from(byte.checked_sub(b'0')?))?;
    }
    Some(number)
}
fn totals(reading: &Reading) -> Option<w::MilestoneTotals> {
    match forge::milestones::answer(reading.pages.get(&3)?, &super::TOTALS)? {
        w::MilestoneResult::Totals(totals) => Some(totals),
        w::MilestoneResult::NotMilestone => None,
    }
}
fn text(reading: &Reading, me: Option<&c::Me>, out: &mut Text) {
    if reading.called.call.tool == rows::MILESTONE_LIST_AT {
        let (rows, after) = selected(page(reading, 1), &reading.called.call);
        for row in rows {
            out.number(row.chip.task);
            out.put(b"  ");
            out.put(row.chip.title.bytes());
            out.put(b"  ");
            out.number(u64::from(row.delegates_done));
            out.put(b" / ");
            out.number(u64::from(row.delegates_total));
            out.put(b" done  ");
            out.put(state(&row.chip));
            out.put(b"\n");
        }
        render::lists::next(&reading.called.call, 2, after, out);
        return;
    }
    show_text(reading, me, out);
}
fn show_text(reading: &Reading, me: Option<&c::Me>, out: &mut Text) {
    let (task, result) = document(reading);
    let facts = milestones::facts(task);
    out.put(b"milestone ");
    out.number(task.chip.task);
    out.put(b" \xc2\xb7 ");
    out.put(&facts.name);
    out.put(b" \xc2\xb7 ");
    match facts.whole {
        Some(at) => {
            out.put(b"whole since ");
            render::document::clock::minute(at, out);
        }
        None => out.put(b"open"),
    }
    out.put(b"\n");
    if let Some(words) = &facts.words {
        out.put(words);
        out.put(b"\n");
    }
    out.put(b"made by ");
    maker(facts.maker, reading, me, out);
    out.put(b" \xc2\xb7 ");
    render::document::clock::minute(facts.made, out);
    out.put(b"\n");
    let goals = goals(page(reading, 2), page(reading, 6));
    totals_text(reading, task, result, &goals, out);
    out.put(b"goals:\n");
    for row in goals.values() {
        out.put(b"  ");
        out.number(row.chip.task);
        out.put(b"  ");
        out.put(row.chip.title.bytes());
        out.put(b"  ");
        out.put(goal_state(&row.chip));
        out.put(b"\n");
    }
    continuations(page(reading, 2), page(reading, 6), task, out);
}
fn totals_text(
    reading: &Reading,
    task: &c::Task,
    result: Option<&c::Result>,
    goals: &BTreeMap<u64, &c::TaskLine>,
    out: &mut Text,
) {
    match totals(reading) {
        None => out.put(b"totals: none\n"),
        Some(totals) => {
            out.put(b"goals: ");
            out.number(u64::from(totals.done()));
            out.put(b" / ");
            out.number(
                u64::from(totals.done())
                    .checked_add(u64::from(totals.failed()))
                    .expect("goal counts")
                    .checked_add(u64::from(totals.live()))
                    .expect("goal counts"),
            );
            out.put(b" done \xc2\xb7 ");
            out.number(u64::from(totals.failed()));
            out.put(b" failed \xc2\xb7 ");
            out.number(u64::from(totals.closed()));
            out.put(b" closed \xc2\xb7 ");
            out.number(u64::from(totals.live()));
            out.put(b" live\nspent: ");
            render::amount(totals.spent(), out);
            out.put(b" of ");
            render::amount(task.budget.numbers.budget, out);
            out.put(b"\n");
            if let Some(at) = totals.ended() {
                out.put(b"ended: ");
                render::document::clock::minute(Wall::from_nanos(*at), out);
                out.put(b", ");
                out.put(ended_state(task, result));
                out.put(b"\n");
            } else if let Some(landing) = totals.last_landing() {
                out.put(b"last landing: ");
                render::document::clock::minute(Wall::from_nanos(landing.at()), out);
                out.put(b", ");
                if let Some(goal) = goals.get(&landing.goal()) {
                    out.put(goal.chip.title.bytes());
                } else if let Some(Readout::Jig(c::ReadResult::Value(value))) = reading.pages.get(&5)
                    && let Some((goal, _)) = render::read_task(value)
                {
                    out.put(goal.chip.title.bytes());
                }
                out.put(b" (task ");
                out.number(landing.goal());
                out.put(b")\n");
            }
        }
    }
}
fn ended_state(task: &c::Task, result: Option<&c::Result>) -> &'static [u8] {
    if let Some(result) = result {
        match &result.ending {
            c::Ending::Cancelled { .. } => return b"closed",
            c::Ending::Failed { .. } => return b"ended",
            c::Ending::Task(ending) => match ending.as_ref() {
                c::ChipEnding::Cancelled { .. } => return b"closed",
                c::ChipEnding::Failed(_) => return b"ended",
                c::ChipEnding::Done(c::ResultValue::Report(report)) if failed(report.bytes()) => return b"ended",
                c::ChipEnding::Done(_) => {}
            },
            c::Ending::Withdrawn { .. } | c::Ending::Decided { .. } | c::Ending::Deleted => {}
            c::Ending::Done => {
                if let Some(c::ResultValue::Report(report)) = &result.value
                    && failed(report.bytes())
                {
                    return b"ended";
                }
            }
        }
    }
    let state = state(&task.chip);
    if state == b"live" { b"complete" } else { state }
}
fn maker(maker: c::Requester, reading: &Reading, me: Option<&c::Me>, out: &mut Text) {
    match maker {
        c::Requester::Person(number) => {
            out.put(b"person ");
            out.number(number);
            if let Some(Readout::Jig(c::ReadResult::Value(c::ReadValue::Project(project)))) = reading.pages.get(&4) {
                for member in &project.members {
                    if member.party == number {
                        out.put(b", ");
                        out.put(member.name.bytes());
                        return;
                    }
                }
            }
            if let Some(me) = me
                && me.party.number == number
            {
                out.put(b", ");
                out.put(me.party.name.bytes());
            }
        }
        c::Requester::Task(task) => {
            out.put(b"task ");
            out.number(task);
        }
        c::Requester::Deployment { project } => {
            out.put(b"deployment ");
            out.number(u64::from(project));
        }
    }
}
fn continuations(page: &c::TaskPage, ended: &c::TaskPage, task: &c::Task, out: &mut Text) {
    if let Some(after) = &page.live_after {
        continuation(task, false, after, out);
    }
    if let c::EndedGroup::Rows { after: Some(after), .. } = &ended.ended {
        continuation(task, true, after, out);
    }
}
fn continuation(task: &c::Task, ended: bool, after: &c::Cursor, out: &mut Text) {
    render::lists::next(&delegate_call(task, ended), 6, Some(after), out);
}
fn delegate_call(task: &c::Task, ended: bool) -> Call {
    Call {
        tool: rows::TASKS_AT,
        form: 0,
        key: None,
        values: Box::new([
            Some(Values::One(Value::Project(Project::Number(task.project)))),
            None,
            None,
            None,
            if ended { Some(Values::One(Value::Switch)) } else { None },
            None,
            None,
            Some(Values::One(Value::Number(task.chip.task))),
            None,
            None,
            None,
        ]),
    }
}
fn encode(reading: &Reading, out: &mut Encoder) {
    out.object_start();
    if reading.called.call.tool == rows::MILESTONE_LIST_AT {
        let (rows, after) = selected(page(reading, 1), &reading.called.call);
        out.key(b"milestones");
        out.array_start();
        for row in rows {
            line(row, state(&row.chip), out);
        }
        out.array_end();
        out.key(b"after");
        match after {
            Some(after) => render::document::hex(after.value.bytes(), out),
            None => out.null(),
        }
        out.key(b"read_on");
        match after {
            Some(after) => out.string(&render::lists::read_on(&reading.called.call, 2, after)),
            None => out.null(),
        }
    } else {
        encode_show(reading, out);
    }
    out.object_end();
}
fn encode_show(reading: &Reading, out: &mut Encoder) {
    let (task, _) = document(reading);
    let facts = milestones::facts(task);
    out.key(b"facts");
    out.object_start();
    out.key(b"name");
    out.string(&facts.name);
    out.key(b"words");
    match facts.words {
        Some(words) => out.string(&words),
        None => out.null(),
    }
    out.key(b"maker");
    render::document::requester_json(&facts.maker, out);
    out.key(b"whole");
    match facts.whole {
        Some(at) => out.unsigned(at.as_nanos()),
        None => out.null(),
    }
    out.key(b"made");
    out.unsigned(facts.made.as_nanos());
    out.object_end();
    out.key(b"totals");
    match totals(reading) {
        Some(totals) => encode_totals(&totals, out),
        None => out.null(),
    }
    out.key(b"goals");
    out.array_start();
    for row in goals(page(reading, 2), page(reading, 6)).values() {
        render::document::task_line(row, out);
    }
    out.array_end();
    out.key(b"read_on");
    out.array_start();
    if let Some(after) = &page(reading, 2).live_after {
        out.string(&render::lists::read_on(&delegate_call(task, false), 6, after));
    }
    if let c::EndedGroup::Rows { after: Some(after), .. } = &page(reading, 6).ended {
        out.string(&render::lists::read_on(&delegate_call(task, true), 6, after));
    }
    out.array_end();
}
fn line(row: &c::TaskLine, state: &[u8], out: &mut Encoder) {
    out.object_start();
    render::document::task_line_fields(row, out);
    out.key(b"name");
    out.string(row.chip.title.bytes());
    out.key(b"state");
    out.string(state);
    out.key(b"delegates_done");
    out.unsigned(u64::from(row.delegates_done));
    out.key(b"delegates_total");
    out.unsigned(u64::from(row.delegates_total));
    out.object_end();
}
fn encode_totals(totals: &w::MilestoneTotals, out: &mut Encoder) {
    out.object_start();
    out.key(b"task");
    out.unsigned(totals.task());
    out.key(b"done");
    out.unsigned(u64::from(totals.done()));
    out.key(b"failed");
    out.unsigned(u64::from(totals.failed()));
    out.key(b"closed");
    out.unsigned(u64::from(totals.closed()));
    out.key(b"live");
    out.unsigned(u64::from(totals.live()));
    out.key(b"ended");
    match totals.ended() {
        Some(at) => out.unsigned(*at),
        None => out.null(),
    }
    out.key(b"last_landing");
    match totals.last_landing() {
        Some(landing) => {
            out.object_start();
            out.key(b"at");
            out.unsigned(landing.at());
            out.key(b"goal");
            out.unsigned(landing.goal());
            out.key(b"change");
            out.unsigned(landing.change());
            out.object_end();
        }
        None => out.null(),
    }
    out.key(b"spent");
    out.unsigned(totals.spent());
    out.object_end();
}
