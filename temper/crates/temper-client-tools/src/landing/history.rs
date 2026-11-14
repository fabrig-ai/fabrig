//! Bounded history reads and change-history paging
//! (temper/docs/design/domain/landing-history.md, 5.1 and 5.2).
mod render;
use crate::{
    jig::{
        self,
        call::{Value, number, value},
        render as print,
        runner::{Called, Out},
        table::{
            Keyed, Kind,
            Occurs::{Optional, Required},
            Tool, flag,
        },
    },
    root::Limits,
};
use alloc::collections::BTreeMap;
use jig_client::{Client, Event, Family, Handle, Readable, Readout, Told, contract as c};
pub use render::{event_json, event_line};
use skein_lib::{Duration, Env, List, Queue, Time};
use temper_client_protocol_wire::{self as wire, v1 as w};
pub const HISTORY: Tool = Tool {
    name: b"landing history", line: b"Read a page of a project's landing history, in event order",
    prints: b"The project's upper bound and whole-since time, an event per line, then the next command or end",
    result: b"history", forms: &[&[
        flag(b"project", b"PROJECT", Kind::Project { empty: false }, Required, b"The project"),
        flag(b"change", b"TASK", Kind::NumberRange { min: 1, max: u64::MAX }, Optional, b"Only this change; not with --landing"),
        flag(b"landing", b"TASK", Kind::NumberRange { min: 1, max: u64::MAX }, Optional, b"Only this landing; not with --change"),
        flag(b"since", b"TIME", Kind::Literal, Optional, b"At or after this UTC minute, YYYY-MM-DDTHH:MMZ"),
        flag(b"until", b"TIME", Kind::Literal, Optional, b"Before this UTC minute, YYYY-MM-DDTHH:MMZ"),
        flag(b"after", b"CURSOR", Kind::Cursor, Optional, b"Continue with the 17-byte cursor, in lowercase hexadecimal"),
    ]], keyed: Keyed::Never, area: 6,
    notes: b"Reads at most 32 events, including events whose change record has gone.\n--since and --until filter time, not a wait; the range is half-open.\nA cursor fixes the first page's upper bound. A swept cursor is refused\n(exit 2), naming the first event that remains and the read again.\n--json prints every event field, full commits and the next cursor.\n",
};
#[derive(Debug)]
pub(crate) struct History {
    calls: BTreeMap<u32, Pending>,
}
#[derive(Debug)]
struct Pending {
    called: Called,
    query: Option<w::HistoryRead>,
    handle: Option<Handle>,
    serial: u32,
    retry: Option<Time>,
    until: Time,
}
impl History {
    pub(crate) fn new() -> Self {
        Self { calls: BTreeMap::new() }
    }
    pub(crate) fn calls(&self) -> usize {
        self.calls.len()
    }
    pub(crate) fn next_deadline(&self) -> Option<Time> {
        let mut next: Option<Time> = None;
        for pending in self.calls.values() {
            if let Some(at) = pending.retry {
                next = Some(match next {
                    Some(old) => at.min(old),
                    None => at,
                });
            }
        }
        next
    }
}
pub(crate) fn check(call: &jig::call::Call) -> Result<(), jig::argv::Usage> {
    if value(call, 1).is_some() && value(call, 2).is_some() {
        return Err(jig::argv::Usage::Kind { tool: call.tool, form: call.form, argument: 2 });
    }
    for index in [3_u8, 4] {
        if let Some(Value::Text(text)) = value(call, usize::from(index))
            && jig::argv::deadline(text).is_none()
        {
            return Err(jig::argv::Usage::Kind { tool: call.tool, form: call.form, argument: index });
        }
    }
    if let Some(Value::Cursor(bytes)) = value(call, 5)
        && (bytes.len() != 17 || bytes.first() != Some(&1))
    {
        return Err(jig::argv::Usage::Kind { tool: call.tool, form: call.form, argument: 5 });
    }
    Ok(())
}
pub(crate) fn call(state: &mut History, client: &Client, env: &Env<Limits>, called: Called, out: &mut Queue<Out>) {
    if called.value.is_some()
        || jig::argv::check(crate::table::TOOLS, &called.call, &env.limits.tools).is_err()
        || check(&called.call).is_err()
    {
        let printed = print::failure(b"invalid history call", called.json, 1);
        out.push(Out::Printed { number: called.number, printed });
        return;
    }
    state.calls.insert(
        called.call_number,
        Pending {
            called,
            query: None,
            handle: None,
            serial: 0,
            retry: None,
            until: env.now.saturating_add(env.limits.tools.wait),
        },
    );
    ready(state, client, env, out);
}
pub(crate) fn ready(state: &mut History, client: &Client, _env: &Env<Limits>, out: &mut Queue<Out>) {
    let Some(me) = crate::forge::me(client) else { return };
    let mut failed = List::with_capacity(u32::try_from(state.calls.len()).expect("bounded calls"));
    for (&id, pending) in &mut state.calls {
        if pending.query.is_some() {
            continue;
        }
        let project = match value(&pending.called.call, 0) {
            Some(Value::Project(jig::call::Project::Number(project))) => Some(*project),
            Some(Value::Project(jig::call::Project::Name(name))) => {
                let mut found = None;
                for project in &me.projects {
                    if project.name.bytes() == name.as_ref() {
                        found = Some(project.project);
                        break;
                    }
                }
                found
            }
            _ => None,
        };
        let Some(project) = project else {
            failed.push(id).expect("bounded calls");
            continue;
        };
        let call = &pending.called.call;
        let of = if let Some(task) = number(call, 1) {
            w::HistoryOf::Change(task_record(task))
        } else if let Some(task) = number(call, 2) {
            w::HistoryOf::Landing(task_record(task))
        } else {
            w::HistoryOf::Project
        };
        let since = time(call, 3);
        let until = time(call, 4);
        let after = match value(call, 5) {
            Some(Value::Cursor(bytes)) => Some(bytes.as_ref().try_into().expect("checked cursor")),
            _ => None,
        };
        pending.query = Some(
            w::HistoryRead::new(&w::CEILINGS, w::HistoryReadParts { project, of, since, until, after })
                .expect("checked query"),
        );
        send(pending, out);
    }
    for &id in &failed {
        let json = state.calls.get(&id).expect("active call").called.json;
        complete(state, id, print::failure(b"unknown project", json, 1), out);
    }
}
fn task_record(task: u64) -> w::HistoryTask {
    w::HistoryTask::new(&w::CEILINGS, w::HistoryTaskParts { task }).expect("scalar task")
}
fn time(call: &jig::call::Call, index: usize) -> Option<u64> {
    let Some(Value::Text(text)) = value(call, index) else { return None };
    Some(jig::argv::deadline(text).expect("checked time").as_nanos())
}
/// The application read used by the command and change-history pager.
pub(crate) fn read(query: w::HistoryRead) -> Readable {
    let project = query.project();
    let reading =
        w::Reading::new(&w::CEILINGS, w::ReadingParts { read: w::Read::History(query) }).expect("history read");
    Readable::Family {
        scope: c::Scope::Project(project),
        window: c::Window { rows: 0, after: None },
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::reading(&reading) },
    }
}
fn send(pending: &mut Pending, out: &mut Queue<Out>) {
    pending.serial = pending.serial.checked_add(1).expect("retry count within wait");
    let handle = Handle {
        asker: temper_client_domain::owner::Owner::Forge.asker(),
        number: (u64::from(pending.called.call_number) << 32_u32) | u64::from(pending.serial),
    };
    pending.handle = Some(handle);
    pending.retry = None;
    out.push(Out::Client { event: Event::Read { handle, read: read(pending.query.clone().expect("resolved query")) } });
}
fn complete(state: &mut History, id: u32, printed: jig::Printed, out: &mut Queue<Out>) {
    let pending = state.calls.remove(&id).expect("active history call");
    out.push(Out::Printed { number: pending.called.number, printed });
}
pub(crate) fn decode(page: &Readout, limits: &Limits) -> Option<w::HistoryReply> {
    let Readout::Family { family: wire::FORGE, record: Some(record) } = page else { return None };
    let bounds = temper_client_domain::forge::wire_limits(&limits.domain.forge).expect("checked wire limits");
    Some(wire::decode_history_answer(record, &bounds)?.into_parts().result)
}
pub(crate) fn told(state: &mut History, client: &Client, env: &Env<Limits>, told: &Told, out: &mut Queue<Out>) {
    let Told::Page { handle, page } = told else { return };
    let id = u32::try_from(handle.number >> 32_u32).expect("call number");
    let Some(pending) = state.calls.get_mut(&id) else { return };
    if pending.handle != Some(*handle) {
        return;
    }
    pending.handle = None;
    let answer = decode(page, &env.limits);
    if answer == Some(w::HistoryReply::Busy) || *page == Readout::Jig(c::ReadResult::None(c::ReadNone::Busy)) {
        if env.now < pending.until {
            pending.retry = Some(env.now.saturating_add(Duration::from_secs(1)).min(pending.until));
            return;
        }
        let printed = print::failure(b"busy", pending.called.json, 1);
        complete(state, id, printed, out);
        return;
    }
    let printed = match answer {
        Some(w::HistoryReply::Page(page)) => {
            render::page(pending.query.as_ref().expect("query"), &page, pending.called.json, &env.limits)
        }
        Some(w::HistoryReply::Gone(gone)) => {
            render::gone(pending.query.as_ref().expect("query"), &gone, pending.called.json)
        }
        Some(w::HistoryReply::Busy) => unreachable!("busy read retried"),
        None => print::read(page, client, *handle, pending.called.json, &env.limits.tools, &crate::root::Parts),
    };
    complete(state, id, printed, out);
}
pub(crate) fn fire(state: &mut History, env: &Env<Limits>, out: &mut Queue<Out>) {
    let mut expired = List::with_capacity(u32::try_from(state.calls.len()).expect("bounded calls"));
    for (&id, pending) in &mut state.calls {
        if let Some(at) = pending.retry
            && at <= env.now
        {
            if env.now >= pending.until {
                expired.push(id).expect("bounded calls");
            } else {
                send(pending, out);
            }
        }
    }
    for &id in &expired {
        let json = state.calls.get(&id).expect("active history call").called.json;
        complete(state, id, print::failure(b"busy", json, 1), out);
    }
}
/// State for reading a change's history, every page to its end, at most 16.
/// Send the read from `read_change_history`, feed each decoded page to `accept`,
/// and send its next read until none. The caller retries Busy within its wait,
/// without accepting or advancing a page.
#[derive(Debug)]
pub struct ChangeHistory {
    query: w::HistoryRead,
    pages: u8,
    events: List<w::HistoryEvent>,
}
/// Begin reading a change's history without depending on its Change or task row.
#[must_use]
pub fn read_change_history(project: u32, change: u64) -> (ChangeHistory, Readable) {
    let query = w::HistoryRead::new(
        &w::CEILINGS,
        w::HistoryReadParts {
            project,
            of: w::HistoryOf::Change(task_record(change)),
            since: None,
            until: None,
            after: None,
        },
    )
    .expect("change history query");
    (ChangeHistory { query: query.clone(), pages: 0, events: List::with_capacity(512) }, read(query))
}
impl ChangeHistory {
    #[must_use]
    pub fn events(&self) -> &List<w::HistoryEvent> {
        &self.events
    }
    pub(crate) fn gone(&self, gone: &w::HistoryGone, json: bool) -> jig::Printed {
        render::gone(&self.query, gone, json)
    }
    pub fn accept(&mut self, page: w::HistoryPage) -> Option<Readable> {
        if self.pages >= 16 {
            return None;
        }
        let parts = page.into_parts();
        for event in parts.events.into_boxed() {
            self.events.push(event).expect("16 pages of 32 events");
        }
        self.pages = self.pages.checked_add(1).expect("at most 16 pages");
        if self.pages == 16 || parts.after.is_none() {
            return None;
        }
        let mut query = self.query.clone().into_parts();
        query.after = parts.after;
        self.query = w::HistoryRead::new(&w::CEILINGS, query).expect("continued query");
        Some(read(self.query.clone()))
    }
}
