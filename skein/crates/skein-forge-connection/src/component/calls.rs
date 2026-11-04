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
        base: &mut Base,
        env: &Env<Limits>,
        io: &mut Queue<skein_io::Request>,
    ) -> bool {
        let waiting = self.waiting.get(id).expect("call");
        if waiting.connection.is_some() {
            return true;
        }
        let forge = waiting.repository.forge;
        if !super::ready(base.start) {
            return false;
        }
        let mut token = None;

        let mut other = None;
        let call_connections = self.call_connections();
        for (owner, connection) in &self.connections {
            let c = self.connection_slab.get(*connection).expect("indexed connection");
            if c.role == connection::Use::Reserved {
                continue;
            }
            if c.phase == Phase::Idle && !c.closing && !self.holders.contains_key(owner) {
                if c.forge == forge && c.origin == waiting.origin() {
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
            let origin = waiting.origin();
            token = Some(self.open_call_origin(&base.forge, origin, env, io));
        }
        if token.is_none() {
            if let Some(other) = other {
                self.connection_slab.get_mut(other).expect("idle other forge").close(env, false, io);
            }
            return false;
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
        match &mut waiting.side {
            Side::Forgejo(call) => call.exchange = connection::forgejo::Exchange::new(),
            Side::GitHub(call) => call.exchange = connection::github::Exchange::new(),
            Side::Unbegun => {}
        }
        true
    }
    fn begin_forgejo_call(
        &mut self,
        call: Token,
        id: Id<Waiting>,
        base: &Base,
        env: &Env<Limits>,
        up: &mut Queue<Event>,
        io: &mut Queue<skein_io::Request>,
    ) -> bool {
        let waiting = self.waiting.get_mut(id).expect("call");
        if waiting.begun() {
            return true;
        }
        let Some(cap) = super::page_cap(base.start) else {
            return false;
        };
        let op = waiting.op.take().expect("waiting operation");
        waiting.page_size = page_size(&op, crate::settings::Kind::Forgejo, cap, &env.limits, &self.shared);
        let mut plan = Plan::new(waiting.repository, op);
        let step = plan.begin(
            bounds(waiting.page_size, &env.limits, &self.shared),
            &self.owner.word,
            &mut self.forgejo_tables,
        );
        waiting.side = Side::Forgejo(Box::new(ForgejoCall { plan, exchange: connection::forgejo::Exchange::new() }));
        match step {
            Ok(step) => !self.call_step(id, step, env, up, io, call),
            Err(error) => {
                self.finish(call, id, Err(error), up);
                false
            }
        }
    }
    fn pump_call(
        &mut self,
        holder: (Token, Id<Waiting>),
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
            if c.phase == Phase::Idle {
                let waiting = self.waiting.get_mut(id).expect("indexed call");
                if waiting.revoked {
                    c.deadline = Some(env.now.saturating_add(env.limits.idle));
                    let error = waiting.error(api::Error::Forbidden);
                    self.finish(call, id, Err(error), up);
                    break;
                }
                let deadline = waiting.deadline;
                let Some(forgejo) = waiting.forgejo() else {
                    break;
                };
                let plan = &mut forgejo.plan;
                if let Some(request) = plan.pending() {
                    c.deadline = Some(deadline);
                    c.raw_room(plan.raw_room(self.shared.answer_bytes));
                    let forge = &base.forge;
                    let authorization = skein_forge::forgejo::credential::authorization(
                        &forge.access.as_ref().expect("held access").credential,
                    )
                    .expect("checked credential length");
                    match forgejo.exchange.start(c, env, forge, &self.owner.user_agent, request, authorization) {
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
            let Some(forgejo) = waiting.forgejo() else { break };
            forgejo.exchange.pump(c, env, io);
            if c.written && !waiting.charged {
                waiting.cost = waiting.cost.checked_add(1).expect("bounded exchanges");
                waiting.charged = true;
            }
            let Some(forgejo) = waiting.forgejo() else { break };
            if let Some(heard) = forgejo.exchange.heard(c) {
                let heard = call_heard(heard, waiting, &mut self.diagnostics);
                let page_size = waiting.page_size;
                let Some(forgejo) = waiting.forgejo() else {
                    break;
                };
                let step = forgejo.plan.heard(
                    heard,
                    bounds(page_size, &env.limits, &self.shared),
                    &self.owner.word,
                    &mut self.forgejo_tables,
                );
                let cut = forgejo.plan.take_cut();
                if cut {
                    forgejo.exchange.cut(c);
                }
                match step {
                    Ok(Some(step)) => {
                        if self.call_step(id, step, env, up, io, call) {
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
            if c.phase == Phase::Closed {
                c.close(env, true, io);
            }
            if !forgejo.exchange.ready(c) {
                break;
            }
            self.wake = Some(env.now);
        }
    }
    fn call_step(
        &mut self,
        id: Id<Waiting>,
        step: Step,
        env: &Env<Limits>,
        up: &mut Queue<Event>,
        io: &mut Queue<skein_io::Request>,
        call: Token,
    ) -> bool {
        let waiting = self.waiting.get_mut(id).expect("indexed call");
        let token = waiting.connection.expect("holder");
        let Some(forgejo) = waiting.forgejo() else {
            return true;
        };
        let connection = *self.connections.get(&token).expect("connection");
        let c = self.connection_slab.get_mut(connection).expect("connection");
        match step {
            Step::Send(exchange) => {
                forgejo.plan.set_pending(exchange);
                if c.phase == Phase::Closed {
                    c.close(env, false, io);
                    let _holder = self.holders.remove(&token);
                    self.waiting.get_mut(id).expect("call").connection = None;
                    self.wake = Some(env.now);
                    return true;
                }
                false
            }
            Step::Done(result) => {
                if c.phase == Phase::Closed {
                    c.close(env, false, io);
                } else {
                    c.deadline = Some(env.now.saturating_add(env.limits.idle));
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
            api::Read::Items { .. } | api::Read::PullFor { .. } => Listing::Items,
            api::Read::Reviews { .. } => Listing::Reviews,
            api::Read::Statuses { .. } | api::Read::Checks { .. } => Listing::Statuses,
            api::Read::Pull { .. } => match kind {
                crate::settings::Kind::Forgejo => Listing::Statuses,
                crate::settings::Kind::GitHub => return 1,
            },
            api::Read::Remarks { .. } => Listing::Remarks,
            api::Read::Branches { .. } => Listing::Branches,
            api::Read::Collaborators { .. } => Listing::Collaborators,
            api::Read::PullFiles { .. } => Listing::PullFiles,
            api::Read::Compare { .. } | api::Read::Commits { .. } => Listing::Compare,
            api::Read::Runs { .. } => Listing::Runs,
            api::Read::Tree { .. } => Listing::Tree,
            api::Read::Locate { .. }
            | api::Read::Item { .. }
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
    limits::page_size(listing, cap, l, s)
}

impl Component {
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
            let base = &mut entry.base;
            match (&waiting.side, &mut entry.dialect) {
                (Side::Unbegun | Side::GitHub(_), Dialect::GitHub(gh)) => {
                    let waiting = self.common.waiting.get_mut(id).expect("call");
                    if waiting.github().is_none() {
                        waiting.side = Side::GitHub(Box::new(GitHubCall {
                            exchange: connection::github::Exchange::new(),
                            plan: None,
                            route: None,
                            discovery: None,
                            discovered: None,
                        }));
                    }
                    self.common.drive_github_call((call, id), base, gh, env, up, io);
                }
                (Side::Unbegun | Side::Forgejo(_), Dialect::Forgejo { .. }) => {
                    if self.common.assign(call, id, base, env, io)
                        && self.common.begin_forgejo_call(call, id, base, env, up, io)
                    {
                        self.common.pump_call((call, id), base, env, up, io);
                    }
                }
                (Side::Forgejo(_), Dialect::GitHub(_)) | (Side::GitHub(_), Dialect::Forgejo { .. }) => {
                    self.common.finish(call, id, Err(api::Error::Unavailable), up);
                }
            }
        }
        for (_, id) in &self.common.connections {
            let c = self.common.connection_slab.get_mut(*id).expect("indexed connection");
            if c.drain_redirect(env, io) {
                self.common.wake = Some(env.now);
            }
            if self.common.lifecycle != Lifecycle::Live
                || (c.phase == Phase::Idle && c.deadline <= Some(env.now))
                || c.phase == Phase::Closed
            {
                c.close(env, c.in_flight(), io);
                if c.link.has_work() {
                    c.drain(env, io);
                }
                if c.link.has_work() {
                    self.common.wake = Some(env.now);
                }
            }
        }
    }
}
