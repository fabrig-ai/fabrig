//! Landed drafts, the writer's door (temper/docs/design/client/ctl/areas.md), area 7.
use crate::{
    jig::{
        self,
        call::{Value, number, value},
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
use jig_client::{Client, Event, Handle, Key, Latest, Outcome, State, Told, Why};
use skein_json::writer::Encoder;
use skein_lib::{Env, Queue, Time, Wall};
use temper_client_domain::{
    forge::{self, drafts},
    owner::Owner,
};
use temper_client_protocol_wire::{self as wire, v1 as w};

const PROJECT: jig::table::Argument =
    flag(b"project", b"PROJECT", Kind::Project { empty: false }, Required, b"The project");
const OWNER: jig::table::Argument =
    flag(b"owner", b"OWNER", Kind::Literal, Optional, b"The repository's owner; give --name too");
const NAME: jig::table::Argument =
    flag(b"name", b"NAME", Kind::Literal, Optional, b"The repository's name; give --owner too");
const PATH: jig::table::Argument =
    positional(b"path", b"PATH", Kind::Literal, Required, b"The landed draft's relative Markdown path");
pub const READY: Tool = Tool {
    name: b"draft ready", line: b"Say a landed draft is ready to start a goal",
    prints: b"`ready: OWNER/NAME PATH at COMMIT on BRANCH`, then its title", result: b"draft",
    forms: &[&[PROJECT, OWNER, NAME, flag(b"chat", b"TASK", Kind::Number, Optional, b"The chat it was written in"), PATH]],
    keyed: Keyed::Required, area: 7,
    notes: b"Give --owner and --name together, or neither for the project's home.\nReads PATH at the head of the repository's landing branch.\nCOMMIT is its first 12 digits; --json keeps the whole row.\n",
};
pub const NOT_READY: Tool = Tool {
    name: b"draft not-ready", line: b"Say a landed draft is no longer ready",
    prints: b"`not ready: OWNER/NAME PATH`", result: b"draft",
    forms: &[&[PROJECT, OWNER, NAME, PATH]], keyed: Keyed::Required, area: 7,
    notes: b"Give --owner and --name together, or neither for the project's home.\nWork stops listing the draft. --json keeps the answered row.\n",
};
pub const LIST: Tool = Tool {
    name: b"draft list", line: b"List a project's landed drafts, newest first",
    prints: b"A line per draft: repository, path, readiness or goal, commit, branch, time, chat and title; `No landed drafts.` with none", result: b"drafts",
    forms: &[&[PROJECT,
        flag(b"until", b"CONDITION", Kind::Word { set: &[b"changed"] }, Optional, b"Wait until the drafts change"),
        flag(b"timeout", b"DURATION", Kind::Duration, Optional, b"How long a wait lasts; 300 seconds when absent, at most 3600"),
    ]], keyed: Keyed::Never, area: 7,
    notes: b"Lists ready and not-ready drafts, including those with a goal.\n--json prints the rows with each commit whole.\n",
};
#[derive(Debug)]
pub(crate) struct Drafts {
    waiting: BTreeMap<u32, Called>,
    pending: BTreeMap<Key, Pending>,
    watches: BTreeMap<Handle, Watching>,
    json: bool,
}
#[derive(Debug)]
struct Pending {
    request: w::SetDraftParts,
    project: Box<[u8]>,
    called: Option<Called>,
}
#[derive(Debug)]
struct Watching {
    called: Called,
    project: u32,
    first: Option<Box<[u8]>>,
    deadline: Option<Time>,
}
impl Drafts {
    pub(crate) fn new() -> Self {
        Self { waiting: BTreeMap::new(), pending: BTreeMap::new(), watches: BTreeMap::new(), json: false }
    }
    pub(crate) fn calls(&self) -> usize {
        self.waiting
            .len()
            .checked_add(self.pending.len())
            .expect("bounded calls")
            .checked_add(self.watches.len())
            .expect("bounded calls")
    }
    pub(crate) fn kept_json(&mut self, json: bool) {
        self.json = json;
    }
    pub(crate) fn next_deadline(&self) -> Option<Time> {
        let mut next = None;
        for watch in self.watches.values() {
            if let Some(at) = watch.deadline {
                next = Some(match next {
                    Some(old) => at.min(old),
                    None => at,
                });
            }
        }
        next
    }
}
fn tool(call: &jig::call::Call) -> &'static Tool {
    crate::table::TOOLS.get(usize::from(call.tool)).expect("parsed tool")
}
pub(crate) fn owns(call: &jig::call::Call) -> bool {
    let name = tool(call).name;
    name == READY.name || name == NOT_READY.name || name == LIST.name
}
fn finish(called: Called, printed: jig::Printed, out: &mut Queue<Out>) {
    out.push(Out::Printed { number: called.number, printed });
}
fn fail(called: Called, why: &[u8], out: &mut Queue<Out>) {
    let printed = render::failure(why, called.json, 1);
    finish(called, printed, out);
}
pub(crate) fn call(state: &mut Drafts, client: &Client, env: &Env<Limits>, called: Called, out: &mut Queue<Out>) {
    if called.value.is_some() || jig::argv::check(crate::table::TOOLS, &called.call, &env.limits.tools).is_err() {
        fail(called, b"invalid draft call", out);
        return;
    }
    if tool(&called.call).name != LIST.name && value(&called.call, 1).is_some() != value(&called.call, 2).is_some() {
        fail(called, b"--owner and --name go together", out);
        return;
    }
    if tool(&called.call).name == LIST.name && value(&called.call, 2).is_some() && value(&called.call, 1).is_none() {
        fail(called, b"--timeout needs --until", out);
        return;
    }
    state.waiting.insert(called.call_number, called);
    ready(state, client, env, out);
}
pub(crate) fn ready(state: &mut Drafts, client: &Client, env: &Env<Limits>, out: &mut Queue<Out>) {
    let Some(me) = crate::forge::me(client) else { return };
    for _ in 0..state.waiting.len() {
        let Some((_, called)) = state.waiting.pop_first() else { break };
        let Some(project) = jig::runner::reads::project(&called.call, me) else {
            fail(called, b"unknown project", out);
            continue;
        };
        if tool(&called.call).name == LIST.name {
            let deadline = if jig::call::waiting(&LIST, &called.call) {
                let Some(Value::Duration(timeout)) = value(&called.call, 2) else { unreachable!("checked timeout") };
                Some(env.now.saturating_add(*timeout))
            } else {
                None
            };
            let handle = Handle { asker: Owner::Forge.asker(), number: (u64::from(called.call_number) << 32u32) | 1 };
            state.watches.insert(handle, Watching { called, project, first: None, deadline });
            out.push(Out::Client {
                event: Event::Watch {
                    handle,
                    subject: drafts::watch(project, &env.limits.domain.forge).expect("checked limits"),
                },
            });
        } else {
            send(state, client, env, called, project, out);
        }
    }
}
fn text(call: &jig::call::Call, at: usize) -> Box<[u8]> {
    let Some(Value::Text(bytes)) = value(call, at) else { unreachable!("checked literal") };
    bytes.clone()
}
fn send(state: &mut Drafts, client: &Client, env: &Env<Limits>, called: Called, project: u32, out: &mut Queue<Out>) {
    let bounds = forge::wire_limits(&env.limits.domain.forge).expect("checked limits");
    let repository = if value(&called.call, 1).is_some() {
        let Ok(repository) = w::DraftRepository::new(
            &bounds,
            w::DraftRepositoryParts { owner: text(&called.call, 1), name: text(&called.call, 2) },
        ) else {
            fail(called, b"repository past the forge's bounds", out);
            return;
        };
        Some(repository)
    } else {
        None
    };
    let is_ready = tool(&called.call).name == READY.name;
    let change = if is_ready {
        w::DraftChange::Ready(
            w::ReadyDraft::new(&bounds, w::ReadyDraftParts { chat: number(&called.call, 3) }).expect("chat fits"),
        )
    } else {
        w::DraftChange::NotReady
    };
    let path = text(&called.call, if is_ready { 4 } else { 3 });
    let Some(request) =
        drafts::ask(project, repository.clone(), path.clone(), change.clone(), &env.limits.domain.forge)
    else {
        fail(called, b"draft past the forge's bounds", out);
        return;
    };
    let key = jig::key::local(Owner::Forge.asker(), called.call_number);
    if client.pending(key).is_some() {
        fail(called, b"tools' bug; client key collision; report it as a finding", out);
        return;
    }
    let project_name = match value(&called.call, 0) {
        Some(Value::Project(jig::call::Project::Name(name))) => name.clone(),
        _ => Box::from(skein_lib::Decimal::of(u64::from(project)).as_bytes()),
    };
    let name = Some(crate::key::engine(called.call.key.as_ref().expect("draft act requires a key")));
    state.pending.insert(
        key,
        Pending {
            request: w::SetDraftParts { project, repository, path, change },
            project: project_name,
            called: Some(called),
        },
    );
    out.push(Out::Client { event: Event::Send { key, name, request } });
}
pub(crate) fn told(state: &mut Drafts, client: &Client, env: &Env<Limits>, told: &Told, out: &mut Queue<Out>) {
    match told {
        Told::Kept { key, request } => {
            let jig_client::Ask::Family { record, .. } = request else { return };
            if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
                return;
            }
            let Some(request) = wire::decode_requesting(
                &record.bytes,
                &forge::wire_limits(&env.limits.domain.forge).expect("checked limits"),
            ) else {
                return;
            };
            let w::Request::SetDraft(request) = request.into_parts().request else { return };
            let request = request.into_parts();
            let project = Box::from(skein_lib::Decimal::of(u64::from(request.project)).as_bytes());
            state.pending.insert(*key, Pending { request, project, called: None });
        }
        Told::Answered { key, outcome } => {
            let Some(pending) = state.pending.remove(key) else { return };
            let json = match &pending.called {
                Some(called) => called.json,
                None => state.json,
            };
            let printed = print_answer(&pending, outcome, json, &env.limits);
            match pending.called {
                Some(called) => finish(called, printed, out),
                None => {
                    let bytes = if printed.status == 0 { printed.stdout } else { printed.stderr };
                    let line = if bytes.ends_with(b"\n") { bytes } else { render::outcome::newline(&bytes) };
                    out.push(Out::Kept { line });
                }
            }
        }
        Told::Changed { handle } if state.watches.contains_key(handle) => changed(state, client, env, *handle, out),
        Told::Ended { handle, why } if state.watches.contains_key(handle) => {
            let watch = state.watches.get(handle).expect("active watch");
            let (words, status): (&[u8], u8) = match why {
                Why::Unknown | Why::SubjectEnded => (b"unknown drafts", 1),
                Why::NotAllowed => (b"not_allowed", 2),
                Why::SignedOut => (b"signed out", 1),
                Why::NotServed => (b"version not served", 1),
                Why::Limit => (b"watch table limit", 1),
            };
            let printed = render::failure(words, watch.called.json, status);
            complete(state, *handle, printed, out);
        }
        Told::Changed { .. }
        | Told::Ended { .. }
        | Told::Pending { .. }
        | Told::Text { .. }
        | Told::Page { .. }
        | Told::Link { .. }
        | Told::Dropped { .. } => {}
    }
}
fn panel(client: &Client, handle: Handle, limits: &Limits) -> Option<w::DraftsPanel> {
    let doc = client.document(handle)?;
    if doc.state != State::Live {
        return None;
    }
    let Latest::Family(record) = doc.latest? else { return None };
    drafts::panel(record, limits.domain.forge)
}
fn changed(state: &mut Drafts, client: &Client, env: &Env<Limits>, handle: Handle, out: &mut Queue<Out>) {
    let Some(doc) = client.document(handle) else { return };
    if doc.state != State::Live {
        return;
    }
    let watch = state.watches.get_mut(&handle).expect("active watch");
    let Some(panel) = panel(client, handle, &env.limits) else {
        let printed = render::failure(b"drafts document decoding failed", watch.called.json, 1);
        complete(state, handle, printed, out);
        return;
    };
    if panel.project() != watch.project {
        let printed = render::failure(b"unexpected drafts project", watch.called.json, 1);
        complete(state, handle, printed, out);
        return;
    }
    let bytes = wire::drafts_panel(&panel);
    let changed = match &watch.first {
        Some(first) => *first != bytes,
        None => false,
    };
    if watch.deadline.is_none() || changed {
        let printed = print_rows(Some(&panel), watch.called.json, false, &env.limits);
        complete(state, handle, printed, out);
    } else if watch.first.is_none() {
        watch.first = Some(bytes);
    }
}
fn complete(state: &mut Drafts, handle: Handle, printed: jig::Printed, out: &mut Queue<Out>) {
    let watch = state.watches.remove(&handle).expect("active watch");
    out.push(Out::Client { event: Event::Unwatch { handle } });
    finish(watch.called, printed, out);
}
pub(crate) fn fire(state: &mut Drafts, client: &Client, env: &Env<Limits>, out: &mut Queue<Out>) {
    for _ in 0..state.watches.len() {
        let mut expired = None;
        for (&handle, watch) in &state.watches {
            if let Some(at) = watch.deadline
                && at <= env.now
            {
                expired = Some(handle);
                break;
            }
        }
        let Some(handle) = expired else { break };
        let watch = state.watches.get(&handle).expect("active watch");
        let panel = panel(client, handle, &env.limits);
        let printed = print_rows(panel.as_ref(), watch.called.json, true, &env.limits);
        complete(state, handle, printed, out);
    }
}
fn identity(row: &w::LandedDraft, out: &mut Text) {
    out.put(row.owner());
    out.put(b"/");
    out.put(row.name());
    out.put(b" ");
    out.put(row.path());
}
fn commit(row: &w::LandedDraft, out: &mut Text) {
    out.put(row.commit().get(..12).unwrap_or(row.commit()));
}
fn ready_text(row: &w::LandedDraft, ready: bool, out: &mut Text) {
    out.put(if ready { b"ready: " } else { b"not ready: " });
    identity(row, out);
    if ready {
        out.put(b" at ");
        commit(row, out);
        out.put(b" on ");
        out.put(row.branch());
    }
    out.put(b"\n");
    if ready && !row.title().is_empty() {
        out.put(row.title());
        out.put(b"\n");
    }
}
fn refusal(p: &Pending, why: &w::DraftRefusal, out: &mut Text) {
    match why {
        w::DraftRefusal::NoRepository => match &p.request.repository {
            Some(repository) => {
                out.put(b"no repository ");
                out.put(repository.owner());
                out.put(b"/");
                out.put(repository.name());
                out.put(b" in ");
                out.put(&p.project);
            }
            None => {
                out.put(&p.project);
                out.put(b" has no home repository: name one with --owner and --name");
            }
        },
        w::DraftRefusal::Path => {
            out.put(&p.request.path);
            out.put(b" is no draft's path: a relative path ending .md");
        }
        w::DraftRefusal::Missing => {
            out.put(b"no ");
            out.put(&p.request.path);
            out.put(b" at the head of the landing branch: land it first");
        }
        w::DraftRefusal::NotAFile => {
            out.put(&p.request.path);
            out.put(b" is a directory");
        }
        w::DraftRefusal::Unreachable => out.put(b"the forge did not answer: try again"),
        w::DraftRefusal::Unknown => {
            out.put(b"no landed draft at ");
            out.put(&p.request.path);
        }
        w::DraftRefusal::Full => {
            out.put(&p.project);
            out.put(b" keeps 64 landed drafts, every one listed: say one not ready first");
        }
    }
}
fn is_ready(change: &w::DraftChange) -> bool {
    match change {
        w::DraftChange::Ready(_) => true,
        w::DraftChange::NotReady | w::DraftChange::Started(_) => false,
    }
}
fn print_answer(p: &Pending, outcome: &Outcome, json: bool, limits: &Limits) -> jig::Printed {
    if let Outcome::Answered(jig_client::Answered::Family(record)) = outcome {
        if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
            return render::failure(b"draft answer decoding failed", json, 1);
        }
        let Ok(answer) = w::Answer::decode(
            &forge::wire_limits(&limits.domain.forge).expect("checked limits"),
            &mut skein_lib::Reader::new(&record.bytes),
        ) else {
            return render::failure(b"draft answer decoding failed", json, 1);
        };
        if let w::Outcome::Drafted(row) = answer.outcome() {
            let bytes = if json {
                let Some(bytes) = json_row(row, limits) else { return render::failure(b"result limit", json, 1) };
                bytes
            } else {
                let mut measure = Text::measure();
                ready_text(row, is_ready(&p.request.change), &mut measure);
                let mut out = Text::writing_exact(measure);
                ready_text(row, is_ready(&p.request.change), &mut out);
                out.finish()
            };
            return printed(bytes, 0, limits);
        }
        if let w::Outcome::DraftRefused(why) = answer.outcome() {
            let mut measure = Text::measure();
            refusal(p, why.reason(), &mut measure);
            let mut out = Text::writing_exact(measure);
            refusal(p, why.reason(), &mut out);
            return render::failure(&out.finish(), json, 2);
        }
        if let Some(forge::Answer::CoreRefused(why)) =
            forge::answer(&limits.domain, &jig_client::Answered::Family(record.clone()))
        {
            return render::outcome::print(
                &operation(p, limits),
                &Outcome::Answered(jig_client::Answered::Jig(jig_client::contract::Answer::Refused {
                    refusal: why,
                    at: None,
                })),
                json,
                None,
                &limits.tools,
                crate::root::DIAGNOSTIC_PREFIX,
            );
        }
        if let w::Outcome::Unknown = answer.outcome() {
            return render::outcome::print(
                &operation(p, limits),
                &Outcome::Unknown,
                json,
                None,
                &limits.tools,
                crate::root::DIAGNOSTIC_PREFIX,
            );
        }
        return render::failure(b"unexpected draft answer", json, 1);
    }
    render::outcome::print(&operation(p, limits), outcome, json, None, &limits.tools, crate::root::DIAGNOSTIC_PREFIX)
}
fn operation(p: &Pending, limits: &Limits) -> render::outcome::Operation {
    let ask = drafts::ask(
        p.request.project,
        p.request.repository.clone(),
        p.request.path.clone(),
        p.request.change.clone(),
        &limits.domain.forge,
    )
    .expect("validated draft");
    let mut operation =
        render::outcome::Operation::for_call(&ask, if is_ready(&p.request.change) { &READY } else { &NOT_READY });
    operation.word = b"draft";
    operation
}
fn encode_row(row: &w::LandedDraft, out: &mut Encoder) {
    out.object_start();
    out.key(b"forge");
    out.unsigned(u64::from(row.forge()));
    for (key, value) in [
        (b"owner".as_slice(), row.owner()),
        (b"name", row.name()),
        (b"path", row.path()),
        (b"branch", row.branch()),
        (b"commit", row.commit()),
        (b"title", row.title()),
    ] {
        out.key(key);
        out.string(value);
    }
    out.key(b"writer");
    out.unsigned(row.writer());
    out.key(b"at");
    out.unsigned(row.at());
    for (key, value) in [(b"chat".as_slice(), row.chat()), (b"goal", row.goal())] {
        out.key(key);
        match value {
            Some(n) => out.unsigned(*n),
            None => out.null(),
        }
    }
    out.key(b"ready");
    out.boolean(row.ready());
    out.object_end();
}
fn json_row(row: &w::LandedDraft, limits: &Limits) -> Option<Box<[u8]>> {
    let bounds = skein_json::writer::Limits { depth: 3, length: limits.tools.text };
    let mut measure = Encoder::measure(&bounds);
    encode_row(row, &mut measure);
    let mut out = Encoder::write(measure.measured().ok()?, &bounds);
    encode_row(row, &mut out);
    Some(out.finish())
}
fn rows_text(panel: Option<&w::DraftsPanel>, timeout: bool, out: &mut Text) {
    if let Some(panel) = panel {
        if panel.drafts().is_empty() {
            out.put(b"No landed drafts.\n");
        }
        for row in panel.drafts() {
            identity(row, out);
            out.put(b" ");
            match row.goal() {
                Some(goal) => {
                    out.put(b"goal ");
                    out.number(*goal);
                }
                None => out.put(if row.ready() { b"ready" } else { b"not ready" }),
            }
            out.put(b" ");
            commit(row, out);
            out.put(b" ");
            out.put(row.branch());
            out.put(b" ");
            render::document::clock::deadline(Wall::from_nanos(row.at()), out);
            if let Some(chat) = row.chat() {
                out.put(b" chat ");
                out.number(*chat);
            }
            if !row.title().is_empty() {
                out.put(b" ");
                out.put(row.title());
            }
            out.put(b"\n");
        }
    }
    if timeout {
        out.put(b"not met: timeout\n");
    }
}
fn encode_rows(panel: Option<&w::DraftsPanel>, out: &mut Encoder) {
    out.array_start();
    if let Some(panel) = panel {
        for row in panel.drafts() {
            encode_row(row, out);
        }
    }
    out.array_end();
}
fn print_rows(panel: Option<&w::DraftsPanel>, json: bool, timeout: bool, limits: &Limits) -> jig::Printed {
    let bytes = if json {
        let bounds = skein_json::writer::Limits { depth: 4, length: limits.tools.text };
        let mut measure = Encoder::measure(&bounds);
        encode_rows(panel, &mut measure);
        let Ok(size) = measure.measured() else { return render::failure(b"result limit", json, 1) };
        let mut out = Encoder::write(size, &bounds);
        encode_rows(panel, &mut out);
        out.finish()
    } else {
        let mut measure = Text::measure();
        rows_text(panel, timeout, &mut measure);
        let mut out = Text::writing_exact(measure);
        rows_text(panel, timeout, &mut out);
        out.finish()
    };
    printed(bytes, if timeout { 3 } else { 0 }, limits)
}
fn printed(bytes: Box<[u8]>, status: u8, limits: &Limits) -> jig::Printed {
    if bytes.len() > usize::try_from(limits.tools.text).expect("text fits") {
        return render::failure(b"result limit", false, 1);
    }
    jig::Printed { file: jig::FileEnd::Keep, status, stdout: bytes, stderr: Box::default() }
}
