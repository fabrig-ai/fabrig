//! Open a person's pushed branch (temper/docs/design/client/ctl/areas.md, area 6).
use temper_client_protocol_wire::v1::Record;
use crate::{
    jig::{
        self,
        call::{Call, Project, Value, number, value},
        render::{self, Text},
        runner::{Called, Out},
        table::{
            Keyed, Kind,
            Occurs::{Optional, Required},
            Tool, flag, switch,
        },
    },
    root::Limits,
};
use alloc::{boxed::Box, collections::BTreeMap};
use jig_client::{Ask, Client, Event, Key, Outcome, Told, contract as c};
use skein_lib::{Env, Queue};
use temper_client_domain_forge::{forge, owner::Owner};
use temper_client_protocol_wire::{self as wire, v1 as w};

pub const ROW: Tool = Tool {
    name: b"change open",
    line: b"Open a change from a branch you pushed",
    prints: b"`task N`, the change task made; a refusal names why nothing was made",
    result: b"change_open",
    forms: &[&[
        flag(b"project", b"PROJECT", Kind::Project { empty: false }, Required, b"The project"),
        flag(b"repository", b"OWNER/NAME", Kind::Literal, Optional, b"Its repository; the project's only repository when absent"),
        flag(b"branch", b"BRANCH", Kind::Literal, Optional, b"Required: the source branch, 1 to 1024 bytes"),
        flag(b"at", b"COMMIT", Kind::Literal, Optional, b"40 or 64 hexadecimal digits; its head when taking the change when absent"),
        flag(b"into", b"BRANCH", Kind::Literal, Optional, b"The landing branch, 1 to 1024 bytes; the repository's default when absent"),
        flag(b"title", b"TEXT", Kind::Text, Optional, b"Required: the pull request's title, 1 to 256 bytes"),
        flag(b"body", b"TEXT", Kind::Text, Optional, b"The pull request's body after the opening line, 1 to 16384 bytes"),
        flag(b"budget", b"AMOUNT", Kind::Amount, Optional, b"What reviews, resolutions and repairs may spend; or --as-allowed"),
        switch(b"as-allowed", b"Take what your role allows instead of a named budget"),
    ]],
    keyed: Keyed::Required,
    area: 6,
    notes: b"Give --branch, --title, and either --budget or --as-allowed.\n--repository is OWNER/NAME. --at is sent lowercase.\nA replay under the same key answers the first. A change that fails later\nwas opened: read it with change show TASK.\n",
};
#[derive(Debug)]
pub(crate) struct Opens {
    waiting: BTreeMap<u32, Called>,
    pending: BTreeMap<Key, Pending>,
    pub(crate) json: bool,
}
#[derive(Debug)]
struct Pending {
    request: w::OpenChange,
    project: Option<Box<[u8]>>,
    called: Option<Called>,
}
impl Opens {
    pub(crate) fn new() -> Self {
        Self { waiting: BTreeMap::new(), pending: BTreeMap::new(), json: false }
    }
    pub(crate) fn calls(&self) -> usize {
        self.waiting.len().checked_add(self.pending.len()).expect("bounded calls")
    }
}
pub(crate) fn owns(call: &Call) -> bool {
    crate::table::TOOLS.get(usize::from(call.tool)).expect("parsed tool").name == ROW.name
}
fn text(call: &Call, at: usize) -> Option<&[u8]> {
    match value(call, at) {
        Some(Value::Text(text)) => Some(text),
        _ => None,
    }
}
fn missing(call: &Call, out: &mut Text) {
    out.put(b"change open needs: ");
    let mut separator = b"".as_slice();
    for (absent, words) in [
        (value(call, 2).is_none(), b"--branch BRANCH".as_slice()),
        (value(call, 5).is_none(), b"--title TEXT"),
        (value(call, 7).is_none() && value(call, 8).is_none(), b"--budget AMOUNT or --as-allowed"),
    ] {
        if absent {
            out.put(separator);
            out.put(words);
            separator = b"; ";
        }
    }
}
fn repository(bytes: &[u8]) -> Option<w::DraftRepository> {
    let slash = skein_lib::bytes::find(bytes, b"/")?;
    let owner = bytes.get(..slash)?;
    let name = bytes.get(slash.checked_add(1)?..)?;
    if owner.is_empty() || name.is_empty() || name.contains(&b'/') {
        return None;
    }
    { let value = w::DraftRepository { owner: owner.into(), name: name.into() }; value.measure(&w::CEILINGS).ok()?; Some(value) }
}
fn hexadecimal(bytes: &[u8]) -> bool {
    for byte in bytes {
        if !byte.is_ascii_hexdigit() {
            return false;
        }
    }
    true
}
fn owned(bytes: Option<&[u8]>) -> Option<Box<[u8]>> {
    Some(Box::from(bytes?))
}
fn checked(call: &Call) -> Result<(), &'static [u8]> {
    if let Some(bytes) = text(call, 1)
        && repository(bytes).is_none()
    {
        return Err(b"change open: --repository must be OWNER/NAME");
    }
    if let Some(bytes) = text(call, 2)
        && (bytes.is_empty() || bytes.len() > 1024)
    {
        return Err(b"change open: --branch must be 1 to 1024 bytes");
    }
    if let Some(bytes) = text(call, 3)
        && ((bytes.len() != 40 && bytes.len() != 64) || !hexadecimal(bytes))
    {
        return Err(b"change open: --at must be 40 or 64 hexadecimal digits");
    }
    if let Some(bytes) = text(call, 4)
        && (bytes.is_empty() || bytes.len() > 1024)
    {
        return Err(b"change open: --into must be 1 to 1024 bytes");
    }
    if let Some(bytes) = text(call, 5)
        && (bytes.is_empty() || bytes.len() > 256)
    {
        return Err(b"change open: --title must be 1 to 256 bytes");
    }
    if let Some(bytes) = text(call, 6)
        && (bytes.is_empty() || bytes.len() > 16_384)
    {
        return Err(b"change open: --body must be 1 to 16384 bytes");
    }
    if value(call, 7).is_some() && value(call, 8).is_some() {
        return Err(b"change open: --budget and --as-allowed are alternatives");
    }
    Ok(())
}
fn fail(called: Called, why: &[u8], out: &mut Queue<Out>) {
    out.push(Out::Printed { number: called.number, printed: render::failure(why, called.json, 1) });
}
pub(crate) fn call(state: &mut Opens, client: &Client, env: &Env<Limits>, called: Called, out: &mut Queue<Out>) {
    if called.value.is_some() || jig::argv::check(crate::table::TOOLS, &called.call, &env.limits.tools).is_err() {
        fail(called, b"invalid change open call", out);
        return;
    }
    if value(&called.call, 2).is_none()
        || value(&called.call, 5).is_none()
        || (value(&called.call, 7).is_none() && value(&called.call, 8).is_none())
    {
        let mut measure = Text::measure();
        missing(&called.call, &mut measure);
        let mut words = Text::writing_exact(measure);
        missing(&called.call, &mut words);
        fail(called, &words.finish(), out);
        return;
    }
    if let Err(why) = checked(&called.call) {
        fail(called, why, out);
        return;
    }
    state.waiting.insert(called.call_number, called);
    ready(state, client, env, out);
}
fn lowercase(bytes: Option<&[u8]>) -> Option<Box<[u8]>> {
    let mut bytes = Box::<[u8]>::from(bytes?);
    bytes.make_ascii_lowercase();
    Some(bytes)
}
fn ask(request: &w::OpenChange, limits: &Limits) -> Option<Ask> {
    let bounds = forge::wire_limits(&limits.domain.forge)?;
    let record =
        w::Requesting { request: w::Request::OpenChange(request.clone()) };
    Some(Ask::Family {
        scope: c::Scope::Project(request.project),
        record: jig_client::Family {
            family: wire::FORGE,
            version: wire::CLIENT_VERSION,
            bytes: record.encode(&bounds).ok()?,
        },
    })
}
pub(crate) fn ready(state: &mut Opens, client: &Client, env: &Env<Limits>, out: &mut Queue<Out>) {
    let Some(me) = crate::forge::me(client) else { return };
    for _ in 0..state.waiting.len() {
        let Some((_, called)) = state.waiting.pop_first() else { break };
        let Some(project) = jig::runner::reads::project(&called.call, me) else {
            fail(called, b"unknown project", out);
            continue;
        };
        let call = &called.call;
        let repository = match text(call, 1) {
            Some(bytes) => repository(bytes),
            None => None,
        };
        let budget = match number(call, 7) {
            Some(amount) => w::OpenBudget::Amount(
                w::OpenAmount { amount },
            ),
            None => w::OpenBudget::AsAllowed,
        };
        let request = w::OpenChange {
                project,
                repository,
                branch: text(call, 2).expect("required branch").into(),
                at: lowercase(text(call, 3)),
                into: owned(text(call, 4)),
                title: text(call, 5).expect("required title").into(),
                body: owned(text(call, 6)),
                budget,
            };
        let Some(sent) = ask(&request, &env.limits) else {
            fail(called, b"change open past the forge's bounds", out);
            continue;
        };
        let project = match value(call, 0) {
            Some(Value::Project(Project::Name(name))) => Some(name.clone()),
            _ => None,
        };
        let key = jig::key::local(Owner::Forge.asker(), called.call_number);
        assert!(client.pending(key).is_none(), "unique call number");
        let name = Some(crate::key::engine(call.key.as_ref().expect("change open requires key")));
        state.pending.insert(key, Pending { request, project, called: Some(called) });
        out.push(Out::Client { event: Event::Send { key, name, request: sent } });
    }
}
pub(crate) fn told(state: &mut Opens, client: &Client, env: &Env<Limits>, told: &Told, out: &mut Queue<Out>) {
    match told {
        Told::Kept { key, request } => {
            let Ask::Family { record, .. } = request else { return };
            if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
                return;
            }
            let bounds = forge::wire_limits(&env.limits.domain.forge).expect("checked limits");
            let Some(request) = wire::decode_requesting(&record.bytes, &bounds) else { return };
            let w::Request::OpenChange(request) = request.request else { return };
            state.pending.insert(*key, Pending { request, project: None, called: None });
        }
        Told::Answered { key, outcome } => {
            let Some(pending) = state.pending.remove(key) else { return };
            let json = match &pending.called {
                Some(called) => called.json,
                None => state.json,
            };
            let printed = print(&pending, client, outcome, json, &env.limits);
            match pending.called {
                Some(called) => out.push(Out::Printed { number: called.number, printed }),
                None => out.push(Out::Kept { line: if printed.status == 0 { printed.stdout } else { printed.stderr } }),
            }
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
fn project_name<'a>(p: &'a Pending, client: &'a Client) -> &'a [u8] {
    if let Some(name) = &p.project {
        return name;
    }
    if let Some(me) = crate::forge::me(client) {
        for project in &me.projects {
            if project.project == p.request.project {
                return project.name.bytes();
            }
        }
    }
    b"(no longer offered)"
}
fn refusal(p: &Pending, project: &[u8], why: &w::OpenRefusal, out: &mut Text) {
    match why {
        w::OpenRefusal::Several(rows) => {
            out.put(b"project ");
            out.put(project);
            out.put(b" has several repositories");
            for repository in &rows.repositories {
                out.put(b", ");
                out.put(repository.owner());
                out.put(b"/");
                out.put(repository.name());
            }
            out.put(b": name one with --repository");
        }
        w::OpenRefusal::NoRepository => {
            out.put(b"project ");
            out.put(project);
            out.put(b" has no repository a change lands into");
        }
        w::OpenRefusal::NotARepository => {
            if let Some(repository) = &p.request.repository {
                out.put(repository.owner());
                out.put(b"/");
                out.put(repository.name());
            }
            out.put(b" is not a repository of project ");
            out.put(project);
        }
        w::OpenRefusal::UnderPrefix(row) => {
            out.put(w::OpenChange::into(p.request).as_deref().unwrap_or(&p.request.branch));
            out.put(b" is under ");
            out.put(&row.owner);
            out.put(b"/");
            out.put(&row.name);
            out.put(b"'s prefix ");
            out.put(&row.prefix);
            out.put(b": a change lands into one of its landing branches");
        }
        w::OpenRefusal::NamePast(row) => {
            let branch = if u64::try_from(p.request.branch.len()).expect("bounded branch") > u64::from(row.bound) {
                &p.request.branch
            } else {
                w::OpenChange::into(p.request).as_deref().unwrap_or(&p.request.branch)
            };
            out.put(branch);
            out.put(b" is past the forge's ");
            out.number(u64::from(row.bound));
            out.put(b"-byte name bound");
        }
    }
}
fn refused(p: &Pending, client: &Client, why: &w::OpenRefusal, json: bool) -> jig::Printed {
    let project = project_name(p, client);
    let mut measure = Text::measure();
    measure.put(b"temper ctl: change open: ");
    refusal(p, project, why, &mut measure);
    let mut out = Text::writing_exact(measure);
    out.put(b"temper ctl: change open: ");
    refusal(p, project, why, &mut out);
    render::failure(&out.finish(), json, 2)
}
fn core_refused(why: &c::Refusal, json: bool, limits: &Limits) -> jig::Printed {
    let book = render::document::phrasebook(limits.tools.text);
    let mut measure = Text::measure();
    measure.put(b"temper ctl: change open: ");
    render::acts::refusal(why, &book, &mut measure);
    let mut out = Text::writing_exact(measure);
    out.put(b"temper ctl: change open: ");
    render::acts::refusal(why, &book, &mut out);
    render::failure(&out.finish(), json, 2)
}
fn opened(task: u64, json: bool, limits: &Limits) -> jig::Printed {
    let bytes = if json {
        let bounds = skein_json::writer::Limits { depth: 1, length: limits.tools.text };
        let mut measure = skein_json::writer::Encoder::measure(&bounds);
        opened_json(task, &mut measure);
        let Ok(length) = measure.measured() else { return render::failure(b"result limit", json, 1) };
        let mut out = skein_json::writer::Encoder::write(length, &bounds);
        opened_json(task, &mut out);
        render::outcome::newline(&out.finish())
    } else {
        let mut measure = Text::measure();
        measure.put(b"task ");
        measure.number(task);
        measure.put(b"\n");
        let mut out = Text::writing_exact(measure);
        out.put(b"task ");
        out.number(task);
        out.put(b"\n");
        out.finish()
    };
    jig::Printed { file: jig::FileEnd::Keep, status: 0, stdout: bytes, stderr: Box::new([]) }
}
fn opened_json(task: u64, out: &mut skein_json::writer::Encoder) {
    out.object_start();
    out.key(b"task");
    out.unsigned(task);
    out.object_end();
}
fn print(p: &Pending, client: &Client, outcome: &Outcome, json: bool, limits: &Limits) -> jig::Printed {
    let request = ask(&p.request, limits).expect("validated request");
    let mut operation = render::outcome::Operation::for_call(&request, &ROW);
    operation.word = ROW.name;
    if let Outcome::Answered(jig_client::Answered::Family(record)) = outcome {
        if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
            return render::failure(b"change open answer decoding failed", json, 1);
        }
        let bounds = forge::wire_limits(&limits.domain.forge).expect("checked limits");
        let mut reader = skein_lib::Reader::new(&record.bytes);
        let Ok(answer) = w::Answer::decode(&bounds, &mut reader) else {
            return render::failure(b"change open answer decoding failed", json, 1);
        };
        if !reader.is_empty() {
            return render::failure(b"change open answer decoding failed", json, 1);
        }
        if let w::Outcome::Opened(row) = answer.outcome() {
            return opened(row.task, json, limits);
        }
        if let w::Outcome::OpenRefused(why) = answer.outcome() {
            return refused(p, client, &why.reason, json);
        }
        if let Some(forge::Answer::CoreRefused(why)) =
            forge::answer(&limits.domain, &jig_client::Answered::Family(record.clone()))
        {
            return core_refused(&why, json, limits);
        }
        if let w::Outcome::Unknown = answer.outcome() {
            return render::outcome::print(
                &operation,
                &Outcome::Unknown,
                json,
                None,
                &limits.tools,
                crate::root::DIAGNOSTIC_PREFIX,
            );
        }
        return render::failure(b"unexpected change open answer", json, 1);
    }
    if let Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused { refusal, .. })) = outcome {
        return core_refused(refusal, json, limits);
    }
    render::outcome::print(&operation, outcome, json, None, &limits.tools, crate::root::DIAGNOSTIC_PREFIX)
}
