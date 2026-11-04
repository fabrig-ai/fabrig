//! GitHub's typed start plans on the forge's reserved connection (forge-connection.md, 3.4).
use super::{Common, Dialect, ForgeEntry, OwnPlan, backoff};
use crate::{
    Event, Limits, StartRead, Stopped, Value,
    connection::{self, Exchange, Phase},
    installations::{Clock, HeldToken, Read, Resume, Selected, Tokens, Work, retry_times},
    start::Start,
};
use skein_forge::{
    api,
    credential::Credential,
    github::{
        self, document,
        plans::{
            Heard,
            plan::{Bounds, Exchange as Pending},
            start::{Identity, InstallationPage, Installations, Minting, Step},
        },
        request::Scope,
    },
};
use skein_lib::{Duration, Env, Lifecycle, Queue};

enum Progress {
    Send(Pending),
    Paged(Result<InstallationPage, api::Error>),
    Minted(Result<document::Minted, api::Error>),
    Identified(Result<document::Viewer, api::Error>),
    LogPage(Result<github::log::Listed, api::Error>),
    LogDelivery(Result<github::log::Logged, api::Error>),
}
impl Common {
    pub(super) fn github_token_refused(&mut self, entry: &mut ForgeEntry, id: u64, revision: u64, env: &Env<Limits>) {
        if self.lifecycle != Lifecycle::Live || !super::ready(entry.base.start) {
            return;
        }
        let gh = &mut entry.github_mut().expect("GitHub").state;
        if !gh.identity_checked {
            return;
        }
        let Some(row) = gh.rows.get_mut(&id) else {
            return;
        };
        if !row.accepted
            || row.suspended
            || row.state == Tokens::Gone
            || row.token_revision != revision
            || row.refused_revision == Some(revision)
        {
            return;
        }
        row.refused_revision = Some(revision);
        let active = if let Some(selected) = gh.selected {
            selected.id == id && selected.connect == gh.connect && selected.revision == row.revision && !gh.stale
        } else {
            false
        };
        for scope in [Scope::Full, Scope::Contents] {
            let running = active && gh.mint == Some((id, scope));
            let clock = row.clock(scope);
            clock.due = None;
            if !running {
                clock.pending = true;
            }
        }
        self.wake = Some(env.now);
    }
    fn github_bounds(&self, l: &Limits) -> Bounds {
        crate::limits::installation_bounds(l, &self.shared).expect("checked installation bounds")
    }
    // Ready-time jobs use the start plans without moving the public start out of Ready.
    fn github_state(entry: &ForgeEntry) -> Start {
        let state = entry.base.start;
        match state {
            Start::Ready { .. } => {
                let gh = &entry.github().expect("GitHub").state;
                match gh.work {
                    Work::Minting { next } => Start::Minting { next },
                    Work::Identity { next } => Start::Identity { next },
                    Work::Idle | Work::Waiting { .. } => state,
                }
            }
            Start::Unconnected
            | Start::Checking { .. }
            | Start::Settings { .. }
            | Start::Listing { .. }
            | Start::Minting { .. }
            | Start::Identity { .. }
            | Start::Waiting { .. }
            | Start::Stopped(_) => state,
        }
    }
    fn github_phase(entry: &mut ForgeEntry, state: Start) {
        let gh = &mut entry.github_mut().expect("GitHub").state;
        if gh.connected {
            gh.work = match state {
                Start::Minting { next } => Work::Minting { next },
                Start::Identity { next } => Work::Identity { next },
                Start::Ready { .. } | Start::Listing { .. } => Work::Idle,
                Start::Unconnected
                | Start::Checking { .. }
                | Start::Settings { .. }
                | Start::Waiting { .. }
                | Start::Stopped(_) => unreachable!("GitHub selection"),
            };
        } else {
            entry.base.start = state;
        }
    }
    pub(super) fn github_wait(entry: &mut ForgeEntry, env: &Env<Limits>) {
        if Self::github_log_active(entry) {
            Self::github_log_failed(entry, env);
            return;
        }
        let state = Self::github_state(entry);
        let gh = &mut entry.github_mut().expect("GitHub").state;
        let next = match state {
            Start::Listing { next } | Start::Minting { next } | Start::Identity { next } => next,
            Start::Ready { .. } => match &gh.read {
                Read::Reading(reading) => reading.next,
                Read::Idle | Read::Changes(_) | Read::Waiting { .. } => return,
            },
            Start::Unconnected
            | Start::Checking { .. }
            | Start::Settings { .. }
            | Start::Waiting { .. }
            | Start::Stopped(_) => return,
        };
        let (until, doubled) = retry_times(next, gh.retry.unwrap_or(Duration::ZERO), env);
        if gh.connected {
            match state {
                Start::Minting { .. } => {
                    let (id, scope) = gh.mint.expect("selected mint");
                    let row = gh.rows.get_mut(&id).expect("selected row");
                    row.state = Tokens::Waiting;
                    if gh.status == Some(403) || gh.status == Some(404) {
                        row.forget();
                        row.state = Tokens::Waiting;
                        gh.held = None;
                        gh.work = Work::Idle;
                        gh.selected = None;
                        gh.mint = None;
                        gh.read = Read::Waiting { until, next: doubled };
                        gh.reread = false;
                        gh.refused_read = true;
                    } else {
                        row.clock(scope).next = doubled;
                        gh.work = Work::Waiting { until, next: doubled, resume: Resume::Minting { scope } };
                    }
                }
                Start::Identity { .. } => {
                    gh.work = Work::Waiting { until, next: doubled, resume: Resume::Identity };
                }
                Start::Listing { .. } | Start::Ready { .. } => gh.read = Read::Waiting { until, next: doubled },
                Start::Unconnected
                | Start::Checking { .. }
                | Start::Settings { .. }
                | Start::Waiting { .. }
                | Start::Stopped(_) => unreachable!("active GitHub work"),
            }
        } else {
            gh.read = Read::Idle;
            gh.forget_tokens();
            entry.base.start = Start::Waiting { until, next: doubled };
        }
        entry.github_mut().expect("GitHub").plan = None;
    }
    pub(super) fn github_stop(entry: &mut ForgeEntry, why: Stopped, up: &mut Queue<Event>) {
        entry.base.forge.access = None;
        if let Some(github) = entry.github_mut() {
            github.state.forget();
            github.plan = None;
        }
        entry.base.start = Start::Stopped(why);
        up.push(Event::Stopped { forge: entry.base.forge.number, why });
    }
    fn github_select(entry: &mut ForgeEntry, next: Duration, env: &Env<Limits>, up: &mut Queue<Event>) {
        let gh = &mut entry.github_mut().expect("GitHub").state;
        gh.mint = None;
        gh.selected = None;
        gh.stale = false;
        let mut chosen = None;
        for (id, row) in &gh.rows {
            if row.accepted && !row.suspended {
                let scope = row.scope();
                if let Some(scope) = scope {
                    chosen = Some((*id, scope));
                    break;
                }
            }
        }
        if let Some((id, scope)) = chosen {
            let row = gh.rows.get_mut(&id).expect("eligible row");
            row.state = Tokens::Minting { scope };
            let next = next.max(row.clock(scope).next);
            row.clock(scope).take();
            gh.selected = Some(Selected { id, connect: gh.connect, revision: row.revision });
            gh.mint = Some((id, scope));
            Self::github_phase(entry, Start::Minting { next });
            return;
        }
        let mut recent = None;
        if let Some(id) = gh.recent
            && let Some(row) = gh.rows.get(&id)
            && row.accepted
            && !row.suspended
            && row.credential(env.now).is_some()
        {
            recent = Some(id);
        }
        if recent.is_none() {
            for (id, row) in &gh.rows {
                if row.accepted && !row.suspended && row.credential(env.now).is_some() {
                    recent = Some(*id);
                }
            }
        }
        gh.recent = recent;
        if let Some(id) = recent
            && !gh.identity_checked
        {
            let row = gh.rows.get(&id).expect("identity row");
            gh.selected = Some(Selected { id, connect: gh.connect, revision: row.revision });
            Self::github_phase(entry, Start::Identity { next });
        } else {
            gh.work = Work::Idle;
            let connected = gh.connected;
            gh.connected = true;
            entry.base.start = Start::Ready { cap: github::pin::PAGE_CAP };
            if !connected {
                up.push(Event::Connected { forge: entry.base.forge.number });
            }
        }
    }
    fn github_progress(&mut self, entry: &mut ForgeEntry, step: Progress, env: &Env<Limits>, up: &mut Queue<Event>) {
        let state = Self::github_state(entry);
        match step {
            Progress::Send(exchange) => {
                let github = entry.github_mut().expect("GitHub");
                match github.plan.as_mut().expect("GitHub plan") {
                    OwnPlan::GitHubInstallations(plan) => plan.set_pending(exchange),
                    OwnPlan::GitHubMinting(plan) => plan.set_pending(exchange),
                    OwnPlan::GitHubIdentity(plan) => plan.set_pending(exchange),
                    OwnPlan::GitHubLogPage(plan) => plan.set_pending(exchange),
                    OwnPlan::GitHubLogDelivery(plan) => plan.set_pending(exchange),
                }
                return;
            }
            Progress::LogPage(answer) => {
                let github = entry.github_mut().expect("GitHub");
                github.plan = None;
                match answer {
                    Ok(github::log::Listed::Page(page)) => github.state.puller.page(page),
                    Ok(github::log::Listed::Unchanged) => github.state.puller.finish(env, false),
                    Err(_) => Self::github_log_error(entry, env, up),
                }
                return;
            }
            Progress::LogDelivery(answer) => {
                entry.github_mut().expect("GitHub").plan = None;
                self.github_log_delivery(entry, answer, up);
                return;
            }
            Progress::Paged(answer) => {
                entry.github_mut().expect("GitHub").plan = None;
                let gh = &mut entry.github_mut().expect("GitHub").state;
                if gh.stale_read() {
                    gh.read = Read::Idle;
                    gh.reread = true;
                    return;
                }
                let result = match answer {
                    Ok(page) => {
                        let reading = match &mut gh.read {
                            Read::Reading(reading) => reading,
                            Read::Idle | Read::Changes(_) | Read::Waiting { .. } => unreachable!("active read"),
                        };
                        match reading.page(page.rows, &gh.rows, &gh.accepted, self.shared.name_bytes) {
                            Ok(()) => {
                                if page.next {
                                    match reading.page.checked_add(1) {
                                        Some(next) => {
                                            reading.page = next;
                                            Ok(())
                                        }
                                        None => Err(api::Error::TooLarge),
                                    }
                                } else {
                                    reading.complete = true;
                                    Ok(())
                                }
                            }
                            Err(error) => Err(error),
                        }
                    }
                    Err(error) => Err(error),
                };
                if let Err(error) = result {
                    Self::github_error(entry, error, env, up);
                }
                return;
            }
            Progress::Minted(answer) => {
                if !Self::github_minted(entry, answer, env, up) {
                    return;
                }
            }
            Progress::Identified(answer) => {
                if !Self::github_identified(entry, answer, env, up) {
                    return;
                }
            }
        }
        entry.github_mut().expect("GitHub").plan = None;
        let gh = &entry.github().expect("GitHub").state;
        let next = if gh.connected { env.limits.start_backoff } else { active_next(state) };
        if gh.reread {
            Self::github_phase(entry, Start::Listing { next });
        } else {
            Self::github_select(entry, next, env, up);
        }
    }
    fn github_identified(
        entry: &mut ForgeEntry,
        answer: Result<document::Viewer, api::Error>,
        env: &Env<Limits>,
        up: &mut Queue<Event>,
    ) -> bool {
        let gh = &entry.github().expect("GitHub").state;
        if !gh.selected_valid() {
            return true;
        }
        match answer {
            Ok(viewer) => {
                if viewer.id != entry.base.forge.access.as_ref().expect("access").user {
                    Self::github_stop(entry, Stopped::OtherUser { user: viewer.id }, up);
                    return false;
                }
                entry.github_mut().expect("GitHub").state.identity_checked = true;
                true
            }
            Err(error) => {
                Self::github_error(entry, error, env, up);
                false
            }
        }
    }
    fn github_minted(
        entry: &mut ForgeEntry,
        answer: Result<document::Minted, api::Error>,
        env: &Env<Limits>,
        up: &mut Queue<Event>,
    ) -> bool {
        let next = active_next(Self::github_state(entry));
        if !entry.github().expect("GitHub").state.selected_valid() {
            return true;
        }
        match answer {
            Ok(minted) => {
                if !crate::limits::header_value(minted.token.expose())
                    || minted.token.len() > usize::try_from(env.limits.token_bytes).expect("token bound")
                    || minted.expires <= env.wall
                {
                    Self::github_wait(entry, env);
                    return false;
                }
                let gh = &mut entry.github_mut().expect("GitHub").state;
                let (id, scope) = gh.mint.expect("selected mint");
                let row = gh.rows.get_mut(&id).expect("selected row");
                if row.accepted && !row.suspended {
                    match scope {
                        Scope::Full => {
                            row.api = Some(minted.token);
                            row.permissions = minted.permissions;
                            row.api_expires = minted.expires;
                            row.full = Clock::minted(minted.expires, next, env);
                            row.token_revision = row.token_revision.checked_add(1).expect("Full token revision space");
                            row.refused_revision = None;
                            row.state = Tokens::Current;
                            gh.recent = Some(id);
                        }
                        Scope::Contents => {
                            row.git_expires = minted.expires;
                            row.contents = Clock::minted(minted.expires, next, env);
                            row.state = Tokens::Current;
                            gh.held = Some(HeldToken {
                                selected: gh.selected.expect("selected Contents"),
                                token: minted.token,
                                expires: minted.expires,
                            });
                        }
                    }
                }
            }
            Err(error) => {
                Self::github_error(entry, error, env, up);
                return false;
            }
        }
        true
    }
    fn github_error(entry: &mut ForgeEntry, _error: api::Error, env: &Env<Limits>, up: &mut Queue<Event>) {
        let gh = &entry.github().expect("GitHub").state;
        if let Some(status) = gh.status
            && (400..500).contains(&status)
            && status != 429
        {
            let why = match Self::github_state(entry) {
                Start::Listing { .. } | Start::Ready { .. } => Some(if status == 401 {
                    Stopped::KeyRefused
                } else {
                    Stopped::Refused { read: StartRead::Installations, status }
                }),
                Start::Minting { .. } => {
                    if status == 401 {
                        Some(Stopped::KeyRefused)
                    } else if status == 403 || status == 404 {
                        None
                    } else {
                        Some(Stopped::TokenRefused { installation: gh.mint.expect("mint").0, status })
                    }
                }
                Start::Identity { .. } => Some(Stopped::Refused { read: StartRead::Viewer, status }),
                Start::Unconnected
                | Start::Checking { .. }
                | Start::Settings { .. }
                | Start::Waiting { .. }
                | Start::Stopped(_) => None,
            };
            if let Some(why) = why {
                Self::github_stop(entry, why, up);
                return;
            }
        }
        Self::github_wait(entry, env);
    }
    fn github_begin(&mut self, entry: &mut ForgeEntry, state: Start, env: &Env<Limits>, up: &mut Queue<Event>) {
        let log_plan = if Self::github_log_selected(entry) {
            entry.github_mut().expect("GitHub").state.puller.request()
        } else {
            None
        };
        let mut plan = if let Some(plan) = log_plan {
            plan
        } else {
            match state {
                Start::Listing { .. } | Start::Ready { .. } => {
                    let gh = &entry.github().expect("GitHub").state;
                    let page = match &gh.read {
                        Read::Reading(reading) => reading.page,
                        Read::Idle | Read::Changes(_) | Read::Waiting { .. } => unreachable!("selected read"),
                    };
                    OwnPlan::GitHubInstallations(Installations::new(page))
                }
                Start::Minting { .. } => {
                    let gh = &entry.github().expect("GitHub").state;
                    let (id, scope) = gh.mint.expect("selected mint");
                    OwnPlan::GitHubMinting(Minting::new(id, scope))
                }
                Start::Identity { .. } => OwnPlan::GitHubIdentity(Identity::new()),
                Start::Unconnected
                | Start::Checking { .. }
                | Start::Settings { .. }
                | Start::Waiting { .. }
                | Start::Stopped(_) => unreachable!("active GitHub start"),
            }
        };
        let bounds = self.github_bounds(&env.limits);
        let progress = match &mut plan {
            OwnPlan::GitHubInstallations(plan) => match plan.begin(bounds) {
                Ok(Step::Send(send)) => Progress::Send(send),
                Ok(Step::Done(done)) => Progress::Paged(done),
                Err(error) => Progress::Paged(Err(error)),
            },
            OwnPlan::GitHubMinting(plan) => match plan.begin(bounds) {
                Ok(Step::Send(send)) => Progress::Send(send),
                Ok(Step::Done(done)) => Progress::Minted(done),
                Err(error) => Progress::Minted(Err(error)),
            },
            OwnPlan::GitHubIdentity(plan) => match plan.begin(bounds) {
                Ok(Step::Send(send)) => Progress::Send(send),
                Ok(Step::Done(done)) => Progress::Identified(done),
                Err(error) => Progress::Identified(Err(error)),
            },
            OwnPlan::GitHubLogPage(plan) => match plan.begin(Self::github_log_bounds(&env.limits, false)) {
                Ok(Step::Send(send)) => Progress::Send(send),
                Ok(Step::Done(done)) => Progress::LogPage(done),
                Err(error) => Progress::LogPage(Err(error)),
            },
            OwnPlan::GitHubLogDelivery(plan) => match plan.begin(Self::github_log_bounds(&env.limits, true)) {
                Ok(Step::Send(send)) => Progress::Send(send),
                Ok(Step::Done(done)) => Progress::LogDelivery(done),
                Err(error) => Progress::LogDelivery(Err(error)),
            },
        };
        entry.github_mut().expect("GitHub").plan = Some(plan);
        self.github_progress(entry, progress, env, up);
    }
    fn github_send(
        &mut self,
        entry: &mut ForgeEntry,
        id: skein_lib::Id<connection::Connection>,
        env: &Env<Limits>,
        up: &mut Queue<Event>,
    ) -> bool {
        let state = Self::github_state(entry);
        let log = Self::github_log_active(entry);
        let c = self.connection_slab.get_mut(id).expect("own connection");
        if c.phase == Phase::Idle {
            let gh = &entry.github().expect("GitHub").state;
            if !log && gh.stale_read() {
                let gh = &mut entry.github_mut().expect("GitHub").state;
                gh.read = Read::Idle;
                gh.reread = true;
                entry.github_mut().expect("GitHub").plan = None;
                return false;
            }
            let selected = match state {
                Start::Minting { .. } | Start::Identity { .. } => true,
                Start::Listing { .. } | Start::Ready { .. } => false,
                Start::Unconnected
                | Start::Checking { .. }
                | Start::Settings { .. }
                | Start::Waiting { .. }
                | Start::Stopped(_) => unreachable!("active GitHub work"),
            };
            if !log && selected && !gh.selected_valid() {
                entry.github_mut().expect("GitHub").plan = None;
                Self::github_select(entry, active_next(state), env, up);
                return false;
            }
            let github = entry.github_mut().expect("GitHub");
            let pending = match github.plan.as_mut().expect("GitHub plan") {
                OwnPlan::GitHubInstallations(plan) => plan.pending(),
                OwnPlan::GitHubMinting(plan) => plan.pending(),
                OwnPlan::GitHubIdentity(plan) => plan.pending(),
                OwnPlan::GitHubLogPage(plan) => plan.pending(),
                OwnPlan::GitHubLogDelivery(plan) => plan.pending(),
            };
            if let Some(exchange) = pending {
                let forge = &entry.base.forge;
                let github = match &mut entry.dialect {
                    Dialect::Forgejo { .. } => return false,
                    Dialect::GitHub(github) => github,
                };
                let gh = &mut github.state;
                gh.status = None;
                gh.retry = None;
                let secret = match state {
                    Start::Identity { .. } => {
                        let row = gh.rows.get(&gh.recent.expect("identity row")).expect("identity row");
                        if row.credential(env.now).is_none() {
                            github.plan = None;
                            let row = gh.rows.get_mut(&gh.recent.expect("identity row")).expect("identity row");
                            row.forget();
                            if row.accepted && !row.suspended {
                                row.unminted();
                            }
                            let next = active_next(state);
                            Self::github_select(entry, next, env, up);
                            return false;
                        }
                        row.api.clone()
                    }
                    Start::Listing { .. } | Start::Minting { .. } | Start::Ready { .. } => {
                        match &forge.access.as_ref().expect("access").credential {
                            Credential::App { id, key } => {
                                gh.jwt.bearer(*id, key, env.wall, env.limits.deadline).cloned()
                            }
                            Credential::Token(_) => unreachable!("checked App access"),
                        }
                    }
                    Start::Unconnected
                    | Start::Checking { .. }
                    | Start::Settings { .. }
                    | Start::Waiting { .. }
                    | Start::Stopped(_) => unreachable!("active GitHub start"),
                };
                let Some(secret) = secret else {
                    Self::github_stop(entry, Stopped::Invalid { value: Value::Key }, up);
                    return false;
                };
                let authorization = github::credential::bearer(&secret).expect("bounded bearer");
                c.deadline = Some(env.now.saturating_add(env.limits.deadline));
                if c.start(
                    env,
                    forge,
                    &self.owner.user_agent,
                    Exchange::GitHub { exchange, authorization: Some(authorization) },
                )
                .is_err()
                {
                    Self::github_wait(entry, env);
                    return false;
                }
            }
        }
        true
    }
    fn github_heard(
        &mut self,
        entry: &mut ForgeEntry,
        heard: connection::Heard,
        env: &Env<Limits>,
        up: &mut Queue<Event>,
    ) {
        let heard = match heard {
            connection::Heard::GitHub(heard) => heard,
            connection::Heard::Forgejo(_) => unreachable!("GitHub connection"),
        };
        let gh = &mut entry.github_mut().expect("GitHub").state;
        match &heard {
            Heard::Head { status, head } => {
                gh.status = Some(*status);
                gh.retry = head.retry_after;
            }
            Heard::Row(
                document::Row::Installation(_)
                | document::Row::Rule(_)
                | document::Row::Log(_)
                | document::Row::Reachable(_)
                | document::Row::Branch(_)
                | document::Row::SkippedBranch
                | document::Row::Issue(_)
                | document::Row::Pull(_)
                | document::Row::Context(_)
                | document::Row::Run(_)
                | document::Row::Comment(_)
                | document::Row::Collaborator(_)
                | document::Row::Remark(_)
                | document::Row::Review(_)
                | document::Row::Commit(_)
                | document::Row::Entry { .. }
                | document::Row::File(_),
            )
            | Heard::Done(_)
            | Heard::Failed(_) => {}
        }
        let bounds = self.github_bounds(&env.limits);
        let log_page_bounds = Self::github_log_bounds(&env.limits, false);
        let log_delivery_bounds = Self::github_log_bounds(&env.limits, true);
        let github = entry.github_mut().expect("GitHub");
        let progress = match github.plan.as_mut().expect("GitHub plan") {
            OwnPlan::GitHubInstallations(plan) => match plan.heard(heard, bounds) {
                Ok(Some(Step::Send(send))) => Some(Progress::Send(send)),
                Ok(Some(Step::Done(done))) => Some(Progress::Paged(done)),
                Ok(None) => None,
                Err(error) => Some(Progress::Paged(Err(error))),
            },
            OwnPlan::GitHubMinting(plan) => match plan.heard(heard, bounds) {
                Ok(Some(Step::Send(send))) => Some(Progress::Send(send)),
                Ok(Some(Step::Done(done))) => Some(Progress::Minted(done)),
                Ok(None) => None,
                Err(error) => Some(Progress::Minted(Err(error))),
            },
            OwnPlan::GitHubIdentity(plan) => match plan.heard(heard, bounds) {
                Ok(Some(Step::Send(send))) => Some(Progress::Send(send)),
                Ok(Some(Step::Done(done))) => Some(Progress::Identified(done)),
                Ok(None) => None,
                Err(error) => Some(Progress::Identified(Err(error))),
            },
            OwnPlan::GitHubLogPage(plan) => match plan.heard(heard, log_page_bounds) {
                Ok(Some(Step::Send(send))) => Some(Progress::Send(send)),
                Ok(Some(Step::Done(done))) => Some(Progress::LogPage(done)),
                Ok(None) => None,
                Err(error) => Some(Progress::LogPage(Err(error))),
            },
            OwnPlan::GitHubLogDelivery(plan) => match plan.heard(heard, log_delivery_bounds) {
                Ok(Some(Step::Send(send))) => Some(Progress::Send(send)),
                Ok(Some(Step::Done(done))) => Some(Progress::LogDelivery(done)),
                Ok(None) => None,
                Err(error) => Some(Progress::LogDelivery(Err(error))),
            },
        };
        if let Some(progress) = progress {
            self.github_progress(entry, progress, env, up);
        }
    }
    // A listing's local output reaches its owner before the next remote exchange.
    fn local_output_pending(entry: &ForgeEntry) -> bool {
        let gh = &entry.github().expect("GitHub").state;
        match &gh.read {
            Read::Changes(_) => true,
            Read::Reading(reading) => reading.fact.is_some() || reading.complete,
            Read::Idle => gh.held.is_some(),
            Read::Waiting { .. } => false,
        }
    }
    pub(super) fn github_local(&mut self, entry: &mut ForgeEntry, env: &Env<Limits>, up: &mut Queue<Event>) -> bool {
        let state = entry.base.start;
        let busy = entry.github().expect("GitHub").plan.is_some();
        let forge = entry.base.forge.number;
        let gh = &mut entry.github_mut().expect("GitHub").state;
        match &gh.read {
            Read::Changes(_) => {
                if let Some(event) = gh.output(forge) {
                    up.push(event);
                } else {
                    gh.read = Read::Idle;
                    if busy {
                        return true;
                    }
                    match state {
                        Start::Listing { next } => Self::github_select(entry, next, env, up),
                        Start::Ready { .. } => {
                            let next = gh.read_next;
                            Self::github_select(entry, next, env, up);
                        }
                        Start::Unconnected
                        | Start::Checking { .. }
                        | Start::Settings { .. }
                        | Start::Minting { .. }
                        | Start::Identity { .. }
                        | Start::Waiting { .. }
                        | Start::Stopped(_) => {}
                    }
                }
                true
            }
            Read::Reading(reading) => {
                if reading.fact.is_some() {
                    up.push(gh.output(forge).expect("held fact"));
                    true
                } else if reading.complete {
                    if gh.stale_read() {
                        gh.read = Read::Idle;
                        gh.reread = true;
                    } else {
                        gh.commit();
                    }
                    true
                } else {
                    false
                }
            }
            Read::Waiting { until, next } => {
                if *until <= env.now {
                    let next = *next;
                    gh.begin_read(next);
                    true
                } else {
                    false
                }
            }
            Read::Idle => self.github_idle(entry, env, up),
        }
    }
    fn github_idle(&mut self, entry: &mut ForgeEntry, env: &Env<Limits>, up: &mut Queue<Event>) -> bool {
        let state = entry.base.start;
        let busy = entry.github().expect("GitHub").plan.is_some();
        let forge = entry.base.forge.number;
        let gh = &mut entry.github_mut().expect("GitHub").state;

        let read_next = match state {
            Start::Listing { next } => Some(next),
            Start::Ready { .. } => Some(env.limits.start_backoff),
            Start::Unconnected
            | Start::Checking { .. }
            | Start::Settings { .. }
            | Start::Minting { .. }
            | Start::Identity { .. }
            | Start::Waiting { .. }
            | Start::Stopped(_) => None,
        };
        if !busy
            && gh.reread
            && let Some(next) = read_next
        {
            gh.begin_read(next);
            return true;
        }
        if let Some(held) = gh.held.take() {
            if gh.valid(held.selected) {
                let row = gh.rows.get(&held.selected.id).expect("eligible held token");
                up.push(Event::GitToken {
                    forge,
                    installation: held.selected.id,
                    account: row.account.clone(),
                    token: held.token,
                    expires: held.expires,
                });
            }
            return true;
        }
        if busy {
            return false;
        }
        match state {
            Start::Listing { next } => {
                gh.begin_read(next);
                true
            }
            Start::Ready { .. } => self.github_ready(entry, env, up),
            Start::Unconnected
            | Start::Checking { .. }
            | Start::Settings { .. }
            | Start::Minting { .. }
            | Start::Identity { .. }
            | Start::Waiting { .. }
            | Start::Stopped(_) => false,
        }
    }
    fn github_ready(&mut self, entry: &mut ForgeEntry, env: &Env<Limits>, up: &mut Queue<Event>) -> bool {
        let gh = &mut entry.github_mut().expect("GitHub").state;
        match gh.work {
            Work::Waiting { until, next, resume } => {
                if !gh.selected_valid() {
                    gh.work = Work::Idle;
                    Self::github_select(entry, next, env, up);
                    return true;
                }
                if until > env.now {
                    return self.github_log_local(entry, env, up);
                }
                let selected = gh.selected.expect("retry selection");
                let row = gh.rows.get_mut(&selected.id).expect("eligible retry");
                match resume {
                    Resume::Minting { scope } => {
                        row.state = Tokens::Minting { scope };
                        gh.mint = Some((selected.id, scope));
                        gh.work = Work::Minting { next };
                    }
                    Resume::Identity => {
                        if row.credential(env.now).is_none() {
                            row.forget();
                            row.unminted();
                            gh.held = None;
                            gh.work = Work::Idle;
                            Self::github_select(entry, next, env, up);
                        } else {
                            gh.work = Work::Identity { next };
                        }
                    }
                }
                true
            }
            Work::Idle => {
                Self::github_select(entry, env.limits.start_backoff, env, up);
                if github_active(Self::github_state(entry)) { true } else { self.github_log_local(entry, env, up) }
            }
            Work::Minting { .. } | Work::Identity { .. } => false,
        }
    }
    fn github_retire_unsent(
        &mut self,
        entry: &mut ForgeEntry,
        id: skein_lib::Id<connection::Connection>,
        env: &Env<Limits>,
        up: &mut Queue<Event>,
        io: &mut Queue<skein_io::Request>,
    ) -> bool {
        let log = Self::github_log_active(entry);
        let gh = &entry.github().expect("GitHub").state;
        let c = self.connection_slab.get_mut(id).expect("own connection");
        let unsent = match c.phase {
            Phase::Sending { written } => !written && !c.written,
            Phase::Connecting | Phase::Handshaking | Phase::Closed | Phase::Reading | Phase::Idle => false,
        };
        if !log && unsent && gh.selected.is_some() && !gh.selected_valid() {
            c.close(env, false, io);
            entry.github_mut().expect("GitHub").plan = None;
            let next = active_next(Self::github_state(entry));
            Self::github_select(entry, next, env, up);
            return true;
        }
        false
    }
    #[expect(clippy::too_many_lines, reason = "the reserved connection's drive keeps its bounded pass together")]
    pub(super) fn drive_github(
        &mut self,
        entry: &mut ForgeEntry,
        env: &Env<Limits>,
        up: &mut Queue<Event>,
        io: &mut Queue<skein_io::Request>,
    ) {
        entry.github_mut().expect("GitHub").state.fire(env.now);
        let state = entry.base.start;
        let gh = &mut entry.github_mut().expect("GitHub").state;
        gh.puller.admitted();
        if self.lifecycle == Lifecycle::Live && super::ready(state) {
            let _started = gh.puller.fire(env);
        }
        if let Some((until, next)) = backoff(entry.base.start)
            && until <= env.now
            && self.lifecycle == Lifecycle::Live
        {
            entry.base.start = Start::Listing { next };
        }
        for turn in 0..64_u32 {
            if self.lifecycle == Lifecycle::Live && Self::local_output_pending(entry) {
                let progressed = self.github_local(entry, env, up);
                assert!(progressed, "pending local output advances before remote work");
                if turn == 63 {
                    self.wake = Some(env.now);
                }
                continue;
            }
            if self.lifecycle == Lifecycle::Live && self.github_local(entry, env, up) {
                if turn == 63 {
                    self.wake = Some(env.now);
                }
                continue;
            }
            let state = Self::github_state(entry);
            let gh = &entry.github().expect("GitHub").state;
            let reading = match gh.read {
                Read::Reading(_) => true,
                Read::Idle | Read::Changes(_) | Read::Waiting { .. } => false,
            };
            let active =
                github_active(state) || reading || Self::github_log_active(entry) || Self::github_log_selected(entry);
            if self.lifecycle == Lifecycle::Live && active && entry.github().expect("GitHub").plan.is_none() {
                self.github_begin(entry, state, env, up);
                continue;
            }
            let mut token = entry.base.own;
            if self.lifecycle == Lifecycle::Live
                && active
                && token.is_none()
                && self.connections.len() < self.connections.capacity()
            {
                token = Some(self.open_own(entry, env, io));
            }
            let Some(token) = token else {
                break;
            };
            let id = *self.connections.get(&token).expect("own connection");
            let c = self.connection_slab.get_mut(id).expect("own connection");
            if self.lifecycle != Lifecycle::Live || !active {
                if c.deadline.is_none() && c.phase == Phase::Idle {
                    c.deadline = Some(env.now.saturating_add(env.limits.idle));
                }
                if self.lifecycle != Lifecycle::Live || c.phase != Phase::Idle || c.deadline <= Some(env.now) {
                    c.close(env, c.in_flight(), io);
                }
                break;
            }
            if c.closing {
                break;
            }
            if c.phase != Phase::Idle
                && let Some(deadline) = c.deadline
                && deadline <= env.now
            {
                c.timeout();
            }
            if self.github_retire_unsent(entry, id, env, up, io) {
                continue;
            }
            if !self.github_send(entry, id, env, up) {
                continue;
            }
            let c = self.connection_slab.get_mut(id).expect("own connection");
            c.pump(env, io);
            if let Some(heard) = c.heard() {
                self.github_heard(entry, heard, env, up);
                let c = self.connection_slab.get_mut(id).expect("own connection");
                // A plan can finish on the head, leaving an unread exchange behind.
                if entry.github().expect("GitHub").plan.is_none() {
                    if c.in_flight() {
                        c.close(env, true, io);
                    } else {
                        c.deadline = None;
                    }
                }
                continue;
            }
            if c.phase == Phase::Closed {
                c.close(env, c.in_flight(), io);
                break;
            }
            if !c.ready() {
                break;
            }
            if turn == 63 {
                self.wake = Some(env.now);
            }
        }
    }
}

fn github_active(state: Start) -> bool {
    match state {
        Start::Listing { .. } | Start::Minting { .. } | Start::Identity { .. } => true,
        Start::Unconnected
        | Start::Checking { .. }
        | Start::Settings { .. }
        | Start::Waiting { .. }
        | Start::Ready { .. }
        | Start::Stopped(_) => false,
    }
}
fn active_next(state: Start) -> Duration {
    match state {
        Start::Listing { next } | Start::Minting { next } | Start::Identity { next } => next,
        Start::Unconnected
        | Start::Checking { .. }
        | Start::Settings { .. }
        | Start::Waiting { .. }
        | Start::Ready { .. }
        | Start::Stopped(_) => unreachable!("active GitHub start"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Component;
    use crate::{
        Account, Change, Fact, Request,
        installations::{Metadata, Row},
        tests::fixtures,
    };
    use alloc::boxed::Box;
    use skein_lib::{List, Time, Wall};

    // Twenty rows include lifecycle then rename on id 1.
    const ROWS: u32 = 20;
    fn pending() -> (Component, Env<Limits>) {
        let limits = Limits { installations: ROWS, ..fixtures::limits() };
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
        let mut component = Component::new(fixtures::github_settings(), &limits, &fixtures::shared()).expect("fixture");
        let gh = &mut component.forges.get_mut(0).expect("forge").github_mut().expect("GitHub").state;
        gh.baseline = false;
        drop(
            gh.rows
                .insert(
                    1,
                    Row {
                        state: Tokens::Gone,
                        revision: 0,
                        token_revision: 0,
                        refused_revision: None,
                        full: Clock::empty(),
                        contents: Clock::empty(),
                        account: Account { id: 22, login: Box::from(b"acme".as_slice()) },
                        suspended: false,
                        accepted: false,
                        api: None,
                        permissions: document::Levels::NONE,
                        api_expires: Wall::EPOCH,
                        git_expires: Wall::EPOCH,
                    },
                )
                .expect("old row"),
        );
        gh.begin_read(limits.start_backoff);
        match &mut gh.read {
            Read::Reading(reading) => {
                let _seen = reading.seen.insert(1).expect("seen");
                for id in 1..=ROWS {
                    drop(
                        reading
                            .staged
                            .insert(
                                u64::from(id),
                                Metadata {
                                    account: Account {
                                        id: u64::from(id).saturating_add(21),
                                        login: Box::from(b"renamed".as_slice()),
                                    },
                                    suspended: true,
                                    seen: true,
                                },
                            )
                            .expect("staged rows"),
                    );
                }
            }
            Read::Idle | Read::Changes(_) | Read::Waiting { .. } => unreachable!("reading"),
        }
        gh.commit();
        (component, env)
    }
    fn installation(id: u64, login: &[u8]) -> document::Installation {
        document::Installation {
            id,
            account: 22,
            login: Box::from(login),
            kind: document::Account::Organization,
            created: Wall::EPOCH,
            suspended: false,
        }
    }
    fn queues() -> (Queue<Event>, Queue<skein_io::Request>) {
        (Queue::with_capacity(128), Queue::with_capacity(128))
    }
    #[test]
    fn installation_output_preserves_lifecycle_then_rename() {
        let (mut component, env) = pending();
        let (mut up, mut io) = queues();
        component.test_local(&env, &mut up);
        match up.pop().expect("first phase") {
            Event::Installation { installation, change, .. } => {
                assert_eq!((installation, change), (1, Change::Suspended));
            }
            Event::Fact { .. }
            | Event::GitToken { .. }
            | Event::Answered(_)
            | Event::Hint(_)
            | Event::Listening { .. }
            | Event::ListenerLost
            | Event::Connected { .. }
            | Event::Stopped { .. }
            | Event::Closed => unreachable!("lifecycle"),
        }
        let mut changes = List::with_capacity(ROWS.saturating_add(1));
        for _ in 0_u32..4 {
            component.fire(&env, &mut up, &mut io);
            for _ in 0..up.len() {
                match up.pop().expect("queued") {
                    Event::Installation { installation, change, .. } => {
                        changes.push((installation, change)).expect("bounded changes");
                    }
                    Event::Connected { .. } => assert_eq!(changes.len(), ROWS),
                    Event::Fact { .. }
                    | Event::GitToken { .. }
                    | Event::Answered(_)
                    | Event::Hint(_)
                    | Event::Listening { .. }
                    | Event::ListenerLost
                    | Event::Stopped { .. }
                    | Event::Closed => unreachable!("change output"),
                }
            }
        }
        assert_eq!(changes.len(), ROWS);
        assert_eq!(changes.get(0), Some(&(1, Change::Renamed)));
        for (index, (id, change)) in changes.as_slice().iter().enumerate().skip(1) {
            assert_eq!((*id, *change), (u64::try_from(index).expect("index").saturating_add(1), Change::Created));
        }
        assert!(io.is_empty());
        assert_eq!(component.next_deadline(), Some(env.now.saturating_add(env.limits.log_poll)));
    }
    #[test]
    fn held_unkept_precedes_the_next_page() {
        let limits = Limits { installations: 1, ..fixtures::limits() };
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
        let mut component = Component::new(fixtures::github_settings(), &limits, &fixtures::shared()).expect("fixture");
        let gh = &mut component.forges.get_mut(0).expect("forge").github_mut().expect("GitHub").state;
        gh.begin_read(limits.start_backoff);
        let mut rows = List::with_capacity(2);
        for id in [1, 2] {
            rows.push(installation(id, b"name")).expect("page");
        }
        match &mut gh.read {
            Read::Reading(reading) => {
                reading
                    .page(rows.into_boxed(), &gh.rows, &gh.accepted, fixtures::shared().name_bytes)
                    .expect("bounded page");
                reading.page = 2;
            }
            Read::Idle | Read::Changes(_) | Read::Waiting { .. } => unreachable!("reading"),
        }
        let (mut up, mut io) = queues();
        component.test_local(&env, &mut up);
        match up.pop().expect("held fact") {
            Event::Fact { fact, .. } => assert_eq!(fact, Fact::Unkept { installation: 2 }),
            Event::Installation { .. }
            | Event::GitToken { .. }
            | Event::Answered(_)
            | Event::Hint(_)
            | Event::Listening { .. }
            | Event::ListenerLost
            | Event::Connected { .. }
            | Event::Stopped { .. }
            | Event::Closed => unreachable!("fact"),
        }
        assert!(io.is_empty());
        component.fire(&env, &mut up, &mut io);
        assert_eq!(io.len(), 1);
        assert!(up.is_empty());
    }
    #[test]
    fn close_abandons_pending_installation_output() {
        let (mut component, env) = pending();
        let (mut up, mut io) = queues();
        component.test_local(&env, &mut up);
        drop(up.pop());
        component.down(&env, Request::Close, &mut up, &mut io);
        match up.pop().expect("closed") {
            Event::Closed => {}
            Event::Installation { .. }
            | Event::Fact { .. }
            | Event::GitToken { .. }
            | Event::Answered(_)
            | Event::Hint(_)
            | Event::Listening { .. }
            | Event::ListenerLost
            | Event::Connected { .. }
            | Event::Stopped { .. } => unreachable!("Closed last"),
        }
        component.fire(&env, &mut up, &mut io);
        assert!(up.is_empty());
        assert!(io.is_empty());
    }
    #[test]
    fn connect_replacement_abandons_pending_installation_output() {
        let (mut component, env) = pending();
        let (mut up, mut io) = queues();
        component.test_local(&env, &mut up);
        drop(up.pop());
        component.down(
            &env,
            Request::Connect {
                forge: fixtures::FORGE,
                access: fixtures::app_access(skein_fake_forge_github::keys::KEYS.first().expect("public key"), 7, 9),
            },
            &mut up,
            &mut io,
        );
        assert!(up.is_empty());
        assert_eq!(io.len(), 1);
        let gh = &component.forges.get(0).expect("forge").github().expect("GitHub").state;
        match gh.read {
            Read::Reading(_) => {}
            Read::Idle | Read::Changes(_) | Read::Waiting { .. } => unreachable!("replacement read"),
        }
    }
    #[test]
    fn installation_page_number_overflow_fails_the_attempt() {
        let limits = fixtures::limits();
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
        let mut component = Component::new(fixtures::github_settings(), &limits, &fixtures::shared()).expect("fixture");
        let gh = &mut component.forges.get_mut(0).expect("forge").github_mut().expect("GitHub").state;
        gh.begin_read(limits.start_backoff);
        match &mut gh.read {
            Read::Reading(reading) => reading.page = u32::MAX,
            Read::Idle | Read::Changes(_) | Read::Waiting { .. } => unreachable!("reading"),
        }
        let (mut up, mut io) = queues();
        component.common.github_progress(
            component.forges.get_mut(0).expect("forge"),
            Progress::Paged(Ok(InstallationPage { rows: Box::from([]), next: true })),
            &env,
            &mut up,
        );
        assert_eq!(
            component.start(fixtures::FORGE),
            Some(Start::Waiting {
                until: env.now.saturating_add(limits.start_backoff),
                next: limits.start_backoff.saturating_mul(2)
            })
        );
        assert!(up.is_empty());
        component.fire(&env, &mut up, &mut io);
        assert!(io.is_empty());
    }

    #[test]
    #[expect(clippy::too_many_lines, reason = "the scenario keeps its committed-table and output assertions together")]
    fn failed_later_page_preserves_committed_table() {
        let limits = Limits { installations: 1, ..fixtures::limits() };
        let mut env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
        let mut component = Component::new(fixtures::github_settings(), &limits, &fixtures::shared()).expect("fixture");
        component.forges.get_mut(0).expect("forge").base.start = Start::Ready { cap: 100 };
        let gh = &mut component.forges.get_mut(0).expect("forge").github_mut().expect("GitHub").state;
        gh.baseline = false;
        gh.connected = true;
        gh.identity_checked = true;
        gh.accept(&[1]);
        let token = b"artificial-api";
        let expiry = Wall::from_nanos(3_600_000_000_000);
        drop(
            gh.rows
                .insert(
                    1,
                    Row {
                        state: Tokens::Current,
                        revision: 0,
                        token_revision: 0,
                        refused_revision: None,
                        full: Clock { expiry: Time::from_nanos(expiry.as_nanos()), ..Clock::empty() },
                        contents: Clock::empty(),
                        account: Account { id: 22, login: Box::from(b"acme".as_slice()) },
                        suspended: false,
                        accepted: true,
                        api: Some(skein_lib::Secret::new(Box::from(token.as_slice()))),
                        permissions: document::Levels::NONE,
                        api_expires: expiry,
                        git_expires: expiry,
                    },
                )
                .expect("committed row"),
        );
        gh.begin_read(limits.start_backoff);
        let (mut up, mut io) = queues();
        let mut staged = List::with_capacity(2);
        for (id, login) in [(1, b"renamed".as_slice()), (2, b"stray".as_slice())] {
            staged.push(installation(id, login)).expect("page rows");
        }
        component.common.github_progress(
            component.forges.get_mut(0).expect("forge"),
            Progress::Paged(Ok(InstallationPage { rows: staged.into_boxed(), next: true })),
            &env,
            &mut up,
        );
        component.test_local(&env, &mut up);
        component.common.github_progress(
            component.forges.get_mut(0).expect("forge"),
            Progress::Paged(Err(api::Error::InvalidAnswer)),
            &env,
            &mut up,
        );
        let gh = &component.forges.get(0).expect("forge").github().expect("GitHub").state;
        let row = gh.rows.get(&1).expect("committed row");
        assert_eq!(
            (row.account.login.as_ref(), row.api_expires, row.git_expires),
            (b"acme".as_slice(), expiry, expiry)
        );
        assert!(row.api.as_ref().expect("API token").expose() == token);
        assert_eq!(gh.rows.len(), 1);
        assert_eq!(up.len(), 1);
        match up.pop().expect("capacity diagnostic survives failure") {
            Event::Fact { fact, .. } => assert_eq!(fact, Fact::Unkept { installation: 2 }),
            Event::Installation { .. }
            | Event::GitToken { .. }
            | Event::Answered(_)
            | Event::Hint(_)
            | Event::Listening { .. }
            | Event::ListenerLost
            | Event::Connected { .. }
            | Event::Stopped { .. }
            | Event::Closed => unreachable!("one fact"),
        }
        env.now = env.now.saturating_add(limits.start_backoff);
        let _progressed = component.common.github_local(component.forges.get_mut(0).expect("forge"), &env, &mut up);
        let page = installation(1, b"acme");
        component.common.github_progress(
            component.forges.get_mut(0).expect("forge"),
            Progress::Paged(Ok(InstallationPage { rows: Box::from([page]), next: false })),
            &env,
            &mut up,
        );
        component.drive(&env, &mut up, &mut io);
        assert!(up.is_empty());
        assert!(io.is_empty());
        assert!(
            component
                .forges
                .get(0)
                .expect("forge")
                .github()
                .expect("GitHub")
                .state
                .rows
                .get(&1)
                .expect("row")
                .api
                .as_ref()
                .expect("preserved")
                .expose()
                == token
        );
    }
}
