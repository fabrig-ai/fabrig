//! Calls share the component's connections and advance their held plans.
use super::{
    Base, Body, Common, Component, Dialect, ForgejoCall, GitHubCall, Heard, Phase, Plan, Side, Start, Step, Waiting,
    api,
};
use crate::connection;
use crate::{
    boundary::{Answered, Event},
    limits::{self, Limits, Listing, Shared},
};
use alloc::boxed::Box;
use skein_forge::forgejo::plans::plan::Bounds;
use skein_lib::{Env, Id, Lifecycle, List, Queue, Token};

/// What `assign` did for a call.
pub(super) enum Assigned {
    /// No connection for it yet.
    Waits,
    /// It keeps the connection it holds.
    Held,
    /// A connection is newly its own: its exchange starts fresh.
    New,
}

impl Common {
    pub(super) fn expire(
        &mut self,
        call: Token,
        id: Id<Waiting>,
        env: &Env<Limits>,
        up: &mut Queue<Event>,
        io: &mut Queue<skein_io::Request>,
    ) {
        let waiting = self.waiting.get(id).expect("indexed call");
        let error = if waiting.cost == 0 { api::Error::Unavailable } else { api::Error::Timeout };
        let error = waiting.error(error);
        if let Some(token) = waiting.connection {
            self.connection_slab
                .get_mut(*self.connections.get(&token).expect("holder"))
                .expect("connection")
                .close(env, true, io);
        }
        self.finish(call, id, Err(error), up);
    }
    pub(super) fn finish(
        &mut self,
        call: Token,
        id: Id<Waiting>,
        result: Result<api::Answer, api::Error>,
        up: &mut Queue<Event>,
    ) {
        let waiting = self.waiting.get(id).expect("indexed call");
        let cost = waiting.cost;
        let free = waiting.free;
        let rate = waiting.rate;
        if let Some(token) = waiting.connection {
            let _holder = self.holders.remove(&token);
        }
        self.waiting.retire(id);
        let _call = self.calls.remove(&call);
        up.push(Event::Answered(Answered { call, cost, free, rate, result }));
    }

    pub(super) fn assign(
        &mut self,
        call: Token,
        id: Id<Waiting>,
        origin: Option<u32>,
        base: &mut Base,
        env: &Env<Limits>,
        io: &mut Queue<skein_io::Request>,
    ) -> Assigned {
        let waiting = self.waiting.get(id).expect("call");
        if waiting.connection.is_some() {
            return Assigned::Held;
        }
        let forge = waiting.repository.forge;
        if !super::ready(base.start) {
            return Assigned::Waits;
        }
        let mut token = None;

        let mut other = None;
        let call_connections = self.call_connections();
        for (owner, connection) in &self.connections {
            let c = self.connection_slab.get(*connection).expect("indexed connection");
            if c.role == connection::Use::Reserved {
                continue;
            }
            if c.phase() == Phase::Idle && !c.closing() && !self.holders.contains_key(owner) {
                if c.forge == forge && c.origin == origin {
                    token = Some(*owner);
                    break;
                }
                if c.role == connection::Use::Calls {
                    other = Some(*connection);
                }
            }
        }
        if token.is_none()
            && call_connections < env.limits.connections
            && self.connections.len() < self.connections.capacity()
        {
            token = Some(self.open_call_origin(&base.forge, origin, env, io));
        }
        if token.is_none() {
            if let Some(other) = other {
                self.connection_slab.get_mut(other).expect("idle other forge").close(env, false, io);
            }
            return Assigned::Waits;
        }
        let token = token.expect("selected connection");
        if base.own == Some(token) {
            base.own = None;
            self.connection_slab
                .get_mut(*self.connections.get(&token).expect("own token"))
                .expect("own connection")
                .role = connection::Use::Calls;
        }
        let _old = self.holders.insert(token, call).expect("one holder per connection");
        let waiting = self.waiting.get_mut(id).expect("indexed call");
        waiting.connection = Some(token);
        Assigned::New
    }
    fn begin_forgejo_call(
        &mut self,
        call: Token,
        id: Id<Waiting>,
        base: &Base,
        env: &Env<Limits>,
        up: &mut Queue<Event>,
        io: &mut Queue<skein_io::Request>,
    ) -> Option<(Box<ForgejoCall>, bool)> {
        let waiting = self.waiting.get_mut(id).expect("call");
        let cap = super::page_cap(base.start)?;
        let op = waiting.op.take().expect("waiting operation");
        waiting.page_size = page_size(&op, crate::settings::Kind::Forgejo, cap, &env.limits, &self.shared);
        let mut plan = Plan::new(waiting.repository, op);
        let step = plan.begin(
            bounds(waiting.page_size, &env.limits, &self.shared),
            &self.owner.word,
            &mut self.forgejo_tables,
        );
        let mut forgejo = Box::new(ForgejoCall { plan, exchange: connection::forgejo::Exchange::new() });
        let pump = match step {
            Ok(step) => !self.call_step(id, &mut forgejo, step, env, up, io, call),
            Err(error) => {
                self.finish(call, id, Err(error), up);
                false
            }
        };
        Some((forgejo, pump))
    }
    fn pump_call(
        &mut self,
        holder: (Token, Id<Waiting>),
        forgejo: &mut ForgejoCall,
        base: &mut Base,
        env: &Env<Limits>,
        up: &mut Queue<Event>,
        io: &mut Queue<skein_io::Request>,
    ) {
        let (call, id) = holder;
        let token = self.waiting.get(id).expect("call").connection.expect("holder");
        let connection = *self.connections.get(&token).expect("indexed connection");
        for _ in 0..64_u32 {
            let c = self.connection_slab.get_mut(connection).expect("holding connection");
            if c.phase() == Phase::Idle {
                let waiting = self.waiting.get_mut(id).expect("indexed call");
                if waiting.revoked {
                    c.until(Some(env.now.saturating_add(env.limits.idle)));
                    let error = forgejo.plan.error(api::Error::Forbidden);
                    self.finish(call, id, Err(error), up);
                    break;
                }
                let deadline = waiting.deadline;
                let plan = &mut forgejo.plan;
                if let Some(request) = plan.pending() {
                    c.raw_room(plan.raw_room(self.shared.answer_bytes));
                    let forge = &base.forge;
                    let authorization = skein_forge::forgejo::credential::authorization(
                        &forge.access.as_ref().expect("held access").credential,
                    )
                    .expect("checked credential length");
                    c.until(Some(deadline));
                    match forgejo.exchange.start(
                        c,
                        env,
                        forge,
                        &self.owner.user_agent,
                        request,
                        authorization,
                        &mut self.kept,
                    ) {
                        Ok(()) => {
                            forgejo.exchange.keep_after(plan.after());
                        }
                        Err(_) => {
                            self.finish(call, id, Err(api::Error::Refused), up);
                            break;
                        }
                    }
                    self.waiting.get_mut(id).expect("call").charged = false;
                }
            }
            let waiting = self.waiting.get_mut(id).expect("indexed call");
            forgejo.exchange.pump(c, env, io, &mut self.kept);
            waiting.free = waiting.free.checked_add(c.take_free()).expect("bounded exchanges");
            if c.written && !waiting.charged {
                waiting.cost = waiting.cost.checked_add(1).expect("bounded exchanges");
                waiting.charged = true;
            }
            if let Some(heard) = forgejo.exchange.heard(c) {
                let heard = call_heard(heard, waiting, &mut self.diagnostics);
                let page_size = waiting.page_size;
                let bounds = bounds(page_size, &env.limits, &self.shared);
                let step = forgejo.plan.heard(heard, bounds, &self.owner.word, &mut self.forgejo_tables);
                let cut = forgejo.plan.take_cut();
                if cut {
                    forgejo.exchange.cut(c);
                }
                match step {
                    Ok(Some(step)) => {
                        if self.call_step(id, forgejo, step, env, up, io, call) {
                            break;
                        }
                    }
                    Ok(None) => {}
                    Err(error) => {
                        c.close(env, true, io);
                        self.finish(call, id, Err(error), up);
                        break;
                    }
                }
                continue;
            }
            if c.phase() == Phase::Closed {
                c.close(env, true, io);
            }
            if !forgejo.exchange.ready(c) {
                break;
            }
            self.wake = Some(env.now);
        }
    }
    #[expect(clippy::too_many_arguments, reason = "the call carries its plan through its checked step")]
    fn call_step(
        &mut self,
        id: Id<Waiting>,
        forgejo: &mut ForgejoCall,
        step: Step,
        env: &Env<Limits>,
        up: &mut Queue<Event>,
        io: &mut Queue<skein_io::Request>,
        call: Token,
    ) -> bool {
        let waiting = self.waiting.get_mut(id).expect("indexed call");
        let token = waiting.connection.expect("holder");
        let connection = *self.connections.get(&token).expect("connection");
        let c = self.connection_slab.get_mut(connection).expect("connection");
        match step {
            Step::Send(exchange) => {
                forgejo.plan.set_pending(exchange);
                if c.phase() == Phase::Closed {
                    c.close(env, false, io);
                    let _holder = self.holders.remove(&token);
                    self.waiting.get_mut(id).expect("call").connection = None;
                    self.wake = Some(env.now);
                    return true;
                }
                false
            }
            Step::Done(result) => {
                if c.phase() == Phase::Closed {
                    c.close(env, false, io);
                } else {
                    c.until(Some(env.now.saturating_add(env.limits.idle)));
                }
                self.finish(call, id, result, up);
                true
            }
        }
    }
}

fn call_heard(heard: Heard, waiting: &mut Waiting, diagnostics: &mut super::Diagnostics) -> Heard {
    match &heard {
        Heard::Head { status, .. } => waiting.answered_head(*status, None),
        Heard::Done(_) | Heard::Row(_) | Heard::Failed(_) => {}
    }
    if skein_forge::forgejo::plans::reads::keep::long(&heard) {
        diagnostics.long_bodies = diagnostics.long_bodies.saturating_add(1);
    }
    let malformed = match &heard {
        Heard::Done(Body::Malformed(_)) => true,
        Heard::Done(Body::Document(_) | Body::Raw { .. } | Body::Empty)
        | Heard::Head { .. }
        | Heard::Row(_)
        | Heard::Failed(_) => false,
    };
    if malformed {
        diagnostics.malformed = diagnostics.malformed.saturating_add(1);
    }
    heard
}

fn bounds(page_size: u32, l: &Limits, s: &Shared) -> Bounds {
    Bounds { page_size, rows: s.rows, answer_bytes: s.answer_bytes, body_bytes: l.body_bytes }
}

pub(super) fn page_size(op: &api::Op, kind: crate::settings::Kind, cap: u32, l: &Limits, s: &Shared) -> u32 {
    let listing = match op {
        api::Op::Read(read) => match read {
            api::Read::Items { single, .. } => {
                if *single {
                    return 1;
                }
                return limits::page_size(Listing::Items, cap, l, s);
            }
            api::Read::PullFor { .. } => return limits::bounded_page_size(Listing::Items, cap, l, s),
            api::Read::Reviews { .. } => return limits::page_size(Listing::Reviews, cap, l, s),
            api::Read::Statuses { .. } | api::Read::Checks { .. } => Listing::Statuses,
            api::Read::Pull { .. } => match kind {
                crate::settings::Kind::Forgejo => Listing::Statuses,
                crate::settings::Kind::GitHub => return 1,
            },
            api::Read::Remarks { .. } => return limits::page_size(Listing::Remarks, cap, l, s),
            api::Read::Branches { .. } => Listing::Branches,
            api::Read::Collaborators { .. } => Listing::Collaborators,
            api::Read::PullFiles { .. } => return limits::page_size(Listing::PullFiles, cap, l, s),
            api::Read::Compare { .. } | api::Read::Commits { .. } => Listing::Compare,
            api::Read::Runs { .. } => Listing::Runs,
            api::Read::Tree { .. } => Listing::Tree,
            api::Read::Item { .. } => {
                return match kind {
                    crate::settings::Kind::Forgejo => 1,
                    crate::settings::Kind::GitHub => limits::page_size(Listing::Items, cap, l, s),
                };
            }
            api::Read::Locate { .. }
            | api::Read::Branch { .. }
            | api::Read::Protection { .. }
            | api::Read::Permission { .. }
            | api::Read::File { .. }
            | api::Read::Run { .. }
            | api::Read::Artifact { .. }
            | api::Read::Job { .. }
            | api::Read::Settings => return 1,
        },
        api::Op::Write(_) => return 1,
    };
    match kind {
        crate::settings::Kind::Forgejo => limits::page_size(listing, cap, l, s),
        crate::settings::Kind::GitHub => limits::bounded_page_size(listing, cap, l, s),
    }
}

impl Component {
    #[expect(clippy::too_many_lines, reason = "the call keeps its side out through each dialect's drive")]
    pub(super) fn drive_calls(&mut self, env: &Env<Limits>, up: &mut Queue<Event>, io: &mut Queue<skein_io::Request>) {
        let mut calls = List::with_capacity(self.common.calls.capacity());
        for (call, id) in &self.common.calls {
            calls.push((*call, *id)).expect("bounded calls");
        }
        for (call, id) in calls.into_boxed() {
            if self.common.lifecycle != Lifecycle::Live {
                break;
            }
            let waiting = self.common.waiting.get(id).expect("indexed call");
            if waiting.deadline <= env.now {
                self.common.expire(call, id, env, up, io);
                continue;
            }
            let forge = waiting.repository.forge;
            let Some(entry) = super::find(&mut self.forges, forge) else {
                self.common.finish(call, id, Err(api::Error::Unavailable), up);
                continue;
            };
            if waiting.connection.is_none() {
                if waiting.revoked {
                    let error = waiting.error(api::Error::Forbidden);
                    self.common.finish(call, id, Err(error), up);
                    continue;
                }
                if !waiting.begun() {
                    match entry.base.start {
                        Start::Unconnected | Start::Stopped(_) => {
                            self.common.finish(call, id, Err(api::Error::NotConnected), up);
                            continue;
                        }
                        Start::Listing { .. }
                        | Start::Minting { .. }
                        | Start::Identity { .. }
                        | Start::Checking { .. }
                        | Start::Settings { .. }
                        | Start::Waiting { .. } => continue,
                        Start::Ready { .. } => {}
                    }
                }
            }
            let own = entry.base.own;
            let base = &mut entry.base;
            let side = core::mem::replace(&mut self.common.waiting.get_mut(id).expect("call").side, Side::Unbegun);
            let side = match (side, &mut entry.dialect) {
                (Side::Unbegun, Dialect::GitHub(gh)) => {
                    let mut github = Box::new(GitHubCall::new());
                    self.common.drive_github_call((call, id), &mut github, base, gh, env, up, io);
                    Side::GitHub(github)
                }
                (Side::GitHub(mut github), Dialect::GitHub(gh)) => {
                    self.common.drive_github_call((call, id), &mut github, base, gh, env, up, io);
                    Side::GitHub(github)
                }
                (Side::Unbegun, Dialect::Forgejo { .. }) => match self.common.assign(call, id, None, base, env, io) {
                    Assigned::Waits => Side::Unbegun,
                    Assigned::Held | Assigned::New => match self.common.begin_forgejo_call(call, id, base, env, up, io)
                    {
                        Some((mut forgejo, pump)) => {
                            if pump {
                                self.common.pump_call((call, id), &mut forgejo, base, env, up, io);
                            }
                            Side::Forgejo(forgejo)
                        }
                        None => Side::Unbegun,
                    },
                },
                (Side::Forgejo(mut forgejo), Dialect::Forgejo { .. }) => {
                    let pump = match self.common.assign(call, id, None, base, env, io) {
                        Assigned::Waits => false,
                        Assigned::Held => true,
                        Assigned::New => {
                            forgejo.exchange = connection::forgejo::Exchange::new();
                            true
                        }
                    };
                    if pump {
                        self.common.pump_call((call, id), &mut forgejo, base, env, up, io);
                    }
                    Side::Forgejo(forgejo)
                }
                (Side::Forgejo(_), Dialect::GitHub(_)) | (Side::GitHub(_), Dialect::Forgejo { .. }) => {
                    self.common.finish(call, id, Err(api::Error::Unavailable), up);
                    Side::Unbegun
                }
            };
            if own.is_some() && entry.base.own.is_none() {
                entry.dialect.forget_exchange();
            }
            self.common.waiting.get_mut(id).expect("call").side = side;
        }
        for (_, id) in &self.common.connections {
            let c = self.common.connection_slab.get_mut(*id).expect("indexed connection");
            if c.drain_redirect(env, io, &mut self.common.kept) {
                self.common.wake = Some(env.now);
            }
            if self.common.lifecycle != Lifecycle::Live
                || (c.phase() == Phase::Idle && c.deadline() <= Some(env.now))
                || c.phase() == Phase::Closed
            {
                c.close(env, c.in_flight(), io);
                if c.link.has_work() {
                    c.drain(env, io, &mut self.common.kept);
                }
                if c.link.has_work() {
                    self.common.wake = Some(env.now);
                }
            }
        }
    }
}
