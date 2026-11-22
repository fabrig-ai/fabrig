//! The deployment owner's act and its terminal (temper/docs/design/protocol/checkpoints.md, 3).
use crate::jig::{
    self, render,
    table::{Keyed, Kind, Occurs, Tool, flag},
};
use alloc::boxed::Box;
use skein_lib::Duration;
use temper_client_protocol_wire::v1 as w;

pub(crate) const ROW: Tool = Tool {
    name: b"deployment checkpoint",
    line: b"Take a checkpoint of the serving deployment as its owner",
    prints: b"The checkpoint's name, commit, secrets position and file figures; --json gives its fields",
    result: b"checkpoint",
    forms: &[&[flag(b"timeout", b"DURATION", Kind::Duration, Occurs::Optional, b"Wait this long; 10 minutes when absent")]],
    keyed: Keyed::Optional,
    area: 6,
    notes: b"Only a deployment owner takes one. A refusal exits 2. Past the wait,\nit exits 3; the checkpoint goes on in the engine.\n",
};
pub(crate) fn owns(call: &jig::call::Call) -> bool {
    crate::table::TOOLS.get(usize::from(call.tool)).expect("parsed tool").name == ROW.name
}
pub(crate) fn wait(call: &jig::call::Call) -> Duration {
    if let Some(jig::call::Value::Duration(value)) = jig::call::value(call, 0) {
        *value
    } else {
        crate::limits::CHECKPOINT_WAIT
    }
}
pub(crate) fn timeout(call: &jig::call::Call) -> jig::Printed {
    let seconds = wait(call).as_nanos().checked_div(1_000_000_000).expect("nonzero divisor");
    let mut measure = render::Text::measure();
    timeout_text(seconds, &mut measure);
    let mut out = render::Text::writing_exact(measure);
    timeout_text(seconds, &mut out);
    jig::Printed { file: jig::FileEnd::Keep, status: 3, stdout: Box::default(), stderr: out.finish() }
}
fn timeout_text(seconds: u64, out: &mut render::Text) {
    out.put(b"temper ctl: no answer within ");
    out.number(seconds);
    out.put(b"s; the checkpoint goes on in the engine\n");
}
pub(crate) fn print(answer: w::Outcome, json: bool, limits: &crate::root::Limits) -> jig::Printed {
    let printed = match answer {
        w::Outcome::Checkpointed(value) => success(&value, json, limits),
        w::Outcome::CheckpointRefused(value) => {
            let mut measure = render::Text::measure();
            refused(&value.why, &mut measure);
            let mut out = render::Text::writing_exact(measure);
            refused(&value.why, &mut out);
            jig::Printed { file: jig::FileEnd::Keep, status: 2, stdout: Box::default(), stderr: out.finish() }
        }
        w::Outcome::Removed
        | w::Outcome::Reset
        | w::Outcome::RemoveRefused(_)
        | w::Outcome::EndRefused(_)
        | w::Outcome::Unknown
        | w::Outcome::DeviceAdded(_)
        | w::Outcome::DeviceRemoved
        | w::Outcome::DeviceRefused(_)
        | w::Outcome::Away
        | w::Outcome::Tried
        | w::Outcome::Filed(_)
        | w::Outcome::Handed
        | w::Outcome::Triaged(_)
        | w::Outcome::FindingRefused(_)
        | w::Outcome::Drafted(_)
        | w::Outcome::DraftRefused(_)
        | w::Outcome::Follow(_)
        | w::Outcome::Opened(_)
        | w::Outcome::OpenRefused(_)
        | w::Outcome::Tier(_)
        | w::Outcome::NotTheirs
        | w::Outcome::Overturned
        | w::Outcome::OverturnRefused(_)
        | w::Outcome::Looked
        | w::Outcome::Dismissed
        | w::Outcome::MakeKept
        | w::Outcome::MakeRefused(_)
        | w::Outcome::Adopted(_)
        | w::Outcome::CoreRefused(_)
        | w::Outcome::ForgeRefused(_)
        | w::Outcome::SettingUp
        | w::Outcome::SetUpRefused(_)
        | w::Outcome::Running
        | w::Outcome::StartRefused(_)
        | w::Outcome::StartFailed(_)
        | w::Outcome::Connecting
        | w::Outcome::Link(_)
        | w::Outcome::ConnectRefused(_)
        | w::Outcome::Page(_)
        | w::Outcome::Accepted(_) => return render::failure(b"checkpoint answer decoding failed", json, 1),
    };
    if printed.stdout.len().max(printed.stderr.len()) > usize::try_from(limits.tools.text).expect("text bound fits") {
        return render::failure(b"result limit", json, 1);
    }
    printed
}
fn refused(why: &w::CheckpointWhy, out: &mut render::Text) {
    out.put(b"temper ctl: checkpoint refused: ");
    match why {
        w::CheckpointWhy::NotOwner => out.put(b"only an owner of the deployment takes a checkpoint"),
        w::CheckpointWhy::InProgress => out.put(b"checkpoint in progress"),
        w::CheckpointWhy::Unavailable => out.put(b"the store is not serving"),
        w::CheckpointWhy::Files => out.put(b"more than 65536 files to take"),
        w::CheckpointWhy::Unrecognized(value) => {
            out.put(b"an unrecognized file, ");
            out.put(&value.path);
        }
        w::CheckpointWhy::Vanished(value) => {
            out.put(b"a selected file vanished, ");
            out.put(&value.path);
        }
        w::CheckpointWhy::Failed(value) => {
            out.put(b"a file failed, ");
            out.put(&value.path);
            out.put(b": ");
            out.put(&value.error);
        }
    }
    out.put(b"\n");
}
fn success(value: &w::Checkpointed, json: bool, limits: &crate::root::Limits) -> jig::Printed {
    let bytes = if json {
        let bounds = skein_json::writer::Limits { depth: 2, length: limits.tools.text };
        let mut measure = skein_json::writer::Encoder::measure(&bounds);
        encode(value, &mut measure);
        let Ok(size) = measure.measured() else { return render::failure(b"result limit", true, 1) };
        let mut out = skein_json::writer::Encoder::write(size, &bounds);
        encode(value, &mut out);
        out.finish()
    } else {
        let mut measure = render::Text::measure();
        text(value, &mut measure);
        let mut out = render::Text::writing_exact(measure);
        text(value, &mut out);
        out.finish()
    };
    jig::Printed { file: jig::FileEnd::Keep, status: 0, stdout: bytes, stderr: Box::default() }
}
fn text(value: &w::Checkpointed, out: &mut render::Text) {
    out.put(b"checkpoint ");
    out.put(&value.name);
    for (words, figure) in [
        (b" at commit ".as_slice(), value.store),
        (b", secrets at ", value.secrets),
        (b": ", u64::from(value.linked)),
        (b" files linked, ", u64::from(value.copied)),
        (b" copied (", value.bytes),
        (b" bytes); paused ", value.paused.checked_div(1_000_000).expect("nonzero divisor")),
        (b" ms, took ", value.took.checked_div(1_000_000).expect("nonzero divisor")),
        (b" ms; ", value.held),
        (b" bytes held; ", u64::from(value.removed)),
        (b" older removed, ", u64::from(value.skipped)),
    ] {
        out.put(words);
        out.number(figure);
    }
    out.put(b" skipped\n");
}
fn encode(value: &w::Checkpointed, out: &mut skein_json::writer::Encoder) {
    out.object_start();
    out.key(b"name");
    out.string(&value.name);
    for (name, number) in [
        (b"store".as_slice(), value.store),
        (b"secrets", value.secrets),
        (b"linked", u64::from(value.linked)),
        (b"copied", u64::from(value.copied)),
        (b"bytes", value.bytes),
        (b"paused", value.paused),
        (b"took", value.took),
        (b"held", value.held),
        (b"removed", u64::from(value.removed)),
        (b"skipped", u64::from(value.skipped)),
    ] {
        out.key(name);
        out.number(skein_lib::Decimal::of(number).as_bytes());
    }
    out.object_end();
}
