//! Making and removing forges and resetting the playground (client/tools.md, 5.5,
//! and temper/docs/design/draft/playground-reset.md, 3, 4 and 6.2).
use super::{Forges, MAKE, PLAYGROUND_RESET, REMOVE, finish};
use crate::{
    jig::{
        self,
        call::{Value, number, value},
        render::{self, Text},
        runner::{Called, Out},
    },
    root::Limits,
};
use alloc::boxed::Box;
use jig_client::{Answered, Ask, Event, Family, Key, Outcome, Told, contract as c};
use skein_json::writer::Encoder;
use skein_lib::{Env, Queue};
use temper_client_domain::owner::Owner;
use temper_client_protocol_wire::{self as wire, v1 as w};

#[derive(Debug, PartialEq, Eq)]
enum Kind {
    Make,
    Remove(u16),
    Reset,
}
impl Kind {
    fn forge(&self) -> Option<u16> {
        match self {
            Self::Remove(forge) => Some(*forge),
            Self::Make | Self::Reset => None,
        }
    }
    fn word(&self) -> &'static [u8] {
        match self {
            Self::Make => MAKE.result,
            Self::Remove(_) => REMOVE.result,
            Self::Reset => PLAYGROUND_RESET.result,
        }
    }
}
#[derive(Debug)]
pub(super) struct Pending {
    kind: Kind,
    request: Ask,
    called: Option<Called>,
}
pub(super) fn owns(call: &jig::call::Call) -> bool {
    let name = crate::table::TOOLS.get(usize::from(call.tool)).expect("parsed tool").name;
    name == MAKE.name || name == REMOVE.name || name == PLAYGROUND_RESET.name
}
fn literal(call: &jig::call::Call, at: usize) -> Option<&[u8]> {
    match value(call, at) {
        Some(Value::Text(bytes)) => Some(bytes),
        _ => None,
    }
}
#[expect(clippy::manual_map, reason = "the domain matches without closures")]
fn request(call: &jig::call::Call, limits: &Limits) -> Option<(Kind, Ask)> {
    let bounds = temper_client_domain::forge::wire_limits(&limits.domain.forge)?;
    let name = crate::table::TOOLS.get(usize::from(call.tool))?.name;
    let (kind, request) = if name == MAKE.name {
        let choice = if call.form == 1 {
            let api = literal(call, 1)?;
            let git = literal(call, 2)?;
            let web = literal(call, 3);
            if api.is_empty() || git.is_empty() || web == Some(b"".as_slice()) {
                return None;
            }
            let web = match web {
                Some(bytes) => Some(bytes.into()),
                None => None,
            };
            w::ForgeChoice::Forgejo(
                w::ForgejoChoice::new(&bounds, w::ForgejoChoiceParts { api: api.into(), git: git.into(), web }).ok()?,
            )
        } else if value(call, 0) == Some(&Value::Word(0)) {
            w::ForgeChoice::Owned
        } else {
            w::ForgeChoice::Github
        };
        (Kind::Make, w::Request::MakeForge(w::MakeForge::new(&bounds, w::MakeForgeParts { choice }).ok()?))
    } else if name == REMOVE.name {
        let forge = u16::try_from(number(call, 0)?).ok()?;
        (
            Kind::Remove(forge),
            w::Request::RemoveForge(w::RemoveForge::new(&bounds, w::RemoveForgeParts { forge }).ok()?),
        )
    } else {
        (Kind::Reset, w::Request::EndPlayground)
    };
    let record = w::Requesting::new(&bounds, w::RequestingParts { request }).ok()?;
    Some((
        kind,
        Ask::Family {
            scope: c::Scope::Deployment,
            record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::requesting(&record) },
        },
    ))
}
#[expect(clippy::manual_map, reason = "the domain matches without closures")]
pub(super) fn send(
    state: &mut Forges,
    _client: &jig_client::Client,
    env: &Env<Limits>,
    called: Called,
    out: &mut Queue<Out>,
) {
    if called.value.is_some() || jig::argv::check(crate::table::TOOLS, &called.call, &env.limits.tools).is_err() {
        let printed = render::failure(b"invalid forge call", called.json, 1);
        finish(called, printed, out);
        return;
    }
    let Some((kind, request)) = request(&called.call, &env.limits) else {
        let printed = render::failure(b"invalid forge call", called.json, 1);
        finish(called, printed, out);
        return;
    };
    let key = Key { asker: Owner::Forge.asker(), number: (u64::from(called.call_number) << 32u32) | 7 };
    let name = match &called.call.key {
        Some(name) => Some(crate::key::engine(name)),
        None => None,
    };
    state.making.insert(key, Pending { kind, request: request.clone(), called: Some(called) });
    out.push(Out::Client { event: Event::Send { key, name, request } });
}
pub(super) fn told(state: &mut Forges, env: &Env<Limits>, told: &Told, out: &mut Queue<Out>) -> bool {
    match told {
        Told::Kept { key, request: request @ Ask::Family { scope: c::Scope::Deployment, record } }
            if record.family == wire::FORGE && record.version == wire::CLIENT_VERSION =>
        {
            let Some(bounds) = temper_client_domain::forge::wire_limits(&env.limits.domain.forge) else { return false };
            let Some(decoded) = wire::decode_requesting(&record.bytes, &bounds) else { return false };
            let kind = match decoded.request() {
                w::Request::MakeForge(_) => Kind::Make,
                w::Request::RemoveForge(value) => Kind::Remove(value.forge()),
                w::Request::EndPlayground => Kind::Reset,
                w::Request::Adopt(_)
                | w::Request::SetUp(_)
                | w::Request::StartPlayground
                | w::Request::Return(_)
                | w::Request::Connect(_)
                | w::Request::Install(_)
                | w::Request::Reachable(_)
                | w::Request::Accept(_) => return false,
            };
            state.making.insert(*key, Pending { kind, request: request.clone(), called: None });
            true
        }
        Told::Answered { key, outcome } => {
            let Some(pending) = state.making.remove(key) else { return false };
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
#[derive(Debug)]
enum Answer {
    Made(u16),
    Full,
    NotOffered,
    Removing(u16),
    NoSuchForge,
    Playground,
    Reset,
    Entrance(c::EntranceRefusal),
    Unknown,
}
fn answer(kind: &Kind, value: &Answered, limits: &Limits) -> Option<Answer> {
    let record = match value {
        Answered::Family(record) => record,
        Answered::Jig(c::Answer::Refused { refusal, .. }) => {
            return Some(Answer::Entrance(c::EntranceRefusal::new(refusal.clone()).ok()?));
        }
        Answered::Jig(c::Answer::Done(_)) => return None,
    };
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    let bounds = temper_client_domain::forge::wire_limits(&limits.domain.forge)?;
    let mut reader = skein_lib::Reader::new(&record.bytes);
    let value = w::Answer::decode(&bounds, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    Some(match value.into_parts().outcome {
        w::Outcome::Made(value) if *kind == Kind::Make => Answer::Made(value.forge()),
        w::Outcome::Full if *kind == Kind::Make => Answer::Full,
        w::Outcome::NotOffered if *kind == Kind::Make || *kind == Kind::Reset => Answer::NotOffered,
        w::Outcome::Removing => match kind.forge() {
            Some(forge) => Answer::Removing(forge),
            None => Answer::Unknown,
        },
        w::Outcome::ConnectRefused(value) if kind.forge().is_some() && *value.why() == w::ForgeRefusal::NoSuchForge => {
            Answer::NoSuchForge
        }
        w::Outcome::Playground if kind.forge().is_some() => Answer::Playground,
        w::Outcome::Reset if *kind == Kind::Reset => Answer::Reset,
        w::Outcome::Made(_)
        | w::Outcome::Full
        | w::Outcome::NotOffered
        | w::Outcome::ConnectRefused(_)
        | w::Outcome::Playground
        | w::Outcome::Reset
        | w::Outcome::Page(_)
        | w::Outcome::Returning
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
        | w::Outcome::Unknown
        | w::Outcome::Accepted(_) => Answer::Unknown,
    })
}
fn text(answer: &Answer, word: &[u8], kept: bool, limits: &Limits, out: &mut Text) {
    if kept {
        out.put(b"kept: ");
        out.put(word);
        out.put(b" ");
    }
    match answer {
        Answer::Made(forge) => {
            out.put(b"made: forge ");
            out.number(u64::from(*forge));
        }
        Answer::Removing(forge) => {
            out.put(b"removing: forge ");
            out.number(u64::from(*forge));
        }
        Answer::Full => out.put(b"refused: full"),
        Answer::NotOffered => out.put(b"refused: not offered"),
        Answer::NoSuchForge => out.put(b"refused: no such forge"),
        Answer::Playground => out.put(b"refused: the playground's forge"),
        Answer::Reset => out.put(b"reset"),
        Answer::Unknown => out.put(b"unknown"),
        Answer::Entrance(why) => {
            render::acts::refusal(why.refusal(), &render::document::phrasebook(limits.tools.text), out);
        }
    }
    out.put(b"\n");
}
fn json(answer: &Answer, word: &[u8], kept: bool, out: &mut Encoder) {
    if kept {
        out.object_start();
        out.key(b"operation");
        out.string(word);
        out.key(b"outcome");
    }
    out.object_start();
    out.key(match answer {
        Answer::Made(_) => b"made",
        Answer::Full => b"full",
        Answer::NotOffered => b"not_offered",
        Answer::Removing(_) => b"removing",
        Answer::NoSuchForge => b"no_such_forge",
        Answer::Playground => b"playground",
        Answer::Reset => b"reset",
        Answer::Entrance(_) => b"entrance_refused",
        Answer::Unknown => b"unknown",
    });
    out.object_start();
    match answer {
        Answer::Made(forge) | Answer::Removing(forge) => {
            out.key(b"forge");
            out.unsigned(u64::from(*forge));
        }
        Answer::Entrance(why) => {
            out.key(b"refusal");
            render::document::refusal_json(why.refusal(), out);
        }
        Answer::Full
        | Answer::NotOffered
        | Answer::NoSuchForge
        | Answer::Playground
        | Answer::Reset
        | Answer::Unknown => {}
    }
    out.object_end();
    out.object_end();
    if kept {
        out.object_end();
    }
}
fn print(pending: &Pending, outcome: &Outcome, structured: bool, limits: &Limits) -> jig::Printed {
    let kept = pending.called.is_none();
    let word = pending.kind.word();
    let answer = match outcome {
        Outcome::Full { held, room_at } => {
            return render::answers_full(crate::root::DIAGNOSTIC_PREFIX, *held, *room_at);
        }
        Outcome::Answered(value) => answer(&pending.kind, value, limits).unwrap_or(Answer::Unknown),
        Outcome::Unknown => Answer::Unknown,
        Outcome::Held | Outcome::Limit | Outcome::Unconfirmed | Outcome::NotKept | Outcome::Unsent | Outcome::Lost => {
            let mut operation = render::outcome::Operation::of(&pending.request);
            operation.word = word;
            if kept {
                return jig::Printed {
                    file: jig::FileEnd::Keep,
                    status: 1,
                    stdout: Box::default(),
                    stderr: render::outcome::kept(
                        &operation,
                        outcome,
                        structured,
                        None,
                        &limits.tools,
                        crate::root::DIAGNOSTIC_PREFIX,
                    ),
                };
            }
            return render::outcome::print(
                &operation,
                outcome,
                structured,
                None,
                &limits.tools,
                crate::root::DIAGNOSTIC_PREFIX,
            );
        }
    };
    let status = match answer {
        Answer::Made(_) | Answer::Removing(_) | Answer::Reset => 0,
        Answer::Unknown => 1,
        Answer::Full | Answer::NotOffered | Answer::NoSuchForge | Answer::Playground | Answer::Entrance(_) => 2,
    };
    let bytes = if structured {
        let bounds = skein_json::writer::Limits { depth: 8, length: limits.tools.text };
        let mut measure = Encoder::measure(&bounds);
        json(&answer, word, kept, &mut measure);
        let Ok(size) = measure.measured() else { return render::failure(b"result limit", true, 1) };
        let mut out = Encoder::write(size, &bounds);
        json(&answer, word, kept, &mut out);
        let bytes = out.finish();
        if kept { render::outcome::newline(&bytes) } else { bytes }
    } else {
        let mut measure = Text::measure();
        text(&answer, word, kept, limits, &mut measure);
        let mut out = Text::writing_exact(measure);
        text(&answer, word, kept, limits, &mut out);
        out.finish()
    };
    if bytes.len() > usize::try_from(limits.tools.text).expect("text fits") {
        return render::failure(b"result limit", structured, 1);
    }
    if status == 0 {
        jig::Printed { file: jig::FileEnd::Keep, status, stdout: bytes, stderr: Box::default() }
    } else {
        jig::Printed { file: jig::FileEnd::Keep, status, stdout: Box::default(), stderr: bytes }
    }
}
