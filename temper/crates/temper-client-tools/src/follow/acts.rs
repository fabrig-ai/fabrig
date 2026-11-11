//! Keyed follow acts (temper/docs/design/client/ctl/tools.md, 3).
use super::{
    BTreeMap, Box, Called, Client, Env, Event, Keyed, Kind, Limits, Out, Owner, Queue, Required, Told, Tool, Value,
    flag, forge, jig, number, render, value, w, wire,
};
use crate::jig::{
    call::{Call, Values},
    table::{Occurs, positional},
};
use jig_client::{Ask, Key, Outcome};
use skein_lib::List;
const PROJECT: jig::table::Argument =
    flag(b"project", b"PROJECT", Kind::Project { empty: false }, Required, b"The project");
const FOLLOW: jig::table::Argument = positional(
    b"follow",
    b"FOLLOW",
    Kind::NumberRange { min: 1, max: 4_294_967_295 },
    Required,
    b"The follow's number",
);
const AFTER: jig::table::Argument =
    positional(b"after", b"COMMIT", Kind::Literal, Required, b"The landing's full lowercase commit id");
const REASON: jig::table::Argument =
    flag(b"reason", b"TEXT", Kind::Text, Required, b"Why this entry waits or is skipped");
pub const START: Tool = Tool {
    name: b"follow start",
    line: b"Start a follow entry or lift its owner's hold",
    prints: b"`kept`, or the entry and why the act was refused",
    result: b"follow-act",
    forms: &[&[PROJECT, FOLLOW, AFTER]],
    keyed: Keyed::Required,
    area: 6,
    notes: b"Gives the owner's word, lifts an owner's hold, or retries an ended entry.\n",
};
pub const HOLD: Tool = Tool {
    name: b"follow hold",
    line: b"Keep a follow entry from producing",
    prints: START.prints,
    result: START.result,
    forms: &[&[PROJECT, FOLLOW, AFTER, REASON]],
    keyed: Keyed::Required,
    area: 6,
    notes: b"The change must not have started producing; follow start lifts the hold.\n",
};
pub const SKIP: Tool = Tool {
    name: b"follow skip",
    line: b"Count a landing followed without its change",
    prints: START.prints,
    result: START.result,
    forms: &[&[PROJECT, FOLLOW, AFTER, REASON]],
    keyed: Keyed::Required,
    area: 6,
    notes: b"Cancels its change, keeps the reason among skips, and keeps the chain whole.\n",
};
pub const MERGE: Tool = Tool {
    name: b"follow merge",
    line: b"Make one change follow consecutive landings",
    prints: START.prints,
    result: START.result,
    forms: &[&[
        PROJECT,
        FOLLOW,
        AFTER,
        positional(
            b"afters",
            b"COMMIT",
            Kind::Literal,
            Occurs::Repeated { most: 31 },
            b"The next landings, oldest first",
        ),
    ]],
    keyed: Keyed::Required,
    area: 6,
    notes: b"Names two or more consecutive entries whose changes have not started producing.\n",
};
#[derive(Debug)]
pub(super) struct Acts {
    waiting: BTreeMap<u32, Called>,
    pending: BTreeMap<Key, Pending>,
    pub(super) json: bool,
}
#[derive(Debug)]
struct Pending {
    request: Ask,
    called: Option<Called>,
}
impl Acts {
    pub(super) fn new() -> Self {
        Self { waiting: BTreeMap::new(), pending: BTreeMap::new(), json: false }
    }
    pub(super) fn calls(&self) -> usize {
        self.waiting.len().saturating_add(self.pending.len())
    }
}
pub(super) fn owns(call: &Call) -> bool {
    let name = crate::table::TOOLS.get(usize::from(call.tool)).expect("tool").name;
    name == START.name || name == HOLD.name || name == SKIP.name || name == MERGE.name
}
pub(super) fn call(state: &mut Acts, client: &Client, env: &Env<Limits>, called: Called, out: &mut Queue<Out>) {
    if crate::forge::me(client).is_some() {
        send(state, client, env, called, out);
    } else {
        state.waiting.insert(called.call_number, called);
    }
}
pub(super) fn ready(state: &mut Acts, client: &Client, env: &Env<Limits>, out: &mut Queue<Out>) {
    if crate::forge::me(client).is_none() {
        return;
    }
    for _ in 0..state.waiting.len() {
        let Some((_, called)) = state.waiting.pop_first() else { break };
        send(state, client, env, called, out);
    }
}
fn commit(bytes: &[u8]) -> Option<[u8; 32]> {
    if bytes.len() != 40 && bytes.len() != 64 {
        return None;
    }
    let mut result = [0_u8; 32];
    for (pair, cell) in bytes.chunks_exact(2).zip(&mut result) {
        for digit in pair {
            let n = match digit {
                b'0'..=b'9' => digit.saturating_sub(b'0'),
                b'a'..=b'f' => digit.saturating_sub(b'a').saturating_add(10),
                _ => return None,
            };
            *cell = cell.saturating_mul(16).saturating_add(n);
        }
    }
    Some(result)
}
fn prepare(call: &Call, project: u32) -> Option<w::FollowActRequest> {
    let Some(Value::Text(bytes)) = value(call, 2) else { return None };
    let after = commit(bytes)?;
    let name = crate::table::TOOLS.get(usize::from(call.tool))?.name;
    let act = if name == START.name {
        w::FollowAct::Start(w::FollowAt::new(&w::CEILINGS, w::FollowAtParts { after }).ok()?)
    } else if name == MERGE.name {
        let mut afters = List::with_capacity(w::CEILINGS.follow_merge_act_afters);
        afters.push(after).ok()?;
        if let Some(Some(Values::Many(values))) = call.values.get(3) {
            for value in values {
                let Value::Text(bytes) = value else { return None };
                afters.push(commit(bytes)?).ok()?;
            }
        }
        if afters.len() < 2 {
            return None;
        }
        w::FollowAct::Merge(w::FollowMergeAct::new(&w::CEILINGS, w::FollowMergeActParts { afters }).ok()?)
    } else {
        let Some(Value::Text(reason)) = value(call, 3) else { return None };
        let act =
            w::FollowReasonAct::new(&w::CEILINGS, w::FollowReasonActParts { after, reason: reason.clone() }).ok()?;
        if name == HOLD.name { w::FollowAct::Hold(act) } else { w::FollowAct::Skip(act) }
    };
    w::FollowActRequest::new(
        &w::CEILINGS,
        w::FollowActRequestParts { project, follow: u32::try_from(number(call, 1)?).ok()?, act },
    )
    .ok()
}
fn send(state: &mut Acts, client: &Client, env: &Env<Limits>, called: Called, out: &mut Queue<Out>) {
    let me = crate::forge::me(client).expect("live me");
    let project = jig::runner::reads::project(&called.call, me);
    let request = match project {
        Some(project) => prepare(&called.call, project),
        None => None,
    };
    let request = match request {
        Some(request)
            if called.value.is_none()
                && jig::argv::check(crate::table::TOOLS, &called.call, &env.limits.tools).is_ok() =>
        {
            request
        }
        Some(_) | None => {
            out.push(Out::Printed {
                number: called.number,
                printed: render::failure(
                    b"invalid follow act: name a project, follow, full lowercase commits and the required reason",
                    called.json,
                    2,
                ),
            });
            return;
        }
    };
    let Some(request) = forge::family::follow_act(request, &w::CEILINGS) else { return };
    let key = jig::key::local(Owner::Forge.asker(), called.call_number);
    if client.pending(key).is_some() {
        unreachable!("unique call number");
    }
    let name = Some(crate::key::engine(called.call.key.as_ref().expect("follow act requires key")));
    state.pending.insert(key, Pending { request: request.clone(), called: Some(called) });
    out.push(Out::Client { event: Event::Send { key, name, request } });
}
fn restored(request: &Ask) -> bool {
    let Ask::Family { record, .. } = request else { return false };
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return false;
    }
    let Some(request) = wire::decode_requesting(&record.bytes, &w::CEILINGS) else { return false };
    match request.into_parts().request {
        w::Request::FollowAct(_) => true,
        w::Request::Adopt(_)
        | w::Request::SetUp(_)
        | w::Request::Connect(_)
        | w::Request::Install(_)
        | w::Request::Reachable(_)
        | w::Request::Accept(_)
        | w::Request::RemoveForge(_)
        | w::Request::StartPlayground
        | w::Request::EndPlayground
        | w::Request::Replace(_)
        | w::Request::Dismiss(_)
        | w::Request::MakeRepository(_)
        | w::Request::DismissMaking(_)
        | w::Request::Looked(_)
        | w::Request::SetAway(_)
        | w::Request::Overturn(_)
        | w::Request::TryAccount(_) => false,
    }
}
pub(super) fn told(state: &mut Acts, env: &Env<Limits>, told: &Told, out: &mut Queue<Out>) {
    match told {
        Told::Kept { key, request } => {
            if restored(request) {
                state.pending.insert(*key, Pending { request: request.clone(), called: None });
            }
        }
        Told::Answered { key, outcome } => {
            let Some(pending) = state.pending.remove(key) else { return };
            let json = match &pending.called {
                Some(called) => called.json,
                None => state.json,
            };
            let printed = print(&pending.request, outcome, json, &env.limits);
            match pending.called {
                Some(called) => out.push(Out::Printed { number: called.number, printed }),
                None => out.push(Out::Kept { line: if printed.status == 0 { printed.stdout } else { printed.stderr } }),
            }
        }
        Told::Changed { .. }
        | Told::Ended { .. }
        | Told::Pending { .. }
        | Told::Dropped { .. }
        | Told::Link { .. }
        | Told::Text { .. }
        | Told::Page { .. } => {}
    }
}
fn print(request: &Ask, outcome: &Outcome, json: bool, limits: &Limits) -> jig::Printed {
    if let Outcome::Answered(jig_client::Answered::Family(record)) = outcome {
        if record.family == wire::FORGE
            && record.version == wire::CLIENT_VERSION
            && let Some(answer) = decode_answer(&record.bytes)
            && let w::Outcome::Follow(value) = answer.into_parts().outcome
        {
            return match value.refusal() {
                Some(reason) => render::failure(reason, json, 2),
                None => jig::Printed {
                    file: jig::FileEnd::Keep,
                    status: 0,
                    stdout: if json { b"{\"kept\":{}}".as_slice().into() } else { b"kept\n".as_slice().into() },
                    stderr: Box::new([]),
                },
            };
        }
        return render::failure(b"unexpected follow answer", json, 1);
    }
    render::outcome::print(
        &render::outcome::Operation::of(request),
        outcome,
        json,
        None,
        &limits.tools,
        crate::root::DIAGNOSTIC_PREFIX,
    )
}

fn decode_answer(bytes: &[u8]) -> Option<w::Answer> {
    let mut reader = skein_lib::Reader::new(bytes);
    let answer = w::Answer::decode(&w::CEILINGS, &mut reader).ok()?;
    if reader.is_empty() { Some(answer) } else { None }
}
