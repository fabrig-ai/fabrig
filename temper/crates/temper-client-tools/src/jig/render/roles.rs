//! Standing roles and their whole Now (client/ctl/tools.md, 2).
use super::{Limits, Printed, Text};
use crate::jig::runner::reads::Reading;
use alloc::boxed::Box;
use jig_client::{Readout, contract as c};
use skein_json::writer::{Encoder, Limits as JsonLimits};

pub(crate) fn refusal(word: &[u8], refusal: &c::Refusal, me: Option<&c::Me>, out: &mut Text) -> bool {
    match (word, refusal) {
        (b"talk", c::Refusal::NotOffered) => out.put(b"not a standing role"),
        (b"talk", c::Refusal::Taken { by }) => {
            out.put(b"talking with ");
            super::acts::party(*by, me, out);
        }
        (b"leave", c::Refusal::NotWaiting) => out.put(b"no talk to leave"),
        _ => return false,
    }
    true
}

pub(crate) fn reply_page(thread: &jig_client::thread::Thread, from: u64, through: Option<u64>, limits: &Limits) -> Result<Box<[u8]>, Printed> {
    let bound = u32::try_from(limits.result_bytes).expect("result bound");
    let mut measure = Text::bounded(bound, 0);
    reply_text(thread, from, through, limits, &mut measure);
    if skein_lib::len_u64(measure.structural()) > limits.result_bytes {
        return Err(super::failure(b"result limit", false, 1));
    }
    let mut out = measure.writing(bound, 0);
    reply_text(thread, from, through, limits, &mut out);
    Ok(out.finish())
}
fn reply_text(thread: &jig_client::thread::Thread, from: u64, through: Option<u64>, limits: &Limits, out: &mut Text) {
    for row in thread.lines.rows.as_slice().iter().rev() {
        if row.source.position > from && row.source.position <= through.unwrap_or(u64::MAX)
            && let Some(turn) = &row.decoded
        {
            super::turn::reply(thread.task, turn.name, &turn.turn, limits, out);
        }
    }
}
pub(crate) fn reply(reply: &crate::jig::runner::talk::Reply, json: bool, limits: &Limits) -> Printed {
    let mut measure = Text::measure();
    for chunk in reply.chunks.values() { measure.put(chunk); }
    let mut out = Text::writing_exact(measure);
    for chunk in reply.chunks.values() { out.put(chunk); }
    let text = out.finish();
    let stdout = if json {
        let bounds = JsonLimits { depth: 2, length: u32::try_from(limits.result_bytes).expect("result bound") };
        let mut measure = Encoder::measure(&bounds);
        encode_reply(&text, reply.met, &mut measure);
        let Ok(size) = measure.measured() else { return super::failure(b"result limit", json, 1) };
        let mut out = Encoder::write(size, &bounds);
        encode_reply(&text, reply.met, &mut out);
        out.finish()
    } else { text };
    Printed { file: crate::jig::FileEnd::Keep, status: if reply.met { 0 } else { 3 }, stdout, stderr: Box::default() }
}
fn encode_reply(text: &[u8], met: bool, out: &mut Encoder) {
    out.object_start();
    out.key(b"reply"); out.string(text);
    out.key(b"met"); out.boolean(met);
    out.object_end();
}

fn note<'a>(page: &'a c::NotePage, description: &[u8]) -> Option<&'a c::Note> {
    let mut found: Option<&c::Note> = None;
    for row in &page.rows {
        let earlier = match found {
            Some(prior) => row.number < prior.number,
            None => true,
        };
        if row.description.bytes() == description && earlier {
            found = Some(row);
        }
    }
    found
}
fn page(read: Option<&Readout>) -> Option<&c::NotePage> {
    match read {
        Some(Readout::Jig(c::ReadResult::Value(c::ReadValue::Notes(page)))) => Some(page),
        _ => None,
    }
}
fn selected(read: Option<&Readout>) -> Option<&c::Note> {
    note(page(read)?, b"now")
}
fn first_line(body: &[u8]) -> &[u8] {
    match skein_lib::bytes::find(body, b"\n") {
        Some(at) => body.get(..at).expect("newline offset"),
        None => body,
    }
}
fn slot(index: usize) -> u32 {
    8u32.checked_add(u32::try_from(index).expect("bounded task rows")).expect("read slot")
}

pub(super) fn list(reading: &Reading, limits: &Limits) -> Printed {
    let Some(Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(tasks)))) = reading.pages.get(&1) else {
        unreachable!("validated task page")
    };
    let stdout = if reading.called.json {
        let bounds = JsonLimits { depth: 8, length: u32::try_from(limits.result_bytes).expect("result bound") };
        let mut measure = Encoder::measure(&bounds);
        encode_list(tasks, reading, &mut measure);
        let Ok(size) = measure.measured() else { return super::failure(b"result limit", true, 1) };
        let mut out = Encoder::write(size, &bounds);
        encode_list(tasks, reading, &mut out);
        out.finish()
    } else {
        let mut measure = Text::measure();
        list_text(tasks, reading, &mut measure);
        if skein_lib::len_u64(measure.structural()) > limits.result_bytes {
            return super::failure(b"result limit", false, 1);
        }
        let mut out = Text::writing_exact(measure);
        list_text(tasks, reading, &mut out);
        out.finish()
    };
    Printed { file: crate::jig::FileEnd::Keep, status: 0, stdout, stderr: Box::default() }
}
fn list_text(tasks: &c::TaskPage, reading: &Reading, out: &mut Text) {
    for (index, row) in tasks.live.as_slice().iter().enumerate() {
        out.number(row.chip.task);
        out.put(b" \xc2\xb7 charter ");
        charter(row.chip.executor, out);
        out.put(b" \xc2\xb7 ");
        out.put(crate::jig::subject::phase_word(row.chip.phase));
        out.put(b" \xc2\xb7 left ");
        out.number(row.budget.left);
        out.put(b" \xc2\xb7 ");
        match selected(reading.pages.get(&slot(index))) {
            Some(now) => out.put(first_line(now.body.bytes())),
            None => out.put(b"no Now yet"),
        }
        out.put(b"\n");
    }
    super::lists::next(&reading.called.call, 1, tasks.live_after.as_ref(), out);
}
fn charter(executor: c::ChipExecutor, out: &mut Text) {
    match executor {
        c::ChipExecutor::Agent { charter } => out.number(u64::from(charter)),
        c::ChipExecutor::Procedure { .. }
        | c::ChipExecutor::Party { .. }
        | c::ChipExecutor::Role { .. }
        | c::ChipExecutor::Job { .. } => out.put(b"none"),
    }
}
fn encode_list(tasks: &c::TaskPage, reading: &Reading, out: &mut Encoder) {
    out.object_start();
    out.key(b"roles");
    out.array_start();
    for (index, row) in tasks.live.as_slice().iter().enumerate() {
        out.object_start();
        out.key(b"task");
        out.unsigned(row.chip.task);
        out.key(b"charter");
        match row.chip.executor {
            c::ChipExecutor::Agent { charter } => out.unsigned(u64::from(charter)),
            c::ChipExecutor::Procedure { .. }
            | c::ChipExecutor::Party { .. }
            | c::ChipExecutor::Role { .. }
            | c::ChipExecutor::Job { .. } => out.null(),
        }
        out.key(b"state");
        out.string(crate::jig::subject::phase_word(row.chip.phase));
        out.key(b"left");
        out.unsigned(row.budget.left);
        out.key(b"now");
        match selected(reading.pages.get(&slot(index))) {
            Some(now) => out.string(first_line(now.body.bytes())),
            None => out.null(),
        }
        out.object_end();
    }
    out.array_end();
    out.key(b"after");
    match &tasks.live_after {
        Some(cursor) => super::document::hex(cursor.value.bytes(), out),
        None => out.null(),
    }
    out.object_end();
}

pub(crate) fn show(task: Printed, reading: &Reading, me: Option<&c::Me>, limits: &Limits) -> Printed {
    let json = reading.called.json;
    let report = crate::jig::call::value(&reading.called.call, 1).is_some();
    let talk = match reading.pages.get(&1) {
        Some(Readout::Jig(c::ReadResult::Value(value))) => match super::read_task(value) {
            Some((task, _)) => task.talk,
            None => None,
        },
        _ => None,
    };
    let read = reading.pages.get(&8);
    if task.status != 0 {
        return task;
    }
    let Some(page) = page(read) else { return super::failure(if report { b"report could not be read" } else { b"Now could not be read" }, json, 1) };
    let now = note(page, if report { b"report" } else { b"now" });
    let suffix = if json {
        let bounds = JsonLimits { depth: 4, length: u32::try_from(limits.result_bytes).expect("result bound") };
        let mut measure = Encoder::measure(&bounds);
        encode_now(now, &mut measure);
        let Ok(size) = measure.measured() else { return super::failure(b"result limit", json, 1) };
        let mut out = Encoder::write(size, &bounds);
        encode_now(now, &mut out);
        out.finish()
    } else {
        let mut measure = Text::measure();
        talk_text(talk, me, &mut measure);
        now_text(now, report, &mut measure);
        let mut out = Text::writing_exact(measure);
        talk_text(talk, me, &mut out);
        now_text(now, report, &mut out);
        out.finish()
    };
    let key = if report { b",\"report\":".as_slice() } else { b",\"now\":" };
    let parts: &[&[u8]] =
        if json { &[b"{\"task\":", &task.stdout, key, &suffix, b"}"] } else { &[&task.stdout, &suffix] };
    let bytes = skein_lib::bytes::concat(parts);
    if skein_lib::len_u64(bytes.len()) > limits.result_bytes {
        return super::failure(b"result limit", json, 1);
    }
    Printed { stdout: bytes, ..task }
}
fn talk_text(talk: Option<c::Talk>, me: Option<&c::Me>, out: &mut Text) {
    if let Some(talk) = talk {
        out.put(b"talking with ");
        super::acts::party(talk.party, me, out);
        out.put(b" since ");
        super::document::clock::deadline(talk.opened, out);
        out.put(b"\n");
    }
}
fn now_text(now: Option<&c::Note>, report: bool, out: &mut Text) {
    let Some(now) = now else {
        out.put(if report { b"no report yet\n" } else { b"no Now yet\n" });
        return;
    };
    out.put(if report { b"report: note " } else { b"Now: note " });
    out.number(now.number);
    out.put(b" revision ");
    out.number(now.revision.value);
    out.put(b" written ");
    match now.changed {
        Some(at) => super::document::clock::deadline(at, out),
        None => out.put(b"unknown"),
    }
    out.put(b"\n");
    out.put(now.body.bytes());
    if !now.body.bytes().ends_with(b"\n") {
        out.put(b"\n");
    }
    if now.body.cut {
        out.put(if report { b"report body cut; report it\n" } else { b"Now body cut; report it\n" });
    }
}
fn encode_now(now: Option<&c::Note>, out: &mut Encoder) {
    let Some(now) = now else {
        out.null();
        return;
    };
    out.object_start();
    out.key(b"number");
    out.unsigned(now.number);
    out.key(b"revision");
    out.unsigned(now.revision.value);
    out.key(b"written");
    match now.changed {
        Some(at) => out.unsigned(at.as_nanos()),
        None => out.null(),
    }
    out.key(b"body");
    out.string(now.body.bytes());
    out.key(b"cut");
    out.boolean(now.body.cut);
    out.object_end();
}
