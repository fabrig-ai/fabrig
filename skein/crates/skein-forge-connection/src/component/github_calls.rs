//! Installation account routing and bounded repository discovery (forge/connection.md, 4).
use super::{Base, Common, GitHubCall, GitHubForge, Waiting, calls};
use crate::{Event, Limits, connection::Phase, installations::Selected};
use alloc::boxed::Box;
use skein_forge::{
    api,
    github::{
        self,
        plans::{
            Body, Heard,
            plan::{Bounds, Plan, Step},
            tables::Tables,
        },
        request::Auth,
    },
};
use skein_lib::{Env, Id, List, Queue, Token};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Route {
    selected: Selected,
    token_revision: u64,
    status: Option<u16>,
    pub(super) origin: Option<u32>,
}
/// One call's finite eligibility snapshot and private validated row.
#[derive(Debug)]
pub(crate) struct Discovery {
    candidates: List<Selected>,
    cursor: u32,
    skipped: Option<api::Error>,
    tables: Tables,
}
impl Discovery {
    fn new(data: &crate::installations::GitHub) -> Self {
        let mut candidates = List::with_capacity(data.rows.capacity());
        let mut recent = None;
        if let Some(id) = data.recent
            && let Some(row) = data.rows.get(&id)
            && eligible(row)
        {
            candidates.push(Selected { id, connect: data.connect, revision: row.revision }).expect("row bound");
            recent = Some(id);
        }
        for (id, row) in &data.rows {
            if Some(*id) != recent && eligible(row) {
                candidates
                    .push(Selected { id: *id, connect: data.connect, revision: row.revision })
                    .expect("row bound");
            }
        }
        Self { candidates, cursor: 0, skipped: None, tables: Tables::new(1, 1) }
    }
    fn advance(&mut self) {
        self.cursor = self.cursor.checked_add(1).expect("bounded candidates");
    }
    fn select(&mut self, data: &crate::installations::GitHub) -> Result<Selected, api::Error> {
        for _ in self.cursor..self.candidates.len() {
            let selected = *self.candidates.get(self.cursor).expect("candidate cursor");
            if selected.connect != data.connect {
                return Err(api::Error::Forbidden);
            }
            if data.valid(selected)
                && data.rows.get(&selected.id).expect("valid row").state != crate::installations::Tokens::Gone
            {
                return Ok(selected);
            }
            self.skipped = Some(match data.rows.get(&selected.id) {
                Some(row) if row.accepted && row.suspended => api::Error::Forbidden,
                Some(_) | None => api::Error::NotInstalled,
            });
            self.advance();
        }
        Err(self.skipped.unwrap_or(api::Error::Missing))
    }
}
fn eligible(row: &crate::installations::Row) -> bool {
    row.accepted && !row.suspended && row.state != crate::installations::Tokens::Gone
}
impl Common {
    #[expect(clippy::manual_map, reason = "the programming model excludes closures")]
    fn github_route(
        &mut self,
        id: Id<Waiting>,
        github: &mut GitHubCall,
        gh: &mut GitHubForge,
        env: &Env<Limits>,
    ) -> Result<bool, api::Error> {
        let waiting = self.waiting.get_mut(id).expect("call");
        let revoked = waiting.revoked;
        let data = &gh.state;
        if let Some(discovery) = &mut github.discovery {
            if revoked {
                return Err(api::Error::Forbidden);
            }
            let selected = discovery.select(data)?;
            if let Some(route) = github.route
                && (route.selected.id != selected.id || route.selected.revision != selected.revision)
            {
                github.plan = None;
                discovery.tables = Tables::new(1, 1);
            }
            let row = data.rows.get(&selected.id).expect("eligible row");
            if github.route.is_none() || github.plan.is_none() {
                github.route = Some(Route { selected, token_revision: row.token_revision, status: None, origin: None });
            }
            return Ok(data.identity_checked && row.credential(env.now).is_some());
        }
        if let Some(route) = github.route {
            if revoked || !data.valid(route.selected) {
                return Err(api::Error::Forbidden);
            }
            return Ok(route.origin.is_some()
                || (data.identity_checked
                    && data.rows.get(&route.selected.id).expect("eligible row").credential(env.now).is_some()));
        }
        let owner = match self.github_tables.repository(waiting.repository) {
            Some(row) => Some(row.owner_id),
            None => None,
        };
        let login = match waiting.op.as_ref() {
            Some(api::Op::Read(api::Read::Locate { owner, .. })) => Some(owner.as_ref()),
            Some(api::Op::Read(_) | api::Op::Write(_)) | None => None,
        };
        let mut chosen = None;
        if owner.is_some() || login.is_some() {
            for (installation, row) in &data.rows {
                if row.accepted && (owner == Some(row.account.id) || login == Some(row.account.login.as_ref())) {
                    chosen = Some(*installation);
                    break;
                }
            }
        } else {
            if let Some(recent) = data.recent
                && let Some(row) = data.rows.get(&recent)
                && row.accepted
                && !row.suspended
                && row.state != crate::installations::Tokens::Gone
            {
                chosen = Some(recent);
            }
            if chosen.is_none() {
                for (installation, row) in &data.rows {
                    if row.accepted && !row.suspended && row.state != crate::installations::Tokens::Gone {
                        chosen = Some(*installation);
                        break;
                    }
                }
            }
        }
        let installation = chosen.ok_or(api::Error::NotInstalled)?;
        let row = data.rows.get(&installation).expect("chosen row");
        if row.suspended {
            return Err(api::Error::Forbidden);
        }
        github.route = Some(Route {
            selected: Selected { id: installation, connect: data.connect, revision: row.revision },
            token_revision: row.token_revision,
            status: None,
            origin: None,
        });
        Ok(data.identity_checked && row.credential(env.now).is_some())
    }
    #[expect(clippy::too_many_arguments, reason = "the call carries its split forge through its checked step")]
    pub(super) fn drive_github_call(
        &mut self,
        holder: (Token, Id<Waiting>),
        github: &mut GitHubCall,
        base: &mut Base,
        gh: &mut GitHubForge,
        env: &Env<Limits>,
        up: &mut Queue<Event>,
        io: &mut Queue<skein_io::Request>,
    ) {
        let (call, id) = holder;
        let waiting = self.waiting.get_mut(id).expect("call");
        if github.plan.is_none()
            && base.forge.log_origins.is_empty()
            && let Some(api::Op::Read(api::Read::Job { .. })) = waiting.op.as_ref()
        {
            self.finish(call, id, Err(api::Error::Refused), up);
            return;
        }
        let repository = waiting.repository;
        let cost = waiting.cost;
        if github.plan.is_none() && github.discovery.is_none() && github.discovered.is_none() && cost == 0 {
            let missing = repository.repository != 0 && self.github_tables.repository(repository).is_none();
            github.route = None;
            if missing {
                let discovery = Discovery::new(&gh.state);
                if discovery.candidates.is_empty() {
                    self.finish(call, id, Err(api::Error::NotInstalled), up);
                    return;
                }
                github.discovery = Some(discovery);
            }
        }
        if self.waiting.get(id).expect("call").connection.is_none() {
            match self.github_route(id, github, gh, env) {
                Ok(true) => {}
                Ok(false) => return,
                Err(error) => {
                    self.finish(call, id, Err(error), up);
                    return;
                }
            }
        }
        if let Some(answer) = github.discovered.take() {
            self.finish(call, id, Ok(answer), up);
            return;
        }
        if let Some(answer) = self.github_bot_permission(id, github, base, gh) {
            self.finish(call, id, Ok(answer), up);
            return;
        }
        match self.assign(call, id, github.origin(), base, env, io) {
            calls::Assigned::Waits => return,
            calls::Assigned::Held => {}
            calls::Assigned::New => github.exchange = crate::connection::github::Exchange::new(),
        }
        if github.plan.is_none() && !self.begin_github_call(base, gh, (call, id), github, env, up, io) {
            return;
        }
        self.pump_github_call(holder, github, base, gh, env, up, io);
    }
    fn github_bot_permission(
        &mut self,
        id: Id<Waiting>,
        github: &mut GitHubCall,
        base: &mut Base,
        gh: &mut GitHubForge,
    ) -> Option<api::Answer> {
        let waiting = self.waiting.get_mut(id).expect("call");
        let user = match waiting.op.as_ref() {
            Some(api::Op::Read(api::Read::Permission { user })) => *user,
            Some(api::Op::Read(_) | api::Op::Write(_)) | None => return None,
        };
        if github.discovery.is_none() && user == base.forge.access.as_ref()?.user {
            let route = github.route.expect("eligible route");
            let row = gh.state.rows.get(&route.selected.id).expect("eligible row");
            return Some(api::Answer::Permission(api::UserPermission {
                user,
                login: Box::new([]),
                permission: row.permissions.permission(),
            }));
        }
        None
    }
    #[expect(clippy::too_many_arguments, reason = "the call carries its split forge through its checked step")]
    pub(super) fn begin_github_call(
        &mut self,
        base: &mut Base,
        gh: &mut GitHubForge,
        holder: (Token, Id<Waiting>),
        github: &mut GitHubCall,
        env: &Env<Limits>,
        up: &mut Queue<Event>,
        io: &mut Queue<skein_io::Request>,
    ) -> bool {
        let (call, id) = holder;
        match self.initialize_github_plan(id, github, env) {
            Ok(step) => !self.github_call_step(base, gh, (call, id), github, step, env, up, io, None),
            Err(error) => {
                self.finish(call, id, Err(error), up);
                false
            }
        }
    }
    fn initialize_github_plan(
        &mut self,
        id: Id<Waiting>,
        github: &mut GitHubCall,
        env: &Env<Limits>,
    ) -> Result<Step, api::Error> {
        let waiting = self.waiting.get_mut(id).expect("call");
        let op = if github.discovery.is_some() {
            api::Op::Read(api::Read::Settings)
        } else {
            waiting.op.take().expect("operation")
        };
        waiting.page_size =
            calls::page_size(&op, crate::settings::Kind::GitHub, github::pin::PAGE_CAP, &env.limits, &self.shared);
        let mut plan = Plan::new(waiting.repository, op);
        let page_size = waiting.page_size;
        let tables = match &mut github.discovery {
            Some(discovery) => &mut discovery.tables,
            None => &mut self.github_tables,
        };
        let step = plan.begin(bounds(page_size, env, &self.shared), &self.owner.word, tables);
        github.plan = Some(plan);
        step
    }
    #[expect(clippy::too_many_arguments, reason = "the call carries its split forge through its checked step")]
    fn pump_github_call(
        &mut self,
        holder: (Token, Id<Waiting>),
        github: &mut GitHubCall,
        base: &mut Base,
        gh: &mut GitHubForge,
        env: &Env<Limits>,
        up: &mut Queue<Event>,
        io: &mut Queue<skein_io::Request>,
    ) {
        let (call, id) = holder;
        let token = self.waiting.get(id).expect("call").connection.expect("holder");
        let connection = *self.connections.get(&token).expect("connection");
        for _ in 0..64_u32 {
            if self.connection_slab.get(connection).expect("connection").phase() == Phase::Idle {
                match self.github_route(id, github, gh, env) {
                    Ok(true) => {}
                    Ok(false) => {
                        self.release_github_call(id, env, io);
                        break;
                    }
                    Err(error) => {
                        self.release_github_call(id, env, io);
                        self.finish(call, id, Err(error), up);
                        break;
                    }
                }
                if github.plan.is_none() && !self.begin_github_call(base, gh, (call, id), github, env, up, io) {
                    break;
                }
                if let Err(error) = self.send_github_call(id, github, base, gh, env) {
                    self.finish(call, id, Err(error), up);
                    break;
                }
            }

            let c = self.connection_slab.get_mut(connection).expect("connection");
            let waiting = self.waiting.get_mut(id).expect("call");
            github.exchange.pump(c, env, io, &mut self.kept);
            waiting.free = waiting.free.checked_add(c.take_free()).expect("bounded exchanges");
            if c.written && !waiting.charged {
                waiting.cost = waiting.cost.checked_add(1).expect("bounded exchanges");
                waiting.charged = true;
                gh.state.recent = Some(github.route.expect("sent route").selected.id);
            }
            if let Some(heard) = github.exchange.heard(c) {
                let (step, valid) = self.hear_github_call(id, github, base, gh, env, heard);
                match step {
                    Ok(Some(step)) => {
                        if self.github_call_step(base, gh, holder, github, step, env, up, io, Some(valid)) {
                            break;
                        }
                    }
                    Ok(None) => {}
                    Err(error) => {
                        self.connection_slab.get_mut(connection).expect("connection").close(env, true, io);
                        self.finish(call, id, Err(error), up);
                        break;
                    }
                }
                continue;
            }
            if c.phase() == Phase::Closed {
                c.close(env, true, io);
            }
            if !github.exchange.ready(c) {
                break;
            }
            self.wake = Some(env.now);
        }
    }
    fn send_github_call(
        &mut self,
        id: Id<Waiting>,
        github: &mut GitHubCall,
        base: &mut Base,
        gh: &mut GitHubForge,
        env: &Env<Limits>,
    ) -> Result<(), api::Error> {
        let waiting = self.waiting.get_mut(id).expect("call");
        let token = waiting.connection.expect("holder");
        let deadline = waiting.deadline;
        let Some(exchange) = github.plan.as_mut().expect("begun plan").pending() else {
            return Ok(());
        };
        let encoded = github::request::encode(&exchange.request, &crate::limits::documents(&env.limits));
        match encoded {
            Ok(encoded) => match encoded.auth {
                Auth::Installation | Auth::None => {}
                Auth::App => return Err(api::Error::Refused),
            },
            Err(_) => return Err(api::Error::Refused),
        }
        let route = github.route.as_mut().expect("eligible route");
        let row = gh.state.rows.get(&route.selected.id).expect("eligible row");
        let authorization = if route.origin.is_some() {
            None
        } else {
            Some(github::credential::bearer(row.credential(env.now).expect("usable Full")).expect("validated token"))
        };
        route.token_revision = row.token_revision;
        route.status = None;
        let c = self.connection_slab.get_mut(*self.connections.get(&token).expect("connection")).expect("connection");
        c.identity = Some(route.selected.id);
        c.until(Some(deadline));
        match github.exchange.start(
            c,
            env,
            &base.forge,
            &self.owner.user_agent,
            exchange,
            authorization,
            &mut self.kept,
        ) {
            Ok(()) => {}
            Err(_) => return Err(api::Error::Refused),
        }
        waiting.charged = false;
        Ok(())
    }
    fn hear_github_call(
        &mut self,
        id: Id<Waiting>,
        github: &mut GitHubCall,
        base: &mut Base,
        gh: &mut GitHubForge,
        env: &Env<Limits>,
        heard: Heard,
    ) -> (Result<Option<Step>, api::Error>, bool) {
        let waiting = self.waiting.get_mut(id).expect("call");
        let refused = match &heard {
            Heard::Head { status, .. } => *status == 401,
            Heard::Done(_) | Heard::Row(_) | Heard::Failed(_) => false,
        };
        let status = match &heard {
            Heard::Head { status, head } => {
                waiting.answered_head(*status, github::header::rate(head));
                Some(*status)
            }
            Heard::Done(Body::Malformed(_)) => {
                self.diagnostics.malformed = self.diagnostics.malformed.saturating_add(1);
                None
            }
            Heard::Done(Body::Document(_) | Body::Raw { .. } | Body::Empty) | Heard::Row(_) | Heard::Failed(_) => None,
        };
        let page_size = waiting.page_size;
        let revoked = waiting.revoked;
        if let Some(status) = status {
            github.route.as_mut().expect("sent route").status = Some(status);
        }
        let route = github.route.expect("sent route");
        let valid = !revoked && gh.state.valid(route.selected);
        let mut discarded = Tables::new(0, 0);
        let tables = if valid {
            match &mut github.discovery {
                Some(discovery) => &mut discovery.tables,
                None => &mut self.github_tables,
            }
        } else {
            &mut discarded
        };
        let plan = github.plan.as_mut().expect("begun plan");
        let step = plan.heard(heard, bounds(page_size, env, &self.shared), &self.owner.word, tables);
        let step =
            if refused && route.origin.is_none() { Ok(Some(Step::Done(Err(api::Error::Forbidden)))) } else { step };
        if refused && route.origin.is_none() && valid {
            self.github_token_refused(base, gh, route.selected.id, route.token_revision, env);
        }
        (step, valid)
    }
    pub(super) fn release_github_call(
        &mut self,
        id: Id<Waiting>,
        env: &Env<Limits>,
        io: &mut Queue<skein_io::Request>,
    ) {
        let waiting = self.waiting.get_mut(id).expect("call");
        if let Some(token) = waiting.connection.take() {
            let c =
                self.connection_slab.get_mut(*self.connections.get(&token).expect("connection")).expect("connection");
            if c.phase() == Phase::Idle && !c.in_flight() {
                c.until(Some(env.now.saturating_add(env.limits.idle)));
            } else if !c.draining() {
                c.close(env, c.in_flight(), io);
            }
            let _holder = self.holders.remove(&token);
        }
    }
    #[expect(clippy::too_many_arguments, reason = "the call carries its split forge through its checked step")]
    fn github_call_step(
        &mut self,
        base: &mut Base,
        gh: &mut GitHubForge,
        holder: (Token, Id<Waiting>),
        github: &mut GitHubCall,
        step: Step,
        env: &Env<Limits>,
        up: &mut Queue<Event>,
        io: &mut Queue<skein_io::Request>,
        checked: Option<bool>,
    ) -> bool {
        let (call, id) = holder;
        match step {
            Step::Send(exchange) => {
                github.plan.as_mut().expect("begun plan").set_pending(exchange);
                let token = self.waiting.get(id).expect("call").connection.expect("holder");
                let connection = *self.connections.get(&token).expect("connection");
                let c = self.connection_slab.get_mut(connection).expect("connection");
                if c.phase() == Phase::Closed || (checked.is_some() && c.in_flight()) {
                    self.release_github_call(id, env, io);
                    self.wake = Some(env.now);
                    return true;
                }
                false
            }
            Step::Redirect { redirect, retention } => {
                self.follow_github_log(base, gh, holder, github, redirect, retention, env, up, io);
                true
            }
            Step::Done(result) => {
                if github.discovery.is_some() {
                    self.github_discovered(
                        base,
                        gh,
                        holder,
                        github,
                        result,
                        env,
                        up,
                        io,
                        checked.expect("discovery response"),
                    );
                    return true;
                }
                self.release_github_call(id, env, io);
                self.finish(call, id, result, up);
                true
            }
        }
    }
    #[expect(clippy::too_many_arguments, reason = "the call carries its split forge through its checked step")]
    fn github_discovered(
        &mut self,
        base: &mut Base,
        gh: &mut GitHubForge,
        holder: (Token, Id<Waiting>),
        github: &mut GitHubCall,
        result: Result<api::Answer, api::Error>,
        env: &Env<Limits>,
        up: &mut Queue<Event>,
        io: &mut Queue<skein_io::Request>,
        valid: bool,
    ) {
        let (call, id) = holder;
        let result = if valid { result } else { Err(api::Error::Forbidden) };
        match result {
            Ok(answer) => {
                let waiting = self.waiting.get_mut(id).expect("call");
                let repository = waiting.repository;
                let mut staged = github.discovery.take().expect("discovery table").tables;
                let row = staged.repository(repository).expect("validated Settings row").clone();
                self.github_tables.keep_user(repository.forge, row.owner_id, row.owner.clone());
                self.github_tables.keep_repository(repository, row);
                github.route = None;
                let usable = match self.github_route(id, github, gh, env) {
                    Ok(usable) => usable,
                    Err(error) => {
                        self.release_github_call(id, env, io);
                        self.finish(call, id, Err(error), up);
                        return;
                    }
                };
                let waiting = self.waiting.get_mut(id).expect("call");
                github.plan = None;
                if waiting.op.as_ref() == Some(&api::Op::Read(api::Read::Settings)) {
                    self.release_github_call(id, env, io);
                    if usable {
                        self.finish(call, id, Ok(answer), up);
                    } else {
                        github.discovered = Some(answer);
                    }
                    return;
                }
                if let Some(answer) = self.github_bot_permission(id, github, base, gh) {
                    self.release_github_call(id, env, io);
                    if usable {
                        self.finish(call, id, Ok(answer), up);
                    } else {
                        self.wake = Some(env.now);
                    }
                    return;
                }
                let step = match self.initialize_github_plan(id, github, env) {
                    Ok(step) => step,
                    Err(error) => {
                        self.release_github_call(id, env, io);
                        self.finish(call, id, Err(error), up);
                        return;
                    }
                };
                if self.github_call_step(base, gh, holder, github, step, env, up, io, None) {
                    return;
                }
                self.release_github_call(id, env, io);
                self.wake = Some(env.now);
            }
            Err(error) => {
                if valid && error == api::Error::Missing && github.route.expect("probe route").status == Some(404) {
                    let discovery = github.discovery.as_mut().expect("discovery");
                    discovery.advance();
                    discovery.tables = Tables::new(1, 1);
                    github.plan = None;
                    github.route = None;
                    self.release_github_call(id, env, io);
                    self.wake = Some(env.now);
                    return;
                }
                self.release_github_call(id, env, io);
                self.finish(call, id, Err(error), up);
            }
        }
    }
}
fn bounds(page_size: u32, env: &Env<Limits>, shared: &crate::Shared) -> Bounds {
    Bounds {
        inner: env.limits.inner,
        page_size,
        rows: shared.rows,
        answer_bytes: shared.answer_bytes,
        body_bytes: env.limits.body_bytes,
    }
}
