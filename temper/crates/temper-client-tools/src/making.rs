//! Keyed repository requests and adoption waits (temper/docs/design/client/ctl/tools.md, 3 and 4).
use crate::{
    jig::{
        self,
        call::{Call, Value, argument},
        render::{self, Text},
        runner::{Called, Out},
        table::{
            Keyed, Kind,
            Occurs::{Optional, Required},
            Tool, flag, positional, switch,
        },
    },
    root::Limits,
};
use alloc::boxed::Box;
use jig_client::{Answered, Ask, Client, Event, Family, Handle, Key, Latest, Outcome, State, Told, contract as c};
pub(crate) use making::phrase;
use skein_json::writer::Encoder;
use skein_lib::{Env, Map, Queue, Reader, Time, bytes};
use temper_client_domain::{
    forge::{
        self,
        making::{self, Making},
    },
    owner::Owner,
};
use temper_client_protocol_wire::{self as wire, source, v1 as w};

pub const MAKE: Tool = Tool {
    name: b"repository make",
    line: b"Make an empty repository and adopt it into a project",
    prints: b"`kept`; with --until adopted, `ORG/NAME is in PROJECT.` once adopted",
    result: b"repository-make",
    forms: &[&[
        flag(b"project", b"PROJECT", Kind::Project { empty: false }, Required, b"The project"),
        flag(b"forge", b"FORGE", Kind::NumberRange { min: 1, max: 65535 }, Optional, b"The forge's number; 1 when absent"),
        positional(b"repository", b"ORG/NAME", Kind::Literal, Required, b"The destination organization and repository"),
        switch(b"no-home", b"Do not make it the project's home"),
        flag(b"until", b"CONDITION", Kind::Word { set: &[b"adopted"] }, Optional, b"Wait until adopted"),
        flag(b"timeout", b"DURATION", Kind::Duration, Optional, b"How long a wait lasts; 300 seconds when absent, at most 3600"),
    ]],
    keyed: Keyed::Required,
    area: 1,
    notes: b"It makes the starter workflow and adopts the default branch, owned,\nwith CI. A project has one home: add --no-home if it has one already,\nand make the call under a new key. --timeout needs --until adopted.\nA wait exits 1 on a failed row, 2 on timeout; the request goes on.\nRepeat the same call with the same key to get kept and wait again.\n",
};
pub const COPY: Tool = Tool {
    name: b"repository copy",
    line: b"Copy a public repository and adopt it into a project",
    prints: MAKE.prints,
    result: b"repository-copy",
    forms: &[&[
        flag(b"project", b"PROJECT", Kind::Project { empty: false }, Required, b"The project"),
        flag(b"forge", b"FORGE", Kind::NumberRange { min: 1, max: 65535 }, Optional, b"The forge's number; 1 when absent"),
        positional(b"address", b"ADDRESS", Kind::Literal, Required, b"HOST/OWNER/NAME, optionally with https:// before and .git after"),
        flag(b"as", b"ORG/NAME", Kind::Literal, Optional, b"The destination; the source's owner and name when absent"),
        switch(b"no-home", b"Do not make it the project's home"),
        flag(b"until", b"CONDITION", Kind::Word { set: &[b"adopted"] }, Optional, b"Wait until adopted"),
        flag(b"timeout", b"DURATION", Kind::Duration, Optional, b"How long a wait lasts; 300 seconds when absent, at most 3600"),
    ]],
    keyed: Keyed::Required,
    area: 1,
    notes: b"ADDRESS is HOST/OWNER/NAME, with optional https:// and .git. It must\nbe public. An invalid address sends nothing. --as chooses another\norganization and name. Adoption is owned, on its default branch,\nwith CI. A home refused needs --no-home and a new key.\n--timeout needs --until adopted. A wait exits 1 on failure, 2 on\ntimeout; the copy goes on. The same key answers kept and waits again.\n",
};
fn tool(call: &Call) -> &'static Tool {
    crate::table::TOOLS.get(usize::from(call.tool)).expect("parsed tool")
}
fn arg<'a>(call: &'a Call, name: &[u8]) -> Option<&'a Value> {
    argument(tool(call), call, name)
}
fn text_arg<'a>(call: &'a Call, name: &[u8]) -> Option<&'a [u8]> {
    match arg(call, name) {
        Some(Value::Text(text)) => Some(text),
        Some(
            Value::Word(_)
            | Value::Number(_)
            | Value::Stdin
            | Value::File(_)
            | Value::Duration(_)
            | Value::Proposal(_)
            | Value::Digest(_)
            | Value::Condition(_)
            | Value::Switch
            | Value::Project(_)
            | Value::Turn(_)
            | Value::Cursor(_)
            | Value::Group(_)
            | Value::Order(_)
            | Value::Tool(_),
        )
        | None => None,
    }
}
pub(crate) fn owns(call: &Call) -> bool {
    tool(call).name == MAKE.name || tool(call).name == COPY.name
}
#[derive(Debug)]
pub(crate) struct Maker {
    waiting: Map<u32, Called>,
    pending: Map<Key, Pending>,
    watches: Map<Handle, Watching>,
    json: bool,
}
#[derive(Debug)]
struct Pending {
    request: Ask,
    make: w::MakeRepository,
    called: Option<Called>,
}
#[derive(Debug)]
struct Watching {
    called: Called,
    make: w::MakeRepository,
    deadline: Time,
}
impl Maker {
    pub(crate) fn new() -> Self {
        Self { waiting: Map::uncapped(), pending: Map::uncapped(), watches: Map::uncapped(), json: false }
    }
    pub(crate) fn calls(&self) -> usize {
        usize::try_from(
            self.waiting
                .len()
                .checked_add(self.pending.len())
                .expect("bounded calls")
                .checked_add(self.watches.len())
                .expect("bounded calls"),
        )
        .expect("calls fit")
    }
    pub(crate) fn kept_json(&mut self, json: bool) {
        self.json = json;
    }
    pub(crate) fn next_deadline(&self) -> Option<Time> {
        let mut next = None;
        for (_, watch) in &self.watches {
            next = Some(match next {
                Some(old) => watch.deadline.min(old),
                None => watch.deadline,
            });
        }
        next
    }
}
pub(crate) fn call(state: &mut Maker, client: &Client, env: &Env<Limits>, called: Called, out: &mut Queue<Out>) {
    if crate::forge::me(client).is_some() {
        send(state, client, env, called, out);
    } else {
        state.waiting.insert(called.call_number, called).expect("correlation map");
    }
}
pub(crate) fn ready(state: &mut Maker, client: &Client, env: &Env<Limits>, out: &mut Queue<Out>) {
    if crate::forge::me(client).is_none() {
        return;
    }
    for _ in 0..state.waiting.len() {
        let Some((&number, _)) = state.waiting.first() else { break };
        let called = state.waiting.remove(&number).expect("waiting call");
        send(state, client, env, called, out);
    }
}
fn send(state: &mut Maker, client: &Client, env: &Env<Limits>, called: Called, out: &mut Queue<Out>) {
    if called.value.is_some() || jig::argv::check(crate::table::TOOLS, &called.call, &env.limits.tools).is_err() {
        fail(called, b"invalid repository call", out);
        return;
    }
    if arg(&called.call, b"timeout").is_some() && arg(&called.call, b"until").is_none() {
        fail(called, b"--timeout needs --until adopted", out);
        return;
    }
    let me = crate::forge::me(client).expect("live me");
    let Some(project) = jig::runner::reads::project(&called.call, me) else {
        fail(called, b"unknown project", out);
        return;
    };
    let mut known = false;
    for row in &me.projects {
        known |= row.project == project;
    }
    if !known {
        fail(called, b"unknown project", out);
        return;
    }
    let make = match prepare(&called.call, project) {
        Ok(make) => make,
        Err(why) => {
            fail(called, why, out);
            return;
        }
    };
    let Some(request) = ask(make.clone(), &env.limits) else {
        fail(called, b"repository encoding failed", out);
        return;
    };
    let key = jig::key::local(Owner::Forge.asker(), called.call_number);
    if client.pending(key).is_some() {
        fail(called, b"tools' bug; client key collision; report it as a finding", out);
        return;
    }
    let name = Some(crate::key::engine(called.call.key.as_ref().expect("repository requires key")));
    state
        .pending
        .insert(key, Pending { request: request.clone(), make, called: Some(called) })
        .expect("correlation map");
    out.push(Out::Client { event: Event::Send { key, name, request } });
}
fn target(name: &[u8]) -> Option<(&[u8], &[u8])> {
    let at = bytes::find(name, b"/")?;
    let org = name.get(..at)?;
    let repository = name.get(at.checked_add(1)?..)?;
    if source::org(org) && source::repository(repository) { Some((org, repository)) } else { None }
}
fn prepare(call: &Call, project: u32) -> Result<w::MakeRepository, &'static [u8]> {
    let copy = tool(call).name == COPY.name;
    let address = if copy { text_arg(call, b"address") } else { None };
    let (org, name) = match address {
        Some(address) => {
            let source = source::parse(address)
                .or(Err(b"ADDRESS must be HOST/OWNER/NAME, with optional https:// and .git".as_slice()))?;
            match text_arg(call, b"as") {
                Some(name) => target(name).ok_or(b"--as must be ORG/NAME".as_slice())?,
                None => (source.owner, source.name),
            }
        }
        None => target(text_arg(call, b"repository").expect("required repository"))
            .ok_or(b"destination must be ORG/NAME".as_slice())?,
    };
    let forge = match arg(call, b"forge") {
        Some(Value::Number(number)) => u16::try_from(*number).expect("checked forge"),
        Some(
            Value::Word(_)
            | Value::Text(_)
            | Value::Stdin
            | Value::File(_)
            | Value::Duration(_)
            | Value::Proposal(_)
            | Value::Digest(_)
            | Value::Condition(_)
            | Value::Switch
            | Value::Project(_)
            | Value::Turn(_)
            | Value::Cursor(_)
            | Value::Group(_)
            | Value::Order(_)
            | Value::Tool(_),
        )
        | None => 1,
    };
    let into = w::MakeInto::new(&w::CEILINGS, w::MakeIntoParts { project, home: arg(call, b"no-home").is_none() })
        .expect("scalar destination");
    making::ask(forge, org, name, address, Some(into))
        .or(Err(b"destination must be ORG/NAME; use --as ORG/NAME for a copy".as_slice()))
}
fn ask(make: w::MakeRepository, limits: &Limits) -> Option<Ask> {
    let bounds = forge::wire_limits(&limits.domain.forge)?;
    let request = w::Requesting::new(&bounds, w::RequestingParts { request: w::Request::MakeRepository(make) }).ok()?;
    Some(Ask::Family {
        scope: c::Scope::Deployment,
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::requesting(&request) },
    })
}
fn restored(request: &Ask, limits: &Limits) -> Option<w::MakeRepository> {
    let Ask::Family { scope: c::Scope::Deployment, record } = request else { return None };
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    let request = wire::decode_requesting(&record.bytes, &forge::wire_limits(&limits.domain.forge)?)?;
    let w::Request::MakeRepository(make) = request.into_parts().request else { return None };
    Some(make)
}
pub(crate) fn told(state: &mut Maker, client: &Client, env: &Env<Limits>, told: &Told, out: &mut Queue<Out>) {
    match told {
        Told::Kept { key, request } => {
            if let Some(make) = restored(request, &env.limits) {
                state
                    .pending
                    .insert(*key, Pending { request: request.clone(), make, called: None })
                    .expect("correlation map");
            }
        }
        Told::Answered { key, outcome } => {
            let Some(pending) = state.pending.remove(key) else { return };
            let json = match &pending.called {
                Some(called) => called.json,
                None => state.json,
            };
            let printed = answer(&pending, outcome, json, &env.limits);
            match pending.called {
                Some(called) if printed.status == 0 && arg(&called.call, b"until").is_some() => {
                    out.push(Out::Progress {
                        line: if json { render::outcome::newline(&printed.stdout) } else { printed.stdout },
                    });
                    let Some(Value::Duration(timeout)) = arg(&called.call, b"timeout") else {
                        unreachable!("defaulted wait")
                    };
                    let deadline = env.now.saturating_add(*timeout);
                    let handle =
                        Handle { asker: Owner::Forge.asker(), number: (u64::from(called.call_number) << 32u32) | 2 };
                    state
                        .watches
                        .insert(handle, Watching { called, make: pending.make, deadline })
                        .expect("correlation map");
                    out.push(Out::Client {
                        event: Event::Watch {
                            handle,
                            subject: forge::forges::watch(&env.limits.domain.forge).expect("checked limits"),
                        },
                    });
                }
                Some(called) => finish(called, printed, out),
                None => out.push(Out::Kept { line: if printed.status == 0 { printed.stdout } else { printed.stderr } }),
            }
        }
        Told::Changed { handle } if state.watches.contains_key(handle) => {
            let Some(rows) = rows(client, *handle, &env.limits) else { return };
            let watch = state.watches.get(handle).expect("active watch");
            if let Some(status) = verdict(&watch.make, &rows) {
                let printed = result(watch, Some(&rows), status, &env.limits);
                complete(state, *handle, printed, out);
            }
        }
        Told::Ended { handle, why } if state.watches.contains_key(handle) => {
            let watch = state.watches.remove(handle).expect("active watch");
            let printed = render::failure(
                match why {
                    jig_client::Why::Unknown => b"unknown".as_slice(),
                    jig_client::Why::NotAllowed => b"not allowed",
                    jig_client::Why::SignedOut => b"signed out",
                    jig_client::Why::NotServed => b"version not served",
                    jig_client::Why::Limit => b"watch table limit",
                    jig_client::Why::SubjectEnded => b"ended",
                },
                watch.called.json,
                1,
            );
            finish(watch.called, printed, out);
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
fn rows(client: &Client, handle: Handle, limits: &Limits) -> Option<Box<[forge::forges::Row]>> {
    let document = client.document(handle)?;
    if document.state != State::Live {
        return None;
    }
    let Latest::Family(record) = document.latest? else { return None };
    forge::forges::rows(record, &limits.domain.forge)
}
fn verdict(make: &w::MakeRepository, rows: &[forge::forges::Row]) -> Option<u8> {
    for forge in rows {
        if forge.number == make.forge() {
            for row in &forge.makings {
                if row.org.as_ref() == make.org() && row.name.as_ref() == make.name() {
                    return match row.state {
                        w::MakingState::Failed(_) => Some(1),
                        w::MakingState::Waiting
                        | w::MakingState::Making
                        | w::MakingState::Made(_)
                        | w::MakingState::Adopting(_) => None,
                    };
                }
            }
            return Some(0);
        }
    }
    None
}
fn complete(state: &mut Maker, handle: Handle, printed: jig::Printed, out: &mut Queue<Out>) {
    let watch = state.watches.remove(&handle).expect("active watch");
    out.push(Out::Client { event: Event::Unwatch { handle } });
    finish(watch.called, printed, out);
}
pub(crate) fn fire(state: &mut Maker, client: &Client, env: &Env<Limits>, out: &mut Queue<Out>) {
    for _ in 0..state.watches.len() {
        let mut expired = None;
        for (handle, watch) in &state.watches {
            if watch.deadline <= env.now {
                expired = Some(*handle);
                break;
            }
        }
        let Some(handle) = expired else { break };
        let watch = state.watches.get(&handle).expect("active watch");
        let rows = rows(client, handle, &env.limits);
        let printed = result(watch, rows.as_deref(), 2, &env.limits);
        complete(state, handle, printed, out);
    }
}
fn fail(called: Called, why: &[u8], out: &mut Queue<Out>) {
    let printed = render::failure(why, called.json, 1);
    finish(called, printed, out);
}
fn finish(called: Called, printed: jig::Printed, out: &mut Queue<Out>) {
    out.push(Out::Printed { number: called.number, printed });
}

#[expect(clippy::too_many_lines, reason = "the reply enumerates every forge outcome at this boundary")]
fn answer(pending: &Pending, outcome: &Outcome, json: bool, limits: &Limits) -> jig::Printed {
    let kept = pending.called.is_none();
    let row = match pending.make.from() {
        w::MakeFrom::Empty => &MAKE,
        w::MakeFrom::Copy(_) => &COPY,
    };
    let mut operation = render::outcome::Operation::for_call(&pending.request, row);
    operation.word = row.result;
    let decoded = match outcome {
        Outcome::Answered(Answered::Family(record))
            if record.family == wire::FORGE && record.version == wire::CLIENT_VERSION =>
        {
            let Some(bounds) = forge::wire_limits(&limits.domain.forge) else {
                return render::failure(b"invalid limits", json, 1);
            };
            let mut reader = Reader::new(&record.bytes);
            match w::Answer::decode(&bounds, &mut reader) {
                Ok(answer) if reader.is_empty() => Some(answer.into_parts().outcome),
                Ok(_) | Err(_) => return render::failure(b"repository answer decoding failed", json, 1),
            }
        }
        Outcome::Answered(_)
        | Outcome::Unknown
        | Outcome::Held
        | Outcome::Limit
        | Outcome::Unconfirmed
        | Outcome::NotKept
        | Outcome::Unsent
        | Outcome::Lost
        | Outcome::Full { .. } => None,
    };
    let (status, words, why): (u8, &[u8], Option<&[u8]>) = match decoded {
        Some(w::Outcome::MakeKept) => (0, b"kept", None),
        Some(w::Outcome::MakeRefused(refused)) => {
            let (code, words) = refusal(refused.why());
            (2, words, Some(code))
        }
        Some(
            w::Outcome::Unknown
            | w::Outcome::DeviceAdded(_)
            | w::Outcome::DeviceRemoved
            | w::Outcome::DeviceRefused(_)
            | w::Outcome::Away
            | w::Outcome::Tried
            | w::Outcome::Drafted(_)
            | w::Outcome::DraftRefused(_)
            | w::Outcome::Follow(_)
            | w::Outcome::NotTheirs
            | w::Outcome::Overturned
            | w::Outcome::OverturnRefused(_)
            | w::Outcome::Looked
            | w::Outcome::Dismissed
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
            | w::Outcome::Accepted(_)
            | w::Outcome::Removed
            | w::Outcome::RemoveRefused(_)
            | w::Outcome::Reset
            | w::Outcome::EndRefused(_),
        ) => return render::failure(b"repository answer decoding failed", json, 1),
        None => {
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
            return render::outcome::print(
                &operation,
                outcome,
                json,
                None,
                &limits.tools,
                crate::root::DIAGNOSTIC_PREFIX,
            );
        }
    };
    let output = if json {
        let bounds = skein_json::writer::Limits { depth: 5, length: limits.tools.text };
        let mut measure = Encoder::measure(&bounds);
        answer_json(why, kept, row.result, &mut measure);
        let Ok(size) = measure.measured() else { return render::failure(b"result limit", json, 1) };
        let mut out = Encoder::write(size, &bounds);
        answer_json(why, kept, row.result, &mut out);
        let bytes = out.finish();
        if kept { render::outcome::newline(&bytes) } else { bytes }
    } else {
        let mut measure = Text::measure();
        answer_text(pending, kept, row.result, words, why.is_some(), &mut measure);
        let mut out = Text::writing_exact(measure);
        answer_text(pending, kept, row.result, words, why.is_some(), &mut out);
        out.finish()
    };
    printed(output, status, limits, json)
}
fn refusal(why: &w::MakeRefusal) -> (&'static [u8], &'static [u8]) {
    match why {
        w::MakeRefusal::NoSuchForge => (b"no_such_forge", b"no such forge"),
        w::MakeRefusal::NotOffered => (b"not_offered", b"not offered"),
        w::MakeRefusal::NotOwner => (b"not_owner", b"not owner"),
        w::MakeRefusal::Invalid => (b"invalid", b"invalid"),
        w::MakeRefusal::HomeHeld => {
            (b"home_held", b"The project has a home already. Add --no-home and make the call under a new key.")
        }
        w::MakeRefusal::Full => (b"full", b"full"),
        w::MakeRefusal::NotFailed => (b"not_failed", b"It is not failed."),
    }
}
fn answer_text(pending: &Pending, kept: bool, operation: &[u8], words: &[u8], refused: bool, out: &mut Text) {
    if kept {
        out.put(b"kept: ");
        out.put(operation);
        out.put(b" ");
    }
    if refused {
        out.put(b"refused: ");
    }
    out.put(words);
    out.put(b"\n");
    if refused && words.starts_with(b"The project has a home") {
        out.put(b"  temper ctl DIR repository ");
        match pending.make.from() {
            w::MakeFrom::Empty => out.put(b"make"),
            w::MakeFrom::Copy(_) => out.put(b"copy"),
        }
        out.put(b" --project ");
        if let Some(into) = w::MakeRepository::into(&pending.make) {
            out.number(u64::from(into.project()));
        }
        out.put(b" --forge ");
        out.number(u64::from(pending.make.forge()));
        out.put(b" ");
        match pending.make.from() {
            w::MakeFrom::Empty => {}
            w::MakeFrom::Copy(copy) => {
                render::lists::argument(copy.address(), out);
                out.put(b" --as ");
            }
        }
        out.put(pending.make.org());
        out.put(b"/");
        out.put(pending.make.name());
        out.put(b" --no-home --key KEY\n");
    }
}
fn answer_json(why: Option<&[u8]>, kept: bool, operation: &[u8], out: &mut Encoder) {
    if kept {
        out.object_start();
        out.key(b"operation");
        out.string(operation);
        out.key(b"outcome");
    }
    out.object_start();
    out.key(if why.is_some() { b"make_refused" } else { b"make_kept" });
    out.object_start();
    if let Some(why) = why {
        out.key(b"why");
        out.string(why);
    }
    out.object_end();
    out.object_end();
    if kept {
        out.object_end();
    }
}
fn result(watch: &Watching, rows: Option<&[forge::forges::Row]>, status: u8, limits: &Limits) -> jig::Printed {
    let mut state = None;
    if let Some(rows) = rows {
        for forge in rows {
            if forge.number == watch.make.forge() {
                for row in &forge.makings {
                    if row.org.as_ref() == watch.make.org() && row.name.as_ref() == watch.make.name() {
                        state = Some(row);
                    }
                }
            }
        }
    }
    let output = if watch.called.json {
        let bounds = skein_json::writer::Limits { depth: 8, length: limits.tools.text };
        let mut measure = Encoder::measure(&bounds);
        result_json(watch, state, status, &mut measure);
        let Ok(size) = measure.measured() else { return render::failure(b"result limit", true, 1) };
        let mut out = Encoder::write(size, &bounds);
        result_json(watch, state, status, &mut out);
        out.finish()
    } else {
        let mut measure = Text::measure();
        result_text(watch, state, status, &mut measure);
        let mut out = Text::writing_exact(measure);
        result_text(watch, state, status, &mut out);
        out.finish()
    };
    let printed = printed(output, status, limits, watch.called.json);
    if status == 2 && printed.status == 2 {
        return jig::Printed { stdout: printed.stderr, stderr: Box::default(), ..printed };
    }
    printed
}
fn result_text(watch: &Watching, row: Option<&Making>, status: u8, out: &mut Text) {
    if status == 0 {
        out.put(watch.make.org());
        out.put(b"/");
        out.put(watch.make.name());
        out.put(b" is in ");
        let Some(into) = w::MakeRepository::into(&watch.make) else { unreachable!("tools adopt into a project") };
        out.number(u64::from(into.project()));
        out.put(b".\n");
    } else {
        match row {
            Some(row) => out.put(&phrase(row)),
            None => out.put(b"Waiting for the repository's row"),
        }
        out.put(b"\n");
        if status == 2 {
            out.put(b"not met: timeout\n");
        }
    }
}
fn result_json(watch: &Watching, row: Option<&Making>, status: u8, out: &mut Encoder) {
    out.object_start();
    out.key(b"adopted");
    out.boolean(status == 0);
    out.key(b"forge");
    out.unsigned(u64::from(watch.make.forge()));
    out.key(b"org");
    out.string(watch.make.org());
    out.key(b"name");
    out.string(watch.make.name());
    out.key(b"project");
    if let Some(into) = w::MakeRepository::into(&watch.make) {
        out.unsigned(u64::from(into.project()));
    } else {
        out.null();
    }
    out.key(b"making");
    match row {
        Some(row) => encode(row, out),
        None => out.null(),
    }
    if status == 2 {
        out.key(b"why");
        out.string(b"timeout");
    }
    out.object_end();
}
fn printed(bytes: Box<[u8]>, status: u8, limits: &Limits, json: bool) -> jig::Printed {
    if bytes.len() > usize::try_from(limits.tools.text).expect("text fits") {
        return render::failure(b"result limit", json, 1);
    }
    if status == 0 {
        jig::Printed { file: jig::FileEnd::Keep, status, stdout: bytes, stderr: Box::default() }
    } else {
        jig::Printed { file: jig::FileEnd::Keep, status, stdout: Box::default(), stderr: bytes }
    }
}
pub(crate) fn settled(row: &Making) -> bool {
    match row.state {
        w::MakingState::Failed(_) => true,
        w::MakingState::Waiting | w::MakingState::Making | w::MakingState::Made(_) | w::MakingState::Adopting(_) => {
            false
        }
    }
}
pub(crate) fn encode(row: &Making, out: &mut Encoder) {
    out.object_start();
    out.key(b"org");
    out.string(&row.org);
    out.key(b"name");
    out.string(&row.name);
    out.key(b"from");
    out.object_start();
    match &row.from {
        w::MakeFrom::Empty => {
            out.key(b"empty");
            out.object_start();
            out.object_end();
        }
        w::MakeFrom::Copy(copy) => {
            out.key(b"copy");
            out.object_start();
            out.key(b"address");
            out.string(copy.address());
            out.object_end();
        }
    }
    out.object_end();
    out.key(b"into");
    match &row.into {
        Some(into) => {
            out.object_start();
            out.key(b"project");
            out.unsigned(u64::from(into.project()));
            out.key(b"home");
            out.boolean(into.home());
            out.object_end();
        }
        None => out.null(),
    }
    out.key(b"state");
    state_json(&row.state, out);
    out.object_end();
}
fn state_json(state: &w::MakingState, out: &mut Encoder) {
    out.object_start();
    match state {
        w::MakingState::Waiting => {
            out.key(b"waiting");
            out.object_start();
            out.object_end();
        }
        w::MakingState::Making => {
            out.key(b"making");
            out.object_start();
            out.object_end();
        }
        w::MakingState::Made(made) | w::MakingState::Adopting(made) => {
            out.key(match state {
                w::MakingState::Made(_) => b"made",
                w::MakingState::Waiting
                | w::MakingState::Making
                | w::MakingState::Adopting(_)
                | w::MakingState::Failed(_) => b"adopting",
            });
            out.object_start();
            out.key(b"id");
            out.unsigned(made.id());
            out.key(b"branch");
            out.string(made.branch());
            out.object_end();
        }
        w::MakingState::Failed(failed) => {
            out.key(b"failed");
            out.object_start();
            out.key(b"step");
            out.string(match failed.step() {
                w::MakeStep::Admin => b"admin",
                w::MakeStep::Org => b"org",
                w::MakeStep::Repository => b"repository",
                w::MakeStep::File => b"file",
                w::MakeStep::Adoption => b"adoption",
            });
            out.key(b"reason");
            out.object_start();
            match failed.reason() {
                w::MakeFailure::Taken => {
                    out.key(b"taken");
                    out.object_start();
                    out.object_end();
                }
                w::MakeFailure::Empty => {
                    out.key(b"empty");
                    out.object_start();
                    out.object_end();
                }
                w::MakeFailure::TimedOut => {
                    out.key(b"timed_out");
                    out.object_start();
                    out.object_end();
                }
                w::MakeFailure::NotAllowed => {
                    out.key(b"not_allowed");
                    out.object_start();
                    out.object_end();
                }
                w::MakeFailure::Other => {
                    out.key(b"other");
                    out.object_start();
                    out.object_end();
                }
                w::MakeFailure::Refused(refused) => {
                    out.key(b"refused");
                    out.object_start();
                    out.key(b"status");
                    out.unsigned(u64::from(refused.status()));
                    out.object_end();
                }
            }
            out.object_end();
            out.key(b"words");
            out.string(failed.words());
            out.object_end();
        }
    }
    out.object_end();
}
