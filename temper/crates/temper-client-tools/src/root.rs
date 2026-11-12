//! The tools' shared client and exhaustive owner routing (client/ctl/README.md, 4).
use crate::jig::{self, runner};
use alloc::{boxed::Box, collections::BTreeMap};
use jig_client::{Client, Told};
use skein_lib::{Env, List, Queue, Time};
use temper_client_domain::owner::Owner;
pub(crate) const DIAGNOSTIC_PREFIX: &[u8] = b"temper ctl";
pub const RUNNER: Owner = Owner::Runner;
#[derive(Debug)]
pub struct Limits {
    pub domain: temper_client_domain::Limits,
    pub tools: jig::Limits,
}
impl Limits {
    #[must_use]
    pub fn new(domain: temper_client_domain::Limits, tools: jig::Limits) -> Self {
        Self { domain, tools }
    }
}
#[derive(Debug)]
pub struct Root {
    client: Client,
    unlinked: bool,
    runner: runner::Runner,
    adopter: crate::forge::Adopter,
    drafts: crate::drafts::Drafts,
    maker: crate::making::Maker,
    forges: crate::forges::Forges,
    changes: crate::change::Changes,
    landings: crate::landing::Landings,
    devices: crate::push::Devices,
    follows: crate::follow::Follows,
    next: Option<u32>,
    recovery: crate::recovery::Recovery,
    deadline: Option<Time>,
    patience: BTreeMap<u32, (u32, Time)>,
    retries: BTreeMap<jig_client::Key, jig_client::Retry>,
}
#[derive(Debug)]
pub enum Event {
    Client { event: jig_client::Event },
    Call { number: u32, call: jig::call::Call, json: bool, value: Option<skein_lib::Secret> },
}
#[derive(Debug)]
pub enum Request {
    Client {
        request: jig_client::Request,
    },
    /// What the unlinked client asks, which its host carries (jig's client/domain/client/README.md, 4.4).
    Unlinked {
        request: jig_client::UnlinkedRequest,
    },
    Ready,
    Record {
        number: u32,
        record: skein_lib::Secret,
    },
    Kept {
        line: Box<[u8]>,
    },
    Progress {
        line: Box<[u8]>,
    },
    Printed {
        number: u32,
        printed: jig::Printed,
    },
    Fact {
        fact: temper_client_domain::Fact,
    },
}
impl Root {
    /// # Errors
    /// Refuses client limits that cannot be represented.
    pub fn new(limits: &Limits, seed: u64) -> Result<Self, jig_client::Unusable> {
        Ok(Self::with_client(Client::new(&limits.domain.client, seed)?, seed, false))
    }
    /// # Errors
    /// Refuses client limits that cannot be represented.
    pub fn unlinked(limits: &Limits, seed: u64) -> Result<Self, jig_client::Unusable> {
        Ok(Self::with_client(Client::unlinked(&limits.domain.client, seed)?, seed, true))
    }
    fn with_client(client: Client, seed: u64, unlinked: bool) -> Self {
        let mut runner = runner::Runner::new(RUNNER.asker(), crate::key::TOOLS, seed);
        runner.parts(&Parts);
        let mut root = Self {
            client,
            unlinked,
            runner,
            adopter: crate::forge::Adopter::new(),
            drafts: crate::drafts::Drafts::new(),
            maker: crate::making::Maker::new(),
            forges: crate::forges::Forges::new(),
            changes: crate::change::Changes::new(),
            landings: crate::landing::Landings::new(),
            devices: crate::push::Devices::new(),
            follows: crate::follow::Follows::new(),
            next: Some(1),
            recovery: crate::recovery::Recovery::new(),
            deadline: None,
            patience: BTreeMap::new(),
            retries: BTreeMap::new(),
        };
        root.changed();
        root
    }
    fn changed(&mut self) {
        // Every mutating entry point settles this snapshot after draining and
        // reclaiming. Between inputs the absolute deadlines cannot change.
        self.deadline = deadline(self);
    }
    #[must_use]
    pub fn client(&self) -> &Client {
        &self.client
    }
    /// The latest call's act, or the kept act it joined, last answered for retry
    /// (client/ctl/results.md, 3). An in-flight retry keeps that last answer.
    #[must_use]
    pub fn last_retry(&self) -> Option<jig_client::Retry> {
        let (prefix, _) = self.patience.last_key_value()?;
        for (key, why) in &self.retries {
            if key.number >> 32u32 == u64::from(*prefix) {
                return Some(*why);
            }
            for (local, kept) in &self.recovery.joined {
                if kept == key && local.number >> 32u32 == u64::from(*prefix) {
                    return Some(*why);
                }
            }
        }
        None
    }
    /// A change's supplemental read is outstanding, including retry after a
    /// restart. A condition wait's host keeps its link deadline while this holds.
    #[must_use]
    pub fn change_read_pending(&self) -> bool {
        self.changes.reading()
    }
    #[must_use]
    pub fn kept(&self) -> u32 {
        self.runner.kept(&self.client)
    }
    #[must_use]
    pub fn replayed(&self) -> bool {
        match crate::forge::me(&self.client) {
            Some(me) => self.recovery.replayed(&self.client, me.party.number),
            None => false,
        }
    }
    #[must_use]
    pub fn unanswered(&self) -> Box<[u8]> {
        match runner::acts::me(&self.runner, &self.client) {
            Some(me) => self.recovery.unanswered(&self.client, me.party.number),
            None => Box::default(),
        }
    }
    pub fn kept_json(&mut self, json: bool) {
        self.runner.kept_json(json);
        self.adopter.kept_json(json);
        self.drafts.kept_json(json);
        self.maker.kept_json(json);
        self.follows.kept_json(json);
        self.forges.kept_json(json);
        self.devices.kept_json(json);
        self.changed();
    }
}
pub fn step(root: &mut Root, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    let mut client = Queue::uncapped();
    let mut child = Queue::uncapped();
    match event {
        Event::Client { event } => {
            if let jig_client::Event::Start { .. } | jig_client::Event::StartUnlinked { .. } = &event {
                root.runner.start(&mut child);
            }
            if let jig_client::Event::Start { kept } = &event {
                root.recovery.load(kept, &link_limits(env));
            }
            if let jig_client::Event::StartUnlinked { kept } = &event {
                root.recovery.load_rows(kept);
            }
            step_client(root, env, event, &mut client);
        }
        Event::Call { number, call, json, value } => {
            let active = root
                .runner
                .calls()
                .checked_add(root.maker.calls())
                .expect("bounded calls")
                .checked_add(root.drafts.calls())
                .expect("bounded calls")
                .checked_add(root.adopter.calls())
                .expect("bounded calls")
                .checked_add(root.forges.calls())
                .expect("bounded calls")
                .checked_add(root.follows.calls())
                .expect("bounded calls")
                .checked_add(root.landings.calls())
                .expect("bounded calls")
                .checked_add(root.changes.calls())
                .expect("bounded calls");
            let active = active.saturating_add(root.devices.calls());
            if active >= usize::try_from(env.limits.tools.rows).expect("rows fit") {
                let mut printed = jig::render::failure(b"call limit", json, 1);
                if call.tool == jig::rows::EXPORT_AT {
                    printed.file = jig::FileEnd::Remove;
                }
                out.push(Request::Printed { number, printed });
            } else if let Some(call_number) = root.next {
                root.next = call_number.checked_add(1);
                root.patience.insert(call_number, (number, env.now.saturating_add(crate::limits::DEADLINE)));
                let forge = crate::table::TOOLS.get(usize::from(call.tool)).expect("parsed tool").name
                    == crate::adopt::ROW.name;
                let called = runner::Called { number, call_number, call, json, value };
                if crate::push::owns(&called.call) {
                    crate::push::call(&mut root.devices, &root.client, env, called, &mut child);
                } else if crate::making::owns(&called.call) {
                    crate::making::call(&mut root.maker, &root.client, env, called, &mut child);
                } else if crate::follow::owns(&called.call) {
                    crate::follow::call(&mut root.follows, &root.client, env, called, &mut child);
                } else if crate::landing::owns(&called.call) {
                    crate::landing::call(&mut root.landings, &root.client, env, called, &mut child);
                } else if crate::change::owns(&called.call) {
                    crate::change::call(&mut root.changes, &root.client, env, called, &mut child);
                } else if crate::forges::owns(&called.call) {
                    crate::forges::call(&mut root.forges, &root.client, env, called, &mut child);
                } else if crate::drafts::owns(&called.call) {
                    crate::drafts::call(&mut root.drafts, &root.client, env, called, &mut child);
                } else if forge {
                    crate::forge::call(&mut root.adopter, &root.client, &env.limits, called, &mut child);
                } else {
                    runner::call(&mut root.runner, &root.client, &tools_env(env), called, &mut child);
                }
            } else {
                let mut printed = jig::render::failure(b"call number exhausted", json, 1);
                if call.tool == jig::rows::EXPORT_AT {
                    printed.file = jig::FileEnd::Remove;
                }
                out.push(Request::Printed { number, printed });
            }
        }
    }
    drain(root, env, &mut client, &mut child, out);
    root.client.reclaim();
    root.changed();
}
enum ClientOutput {
    Linked(jig_client::Request),
    Unlinked(jig_client::UnlinkedRequest),
}
fn step_client(root: &mut Root, env: &Env<Limits>, event: jig_client::Event, out: &mut Queue<ClientOutput>) {
    if root.unlinked {
        let mut requests = Queue::uncapped();
        root.client.step_unlinked(env.now, env.wall, event, &mut requests);
        while let Some(request) = requests.pop() {
            out.push(ClientOutput::Unlinked(request));
        }
    } else {
        let mut requests = Queue::uncapped();
        root.client.step(env.now, env.wall, event, &mut requests);
        while let Some(request) = requests.pop() {
            out.push(ClientOutput::Linked(request));
        }
    }
}
fn tools_env(env: &Env<Limits>) -> Env<jig::Limits> {
    env.with(env.limits.tools)
}
fn drain(
    root: &mut Root,
    env: &Env<Limits>,
    client: &mut Queue<ClientOutput>,
    child: &mut Queue<runner::Out>,
    out: &mut Queue<Request>,
) {
    // Settle admitted calls: owner fan-out is bounded by their retained state;
    // subsequent engine, storage and timer answers arrive in later inputs.
    while !client.is_empty() || !child.is_empty() {
        let client_outputs_dispatched = client.is_empty();
        if !client_outputs_dispatched {
            let request = client.pop().expect("client output pending");
            match request {
                ClientOutput::Linked(request) => match request {
                    jig_client::Request::Told { told } => route(root, env, told, child, out),
                    // M4's Authorize place answers it
                    jig_client::Request::Approval(_) | jig_client::Request::Authorized(_) => {}
                    request @ (jig_client::Request::Exchange { .. }
                    | jig_client::Request::OpenStream { .. }
                    | jig_client::Request::CloseStream
                    | jig_client::Request::Save { .. }
                    | jig_client::Request::Erase { .. }
                    | jig_client::Request::ServiceCredential { .. }) => {
                        root.recovery.output(&request, &link_limits(env));
                        out.push(Request::Client { request });
                    }
                },
                ClientOutput::Unlinked(request) => match request {
                    jig_client::UnlinkedRequest::Told { told } => route(root, env, told, child, out),
                    jig_client::UnlinkedRequest::ApprovalAct(_)
                    | jig_client::UnlinkedRequest::Approval(_)
                    | jig_client::UnlinkedRequest::Authorized(_) => {}
                    request @ (jig_client::UnlinkedRequest::Down { .. } | jig_client::UnlinkedRequest::SignOut) => {
                        root.recovery.output_unlinked(&request);
                        out.push(Request::Unlinked { request });
                    }
                },
            }
        } else if let Some(request) = child.pop() {
            // All client outputs, including a retired request's erase, have
            // been taken. A follow-on ask may now reuse its engine name.
            root.client.reclaim();
            match request {
                runner::Out::Client { event } => {
                    if let jig_client::Event::Send { key, name: Some(name), request } = &event
                        && let Some(me) = crate::forge::me(&root.client)
                        && let Some(kept) = root.recovery.same(*name, request, me.party.number, &root.client)
                    {
                        root.recovery.joined.insert(*key, kept);
                        if let Some(pending) = root.client.pending(kept)
                            && let jig_client::Progress::Retrying { why: jig_client::Retry::Full { .. } } =
                                pending.progress
                        {
                            route(root, env, Told::Pending { key: kept }, child, out);
                        }
                    } else {
                        step_client(root, env, event, client);
                    }
                }
                runner::Out::Record { number, record } => out.push(Request::Record { number, record }),
                runner::Out::Ready => out.push(Request::Ready),
                runner::Out::Kept { line } => out.push(Request::Kept { line }),
                runner::Out::Progress { line } => out.push(Request::Progress { line }),
                runner::Out::Printed { number, printed } => {
                    let mut finished = List::with_capacity(env.limits.tools.rows);
                    for (prefix, (call, _)) in &root.patience {
                        if *call == number {
                            finished.push(*prefix).expect("bounded calls");
                        }
                    }
                    for prefix in &finished {
                        root.patience.remove(prefix);
                    }
                    out.push(Request::Printed { number, printed });
                }
                runner::Out::DraftAmendment { called, task } => {
                    let prepared = match runner::acts::me(&root.runner, &root.client) {
                        Some(me) => crate::amend::prepare(&called.call, &task, me, &env.limits.tools),
                        None => Err(runner::acts::Failed::Fixed(b"signed out")),
                    };
                    runner::acts::prepared(&mut root.runner, &root.client, &tools_env(env), *called, prepared, child);
                }
            }
        }
    }
    assert!(client.is_empty() && child.is_empty(), "client and child hand-offs settled");
}
fn past_patience(root: &Root, env: &Env<Limits>, key: jig_client::Key, room_at: skein_lib::Wall) -> bool {
    let prefix = u32::try_from(key.number >> 32u32).expect("call prefix");
    let Some((_, until)) = root.patience.get(&prefix) else { return false };
    room_at.as_nanos().saturating_sub(env.wall.as_nanos()) > until.as_nanos().saturating_sub(env.now.as_nanos())
}
fn route(root: &mut Root, env: &Env<Limits>, told: Told, child: &mut Queue<runner::Out>, out: &mut Queue<Request>) {
    if let Told::Pending { key } = &told
        && let Some(pending) = root.client.pending(*key)
        && let jig_client::Progress::Retrying { why } = pending.progress
    {
        root.retries.insert(*key, why);
    }
    if let Told::Answered { key, .. } = &told {
        root.retries.remove(key);
    }
    if let Told::Pending { key } = &told
        && let Some(pending) = root.client.pending(*key)
        && let jig_client::Progress::Retrying { why: jig_client::Retry::Full { held, room_at } } = pending.progress
    {
        let own = past_patience(root, env, *key, room_at);
        let mut refused = List::with_capacity(env.limits.tools.rows);
        for (local, kept) in &root.recovery.joined {
            if kept == key && past_patience(root, env, *local, room_at) {
                refused.push(*local).expect("bounded calls");
            }
        }
        for local in &refused {
            root.recovery.joined.remove(local);
            route_owner(
                root,
                env,
                Told::Answered { key: *local, outcome: jig_client::Outcome::Full { held, room_at } },
                child,
                out,
            );
        }
        if own {
            // End only this call. The engine key stays in the small store,
            // with its recovery bookkeeping, for another call once room returns.
            route_owner(
                root,
                env,
                Told::Answered { key: *key, outcome: jig_client::Outcome::Full { held, room_at } },
                child,
                out,
            );
            return;
        }
    }
    if let Told::Kept { key, .. } = &told {
        jig::key::reserve(&mut root.next, *key);
        root.recovery.kept(*key);
    }
    if let Told::Answered { key, outcome } = &told {
        root.recovery.answered(*key);
        let mut joined = List::with_capacity(env.limits.tools.rows);
        for (local, kept) in &root.recovery.joined {
            if kept == key {
                joined.push(*local).expect("bounded calls");
            }
        }
        if !joined.is_empty() {
            // Clear the restored owners' bookkeeping without printing a second answer.
            let mut discard = Queue::uncapped();
            route_owner(root, env, told.clone(), &mut discard, out);
            for key in &joined {
                root.recovery.joined.remove(key);
                route_owner(root, env, Told::Answered { key: *key, outcome: outcome.clone() }, child, out);
            }
            return;
        }
    }
    route_owner(root, env, told, child, out);
}
fn link_limits(env: &Env<Limits>) -> jig_client_protocol_link::Limits {
    jig_client_protocol_link::Limits::new(env.limits.domain.client.domain, env.limits.domain.client.exchanges)
        .expect("checked client limits")
}
fn route_owner(
    root: &mut Root,
    env: &Env<Limits>,
    told: Told,
    child: &mut Queue<runner::Out>,
    out: &mut Queue<Request>,
) {
    let asker = match &told {
        Told::Link { .. } | Told::Dropped { .. } => RUNNER.asker(),
        Told::Kept { key, .. } | Told::Pending { key } | Told::Answered { key, .. } => key.asker,
        Told::Changed { handle } | Told::Text { handle } | Told::Ended { handle, .. } | Told::Page { handle, .. } => {
            handle.asker
        }
    };
    let me_changed = match &told {
        Told::Changed { handle } => handle.asker == RUNNER.asker() && handle.number == 0,
        Told::Link { .. }
        | Told::Pending { .. }
        | Told::Answered { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. }
        | Told::Text { .. }
        | Told::Ended { .. }
        | Told::Page { .. } => false,
    };
    match Owner::of(asker) {
        Some(Owner::Runner) => runner::told(&mut root.runner, &root.client, &tools_env(env), told, child),
        Some(Owner::Forge) => {
            match &told {
                Told::Kept { key, request } => runner::told(
                    &mut root.runner,
                    &root.client,
                    &tools_env(env),
                    Told::Kept { key: *key, request: request.clone() },
                    child,
                ),
                Told::Answered { key, .. } => root.runner.forget_kept(*key),
                Told::Link { .. }
                | Told::Pending { .. }
                | Told::Dropped { .. }
                | Told::Changed { .. }
                | Told::Text { .. }
                | Told::Ended { .. }
                | Told::Page { .. } => {}
            }
            crate::push::told(&mut root.devices, &root.client, env, &told, child);
            crate::making::told(&mut root.maker, &root.client, env, &told, child);
            crate::follow::told(&mut root.follows, &root.client, env, &told, child);
            crate::landing::told(&mut root.landings, &root.client, env, &told, child);
            crate::change::told(&mut root.changes, &root.client, env, &told, child);
            crate::forges::told(&mut root.forges, &root.client, env, &told, child);
            crate::drafts::told(&mut root.drafts, &root.client, env, &told, child);
            crate::forge::told(&mut root.adopter, &env.limits, told, child);
        }
        Some(Owner::Pages) | None => out.push(Request::Fact { fact: temper_client_domain::Fact::UnknownAsker }),
    }
    if me_changed {
        crate::push::ready(&mut root.devices, &root.client, env, child);
        crate::drafts::ready(&mut root.drafts, &root.client, env, child);
        crate::making::ready(&mut root.maker, &root.client, env, child);
        crate::follow::ready(&mut root.follows, &root.client, env, child);
        crate::landing::ready(&mut root.landings, &root.client, env, child);
        crate::change::ready(&mut root.changes, &root.client, env, child);
        crate::forge::ready(&mut root.adopter, &root.client, &env.limits, child);
        crate::forges::ready(&mut root.forges, &root.client, env, child);
    }
}
pub fn fire(root: &mut Root, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut client = Queue::uncapped();
    let mut child = Queue::uncapped();
    if root.unlinked {
        let mut requests = Queue::uncapped();
        root.client.fire_unlinked(env.now, env.wall, &mut requests);
        while let Some(request) = requests.pop() {
            client.push(ClientOutput::Unlinked(request));
        }
    } else {
        let mut requests = Queue::uncapped();
        root.client.fire(env.now, env.wall, &mut requests);
        while let Some(request) = requests.pop() {
            client.push(ClientOutput::Linked(request));
        }
    }
    drain(root, env, &mut client, &mut child, out);
    jig::wait::fire(&mut root.runner, &root.client, &tools_env(env), &mut child);
    crate::making::fire(&mut root.maker, &root.client, env, &mut child);
    crate::change::fire(&mut root.changes, env, &mut child);
    crate::drafts::fire(&mut root.drafts, &root.client, env, &mut child);
    crate::forges::fire(&mut root.forges, &root.client, env, &mut child);
    drain(root, env, &mut client, &mut child, out);
    root.client.reclaim();
    root.changed();
}
#[must_use]
pub fn next_deadline(root: &Root) -> Option<Time> {
    root.deadline
}
fn deadline(root: &Root) -> Option<Time> {
    root.client
        .next_deadline()
        .into_iter()
        .chain(jig::wait::next_deadline(&root.runner))
        .chain(root.drafts.next_deadline())
        .chain(root.forges.next_deadline())
        .chain(root.maker.next_deadline())
        .chain(root.changes.next_deadline())
        .min()
}

#[cfg(test)]
#[path = "tests/root.rs"]
mod tests;

/// Forge parts use the domain's shared words and whole resource paths.
#[derive(Debug)]
pub struct Parts;
impl jig::render::Parts for Parts {
    fn prepare_setting(&self, call: &mut jig::call::Call, me: Option<&jig_client::contract::Me>) -> bool {
        crate::settings::resolve_project(call, me)
    }
    fn setting_selector(&self, call: &jig::call::Call) -> Option<jig_client::contract::Setting> {
        crate::settings::selector(call)
    }
    fn application_read(&self, query: runner::application::Query) -> jig_client::Readable {
        crate::settings::read(query)
    }
    fn application_answer(
        &self,
        query: runner::application::Query,
        page: &jig_client::Readout,
    ) -> Option<runner::application::Answer> {
        crate::settings::decode(query, page)
    }
    fn setting_name(&self, setting: jig_client::contract::Setting, out: &mut jig::render::Text) -> bool {
        crate::settings::name(setting, out)
    }
    fn setting_value(&self, kind: u32, bytes: &[u8], out: &mut skein_json::writer::Encoder) -> bool {
        crate::settings::render(kind, bytes, out)
    }
    fn diagnostic_prefix(&self) -> &'static [u8] {
        DIAGNOSTIC_PREFIX
    }
    fn command_prefix(&self) -> &'static [u8] {
        b"temper ctl DIR "
    }
    fn failures_recovery(&self) -> &'static [u8] {
        b"the engine's log names each failure (temper help hold), then "
    }
    fn phrasebook(&self, text: u32) -> skein_ui::phrases::Phrasebook {
        temper_client_domain::words::phrasebook(text)
    }
    fn release_again_refusal(&self, task: &jig_client::contract::Task) -> Option<&'static [u8]> {
        if task.chip.executor
            == (jig_client::contract::ChipExecutor::Procedure {
                connector: temper_client_protocol_wire::FORGE,
                kind: 2,
            })
        {
            Some(b"task release: a change has no run of its own to repeat: release its hold, and it decides afresh")
        } else {
            None
        }
    }
    fn task_next(&self, task: &jig_client::contract::Task, out: &mut jig::render::Text) {
        if let jig_client::contract::TaskContractDocument::Change { connector, kind: 1, .. } = task.result_contract
            && connector == temper_client_protocol_wire::FORGE
        {
            out.put(b"next: temper ctl DIR change show ");
            out.number(task.chip.task);
            out.put(b"\n");
        }
    }
    fn owner_part(&self, part: &jig_client::contract::Part, out: &mut jig::render::Text) {
        crate::forge::part(part, out);
    }
    fn part(&self, part: &jig_client::contract::ApplicationPart, out: &mut jig::render::Text) {
        crate::forge::result_part(part, out);
    }
    fn change_kind(&self, connector: u16, kind: u16, out: &mut jig::render::Text) {
        crate::forge::change_kind(connector, kind, out);
    }
    fn change(
        &self,
        kind: u16,
        part: &jig_client::contract::Part,
        words: &jig_client::contract::DocumentText,
        out: &mut jig::render::Text,
    ) {
        crate::forge::change(kind, part, words, out);
    }
}

#[cfg(test)]
#[path = "tests/fixtures/root_acts.rs"]
pub(crate) mod act_fixture;

impl Root {
    #[must_use]
    pub fn file_end(&self, number: u32) -> jig::FileEnd {
        runner::file_end(&self.runner, number)
    }
    pub fn recorded(
        &mut self,
        env: &Env<Limits>,
        number: u32,
        result: Result<(), Box<[u8]>>,
        out: &mut Queue<Request>,
    ) {
        let mut client = Queue::uncapped();
        let mut child = Queue::uncapped();
        runner::recorded(&mut self.runner, &self.client, &tools_env(env), number, result, &mut child);
        drain(self, env, &mut client, &mut child, out);
        self.client.reclaim();
        self.changed();
    }
}
