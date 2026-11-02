//! Forge removal and playground reset (temper/docs/design/client/tools.md, 5.5).
use super::{Forges, PLAYGROUND_END, REMOVE, finish};
use crate::{
    jig::{
        self, render,
        runner::{Called, Out},
    },
    root::Limits,
};
use alloc::boxed::Box;
use jig_client::{Answered, Ask, Event, Family, Key, Outcome, Told, contract as c};
use skein_lib::{Env, Queue, Reader};
use temper_client_domain::owner::Owner;
use temper_client_protocol_wire::{self as wire, v1 as w};

#[derive(Debug)]
pub(super) struct Pending {
    request: Ask,
    end: bool,
    called: Option<Called>,
}
pub(super) fn owns(call: &jig::call::Call) -> bool {
    let name = crate::table::TOOLS.get(usize::from(call.tool)).expect("parsed tool").name;
    name == REMOVE.name || name == PLAYGROUND_END.name
}
pub(super) fn ask(request: w::Request, limits: &Limits) -> Option<Ask> {
    let bounds = temper_client_domain::forge::wire_limits(&limits.domain.forge)?;
    let record = w::Requesting::new(&bounds, w::RequestingParts { request }).ok()?;
    Some(Ask::Family {
        scope: c::Scope::Deployment,
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::requesting(&record) },
    })
}
#[expect(clippy::manual_map, reason = "the domain matches without closures")]
pub(super) fn send(state: &mut Forges, env: &Env<Limits>, called: Called, out: &mut Queue<Out>) {
    if called.value.is_some() || jig::argv::check(crate::table::TOOLS, &called.call, &env.limits.tools).is_err() {
        let printed = render::failure(b"invalid forge call", called.json, 1);
        finish(called, printed, out);
        return;
    }
    let bounds = temper_client_domain::forge::wire_limits(&env.limits.domain.forge).expect("checked limits");
    let end = crate::table::TOOLS.get(usize::from(called.call.tool)).expect("parsed tool").name == PLAYGROUND_END.name;
    let request = if end {
        w::Request::EndPlayground
    } else {
        w::Request::RemoveForge(
            w::RemoveForge::new(
                &bounds,
                w::RemoveForgeParts { forge: u16::try_from(super::number_arg(&called.call)).expect("checked forge") },
            )
            .expect("checked forge"),
        )
    };
    let request = ask(request, &env.limits).expect("checked limits");
    let key = Key { asker: Owner::Forge.asker(), number: (u64::from(called.call_number) << 32u32) | 7 };
    let name = match &called.call.key {
        Some(name) => Some(crate::key::engine(name)),
        None => None,
    };
    state.lifecycle.insert(key, Pending { request: request.clone(), end, called: Some(called) });
    out.push(Out::Client { event: Event::Send { key, name, request } });
}
pub(super) fn told(state: &mut Forges, env: &Env<Limits>, told: &Told, out: &mut Queue<Out>) -> bool {
    match told {
        Told::Kept { key, request: Ask::Family { scope: c::Scope::Deployment, record } }
            if record.family == wire::FORGE && record.version == wire::CLIENT_VERSION =>
        {
            let Some(bounds) = temper_client_domain::forge::wire_limits(&env.limits.domain.forge) else { return false };
            let Some(request) = wire::decode_requesting(&record.bytes, &bounds) else { return false };
            let end = match request.request() {
                w::Request::EndPlayground => true,
                w::Request::RemoveForge(_) => false,
                w::Request::Adopt(_)
                | w::Request::SetUp(_)
                | w::Request::Return(_)
                | w::Request::Connect(_)
                | w::Request::Install(_)
                | w::Request::Reachable(_)
                | w::Request::Accept(_)
                | w::Request::StartPlayground => return false,
            };
            state.lifecycle.insert(
                *key,
                Pending {
                    request: Ask::Family { scope: c::Scope::Deployment, record: record.clone() },
                    end,
                    called: None,
                },
            );
            true
        }
        Told::Answered { key, outcome } => {
            let Some(pending) = state.lifecycle.remove(key) else { return false };
            let json = match &pending.called {
                Some(called) => called.json,
                None => state.json,
            };
            let printed = print(&pending, outcome, json, &env.limits);
            match pending.called {
                Some(called) => finish(called, printed, out),
                None => out.push(Out::Kept { line: if printed.status == 0 { printed.stdout } else { printed.stderr } }),
            }
            true
        }
        Told::Kept { .. }
        | Told::Link { .. }
        | Told::Dropped { .. }
        | Told::Pending { .. }
        | Told::Changed { .. }
        | Told::Ended { .. }
        | Told::Text { .. }
        | Told::Page { .. } => false,
    }
}
fn print(pending: &Pending, outcome: &Outcome, json: bool, limits: &Limits) -> jig::Printed {
    let kept = pending.called.is_none();
    let mut operation = render::outcome::Operation::of(&pending.request);
    operation.word = if pending.end { b"playground-end" } else { b"forge-remove" };
    let answer = match outcome {
        Outcome::Full { held, room_at } => {
            return render::answers_full(crate::root::DIAGNOSTIC_PREFIX, *held, *room_at);
        }
        Outcome::Answered(Answered::Family(record))
            if record.family == wire::FORGE && record.version == wire::CLIENT_VERSION =>
        {
            let Some(bounds) = temper_client_domain::forge::wire_limits(&limits.domain.forge) else {
                return render::failure(b"invalid limits", json, 1);
            };
            let mut reader = Reader::new(&record.bytes);
            let Ok(answer) = w::Answer::decode(&bounds, &mut reader) else {
                return render::failure(b"forge answer decoding failed", json, 1);
            };
            if !reader.is_empty() {
                return render::failure(b"forge answer decoding failed", json, 1);
            }
            Some(answer.into_parts().outcome)
        }
        Outcome::Unknown => Some(w::Outcome::Unknown),
        Outcome::Answered(_)
        | Outcome::Held
        | Outcome::Limit
        | Outcome::Unconfirmed
        | Outcome::NotKept
        | Outcome::Unsent
        | Outcome::Lost => None,
    };
    let Some(answer) = answer else {
        if kept {
            return jig::Printed {
                file: jig::FileEnd::Keep,
                status: 1,
                stdout: Box::default(),
                stderr: render::outcome::kept(
                    &operation,
                    outcome,
                    json,
                    None,
                    &limits.tools,
                    crate::root::DIAGNOSTIC_PREFIX,
                ),
            };
        }
        return render::outcome::print(&operation, outcome, json, None, &limits.tools, crate::root::DIAGNOSTIC_PREFIX);
    };
    let Some((word, why, status)) = answer_words(answer, pending.end) else {
        return render::failure(b"forge answer decoding failed", json, 1);
    };
    let bytes = if json {
        let bounds = skein_json::writer::Limits { depth: 8, length: limits.tools.text };
        let mut measure = skein_json::writer::Encoder::measure(&bounds);
        encode(word, why, kept, operation.word, &mut measure);
        let Ok(size) = measure.measured() else { return render::failure(b"result limit", true, 1) };
        let mut out = skein_json::writer::Encoder::write(size, &bounds);
        encode(word, why, kept, operation.word, &mut out);
        let bytes = out.finish();
        if kept { render::outcome::newline(&bytes) } else { bytes }
    } else {
        let mut measure = render::Text::measure();
        text(word, why, kept, operation.word, &mut measure);
        let mut out = render::Text::writing_exact(measure);
        text(word, why, kept, operation.word, &mut out);
        out.finish()
    };
    if bytes.len() > usize::try_from(limits.tools.text).expect("text fits") {
        return render::failure(b"result limit", json, 1);
    }
    if status == 0 {
        jig::Printed { file: jig::FileEnd::Keep, status, stdout: bytes, stderr: Box::default() }
    } else {
        jig::Printed { file: jig::FileEnd::Keep, status, stdout: Box::default(), stderr: bytes }
    }
}
fn text(word: &[u8], why: Option<(&[u8], &[u8])>, kept: bool, operation: &[u8], out: &mut render::Text) {
    if kept {
        out.put(b"kept: ");
        out.put(operation);
        out.put(b" ");
    }
    match why {
        Some((_, words)) => {
            out.put(b"refused: ");
            out.put(words);
        }
        None => out.put(word),
    }
    out.put(b"\n");
}
fn encode(
    word: &[u8],
    why: Option<(&[u8], &[u8])>,
    kept: bool,
    operation: &[u8],
    out: &mut skein_json::writer::Encoder,
) {
    if kept {
        out.object_start();
        out.key(b"operation");
        out.string(operation);
        out.key(b"outcome");
    }
    out.object_start();
    out.key(word);
    out.object_start();
    if let Some((why, _)) = why {
        out.key(b"why");
        out.string(why);
    }
    out.object_end();
    out.object_end();
    if kept {
        out.object_end();
    }
}

type RefusalWords = (&'static [u8], &'static [u8]);
type AnswerWords = (&'static [u8], Option<RefusalWords>, u8);
fn answer_words(answer: w::Outcome, end: bool) -> Option<AnswerWords> {
    Some(match answer {
        w::Outcome::Removed if !end => (b"removed", None, 0),
        w::Outcome::Reset if end => (b"reset", None, 0),
        w::Outcome::RemoveRefused(value) if !end => {
            (b"remove_refused", Some(temper_client_domain::forge::forges::connect_refusal(value.why())), 2)
        }
        w::Outcome::EndRefused(value) if end => match value.why() {
            w::EndRefusal::NotOffered => (b"end_refused", Some((b"not_offered", b"not offered")), 2),
        },
        w::Outcome::Unknown => (b"unknown", None, 1),
        w::Outcome::Removed
        | w::Outcome::Reset
        | w::Outcome::RemoveRefused(_)
        | w::Outcome::EndRefused(_)
        | w::Outcome::Adopted(_)
        | w::Outcome::CoreRefused(_)
        | w::Outcome::ForgeRefused(_)
        | w::Outcome::SettingUp
        | w::Outcome::SetUpRefused(_)
        | w::Outcome::Running
        | w::Outcome::StartRefused(_)
        | w::Outcome::StartFailed(_)
        | w::Outcome::Returning
        | w::Outcome::Connecting
        | w::Outcome::Link(_)
        | w::Outcome::ConnectRefused(_)
        | w::Outcome::Page(_)
        | w::Outcome::Accepted(_) => return None,
    })
}
