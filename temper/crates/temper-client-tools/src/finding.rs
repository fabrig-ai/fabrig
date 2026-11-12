//! Findings on the forge family (temper/docs/design/client/ctl/tools.md, 2 and 3).
mod render;
use crate::{
    jig::{
        self,
        call::{Call, Project, Value, value},
        rows,
        runner::{Called, Out},
    },
    root::Limits,
};
use alloc::{boxed::Box, collections::BTreeMap};
use jig_client::{
    Answered, Ask, Client, Event, Family, Handle, Key, Latest, Outcome, Readable, Readout, State, Told, Watchable, Why,
    contract as c,
};
use skein_lib::{Env, Queue};
use temper_client_domain::{forge, owner::Owner};
use temper_client_protocol_wire::{self as wire, v1 as w};

#[derive(Debug)]
pub(crate) struct Findings {
    waiting: BTreeMap<u32, Called>,
    sent: BTreeMap<Key, Pending>,
    views: BTreeMap<Handle, Pending>,
    reads: BTreeMap<Handle, Pending>,
    pub(crate) json: bool,
}
#[derive(Clone, Copy, Debug)]
enum Act {
    List,
    Show,
    File,
    Hand,
    Route,
    Backlog,
    Drop,
}
#[derive(Debug)]
struct Pending {
    called: Option<Called>,
    json: bool,
    project: u32,
    number: u32,
    area: Option<Box<[u8]>>,
    act: Act,
}
impl Findings {
    pub(crate) fn new() -> Self {
        Self {
            waiting: BTreeMap::new(),
            sent: BTreeMap::new(),
            views: BTreeMap::new(),
            reads: BTreeMap::new(),
            json: false,
        }
    }
    pub(crate) fn calls(&self) -> usize {
        self.waiting
            .len()
            .checked_add(self.sent.len())
            .expect("bounded calls")
            .checked_add(self.views.len())
            .expect("bounded calls")
            .checked_add(self.reads.len())
            .expect("bounded calls")
    }
}
pub(crate) fn owns(call: &Call) -> bool {
    (rows::FINDING_LIST_AT..=rows::FINDING_DROP_AT).contains(&call.tool)
}
pub(crate) fn call(state: &mut Findings, client: &Client, env: &Env<Limits>, called: Called, out: &mut Queue<Out>) {
    state.waiting.insert(called.call_number, called);
    ready(state, client, env, out);
}
pub(crate) fn ready(state: &mut Findings, client: &Client, env: &Env<Limits>, out: &mut Queue<Out>) {
    let Some(me) = crate::forge::me(client) else { return };
    for _ in 0..state.waiting.len() {
        let Some((&number, _)) = state.waiting.first_key_value() else { break };
        let called = state.waiting.remove(&number).expect("waiting call");
        if called.value.is_some() || jig::argv::check(crate::table::TOOLS, &called.call, &env.limits.tools).is_err() {
            fail(called, b"invalid finding call", out);
            continue;
        }
        let prepared = prepare(&called.call, me);
        let (project, finding, area, act, request) = match prepared {
            Ok(prepared) => prepared,
            Err(why) => {
                fail(called, why, out);
                continue;
            }
        };
        let pending = Pending { json: called.json, called: Some(called), project, number: finding, area, act };
        let handle = Handle { asker: Owner::Forge.asker(), number: (u64::from(number) << 32_u32) | 2 };
        match act {
            Act::List => watch(state, handle, pending, out),
            Act::Show => read(state, handle, pending, out),
            Act::File | Act::Hand | Act::Route | Act::Backlog | Act::Drop => {
                let key = jig::key::local(Owner::Forge.asker(), number);
                let request =
                    w::Requesting::new(&w::CEILINGS, w::RequestingParts { request: request.expect("act request") })
                        .expect("request");
                let name = Some(crate::key::engine(
                    pending.called.as_ref().expect("live call").call.key.as_ref().expect("keyed act"),
                ));
                out.push(Out::Client {
                    event: Event::Send {
                        key,
                        name,
                        request: Ask::Family {
                            scope: c::Scope::Project(project),
                            record: family(wire::requesting(&request)),
                        },
                    },
                });
                state.sent.insert(key, pending);
            }
        }
    }
}
fn family(bytes: Box<[u8]>) -> Family {
    Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes }
}
fn text(call: &Call, index: usize) -> Result<&[u8], &'static [u8]> {
    match value(call, index) {
        Some(Value::Text(text)) => Ok(text),
        _ => Err(b"finding text or body not loaded"),
    }
}
fn selected(project: &Project, me: &c::Me) -> Option<u32> {
    match project {
        Project::Number(number) => Some(*number),
        Project::Name(name) => {
            for row in &me.projects {
                if row.name.bytes() == name.as_ref() {
                    return Some(row.project);
                }
            }
            None
        }
    }
}
// The file constructor owns the wire bounds; empty required words are checked here.
#[expect(clippy::type_complexity, reason = "one prepared call carries its read or request context")]
fn prepare(call: &Call, me: &c::Me) -> Result<(u32, u32, Option<Box<[u8]>>, Act, Option<w::Request>), &'static [u8]> {
    let (project, number) = reference(call, me)?;
    let mut area = None;
    let (act, request) = match call.tool {
        rows::FINDING_LIST_AT => (Act::List, None),
        rows::FINDING_SHOW_AT => (Act::Show, None),
        rows::FINDING_FILE_AT => {
            let kind = match value(call, 1) {
                Some(Value::Word(0)) => w::FindingKind::Qa,
                Some(Value::Word(1)) => w::FindingKind::Flake,
                Some(Value::Word(2)) => w::FindingKind::Red,
                Some(Value::Word(3)) => w::FindingKind::Review,
                Some(Value::Word(4)) => w::FindingKind::Feedback,
                _ => return Err(b"invalid finding kind"),
            };
            let severity = match value(call, 2) {
                Some(Value::Word(0)) => w::Severity::Low,
                Some(Value::Word(2)) => w::Severity::High,
                _ => w::Severity::Medium,
            };
            if value(call, 3).is_some() {
                area = Some(Box::from(text(call, 3)?));
            }
            let title = text(call, 4)?;
            if title.is_empty() {
                return Err(b"finding title must not be empty");
            }
            let file = w::FileFinding::new(
                &w::CEILINGS,
                w::FileFindingParts {
                    project,
                    kind,
                    severity,
                    area: area.clone(),
                    title: title.into(),
                    body: text(call, 5)?.into(),
                },
            )
            .or(Err(b"finding words past their bounds".as_slice()))?;
            (Act::File, Some(w::Request::FileFinding(file)))
        }
        rows::FINDING_HAND_AT => {
            area = Some(Box::from(text(call, 1)?));
            let why = text(call, 2)?;
            if why.is_empty() {
                return Err(b"hand-off why must not be empty");
            }
            let hand = w::HandFinding::new(
                &w::CEILINGS,
                w::HandFindingParts { project, number, area: text(call, 1)?.into(), why: why.into() },
            )
            .or(Err(b"hand-off words past their bounds".as_slice()))?;
            (Act::Hand, Some(w::Request::HandFinding(hand)))
        }
        rows::FINDING_ROUTE_AT | rows::FINDING_BACKLOG_AT | rows::FINDING_DROP_AT => {
            let (act, triage) = match call.tool {
                rows::FINDING_ROUTE_AT => (
                    Act::Route,
                    w::Triage::Route(
                        w::FindingRoute::new(
                            &w::CEILINGS,
                            w::FindingRouteParts { task: jig::call::number(call, 1).expect("goal") },
                        )
                        .expect("scalar goal"),
                    ),
                ),
                rows::FINDING_BACKLOG_AT => (Act::Backlog, w::Triage::Backlog),
                _ => {
                    let reason = text(call, 1)?;
                    if reason.is_empty() {
                        return Err(b"drop reason must not be empty");
                    }
                    (
                        Act::Drop,
                        w::Triage::Drop(
                            w::FindingDrop::new(&w::CEILINGS, w::FindingDropParts { reason: reason.into() })
                                .or(Err(b"drop reason past 400 bytes".as_slice()))?,
                        ),
                    )
                }
            };
            (
                act,
                Some(w::Request::TriageFinding(
                    w::TriageFinding::new(&w::CEILINGS, w::TriageFindingParts { project, number, act: triage })
                        .expect("triage"),
                )),
            )
        }
        _ => unreachable!("finding tool"),
    };
    Ok((project, number, area, act, request))
}
fn reference(call: &Call, me: &c::Me) -> Result<(u32, u32), &'static [u8]> {
    if call.tool == rows::FINDING_FILE_AT || call.tool == rows::FINDING_LIST_AT {
        let Some(Value::Project(project)) = value(call, 0) else { return Err(b"missing project") };
        Ok((selected(project, me).ok_or(b"unknown project".as_slice())?, 0))
    } else {
        let (project, number) = jig::argv::value::finding(text(call, 0)?).ok_or(b"FINDING must be P/N".as_slice())?;
        let parsed = jig::argv::value::decode(
            crate::table::TOOLS,
            jig::table::Kind::Project { empty: false },
            jig::table::Bound::None,
            project,
        )
        .expect("project part");
        let Value::Project(project) = parsed else { unreachable!("project kind") };
        Ok((selected(&project, me).ok_or(b"unknown project".as_slice())?, number))
    }
}
pub(crate) fn subject(project: u32, state: Option<w::FindingState>, mine: bool) -> Watchable {
    let subject =
        w::FindingsSubject::new(&w::CEILINGS, w::FindingsSubjectParts { project, state, mine }).expect("subject");
    let watching =
        w::Watching::new(&w::CEILINGS, w::WatchingParts { watch: w::Watch::Findings(subject) }).expect("watch");
    Watchable::Family {
        scope: c::Scope::Project(project),
        window: c::Window { rows: 0, after: None },
        record: family(wire::watching(&watching)),
    }
}
fn watch(state: &mut Findings, handle: Handle, pending: Pending, out: &mut Queue<Out>) {
    let (filter, mine) = match pending.act {
        Act::List => {
            let call = &pending.called.as_ref().expect("live list").call;
            let filter = match value(call, 1) {
                Some(Value::Word(0)) => Some(w::FindingState::New),
                Some(Value::Word(1)) => Some(w::FindingState::Routed),
                Some(Value::Word(2)) => Some(w::FindingState::Backlog),
                Some(Value::Word(3)) => Some(w::FindingState::Dropped),
                _ => None,
            };
            (filter, value(call, 2).is_some())
        }
        Act::Show | Act::File | Act::Hand | Act::Route | Act::Backlog | Act::Drop => (None, false),
    };
    out.push(Out::Client { event: Event::Watch { handle, subject: subject(pending.project, filter, mine) } });
    state.views.insert(handle, pending);
}
fn read(state: &mut Findings, handle: Handle, pending: Pending, out: &mut Queue<Out>) {
    let read = w::Read::Finding(
        w::FindingRead::new(&w::CEILINGS, w::FindingReadParts { project: pending.project, number: pending.number })
            .expect("scalar read"),
    );
    let reading = w::Reading::new(&w::CEILINGS, w::ReadingParts { read }).expect("read");
    out.push(Out::Client {
        event: Event::Read {
            handle,
            read: Readable::Family {
                window: c::Window { rows: 0, after: None },
                scope: c::Scope::Project(pending.project),
                record: family(wire::reading(&reading)),
            },
        },
    });
    state.reads.insert(handle, pending);
}
fn fail(called: Called, why: &[u8], out: &mut Queue<Out>) {
    let printed = jig::render::failure(why, called.json, 1);
    out.push(Out::Printed { number: called.number, printed });
}
fn finish(pending: Pending, printed: jig::Printed, out: &mut Queue<Out>) {
    match pending.called {
        Some(called) => out.push(Out::Printed { number: called.number, printed }),
        None => out.push(Out::Kept {
            line: if printed.status == 0 { jig::render::outcome::newline(&printed.stdout) } else { printed.stderr },
        }),
    }
}
pub(crate) fn told(state: &mut Findings, client: &Client, env: &Env<Limits>, told: &Told, out: &mut Queue<Out>) {
    match told {
        Told::Kept { key, request } => {
            if let Some(pending) = restored(request, state.json) {
                state.sent.insert(*key, pending);
            }
        }
        Told::Answered { key, outcome } => {
            answered(state, client, env, *key, outcome, out);
        }
        Told::Changed { handle } => {
            let Some(pending) = state.views.get(handle) else { return };
            let Some(document) = client.document(*handle) else { return };
            if (document.state == State::Opening || document.state == State::Behind) && document.latest.is_none() {
                return;
            }
            let bounds = forge::wire_limits(&env.limits.domain.forge).expect("checked limits");
            let findings = match document.latest {
                Some(Latest::Family(record))
                    if record.family == wire::FORGE && record.version == wire::CLIENT_VERSION =>
                {
                    wire::decode_findings(&record.bytes, &bounds)
                }
                _ => None,
            };
            let printed = match findings {
                Some(findings) if findings.project() == pending.project => match pending.act {
                    Act::List => render::list(&findings, pending.json, &env.limits),
                    Act::File | Act::Hand | Act::Show | Act::Route | Act::Backlog | Act::Drop => {
                        unreachable!("finding watch context")
                    }
                },
                _ => jig::render::failure(b"unexpected findings document", pending.json, 1),
            };
            let pending = state.views.remove(handle).expect("view");
            out.push(Out::Client { event: Event::Unwatch { handle: *handle } });
            finish(pending, printed, out);
        }
        Told::Page { handle, page } => {
            let Some(pending) = state.reads.remove(handle) else { return };
            let bounds = forge::wire_limits(&env.limits.domain.forge).expect("checked limits");
            let answer = match page {
                Readout::Family { family: wire::FORGE, record: Some(record) } => {
                    wire::decode_finding_answer(record, &bounds)
                }
                Readout::Limit | Readout::NotServed { .. } | Readout::Jig(_) | Readout::Family { .. } => None,
            };
            let printed = match answer {
                Some(answer) => match answer.result() {
                    w::FindingResult::Finding(detail) if detail.row().number() == pending.number => match pending.act {
                        Act::Show => render::show(pending.project, detail, pending.json, &env.limits),
                        Act::File | Act::Hand => render::acted(&pending, Some(detail.row().holder()), &env.limits),
                        Act::List | Act::Route | Act::Backlog | Act::Drop => unreachable!("finding read context"),
                    },
                    w::FindingResult::NoFinding => jig::render::failure(b"no finding", pending.json, 2),
                    w::FindingResult::Finding(_) => jig::render::failure(b"unexpected finding", pending.json, 1),
                },
                None => jig::render::read(page, client, *handle, pending.json, &env.limits.tools, &crate::root::Parts),
            };
            finish(pending, printed, out);
        }
        Told::Ended { handle, why } => {
            let Some(pending) = state.views.remove(handle) else { return };
            let (words, status): (&[u8], u8) = match why {
                Why::NotAllowed => (b"not_allowed", 2),
                Why::Unknown | Why::SubjectEnded => (b"no findings", 2),
                Why::SignedOut => (b"signed out", 1),
                Why::NotServed => (b"version not served", 1),
                Why::Limit => (b"watch table limit", 1),
            };
            let printed = jig::render::failure(words, pending.json, status);
            finish(pending, printed, out);
        }
        Told::Link { .. } | Told::Pending { .. } | Told::Dropped { .. } | Told::Text { .. } => {}
    }
}
fn answered(
    state: &mut Findings,
    client: &Client,
    env: &Env<Limits>,
    key: Key,
    outcome: &Outcome,
    out: &mut Queue<Out>,
) {
    let Some(mut pending) = state.sent.remove(&key) else { return };
    let bounds = forge::wire_limits(&env.limits.domain.forge).expect("checked limits");
    let answer = match outcome {
        Outcome::Answered(Answered::Family(record))
            if record.family == wire::FORGE && record.version == wire::CLIENT_VERSION =>
        {
            w::Answer::decode(&bounds, &mut skein_lib::Reader::new(&record.bytes)).ok()
        }
        Outcome::NotKept
        | Outcome::Answered(_)
        | Outcome::Unknown
        | Outcome::Limit
        | Outcome::Unconfirmed
        | Outcome::Held
        | Outcome::Unsent
        | Outcome::Full { .. }
        | Outcome::Lost => None,
    };
    let Some(answer) = answer else {
        let operation = jig::render::outcome::Operation::of(&Ask::Family {
            scope: c::Scope::Project(pending.project),
            record: family(Box::default()),
        });
        let printed = jig::render::outcome::print(
            &operation,
            outcome,
            pending.json,
            crate::forge::me(client),
            &env.limits.tools,
            crate::root::DIAGNOSTIC_PREFIX,
        );
        finish(pending, printed, out);
        return;
    };
    match answer.outcome() {
        w::Outcome::Filed(filed) => {
            pending.number = filed.number();
        }
        w::Outcome::Handed => {}
        w::Outcome::Triaged => {
            let printed = render::acted(&pending, None, &env.limits);
            finish(pending, printed, out);
            return;
        }
        w::Outcome::FindingRefused(why) => {
            let printed = render::refused(&pending, why.reason());
            finish(pending, printed, out);
            return;
        }
        w::Outcome::Adopted(_)
        | w::Outcome::CoreRefused(_)
        | w::Outcome::ForgeRefused(_)
        | w::Outcome::Unknown
        | w::Outcome::SettingUp
        | w::Outcome::SetUpRefused(_)
        | w::Outcome::Running
        | w::Outcome::StartRefused(_)
        | w::Outcome::StartFailed(_)
        | w::Outcome::Connecting
        | w::Outcome::Link(_)
        | w::Outcome::ConnectRefused(_)
        | w::Outcome::Page(_)
        | w::Outcome::Accepted(_)
        | w::Outcome::Removed
        | w::Outcome::RemoveRefused(_)
        | w::Outcome::Reset
        | w::Outcome::EndRefused(_)
        | w::Outcome::Dismissed
        | w::Outcome::MakeKept
        | w::Outcome::MakeRefused(_)
        | w::Outcome::Looked
        | w::Outcome::Away
        | w::Outcome::Overturned
        | w::Outcome::OverturnRefused(_)
        | w::Outcome::Tried
        | w::Outcome::NotTheirs => {
            let printed = jig::render::failure(b"unexpected finding outcome", pending.json, 1);
            finish(pending, printed, out);
            return;
        }
    }
    let handle = Handle { asker: key.asker, number: key.number.checked_add(1).expect("read slot") };
    read(state, handle, pending, out);
}
fn restored(ask: &Ask, json: bool) -> Option<Pending> {
    let Ask::Family { record, .. } = ask else { return None };
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    let request = wire::decode_requesting(&record.bytes, &w::CEILINGS)?;
    let (project, number, area, act) = match request.request() {
        w::Request::FileFinding(file) => (
            file.project(),
            0,
            match file.area() {
                Some(area) => Some(area.clone()),
                None => None,
            },
            Act::File,
        ),
        w::Request::HandFinding(hand) => (hand.project(), hand.number(), Some(hand.area().into()), Act::Hand),
        w::Request::TriageFinding(triage) => (
            triage.project(),
            triage.number(),
            None,
            match triage.act() {
                w::Triage::Route(_) => Act::Route,
                w::Triage::Backlog => Act::Backlog,
                w::Triage::Drop(_) => Act::Drop,
            },
        ),
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
        | w::Request::TryAccount(_) => return None,
    };
    Some(Pending { called: None, json, project, number, area, act })
}
