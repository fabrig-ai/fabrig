//! Findings on the forge family (temper/docs/design/client/ctl/tools.md, 2 and 3).
pub(crate) mod render;
use crate::{
    jig::{
        self,
        call::{Call, Project, Value, Values, value},
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
use skein_lib::{Env, List, Queue};
use temper_client_domain_forge::{forge, owner::Owner};
use temper_client_protocol_wire::v1::Record;
use temper_client_protocol_wire::{self as wire, v1 as w};

#[derive(Debug)]
pub(crate) struct Findings {
    waiting: BTreeMap<u32, Called>,
    sent: BTreeMap<Key, Pending>,
    views: BTreeMap<Handle, Pending>,
    reads: BTreeMap<Handle, Pending>,
    pub(crate) json: bool,
}
#[derive(Clone, Debug)]
enum Act {
    List,
    Show,
    File,
    Hand,
    Route,
    Backlog,
    Drop,
    Repair(Box<[u32]>),
    Blame(Option<Box<[u8]>>),
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
        || call.tool == rows::FINDING_REPAIR_AT
        || call.tool == rows::FINDING_BLAME_AT
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
        if check(&called.call).is_err() {
            fail(called, FILTER_FLAGS, out);
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
        match &pending.act {
            Act::List => watch(state, handle, pending, out),
            Act::Show => read(state, handle, pending, out),
            Act::File | Act::Hand | Act::Route | Act::Backlog | Act::Drop | Act::Repair(_) | Act::Blame(_) => {
                let key = jig::key::local(Owner::Forge.asker(), number);
                let request = w::Requesting { request: request.expect("act request") };
                let name = Some(crate::key::engine(
                    pending.called.as_ref().expect("live call").call.key.as_ref().expect("keyed act"),
                ));
                out.push(Out::Client {
                    event: Event::Send {
                        key,
                        name,
                        request: Ask::Family {
                            scope: c::Scope::Project(project),
                            record: family(request.encode(&w::CEILINGS).expect("request")),
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
            let file = {
                let value = w::FileFinding {
                    project,
                    kind,
                    severity,
                    area: area.clone(),
                    title: title.into(),
                    body: text(call, 5)?.into(),
                };
                match value.encode(&w::CEILINGS) {
                    Ok(_) => Ok(value),
                    Err(problem) => Err(problem),
                }
            }
            .or(Err(b"finding words past their bounds".as_slice()))?;
            (Act::File, Some(w::Request::FileFinding(file)))
        }
        rows::FINDING_HAND_AT => {
            area = Some(Box::from(text(call, 1)?));
            let why = text(call, 2)?;
            if why.is_empty() {
                return Err(b"hand-off why must not be empty");
            }
            let hand = {
                let value = w::HandFinding { project, number, area: text(call, 1)?.into(), why: why.into() };
                match value.encode(&w::CEILINGS) {
                    Ok(_) => Ok(value),
                    Err(problem) => Err(problem),
                }
            }
            .or(Err(b"hand-off words past their bounds".as_slice()))?;
            (Act::Hand, Some(w::Request::HandFinding(hand)))
        }
        rows::FINDING_REPAIR_AT => {
            let repair = repair(call, me, project, number)?;
            let act = Act::Repair(repair.more.clone().into_boxed());
            (act, Some(w::Request::TriageFinding(w::TriageFinding { project, number, act: w::Triage::Repair(repair) })))
        }
        rows::FINDING_BLAME_AT => {
            let blame = blame(call)?;
            let act = Act::Blame(blame.landing.clone());
            (act, Some(w::Request::TriageFinding(w::TriageFinding { project, number, act: w::Triage::Blame(blame) })))
        }
        rows::FINDING_ROUTE_AT | rows::FINDING_BACKLOG_AT | rows::FINDING_DROP_AT => {
            let (act, triage) = match call.tool {
                rows::FINDING_ROUTE_AT => {
                    (Act::Route, w::Triage::Route(w::FindingRoute { task: jig::call::number(call, 1).expect("goal") }))
                }
                rows::FINDING_BACKLOG_AT => (Act::Backlog, w::Triage::Backlog),
                _ => {
                    let reason = text(call, 1)?;
                    if reason.is_empty() {
                        return Err(b"drop reason must not be empty");
                    }
                    (
                        Act::Drop,
                        w::Triage::Drop(
                            {
                                let value = w::FindingDrop { reason: reason.into() };
                                match value.encode(&w::CEILINGS) {
                                    Ok(_) => Ok(value),
                                    Err(problem) => Err(problem),
                                }
                            }
                            .or(Err(b"drop reason past 400 bytes".as_slice()))?,
                        ),
                    )
                }
            };
            (act, Some(w::Request::TriageFinding(w::TriageFinding { project, number, act: triage })))
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
        let bytes = if call.tool == rows::FINDING_REPAIR_AT {
            let Some(Some(Values::Many(findings))) = call.values.first() else { return Err(b"missing findings") };
            let Some(Value::Text(first)) = findings.first() else { return Err(b"missing findings") };
            first.as_ref()
        } else {
            text(call, 0)?
        };
        finding_reference(bytes, me)
    }
}
fn finding_reference(bytes: &[u8], me: &c::Me) -> Result<(u32, u32), &'static [u8]> {
    let (project, number) = jig::argv::value::finding(bytes).ok_or(b"FINDING must be P/N".as_slice())?;
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
fn repair(call: &Call, me: &c::Me, project: u32, number: u32) -> Result<w::FindingRepair, &'static [u8]> {
    let Some(Some(Values::Many(findings))) = call.values.first() else { return Err(b"missing findings") };
    let mut more = List::with_capacity(w::CEILINGS.finding_repair_more);
    for finding in findings.iter().skip(1) {
        let Value::Text(bytes) = finding else { return Err(b"FINDING must be P/N") };
        let (next_project, next_number) = finding_reference(bytes, me)?;
        if next_project != project {
            return Err(b"repair findings must be of one project");
        }
        if next_number == number || more.as_slice().contains(&next_number) {
            return Err(b"repair finding named twice");
        }
        more.push(next_number).or(Err(b"repair takes at most 16 findings".as_slice()))?;
    }
    let words = text(call, 1)?;
    if words.is_empty() {
        return Err(b"repair words must not be empty");
    }
    let repair = w::FindingRepair { more, words: words.into() };
    repair.measure(&w::CEILINGS).or(Err(b"repair words past 4096 bytes".as_slice()))?;
    Ok(repair)
}
fn blame(call: &Call) -> Result<w::FindingBlameAct, &'static [u8]> {
    let landing = if call.form == 0 {
        Some(commit(text(call, 1)?).ok_or(b"landing must be 40 or 64 hex digits".as_slice())?)
    } else {
        None
    };
    let why = text(call, 2)?;
    if why.is_empty() {
        return Err(b"blame why must not be empty");
    }
    let blame = w::FindingBlameAct { landing, why: why.into() };
    blame.measure(&w::CEILINGS).or(Err(b"blame why past 200 bytes".as_slice()))?;
    Ok(blame)
}
fn commit(bytes: &[u8]) -> Option<Box<[u8]>> {
    if bytes.len() != 40 && bytes.len() != 64 {
        return None;
    }
    let mut commit = List::with_capacity(32);
    for [high, low] in bytes.as_chunks::<2>().0 {
        let high = hex_digit(*high)?;
        let low = hex_digit(*low)?;
        commit.push(high.checked_mul(16)?.checked_add(low)?).ok()?;
    }
    Some(commit.into_boxed())
}
fn hex_digit(digit: u8) -> Option<u8> {
    if digit.is_ascii_digit() {
        digit.checked_sub(b'0')
    } else if (b'a'..=b'f').contains(&digit) {
        digit.checked_sub(b'a')?.checked_add(10)
    } else if (b'A'..=b'F').contains(&digit) {
        digit.checked_sub(b'A')?.checked_add(10)
    } else {
        None
    }
}
pub(crate) const FILTER_FLAGS: &[u8] =
    b"finding list: --goal and --milestone take neither each other nor --state or --mine";
pub(crate) fn check(call: &Call) -> Result<(), jig::argv::Usage> {
    if call.tool == rows::FINDING_LIST_AT {
        for (left, right) in [(3, 4), (3, 1), (3, 2), (4, 1), (4, 2)] {
            if value(call, left).is_some() && value(call, right).is_some() {
                return Err(jig::argv::Usage::Together {
                    tool: call.tool,
                    form: call.form,
                    first: u8::try_from(left).expect("filter slot"),
                    second: u8::try_from(right).expect("filter slot"),
                });
            }
        }
    }
    Ok(())
}
pub(crate) fn subject(project: u32, state: Option<w::FindingState>, mine: bool) -> Watchable {
    filtered_subject(project, w::FindingsFilter::Triage(w::FindingsTriage { state, mine }))
}
pub(crate) fn filtered_subject(project: u32, filter: w::FindingsFilter) -> Watchable {
    let subject = w::FindingsSubject { project, filter };
    let watching = w::Watching { watch: w::Watch::Findings(subject) };
    Watchable::Family {
        scope: c::Scope::Project(project),
        window: c::Window { rows: 0, after: None },
        record: family(watching.encode(&w::CEILINGS).expect("watch")),
    }
}
fn watch(state: &mut Findings, handle: Handle, pending: Pending, out: &mut Queue<Out>) {
    let (filter, mine) = match &pending.act {
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
        Act::Show | Act::File | Act::Hand | Act::Route | Act::Backlog | Act::Drop | Act::Repair(_) | Act::Blame(_) => {
            (None, false)
        }
    };
    let call = &pending.called.as_ref().expect("live list").call;
    let watching = if let Some(goal) = jig::call::number(call, 3) {
        filtered_subject(pending.project, w::FindingsFilter::Goal(w::FindingsGoal { goal }))
    } else if let Some(milestone) = jig::call::number(call, 4) {
        filtered_subject(pending.project, w::FindingsFilter::Milestone(w::FindingsMilestone { milestone }))
    } else {
        subject(pending.project, filter, mine)
    };
    out.push(Out::Client { event: Event::Watch { handle, subject: watching } });
    state.views.insert(handle, pending);
}
fn read(state: &mut Findings, handle: Handle, pending: Pending, out: &mut Queue<Out>) {
    let read = w::Read::Finding(w::FindingRead { project: pending.project, number: pending.number });
    let reading = w::Reading { read };
    out.push(Out::Client {
        event: Event::Read {
            handle,
            read: Readable::Family {
                window: c::Window { rows: 0, after: None },
                scope: c::Scope::Project(pending.project),
                record: family(reading.encode(&w::CEILINGS).expect("read")),
            },
        },
    });
    state.reads.insert(handle, pending);
}
fn fail(called: Called, why: &[u8], out: &mut Queue<Out>) {
    let status =
        if called.call.tool == rows::FINDING_REPAIR_AT || called.call.tool == rows::FINDING_BLAME_AT { 2 } else { 1 };
    let printed = jig::render::failure(why, called.json, status);
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
                Some(findings) if findings.project == pending.project => match &pending.act {
                    Act::List => render::list(&findings, pending.json, &env.limits),
                    Act::File
                    | Act::Hand
                    | Act::Show
                    | Act::Route
                    | Act::Backlog
                    | Act::Drop
                    | Act::Repair(_)
                    | Act::Blame(_) => {
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
                Some(answer) => match &answer.result {
                    w::FindingResult::Finding(detail) if detail.row.number == pending.number => match &pending.act {
                        Act::Show => render::show(pending.project, detail, pending.json, &env.limits),
                        Act::File | Act::Hand => render::acted(&pending, Some(&detail.row.holder), None, &env.limits),
                        Act::List | Act::Route | Act::Backlog | Act::Drop | Act::Repair(_) | Act::Blame(_) => {
                            unreachable!("finding read context")
                        }
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
#[expect(clippy::too_many_lines, reason = "one exhaustive family outcome dispatcher refuses other request outcomes")]
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
        let tool = match &pending.act {
            Act::File => &rows::FINDING_FILE,
            Act::Hand => &rows::FINDING_HAND,
            Act::Route => &rows::FINDING_ROUTE,
            Act::Backlog => &rows::FINDING_BACKLOG,
            Act::Drop => &rows::FINDING_DROP,
            Act::Repair(_) => &rows::FINDING_REPAIR,
            Act::Blame(_) => &rows::FINDING_BLAME,
            Act::List | Act::Show => unreachable!("act refusal"),
        };
        let operation = jig::render::outcome::Operation::for_call(
            &Ask::Family { scope: c::Scope::Project(pending.project), record: family(Box::default()) },
            tool,
        );
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
    match &answer.outcome {
        w::Outcome::Filed(filed) => {
            pending.number = filed.number;
        }
        w::Outcome::Handed => {}
        w::Outcome::Triaged(value) => {
            let printed = render::acted(&pending, None, value.goal, &env.limits);
            finish(pending, printed, out);
            return;
        }
        w::Outcome::FindingRefused(why) => {
            let printed = render::refused(&pending, &why.reason);
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
        | w::Outcome::Checkpointed(_)
        | w::Outcome::CheckpointRefused(_)
        | w::Outcome::EndRefused(_)
        | w::Outcome::Dismissed
        | w::Outcome::MakeKept
        | w::Outcome::MakeRefused(_)
        | w::Outcome::Looked
        | w::Outcome::Away
        | w::Outcome::Overturned
        | w::Outcome::OverturnRefused(_)
        | w::Outcome::Tried
        | w::Outcome::Drafted(_)
        | w::Outcome::DraftRefused(_)
        | w::Outcome::Follow(_)
        | w::Outcome::Opened(_)
        | w::Outcome::OpenRefused(_)
        | w::Outcome::Tier(_)
        | w::Outcome::DeviceAdded(_)
        | w::Outcome::DeviceRemoved
        | w::Outcome::DeviceRefused(_)
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
    let (project, number, area, act) = match &request.request {
        w::Request::FileFinding(file) => (
            file.project,
            0,
            match &file.area {
                Some(area) => Some(area.clone()),
                None => None,
            },
            Act::File,
        ),
        w::Request::HandFinding(hand) => (hand.project, hand.number, Some(hand.area.clone()), Act::Hand),
        w::Request::TriageFinding(triage) => (
            triage.project,
            triage.number,
            None,
            match &triage.act {
                w::Triage::Route(_) => Act::Route,
                w::Triage::Backlog => Act::Backlog,
                w::Triage::Drop(_) => Act::Drop,
                w::Triage::Repair(value) => Act::Repair(value.more.clone().into_boxed()),
                w::Triage::Blame(value) => Act::Blame(value.landing.clone()),
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
        | w::Request::SetDraft(_)
        | w::Request::OpenChange(_)
        | w::Request::TierAct(_)
        | w::Request::FollowAct(_)
        | w::Request::AddDevice(_)
        | w::Request::RemoveDevice(_)
        | w::Request::Checkpoint
        | w::Request::TryAccount(_) => return None,
    };
    Some(Pending { called: None, json, project, number, area, act })
}
