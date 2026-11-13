//! Waiting calls and returned answers (skein/docs/design/forge/connection.md, sections 2, 7.1 and 8).
use crate::{
    boundary::{Answered, Event, Request},
    connection::{self, Connection, Phase, Use},
    limits::{self, Limits, Shared, Unusable},
    settings::{Forge, Owner, Settings},
    start,
    start::Start,
};
use alloc::boxed::Box;
use skein_lib::{Env, Id, Lifecycle, List, Map, Queue, Slab, Time, Token, earliest};
mod calls;
pub(crate) mod entry;
mod seam;
use entry::{Base, Dialect, ForgeEntry, GitHubForge};
mod github;
mod github_calls;
mod github_logs;
mod puller;
mod runtime;
use skein_forge::api;
use skein_forge::forgejo::plans::{
    Body, Heard,
    plan::{Plan, Step},
    start::{Heard as StartHeard, Plan as StartPlan, Read},
    tables::Tables,
};
/// Malformed answers and long bodies counted by the component.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Diagnostics {
    pub long_bodies: u64,
    pub malformed: u64,
}
/// Out of its `Waiting` while `drive_calls` drives the call.
#[derive(Debug)]
pub(crate) enum Side {
    Unbegun,
    Forgejo(Box<ForgejoCall>),
    GitHub(Box<GitHubCall>),
}
#[derive(Debug)]
pub(crate) struct ForgejoCall {
    pub exchange: connection::forgejo::Exchange,
    pub plan: Plan,
}
#[derive(Debug)]
pub(crate) struct GitHubCall {
    pub exchange: connection::github::Exchange,
    pub plan: Option<skein_forge::github::plans::plan::Plan>,
    pub route: Option<github_calls::Route>,
    pub discovery: Option<github_calls::Discovery>,
    pub discovered: Option<api::Answer>,
}
impl GitHubCall {
    fn new() -> Self {
        Self {
            exchange: connection::github::Exchange::new(),
            plan: None,
            route: None,
            discovery: None,
            discovered: None,
        }
    }
    fn origin(&self) -> Option<u32> {
        self.route?.origin
    }
}
#[derive(Debug)]
#[expect(clippy::enum_variant_names, reason = "the GitHub own plans retain their shared names")]
pub(crate) enum OwnPlan {
    GitHubInstallations(skein_forge::github::plans::start::Installations),
    GitHubMinting(skein_forge::github::plans::start::Minting),
    GitHubIdentity(skein_forge::github::plans::start::Identity),
    GitHubLogPage(Box<skein_forge::github::plans::start::LogPage>),
    GitHubLogDelivery(Box<skein_forge::github::plans::start::LogDelivery>),
}
#[derive(Debug)]
pub(crate) struct Waiting {
    pub repository: api::Repository,
    pub op: Option<api::Op>,
    pub side: Side,
    pub page_size: u32,
    pub connection: Option<Token>,
    pub cost: u32,
    pub free: u32,
    pub rate: Option<api::Rate>,
    pub charged: bool,
    pub revoked: bool,
    pub deadline: Time,
}
impl Waiting {
    fn new(repository: api::Repository, op: api::Op, deadline: Time) -> Self {
        Self {
            repository,
            op: Some(op),
            side: Side::Unbegun,
            page_size: 1,
            connection: None,
            cost: 0,
            free: 0,
            rate: None,
            charged: false,
            revoked: false,
            deadline,
        }
    }
    fn begun(&self) -> bool {
        match &self.side {
            Side::Unbegun => false,
            Side::Forgejo(_) => true,
            Side::GitHub(call) => call.plan.is_some(),
        }
    }
    fn error(&self, error: api::Error) -> api::Error {
        match &self.side {
            Side::Forgejo(call) => call.plan.error(error),
            Side::Unbegun | Side::GitHub(_) => error,
        }
    }
    fn answered_head(&mut self, status: u16, rate: Option<api::Rate>) {
        if status == 304 {
            self.free = self.free.checked_add(1).expect("bounded exchanges");
        }
        if let Some(rate) = rate {
            self.rate = Some(rate);
        }
    }
}
#[derive(Debug)]
pub(crate) struct Held {
    pub answer: Answered,
    pub offer: bool,
}
/// One component per deployment, owning every configured token and start.
#[derive(Debug)]
pub struct Component {
    forges: List<ForgeEntry>,
    common: Common,
}

/// What the configured forges share (skein/docs/design/forge/connection.md, 7.1).
#[derive(Debug)]
pub(crate) struct Common {
    kept: crate::kept::Store,
    hooks: crate::hooks::Hooks,
    owner: Owner,
    forgejo_tables: Tables,
    github_tables: skein_forge::github::plans::tables::Tables,
    waiting: Slab<Waiting>,
    calls: Map<Token, Id<Waiting>>,
    held: Map<Token, Held>,
    holders: Map<Token, Token>,
    lifecycle: Lifecycle,
    diagnostics: Diagnostics,
    shared: Shared,
    connections: Map<Token, Id<Connection>>,
    connection_slab: Slab<Connection>,
    next_token: u64,
    wake: Option<Time>,
}
impl Component {
    pub fn new(settings: Settings, limits: &Limits, shared: &Shared) -> Result<Self, Unusable> {
        limits::check(limits, shared, &settings)?;
        let mut forges = List::with_capacity(limits.forges);
        for forge in settings.forges.into_boxed() {
            forges.push(ForgeEntry::new(forge, limits)).expect("checked forge count");
        }
        let owner = Owner { webhooks: settings.webhooks, word: settings.word, user_agent: settings.user_agent };
        Ok(Self {
            forges,
            common: Common {
                kept: crate::kept::Store::new(limits.kept_bytes),
                hooks: crate::hooks::Hooks::new(limits),
                owner,
                forgejo_tables: Tables::new(
                    if limits.forges > 0 { limits.repositories } else { 0 },
                    if limits.forges > 0 { limits.users } else { 0 },
                ),
                github_tables: skein_forge::github::plans::tables::Tables::new(
                    if limits.forges > 0 { limits.repositories } else { 0 },
                    if limits.forges > 0 { limits.users } else { 0 },
                ),
                waiting: Slab::with_capacity(shared.calls),
                calls: Map::with_capacity(shared.calls),
                held: Map::with_capacity(shared.calls),
                holders: Map::with_capacity(shared.calls),
                lifecycle: Lifecycle::Live,
                diagnostics: Diagnostics { long_bodies: 0, malformed: 0 },
                shared: *shared,
                connections: Map::with_capacity(
                    limits.connections.checked_add(limits.forges).expect("checked connections"),
                ),
                connection_slab: Slab::with_capacity(
                    limits.connections.checked_add(limits.forges).expect("checked connections"),
                ),
                next_token: 1,
                wake: None,
            },
        })
    }
    /// Opens each forge's start within its connection slots.
    pub fn begin(&mut self, env: &Env<Limits>, up: &mut Queue<Event>, io: &mut Queue<skein_io::Request>) {
        let Self { forges, common } = self;
        if common.lifecycle != Lifecycle::Live {
            return;
        }
        if let Some(settings) = &common.owner.webhooks {
            common.hooks.begin(settings, io);
        }
        for entry in forges.iter_mut() {
            if entry.base.forge.access.is_some() {
                entry.base.start = initial(&entry.dialect, env.limits.start_backoff);
            }
        }
        self.drive(env, up, io);
    }
    #[expect(clippy::too_many_lines, reason = "the entrance keeps lifecycle and request dispatch together")]
    pub fn down(
        &mut self,
        env: &Env<Limits>,
        request: Request,
        up: &mut Queue<Event>,
        io: &mut Queue<skein_io::Request>,
    ) {
        let Self { common, .. } = self;
        match common.lifecycle {
            Lifecycle::Live => {}
            Lifecycle::Closing | Lifecycle::Aborting => {
                match request {
                    Request::Call { call, .. } => up.push(Event::Answered(Answered {
                        call,
                        cost: 0,
                        free: 0,
                        rate: None,
                        result: Err(api::Error::Unavailable),
                    })),
                    Request::Abort => {
                        if common.lifecycle == Lifecycle::Closing {
                            self.abort(env, up, io);
                        }
                    }
                    Request::Add(_)
                    | Request::Remove { .. }
                    | Request::Connect { .. }
                    | Request::Installations { .. }
                    | Request::RefusedHint(_)
                    | Request::Refused(_)
                    | Request::Returned(_)
                    | Request::Offer
                    | Request::Close => {}
                }
                return;
            }
            Lifecycle::Closed => return,
        }
        match request {
            Request::Add(forge) => {
                self.add(env, forge);
                self.drive(env, up, io);
            }
            Request::Remove { forge } => {
                self.remove(env, forge, up, io);
                self.drive(env, up, io);
            }
            Request::Connect { forge, access } => {
                self.connect(env, forge, access, up, io);
                self.drive(env, up, io);
            }
            Request::Installations { forge, accepted } => {
                self.installations(forge, &accepted);
                self.drive(env, up, io);
            }
            Request::Call { call, repository, op } => {
                assert!(!common.calls.contains_key(&call), "one live call per token");
                let id = common
                    .waiting
                    .insert(Waiting::new(repository, op, env.now.saturating_add(env.limits.deadline)))
                    .expect("client bounds admitted calls");
                let _previous: Option<Id<Waiting>> =
                    common.calls.insert(call, id).expect("call index matches its slab");
                self.drive(env, up, io);
            }
            Request::RefusedHint(hint) => self.refused_hint(hint),
            Request::Refused(answer) => {
                drop(
                    common
                        .held
                        .insert(answer.call, Held { answer, offer: true })
                        .expect("at most one held answer per client call"),
                );
            }
            Request::Returned(answer) => {
                drop(
                    common
                        .held
                        .insert(answer.call, Held { answer, offer: false })
                        .expect("at most one held answer per client call"),
                );
            }
            Request::Offer => {
                common.hooks.offer();
                self.offer_pullers();
                let Self { forges, common } = self;
                if common.owner.webhooks.is_some() {
                    common.hooks.drive(env, forges, &common.owner.word, up, io);
                }
                for _ in 0..common.held.len() {
                    let mut next = None;
                    for (call, held) in &common.held {
                        if held.offer {
                            next = Some(*call);
                            break;
                        }
                    }
                    let Some(call) = next else {
                        break;
                    };
                    if let Some(held) = common.held.remove(&call) {
                        up.push(Event::Answered(held.answer));
                    }
                }
                self.drive(env, up, io);
            }
            Request::Close => {
                self.shed();
                self.common.lifecycle = Lifecycle::Closing;
                self.common.hooks.close();
                self.drive(env, up, io);
            }
            Request::Abort => {
                for _ in 0..common.calls.len() {
                    let Some((call, id)) = common.calls.first() else {
                        break;
                    };
                    let (call, id) = (*call, *id);
                    common.expire(call, id, env, up, io);
                }
                self.shed();
                self.abort(env, up, io);
            }
        }
    }
    /// Ends each open call with no answer and drops what the component holds for the
    /// owner and each forge (forge/connection.md, 8.5).
    fn shed(&mut self) {
        let Self { forges, common } = self;
        for _ in 0..common.calls.len() {
            let Some((call, id)) = common.calls.first() else {
                break;
            };
            let (call, id) = (*call, *id);
            common.waiting.retire(id);
            let _removed: Option<Id<Waiting>> = common.calls.remove(&call);
        }
        common.held = Map::with_capacity(common.held.capacity());
        common.holders = Map::with_capacity(common.holders.capacity());
        for entry in forges.iter_mut() {
            entry.base.forge.access = None;
            entry.dialect.forget_exchange();
            match &mut entry.dialect {
                Dialect::Forgejo { plan, .. } => *plan = None,
                Dialect::GitHub(github) => {
                    github.state.forget();
                    github.plan = None;
                }
            }
        }
    }
    /// A close that does not wait (programming-model.md, 5.2), once the calls are ended
    /// and the rest shed: every connection and the webhooks are aborted, a graceful
    /// close already sent among them.
    fn abort(&mut self, env: &Env<Limits>, up: &mut Queue<Event>, io: &mut Queue<skein_io::Request>) {
        self.common.lifecycle = Lifecycle::Aborting;
        self.common.hooks.abort(io);
        self.drive(env, up, io);
    }
    fn offer_pullers(&mut self) {
        for entry in self.forges.iter_mut() {
            match &mut entry.dialect {
                Dialect::Forgejo { .. } => {}
                Dialect::GitHub(gh) => gh.state.puller.offer(),
            }
        }
    }
    fn refused_hint(&mut self, hint: crate::hooks::Hinted) {
        for ForgeEntry { base, dialect } in self.forges.iter_mut() {
            if base.forge.number == hint.hint.repository.forge {
                match dialect {
                    Dialect::Forgejo { .. } => {}
                    Dialect::GitHub(gh) => {
                        if gh.state.puller.source == Some(hint.connection) {
                            gh.state.puller.refused(hint.hint);
                            return;
                        }
                    }
                }
            }
        }
        self.common.hooks.refused(hint);
    }
    fn installations(&mut self, forge: u16, accepted: &[u64]) {
        if let Some(ForgeEntry { dialect, .. }) = find(&mut self.forges, forge) {
            match dialect {
                Dialect::Forgejo { .. } => {}
                Dialect::GitHub(gh) => gh.state.accept(accepted),
            }
        }
    }
    fn connect(
        &mut self,
        env: &Env<Limits>,
        number: u16,
        access: crate::Access,
        up: &mut Queue<Event>,
        io: &mut Queue<skein_io::Request>,
    ) {
        let Self { forges, common } = self;
        let Some(ForgeEntry { base, dialect }) = find(forges, number) else {
            return;
        };
        let forge = &mut base.forge;
        let mut checked = limits::access(&env.limits, forge.kind, common.owner.webhooks.is_some(), &access);
        if checked.is_ok()
            && let Some(held) = &forge.access
            && held.user != access.user
        {
            checked = Err(crate::Value::User);
        }
        for (_, id) in &common.calls {
            let call = common.waiting.get_mut(*id).expect("indexed call");
            if call.repository.forge == number && (call.begun() || call.cost > 0) {
                call.revoked = true;
            }
        }
        if let Some(token) = base.own.take()
            && let Some(id) = common.connections.get(&token)
        {
            let c = common.connection_slab.get_mut(*id).expect("start connection");
            c.role = Use::Calls;
            c.close(env, c.in_flight(), io);
        }
        common.forget_kept(number);
        dialect.forget_exchange();
        match &mut *dialect {
            Dialect::Forgejo { plan, .. } => *plan = None,
            Dialect::GitHub(github) => {
                github.state.forget();
                github.plan = None;
            }
        }
        match checked {
            Ok(()) => {
                forge.access = Some(access);
                base.start = initial(dialect, env.limits.start_backoff);
            }
            Err(value) => {
                forge.access = None;
                let why = crate::Stopped::Invalid { value };
                base.start = Start::Stopped(why);
                up.push(Event::Stopped { forge: number, why });
            }
        }
    }
    /// Routes io by this component's connection owner; stale owners are inert.
    pub fn up(
        &mut self,
        env: &Env<Limits>,
        event: skein_io::Event,
        up: &mut Queue<Event>,
        io: &mut Queue<skein_io::Request>,
    ) {
        let Self { common, .. } = self;
        let owner = match &event {
            skein_io::Event::Listening { owner, .. }
            | skein_io::Event::Accepted { owner, .. }
            | skein_io::Event::Stream { owner, .. }
            | skein_io::Event::Failed { owner, .. }
            | skein_io::Event::Closed { owner } => Some(*owner),
            skein_io::Event::Connecting { .. }
            | skein_io::Event::Connected { .. }
            | skein_io::Event::Output { .. }
            | skein_io::Event::Spawned { .. }
            | skein_io::Event::Exited { .. }
            | skein_io::Event::Usage { .. }
            | skein_io::Event::Resized
            | skein_io::Event::Window { .. }
            | skein_io::Event::Shutdown { .. } => None,
        };
        if let Some(owner) = owner
            && common.hooks.owns(owner)
        {
            common.hooks.up(env, event, up);
            self.drive(env, up, io);
            return;
        }
        match event {
            skein_io::Event::Connecting { owner, socket } => {
                if let Some(id) = common.connections.get(&owner)
                    && let Some(c) = common.connection_slab.get_mut(*id)
                {
                    c.link.connecting(socket, io);
                }
            }
            skein_io::Event::Connected { owner } => {
                if let Some(id) = common.connections.get(&owner)
                    && let Some(c) = common.connection_slab.get_mut(*id)
                {
                    c.connect(env);
                }
            }
            skein_io::Event::Stream { owner, up } => {
                if let Some(id) = common.connections.get(&owner)
                    && let Some(c) = common.connection_slab.get_mut(*id)
                {
                    c.stream(env, up);
                }
            }
            skein_io::Event::Failed { owner, .. } => {
                if let Some(id) = common.connections.get(&owner)
                    && let Some(c) = common.connection_slab.get_mut(*id)
                {
                    c.failed(connection::Failure::Unseen);
                }
            }
            skein_io::Event::Closed { owner } => {
                if let Some(id) = common.connections.remove(&owner) {
                    let c = common.connection_slab.get_mut(id).expect("indexed connection");
                    let closing = c.closing();
                    if let Some(call) = common.holders.remove(&owner)
                        && common.lifecycle == Lifecycle::Live
                    {
                        let call_id = *common.calls.get(&call).expect("holding call");
                        let c = common.connection_slab.get(id).expect("closed connection");
                        let error = if c.written { api::Error::Timeout } else { api::Error::Unavailable };
                        common.finish(call, call_id, Err(error), up);
                    }
                    common.connection_slab.retire(id);
                    self.own_closed(owner, closing, env, up);
                }
            }
            skein_io::Event::Listening { .. }
            | skein_io::Event::Accepted { .. }
            | skein_io::Event::Output { .. }
            | skein_io::Event::Spawned { .. }
            | skein_io::Event::Exited { .. }
            | skein_io::Event::Usage { .. }
            | skein_io::Event::Resized
            | skein_io::Event::Window { .. }
            | skein_io::Event::Shutdown { .. } => {}
        }
        self.drive(env, up, io);
    }
    fn own_closed(&mut self, owner: Token, closing: bool, env: &Env<Limits>, up: &mut Queue<Event>) {
        let Self { forges, common } = self;
        for ForgeEntry { base, dialect } in forges.iter_mut() {
            if base.own != Some(owner) {
                continue;
            }
            base.own = None;
            dialect.forget_exchange();
            let retry = common.lifecycle == Lifecycle::Live && !closing;
            match dialect {
                Dialect::Forgejo { plan, .. } => {
                    *plan = None;
                    if retry {
                        common.start_heard(base, StartHeard::Unanswered, env, up);
                    }
                }
                Dialect::GitHub(gh) => {
                    if retry {
                        Common::github_wait(base, gh, env);
                    }
                }
            }
        }
    }
    pub fn fire(&mut self, env: &Env<Limits>, up: &mut Queue<Event>, io: &mut Queue<skein_io::Request>) {
        let Self { common, .. } = self;
        for (_, id) in &common.connections {
            common.connection_slab.get_mut(*id).expect("indexed connection").link.fire(env.now, io);
        }
        if common.lifecycle != Lifecycle::Live {
            self.drive(env, up, io);
            return;
        }
        for _ in 0..common.calls.len() {
            let mut due = None;
            for (call, id) in &common.calls {
                if common.waiting.get(*id).expect("live call").deadline <= env.now {
                    due = Some((*call, *id));
                    break;
                }
            }
            let Some((call, id)) = due else {
                break;
            };
            common.expire(call, id, env, up, io);
        }
        self.drive(env, up, io);
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        let mut next = earliest(self.common.wake, self.common.hooks.deadline());
        for entry in &self.forges {
            match &entry.dialect {
                Dialect::Forgejo { .. } => {}
                Dialect::GitHub(gh) => {
                    next = earliest(next, gh.state.puller.due);
                    next = earliest(next, gh.state.deadline());
                }
            }
        }
        for (_, id) in &self.common.connections {
            let c = self.common.connection_slab.get(*id).expect("indexed connection");
            next = earliest(next, c.deadline());
            next = earliest(next, c.link.next_deadline());
        }
        if self.common.lifecycle == Lifecycle::Live {
            for entry in &self.forges {
                if let Some((until, _)) = backoff(entry.base.start) {
                    next = earliest(next, Some(until));
                }
            }
        }
        for (_, id) in &self.common.calls {
            next = earliest(next, Some(self.common.waiting.get(*id).expect("indexed live call").deadline));
        }
        next
    }

    fn drive_own(&mut self, env: &Env<Limits>, up: &mut Queue<Event>, io: &mut Queue<skein_io::Request>) {
        self.common.wake = None;
        let Self { forges, common } = self;
        for ForgeEntry { base, dialect } in forges.iter_mut() {
            match dialect {
                Dialect::Forgejo { plan, exchange } => common.drive_forgejo(base, plan, exchange, env, up, io),
                Dialect::GitHub(gh) => common.drive_github(base, gh, env, up, io),
            }
        }
    }

    fn drive(&mut self, env: &Env<Limits>, up: &mut Queue<Event>, io: &mut Queue<skein_io::Request>) {
        let Self { forges, common } = self;
        common.kept.reset_active();
        for (_, id) in &common.connections {
            let c = common.connection_slab.get(*id).expect("indexed connection");
            if let Some(pin) = c.kept_pin() {
                common.kept.repin(pin);
            }
            common.kept.held_room(c.kept_room());
        }
        common.kept.prune();
        for entry in forges.iter_mut() {
            if stopped(entry.base.start).is_some() || common.lifecycle != Lifecycle::Live {
                common.forget_kept(entry.base.forge.number);
            }
        }
        // Each pass includes removed connections whose socket io names later.
        for (_, id) in &common.connections {
            let c = common.connection_slab.get_mut(*id).expect("indexed connection");
            if common.lifecycle == Lifecycle::Aborting || c.membership == connection::Membership::Removed {
                c.abort(env, io);
            }
        }
        if common.owner.webhooks.is_some() {
            common.hooks.drive(env, forges, &common.owner.word, up, io);
        }
        self.drive_own(env, up, io);
        self.drive_calls(env, up, io);
        let common = &mut self.common;
        match common.lifecycle {
            Lifecycle::Closing | Lifecycle::Aborting => {
                if common.connections.is_empty() && common.hooks.closed() {
                    common.lifecycle = Lifecycle::Closed;
                    up.push(Event::Closed);
                }
            }
            Lifecycle::Live | Lifecycle::Closed => {}
        }
    }

    #[must_use]
    pub fn diagnostics(&self) -> Diagnostics {
        self.common.diagnostics
    }
    pub fn reclaim(&mut self) {
        self.common.waiting.reclaim();
        self.common.connection_slab.reclaim();
    }
    /// The bounded resolution tables owned by this component.
    pub fn forgejo_tables(&mut self) -> &mut Tables {
        &mut self.common.forgejo_tables
    }
    /// GitHub repository owners and user logins, shared by its configured forges.
    pub fn github_tables(&mut self) -> &mut skein_forge::github::plans::tables::Tables {
        &mut self.common.github_tables
    }
    /// The pure start, in configured forge order.
    #[must_use]
    pub fn start(&self, forge: u16) -> Option<Start> {
        for entry in &self.forges {
            if entry.base.forge.number == forge {
                return Some(entry.base.start);
            }
        }
        None
    }
}

impl Common {
    fn forget_kept(&mut self, forge: u16) {
        for (_, id) in &self.connections {
            let c = self.connection_slab.get_mut(*id).expect("indexed connection");
            if c.forge == forge {
                c.forget_kept();
            }
        }
        self.kept.forget(forge);
    }
    fn start_heard(&mut self, base: &mut Base, heard: StartHeard, env: &Env<Limits>, up: &mut Queue<Event>) {
        let previous = base.start;
        base.start =
            start::heard(previous, heard, base.forge.access.as_ref().expect("held access").user, env.now, &env.limits);
        if start_read(base.start) == Some(Read::ApiSettings)
            && base.start != previous
            && let Some(token) = base.own
            && let Some(id) = self.connections.get(&token)
            && let Some(c) = self.connection_slab.get_mut(*id)
        {
            c.until(Some(env.now.saturating_add(env.limits.deadline)));
        }
        if let Some(reason) = stopped(base.start)
            && base.start != previous
        {
            base.forge.access = None;
            up.push(Event::Stopped { forge: base.forge.number, why: reason });
        } else if ready(base.start) && base.start != previous {
            up.push(Event::Connected { forge: base.forge.number });
        }
    }
    fn drive_forgejo(
        &mut self,
        base: &mut Base,
        plan: &mut Option<StartPlan>,
        exchange: &mut connection::forgejo::Exchange,
        env: &Env<Limits>,
        up: &mut Queue<Event>,
        io: &mut Queue<skein_io::Request>,
    ) {
        if let Some((until, next)) = backoff(base.start)
            && until <= env.now
            && self.lifecycle == Lifecycle::Live
        {
            base.start = Start::Checking { next };
        }
        let mut token = base.own;
        if self.lifecycle == Lifecycle::Live
            && token.is_none()
            && start_read(base.start).is_some()
            && self.call_connections() < env.limits.connections
            && self.connections.len() < self.connections.capacity()
        {
            token = Some(self.open_own(base, Use::Own, env, io));
            *plan = Some(StartPlan::new(start_read(base.start).expect("active start read")));
        }
        let Some(token) = token else {
            return;
        };
        let id = *self.connections.get(&token).expect("indexed token");
        for _ in 0..64_u32 {
            let c = self.connection_slab.get_mut(id).expect("indexed connection");
            if self.lifecycle != Lifecycle::Live {
                *plan = None;
                c.close(env, c.in_flight(), io);
                break;
            }
            let state = base.start;
            if let Some(deadline) = c.deadline()
                && deadline <= env.now
            {
                if c.phase() == Phase::Idle && ready(state) {
                    c.close(env, false, io);
                    break;
                }
                c.timeout();
            }
            if c.phase() == Phase::Idle
                && let Some(read) = start_read(state)
            {
                if plan.is_none() {
                    *plan = Some(StartPlan::new(read));
                }
                let plan = plan.as_mut().expect("active start read");
                send_start(c, exchange, env, &base.forge, &self.owner.user_agent, plan, &mut self.kept);
            }
            exchange.pump(c, env, io, &mut self.kept);
            let _free = c.take_free();
            if let Some(heard) = exchange.heard(c) {
                let step = match plan.as_mut() {
                    Some(plan) => plan.heard(heard),
                    None => None,
                };
                if let Some(skein_forge::forgejo::plans::start::Step::Done(heard)) = step {
                    *plan = None;
                    self.start_heard(base, heard, env, up);
                }
                let c = self.connection_slab.get_mut(id).expect("indexed connection");
                if backoff(base.start).is_some() || stopped(base.start).is_some() {
                    c.close(env, true, io);
                }
                continue;
            }
            if c.phase() == Phase::Closed {
                c.close(env, c.in_flight(), io);
            }
            if !exchange.ready(c) {
                break;
            }
            self.wake = Some(env.now);
        }
    }
    fn call_connections(&self) -> u32 {
        let mut count = 0_u32;
        for (_, id) in &self.connections {
            let c = self.connection_slab.get(*id).expect("indexed connection");
            if c.membership == connection::Membership::Removed {
                continue;
            }
            if c.role != Use::Reserved {
                count = count.checked_add(1).expect("bounded connections");
            }
        }
        count
    }

    fn open_call_origin(
        &mut self,
        forge: &Forge,
        origin: Option<u32>,
        env: &Env<Limits>,
        io: &mut Queue<skein_io::Request>,
    ) -> Token {
        assert!(self.next_token < 1_u64.checked_shl(59).expect("token prefix"), "connection token prefix");
        let owner = Token::new(self.next_token);
        self.next_token = self.next_token.checked_add(1).expect("connection token space");
        self.connection_slab.reclaim();
        let deadline = env.now.saturating_add(env.limits.deadline);
        let connection = Connection::at_origin(forge, origin, &env.limits, self.shared, deadline);
        let id = self.connection_slab.insert(connection).expect("free connection slot");
        let _previous = self.connections.insert(owner, id).expect("connection index has room");
        let address = match origin {
            Some(origin) => forge.log_origins.get(origin).expect("origin").address,
            None => forge.address,
        };
        io.push(skein_io::Request::Connect { owner, addr: address });
        owner
    }
    fn open_own(&mut self, base: &mut Base, role: Use, env: &Env<Limits>, io: &mut Queue<skein_io::Request>) -> Token {
        let owner = self.open_call_origin(&base.forge, None, env, io);
        base.own = Some(owner);
        self.connection_slab.get_mut(*self.connections.get(&owner).expect("own token")).expect("own connection").role =
            role;
        owner
    }
}

fn send_start(
    connection: &mut Connection,
    exchange: &mut connection::forgejo::Exchange,
    env: &Env<Limits>,
    forge: &Forge,
    user_agent: &[u8],
    plan: &mut StartPlan,
    kept: &mut crate::kept::Store,
) {
    let skein_forge::forgejo::plans::start::Step::Send(pending) = plan.begin() else {
        unreachable!("a new start plan sends its read");
    };
    let authorization =
        skein_forge::forgejo::credential::authorization(&forge.access.as_ref().expect("held access").credential)
            .expect("checked credential length");
    exchange.start(connection, env, forge, user_agent, pending, authorization, kept).expect("checked start request");
}

fn start_read(state: Start) -> Option<Read> {
    match state {
        Start::Checking { .. } => Some(Read::User),
        Start::Settings { .. } => Some(Read::ApiSettings),
        Start::Listing { .. }
        | Start::Minting { .. }
        | Start::Identity { .. }
        | Start::Unconnected
        | Start::Waiting { .. }
        | Start::Ready { .. }
        | Start::Stopped(_) => None,
    }
}
fn backoff(state: Start) -> Option<(Time, skein_lib::Duration)> {
    match state {
        Start::Waiting { until, next } => Some((until, next)),
        Start::Listing { .. }
        | Start::Minting { .. }
        | Start::Identity { .. }
        | Start::Unconnected
        | Start::Checking { .. }
        | Start::Settings { .. }
        | Start::Ready { .. }
        | Start::Stopped(_) => None,
    }
}
fn stopped(state: Start) -> Option<crate::Stopped> {
    match state {
        Start::Stopped(reason) => Some(reason),
        Start::Listing { .. }
        | Start::Minting { .. }
        | Start::Identity { .. }
        | Start::Unconnected
        | Start::Checking { .. }
        | Start::Settings { .. }
        | Start::Waiting { .. }
        | Start::Ready { .. } => None,
    }
}
fn ready(state: Start) -> bool {
    page_cap(state).is_some()
}
fn page_cap(state: Start) -> Option<u32> {
    match state {
        Start::Ready { cap } => Some(cap),
        Start::Listing { .. }
        | Start::Minting { .. }
        | Start::Identity { .. }
        | Start::Unconnected
        | Start::Checking { .. }
        | Start::Settings { .. }
        | Start::Waiting { .. }
        | Start::Stopped(_) => None,
    }
}

#[cfg(test)]
mod tests;

fn initial(dialect: &Dialect, next: skein_lib::Duration) -> Start {
    match dialect {
        Dialect::Forgejo { .. } => Start::Checking { next },
        Dialect::GitHub(_) => Start::Listing { next },
    }
}

#[expect(clippy::manual_find, reason = "the programming model excludes closures")]
fn find(forges: &mut List<ForgeEntry>, number: u16) -> Option<&mut ForgeEntry> {
    for entry in forges.iter_mut() {
        if entry.base.forge.number == number {
            return Some(entry);
        }
    }
    None
}
