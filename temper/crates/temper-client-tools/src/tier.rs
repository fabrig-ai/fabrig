//! A person's tier controls (temper/docs/design/client/ctl/tools.md, 3).
use crate::{
    jig::{
        self,
        call::{Call, Value, value},
        render::{self, Text},
        runner::{Called, Out},
        table::{
            Keyed, Kind,
            Occurs::{Optional, Required},
            Tool, flag, positional,
        },
    },
    root::Limits,
};
use alloc::{boxed::Box, collections::BTreeMap};
use jig_client::{Ask, Client, Event, Handle, Key, Outcome, Readable, Readout, Told, contract as c};
use skein_lib::{Env, Queue};
use temper_client_domain::{forge, owner::Owner};
use temper_client_protocol_wire::{self as wire, v1 as w};
const PROJECT: jig::table::Argument =
    flag(b"project", b"PROJECT", Kind::Project { empty: false }, Required, b"The project");
const OWNER: jig::table::Argument =
    flag(b"owner", b"OWNER", Kind::Literal, Optional, b"The repository's owner; give --name too");
const NAME: jig::table::Argument =
    flag(b"name", b"NAME", Kind::Literal, Optional, b"The repository's name; give --owner too");
const TIER: jig::table::Argument = positional(b"tier", b"TIER", Kind::Literal, Required, b"The tier's name");
const REASON: jig::table::Argument =
    flag(b"reason", b"TEXT", Kind::Text, Required, b"Why the tier waits; 1 to 1024 UTF-8 bytes");
pub const HOLD: Tool = Tool {
    name: b"tier hold",
    line: b"Pause a repository's after-landing tier",
    prints: b"`held: TIER of OWNER/NAME`, or a refusal (exit 2)",
    result: b"tier",
    forms: &[&[PROJECT, OWNER, NAME, TIER, REASON]],
    keyed: Keyed::Required,
    area: 6,
    notes: b"Give --owner and --name together, or neither for the project's home. A run in flight is let finish.\n",
};
pub const RELEASE: Tool = Tool {
    name: b"tier release",
    line: b"Resume a repository's held tier",
    prints: b"`released: TIER of OWNER/NAME`, or a refusal (exit 2)",
    result: b"tier",
    forms: &[&[PROJECT, OWNER, NAME, TIER]],
    keyed: Keyed::Required,
    area: 6,
    notes: b"Starts the next due run at the newest tip, covering the landings made while held.\n",
};
#[derive(Debug)]
pub(crate) struct Tiers {
    waiting: BTreeMap<u32, Called>,
    pending: BTreeMap<Key, Pending>,
    pub(crate) json: bool,
    reading: BTreeMap<Handle, (Pending, Outcome)>,
    next_read: u64,
}
#[derive(Debug)]
struct Pending {
    request: Ask,
    called: Option<Called>,
}
impl Tiers {
    pub(crate) fn new() -> Self {
        Self { waiting: BTreeMap::new(), pending: BTreeMap::new(), reading: BTreeMap::new(), next_read: 0, json: false }
    }
    pub(crate) fn calls(&self) -> usize {
        self.waiting.len().saturating_add(self.pending.len()).saturating_add(self.reading.len())
    }
}
pub(crate) fn owns(call: &Call) -> bool {
    let name = crate::table::TOOLS.get(usize::from(call.tool)).expect("tool").name;
    name == HOLD.name || name == RELEASE.name
}
pub(crate) fn call(state: &mut Tiers, client: &Client, env: &Env<Limits>, called: Called, out: &mut Queue<Out>) {
    if crate::forge::me(client).is_some() {
        send(state, client, env, called, out);
    } else {
        state.waiting.insert(called.call_number, called);
    }
}
pub(crate) fn ready(state: &mut Tiers, client: &Client, env: &Env<Limits>, out: &mut Queue<Out>) {
    if crate::forge::me(client).is_none() {
        return;
    }
    for _ in 0..state.waiting.len() {
        let Some((_, called)) = state.waiting.pop_first() else { break };
        send(state, client, env, called, out);
    }
}
fn prepare(call: &Call, project: u32) -> Option<w::TierActRequest> {
    let repository = if value(call, 1).is_none() && value(call, 2).is_none() {
        None
    } else {
        let Some(Value::Text(owner)) = value(call, 1) else { return None };
        let Some(Value::Text(name)) = value(call, 2) else { return None };
        Some(
            w::DraftRepository::new(&w::CEILINGS, w::DraftRepositoryParts { owner: owner.clone(), name: name.clone() })
                .ok()?,
        )
    };
    let Some(Value::Text(tier)) = value(call, 3) else { return None };
    let name = crate::table::TOOLS.get(usize::from(call.tool))?.name;
    let act = if name == HOLD.name {
        let Some(Value::Text(reason)) = value(call, 4) else { return None };
        w::TierAct::Hold(w::TierHoldAct::new(&w::CEILINGS, w::TierHoldActParts { reason: reason.clone() }).ok()?)
    } else {
        w::TierAct::Release
    };
    w::TierActRequest::new(&w::CEILINGS, w::TierActRequestParts { project, repository, tier: tier.clone(), act }).ok()
}
fn send(state: &mut Tiers, client: &Client, env: &Env<Limits>, called: Called, out: &mut Queue<Out>) {
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
            out.push(Out::Printed { number: called.number, printed: render::failure(
                b"invalid tier act: name a project and tier, --owner and --name together, and a reason of 1 to 1024 UTF-8 bytes", called.json, 2) });
            return;
        }
    };
    let Some(request) = forge::family::tier_act(request, &w::CEILINGS) else { return };
    let key = jig::key::local(Owner::Forge.asker(), called.call_number);
    if client.pending(key).is_some() {
        unreachable!("unique call number");
    }
    let name = Some(crate::key::engine(called.call.key.as_ref().expect("tier act requires key")));
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
        w::Request::TierAct(_) => true,
        w::Request::FollowAct(_)
        | w::Request::Adopt(_)
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
        | w::Request::SetDraft(_)
        | w::Request::AddDevice(_)
        | w::Request::RemoveDevice(_)
        | w::Request::FileFinding(_)
        | w::Request::HandFinding(_)
        | w::Request::TriageFinding(_)
        | w::Request::TryAccount(_) => false,
    }
}
pub(crate) fn told(state: &mut Tiers, env: &Env<Limits>, told: &Told, out: &mut Queue<Out>) {
    match told {
        Told::Kept { key, request } => {
            if restored(request) {
                state.pending.insert(*key, Pending { request: request.clone(), called: None });
            }
        }
        Told::Answered { key, outcome } => {
            let Some(pending) = state.pending.remove(key) else { return };
            let request = decoded(&pending.request).expect("tier request");
            if request.repository().is_none() {
                state.next_read = state.next_read.checked_add(1).expect("tier home read sequence");
                let handle = Handle { asker: Owner::Forge.asker(), number: (state.next_read << 32) | 0x74 };
                out.push(Out::Client {
                    event: Event::Read { handle, read: Readable::Jig(c::Read::Project { project: request.project() }) },
                });
                state.reading.insert(handle, (pending, outcome.clone()));
            } else {
                finish(state, pending, outcome, None, &env.limits, out);
            }
        }
        Told::Page { handle, page } => {
            let Some((pending, outcome)) = state.reading.remove(handle) else { return };
            let repository = home(page);
            finish(state, pending, &outcome, repository.as_ref(), &env.limits, out);
        }
        Told::Changed { .. }
        | Told::Ended { .. }
        | Told::Pending { .. }
        | Told::Dropped { .. }
        | Told::Link { .. }
        | Told::Text { .. } => {}
    }
}
fn home(result: &Readout) -> Option<w::DraftRepository> {
    let Readout::Jig(c::ReadResult::Value(c::ReadValue::Project(project))) = result else { return None };
    for resource in &project.resources {
        if !resource.home || resource.part.connector != wire::number(wire::Owner::Forge) {
            continue;
        }
        match &resource.part.key {
            c::PartKey::Path(path) => {
                if let [prefix, _host, owner, name] = path.as_slice()
                    && prefix.as_ref() == b"forge"
                {
                    return w::DraftRepository::new(
                        &w::CEILINGS,
                        w::DraftRepositoryParts { owner: owner.clone(), name: name.clone() },
                    )
                    .ok();
                }
            }
            c::PartKey::Number(_) => {}
        }
    }
    None
}
fn finish(
    state: &Tiers,
    pending: Pending,
    outcome: &Outcome,
    home: Option<&w::DraftRepository>,
    limits: &Limits,
    out: &mut Queue<Out>,
) {
    let json = match &pending.called {
        Some(called) => called.json,
        None => state.json,
    };
    let printed = print(&pending.request, outcome, home, json, limits);
    match pending.called {
        Some(called) => out.push(Out::Printed { number: called.number, printed }),
        None => out.push(Out::Kept { line: if printed.status == 0 { printed.stdout } else { printed.stderr } }),
    }
}
fn decoded(request: &Ask) -> Option<w::TierActRequest> {
    let Ask::Family { record, .. } = request else { return None };
    let w::Request::TierAct(request) = wire::decode_requesting(&record.bytes, &w::CEILINGS)?.into_parts().request
    else {
        return None;
    };
    Some(request)
}
fn words(
    request: &w::TierActRequest,
    home: Option<&w::DraftRepository>,
    refusal: Option<&w::TierRefusal>,
    out: &mut Text,
) {
    let repository = request.repository().as_ref().or(home);
    let mut measure = Text::measure();
    repository_words(repository, request.project(), &mut measure);
    let mut repo = Text::writing_exact(measure);
    repository_words(repository, request.project(), &mut repo);
    let repo = repo.finish();
    match refusal {
        None => {
            out.put(match request.act() {
                w::TierAct::Hold(_) => b"held: ",
                w::TierAct::Release => b"released: ",
            });
            out.put(request.tier());
            out.put(b" of ");
            out.put(&repo);
        }
        Some(w::TierRefusal::NoRepository) => {
            out.put(b"no repository ");
            out.put(&repo);
        }
        Some(w::TierRefusal::NoTier) => {
            out.put(b"no tier ");
            out.put(request.tier());
            out.put(b" on ");
            out.put(&repo);
        }
        Some(w::TierRefusal::Held(held)) => {
            out.put(request.tier());
            out.put(b" of ");
            out.put(&repo);
            out.put(b" is held already by party ");
            out.number(held.party());
            out.put(b": ");
            out.put(held.reason());
        }
        Some(w::TierRefusal::NotHeld) => {
            out.put(request.tier());
            out.put(b" of ");
            out.put(&repo);
            out.put(b" is not held");
        }
        Some(w::TierRefusal::Words) => out.put(b"reason must be 1 to 1024 bytes of UTF-8"),
    }
}
fn repository_words(repository: Option<&w::DraftRepository>, project: u32, out: &mut Text) {
    if let Some(repository) = repository {
        out.put(repository.owner());
        out.put(b"/");
        out.put(repository.name());
    } else {
        out.put(b"project ");
        out.number(u64::from(project));
        out.put(b"'s home");
    }
}
fn print(
    request: &Ask,
    outcome: &Outcome,
    home: Option<&w::DraftRepository>,
    json: bool,
    limits: &Limits,
) -> jig::Printed {
    if let Outcome::Answered(jig_client::Answered::Family(record)) = outcome {
        if record.family == wire::FORGE
            && record.version == wire::CLIENT_VERSION
            && let Some(answer) = decode_answer(&record.bytes)
            && let w::Outcome::Tier(answer) = answer.into_parts().outcome
        {
            let request = decoded(request).expect("tier request");
            let mut measure = Text::measure();
            words(&request, home, answer.refusal().as_ref(), &mut measure);
            let mut out = Text::writing_exact(measure);
            words(&request, home, answer.refusal().as_ref(), &mut out);
            let bytes = out.finish();
            if answer.refusal().is_some() {
                return render::failure(&bytes, json, 2);
            }
            return jig::Printed {
                file: jig::FileEnd::Keep,
                status: 0,
                stdout: if json {
                    let Some(bytes) = json_words(&bytes, limits) else {
                        return render::failure(b"result limit", json, 1);
                    };
                    bytes
                } else {
                    render::outcome::newline(&bytes)
                },
                stderr: Box::new([]),
            };
        }
        if let Some(forge::Answer::CoreRefused(why)) =
            forge::answer(&limits.domain, &jig_client::Answered::Family(record.clone()))
        {
            return render::outcome::print(
                &render::outcome::Operation::of(request),
                &Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused { refusal: why, at: None })),
                json,
                None,
                &limits.tools,
                crate::root::DIAGNOSTIC_PREFIX,
            );
        }
        return render::failure(b"unexpected tier answer", json, 1);
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
fn json_words(bytes: &[u8], limits: &Limits) -> Option<Box<[u8]>> {
    let bounds = skein_json::writer::Limits { depth: 1, length: u32::try_from(limits.tools.result_bytes).ok()? };
    let mut measure = skein_json::writer::Encoder::measure(&bounds);
    measure.string(bytes);
    let mut out = skein_json::writer::Encoder::write(measure.measured().ok()?, &bounds);
    out.string(bytes);
    Some(out.finish())
}
