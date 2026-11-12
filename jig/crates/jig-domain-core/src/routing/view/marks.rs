//! Bounded subject marks gathered while a decision routes.
//! `client/domain/contract/README.md`, 5.2. Task saves gather the affected watched
//! subjects; party-entry updates gather frame and inbox subjects. Marks hold
//! no state snapshots and coalesce in a set. Once projection handoffs finish,
//! the views child asks for documents from the decision's final live state.

use crate::{
    Core, Limits, Request,
    documents::{self, Proposer, Subject, Touch},
    routing::Work,
};
use jig_domain_people as people;
use jig_domain_people::ApplicationScope;
use jig_domain_tasks as tasks;
use jig_domain_views as views;
use skein_lib::{Env, List, Queue, Set, Token};

impl Core {
    /// A list answer changes account lines and deployment facts without marking frames.
    pub(crate) fn touch_account_lines(&mut self, party: Option<u64>) {
        self.touch_views(Touch::AccountLines(party));
    }

    /// Connector settlement changes the chips nested in watched documents.
    pub(crate) fn touch_task_number(&mut self, limits: &Limits, task: u64) {
        touch_number(self, limits, task);
    }

    /// Gather this party's frame, inbox and conversations after owned facts change.
    pub fn touch_party(&mut self, party: u64) {
        self.start_input();
        self.touch_views(Touch::Party(party));
    }

    /// The configured creation period governs every project's current pool join.
    pub(crate) fn touch_frames(&mut self) {
        self.touch_views(Touch::Frames);
    }

    /// Gather every settings watch after the inventory or its holders changes.
    pub fn touch_settings(&mut self) {
        self.start_input();
        self.touch_views(Touch::Settings);
    }

    /// Gather deployment facts after host, account or capacity changes.
    pub fn touch_deployment(&mut self) {
        self.start_input();
        self.touch_views(Touch::Deployment);
    }

    /// Host or account changes can replace a due agent's next-run wait.
    pub(crate) fn touch_run_waits(&mut self) {
        for (_, flight) in &self.flights {
            let crate::routing::flight::Flight::Watch { binding, .. } = flight else { continue };
            let Subject::Task { task } = &self.view_names.get(binding.subject).expect("bound subject").subject else {
                continue;
            };
            let Some(row) = self.tasks.task(*task) else { continue };
            if let tasks::Executor::Agent { .. } = row.executor
                && let tasks::Phase::Active(tasks::Active::Due | tasks::Active::Preparing) = row.phase
            {
                let _new = self.changed_views.insert(binding.subject).expect("bounded watched marks");
            }
        }
    }

    /// An asker's own sign-in, or shared source state that can serve every party.
    pub(crate) fn touch_accounts(&mut self, party: Option<u64>) {
        self.touch_views(Touch::Accounts(party));
    }

    fn touch_views(&mut self, touch: Touch) {
        for (_, flight) in &self.flights {
            let crate::routing::flight::Flight::Watch { binding, .. } = flight else { continue };
            let subject = &self.view_names.get(binding.subject).expect("bound subject").subject;
            if subject.touched_by(touch) {
                let _new = self.changed_views.insert(binding.subject).expect("bounded watched marks");
            }
        }
    }

    pub(crate) fn touch_project_parties(&mut self, project: u32) {
        self.touch_project_objects(project);
        let mut parties = List::with_capacity(self.watch_flights());
        for (_, flight) in &self.flights {
            let crate::routing::flight::Flight::Watch { binding, .. } = flight else { continue };
            if self.people.role(binding.party, project).is_some() {
                parties.push(binding.party).expect("one party per watch");
            }
        }
        for party in parties.into_boxed() {
            self.touch_party(party);
        }
    }

    /// Role, policy and funding facts govern offers on every project object.
    pub(crate) fn touch_project_objects(&mut self, project: u32) {
        for (_, flight) in &self.flights {
            let crate::routing::flight::Flight::Watch { binding, .. } = flight else { continue };
            let subject = &self.view_names.get(binding.subject).expect("bound subject").subject;
            let touched = match subject {
                Subject::Project { project: owned } => *owned == project,
                // Store-only note names do not retain their scope in this core.
                Subject::Query { query } => query.project == Some(project),
                Subject::Note { .. } => true,
                Subject::Task { task }
                | Subject::Tree { task }
                | Subject::Run { task }
                | Subject::Escalation { task }
                | Subject::Question { task, .. }
                | Subject::PersonTask { task }
                | Subject::Result { task }
                | Subject::Proposal { proposer: Proposer::Task(task), .. } => match self.tasks.task(*task) {
                    Some(row) => row.project == project,
                    None => false,
                },
                Subject::Proposal { proposer: Proposer::Party(party), number } => {
                    match self.tasks.person_proposal(*party, *number) {
                        Some(row) => row.project == project,
                        None => false,
                    }
                }
                Subject::Chips { tasks } => {
                    let mut touched = false;
                    for task in tasks {
                        touched |= match self.tasks.task(*task) {
                            Some(row) => row.project == project,
                            None => false,
                        };
                    }
                    touched
                }
                Subject::Terminal { .. } | Subject::Accounts { .. }
                | Subject::Frame { .. }
                | Subject::Inbox { .. }
                | Subject::Settings
                | Subject::Deployment
                | Subject::Application { .. } => false,
            };
            if touched {
                let _new = self.changed_views.insert(binding.subject).expect("bounded watched marks");
            }
        }
    }
}

pub(crate) fn touch_task(core: &mut Core, limits: &Limits, row: &tasks::TaskRecord) {
    if core.watch_flights() == 0 {
        return;
    }
    core.touch_deployment();
    let mut trees = List::with_capacity(limits.tasks.depth.checked_add(1).expect("bounded lineage"));
    trees.push(row.number).expect("a task is in its own tree");
    let mut requester = row.requester;
    for _ in 0..limits.tasks.depth {
        requester = match requester {
            tasks::Party::Task(parent) => {
                trees.push(parent).expect("bounded task lineage");
                match core.tasks.delegation(parent) {
                    Some(parent) => parent.requester,
                    None => break,
                }
            }
            tasks::Party::Person(_) | tasks::Party::Deployment { .. } => break,
        };
    }
    for (_, flight) in &core.flights {
        let crate::routing::flight::Flight::Watch { binding, .. } = flight else { continue };
        let subject = &core.view_names.get(binding.subject).expect("bound subject").subject;
        let touched = match subject {
            Subject::Task { task } | Subject::Tree { task } => trees.as_slice().contains(task),
            Subject::Run { task }
            | Subject::Escalation { task }
            | Subject::Question { task, .. }
            | Subject::PersonTask { task }
            | Subject::Result { task }
            | Subject::Proposal { proposer: Proposer::Task(task), .. } => *task == row.number,
            Subject::Project { project } => *project == row.project,
            Subject::Query { query } => {
                let mut matches = documents::task_query::may_touch(query, row);
                for number in &trees {
                    if let Some(ancestor) = core.tasks.task(*number) {
                        matches |= documents::task_query::may_touch(query, ancestor);
                    }
                }
                matches
            }
            Subject::Chips { tasks } => tasks.as_slice().contains(&row.number),
            Subject::Inbox { party } => core.people.role(*party, row.project).is_some(),
            Subject::Terminal { .. } | Subject::Accounts { .. }
            | Subject::Frame { .. }
            | Subject::Settings
            | Subject::Deployment
            | Subject::Note { .. }
            | Subject::Application { .. }
            | Subject::Proposal { proposer: Proposer::Party(_), .. } => false,
        };
        if touched {
            let _new = core.changed_views.insert(binding.subject).expect("bounded watched marks");
        }
    }
}

pub(crate) fn touch_number(core: &mut Core, limits: &Limits, task: u64) {
    if core.watch_flights() == 0 {
        return;
    }
    // The row belongs to the hub; this temporary copy ends in this call.
    let row = match core.tasks.task(task) {
        Some(row) => row.clone(),
        None => return,
    };
    touch_task(core, limits, &row);
}

pub(crate) fn touch_people_row(core: &mut Core, row: &people::Stored) {
    match row {
        people::Stored::Person { number, .. }
        | people::Stored::ReadResults { party: number, .. }
        | people::Stored::EndedRead { party: number, .. }
        | people::Stored::ReadConversation { party: number, .. } => {
            core.touch_party(*number);
        }
        people::Stored::Roles { holdings, .. } => {
            for holding in holdings {
                core.touch_party(holding.person);
            }
        }
        people::Stored::Answer { .. } | people::Stored::Acted { .. } => core.touch_deployment(),
        people::Stored::SignIn { .. }
        | people::Stored::Deletion { .. }
        | people::Stored::Deleted { .. }
        | people::Stored::Public { .. }
        | people::Stored::Policy { .. } => {}
    }
}

pub(crate) fn touch_people_event(core: &mut Core, limits: &Limits, event: &people::Event) {
    match event {
        people::Event::Roles { project, .. } => {
            core.touch_accounts(None);
            core.touch_deployment();
            core.touch_project_parties(*project);
        }
        people::Event::ChatState { party, .. } => core.touch_party(*party),
        people::Event::MoveChat { from, to, .. } => {
            core.touch_party(*from);
            core.touch_party(*to);
        }
        people::Event::ConversationSeen { request, .. } => {
            if let Some((party, _)) = core.people.seen_context(*request) {
                core.touch_party(party);
            }
        }
        people::Event::Waiting { task, entries } => {
            let mut parties = List::with_capacity(core.watch_flights());
            for (_, flight) in &core.flights {
                let crate::routing::flight::Flight::Watch { binding, .. } = flight else { continue };
                let old = core.people.cached_waiting(&limits.people, binding.party);
                let mut touched = core.people.task_needs(binding.party, *task)
                    || match old {
                        Some(old) => {
                            let mut present = false;
                            for entry in &old {
                                present |= entry.task == *task;
                            }
                            present
                        }
                        None => false,
                    };
                for entry in entries {
                    touched |= match entry.whom {
                        people::Whom::Person(party) => party == binding.party,
                        people::Whom::Role { project, role } => match core.people.role(binding.party, project) {
                            Some(held) => held.number() == role,
                            None => false,
                        },
                    };
                }
                if touched {
                    parties.push(binding.party).expect("bounded watched parties");
                }
            }
            for party in parties.into_boxed() {
                core.touch_party(party);
            }
        }
        people::Event::Forget { party, .. } => {
            if let Some(party) = party {
                core.touch_party(*party);
            }
        }
        people::Event::Deleting { .. }
        | people::Event::CloseProject { .. }
        | people::Event::ForgetProject { .. }
        | people::Event::MakeService { .. }
        | people::Event::Seed { .. }
        | people::Event::ApplyRoles { .. }
        | people::Event::SignedIn { .. }
        | people::Event::Whose { .. }
        | people::Event::SignOut { .. }
        | people::Event::Act { .. }
        | people::Event::Ask { .. }
        | people::Event::Decided { .. }
        | people::Event::Restore { .. } => {}
    }
}

#[derive(Debug)]
pub(crate) enum Reconciliation {
    Ended { run: Token },
    Committed { run: Token, attempt: u64, turn: u32 },
}

fn reconcile(core: &Core, token: Token, retained: &views::RetainedRun) -> Option<Reconciliation> {
    let (attempt, position, committed) = retained.metadata();
    match core.proofs.get(&token.raw()) {
        Some(proof) if proof.terminal.is_none() && attempt == proof.attempt => match position {
            Some(writing) if writing.attempt != proof.attempt => Some(Reconciliation::Ended { run: token }),
            Some(writing) => match proof.turn {
                Some(kept) if writing.turn <= kept.turn && !committed => {
                    Some(Reconciliation::Committed { run: token, attempt: proof.attempt, turn: kept.turn })
                }
                Some(_) | None => None,
            },
            None => None,
        },
        Some(_) | None => Some(Reconciliation::Ended { run: token }),
    }
}

pub(crate) fn retention_owed(core: &Core) -> bool {
    for (token, retained) in core.views.retained_run_states() {
        if reconcile(core, *token, retained).is_some() {
            return true;
        }
    }
    false
}

fn timeless_ending(core: &Core, watcher: Token, binding: &crate::WatchBinding) -> Option<views::End> {
    if core.stopping {
        Some(views::End::Stopping)
    } else if core.people.sign_in_person(binding.sign_in) != Some(binding.party) {
        Some(views::End::SignedOut)
    } else {
        visibility_end(core, watcher, binding)
    }
}

pub(crate) fn endings_owed(core: &Core) -> bool {
    for (id, flight) in &core.flights {
        let crate::routing::flight::Flight::Watch { binding, .. } = flight else { continue };
        let watcher = &id.token();
        if !core.views.is_closing(*watcher) && timeless_ending(core, *watcher, binding).is_some() {
            return true;
        }
    }
    false
}

fn ending(core: &Core, watcher: Token, binding: &crate::WatchBinding, env: &Env<Limits>) -> Option<views::End> {
    if core.stopping {
        Some(views::End::Stopping)
    } else if core.people.person(binding.sign_in, env.now, env.wall) != Some(binding.party) {
        Some(views::End::SignedOut)
    } else {
        timeless_ending(core, watcher, binding)
    }
}

pub(crate) fn finish_watches(core: &mut Core, env: &Env<Limits>) -> Queue<Request> {
    let reconcile_runs = retention_owed(core);
    if core.changed_views.is_empty() && !reconcile_runs {
        return end_invalid(core, env);
    }
    let mut out = Queue::uncapped();
    append(end_invalid(core, env), &mut out);
    // Retained runs are live independently of watches.
    // A committed turn stops being provisional; only an ended attempt frees
    // its retained-run admission. Both are fenced by the core's durable proof.
    if reconcile_runs {
        let mut pending = List::with_capacity(core.views.runs());
        for (token, retained) in core.views.retained_run_states() {
            if let Some(event) = reconcile(core, *token, retained) {
                pending.push(event).expect("one reconciliation per retained run");
            }
        }
        for event in &pending {
            let event = match event {
                Reconciliation::Ended { run } => views::Event::RunEnded { run: *run },
                Reconciliation::Committed { run, attempt, turn } => {
                    views::Event::TurnCommitted { run: *run, attempt: *attempt, turn: *turn }
                }
            };
            let mut quiet = Queue::uncapped();
            crate::routing::step::step_views(core, env, event, &mut quiet);
            assert!(quiet.is_empty(), "ending writing emits no output");
        }
    }
    let changed = core::mem::replace(&mut core.changed_views, Set::with_capacity(env.limits.views.watchers));
    for name in &changed {
        let mut subjects = List::with_capacity(1);
        subjects.push(name.token()).expect("one opaque subject mark");
        append(super::routes::route_view(core, env, views::Event::Changed { subjects }), &mut out);
    }
    out
}

pub(crate) fn end_invalid(core: &mut Core, env: &Env<Limits>) -> Queue<Request> {
    let mut any = core.draining == Some(true) && (core.watch_flights() == 0);
    for (id, flight) in &core.flights {
        let crate::routing::flight::Flight::Watch { binding, .. } = flight else { continue };
        let watcher = &id.token();
        if !core.views.is_closing(*watcher) && ending(core, *watcher, binding, env).is_some() {
            any = true;
            break;
        }
    }
    if !any {
        return Queue::uncapped();
    }
    let mut out = Queue::uncapped();
    let mut pending = skein_lib::Map::with_capacity(core.watch_flights());
    for (id, flight) in &core.flights {
        let crate::routing::flight::Flight::Watch { watcher: external, binding } = flight else { continue };
        let watcher = &id.token();
        if !core.views.is_closing(*watcher)
            && let Some(end) = ending(core, *watcher, binding, env)
        {
            pending.insert(*external, (*watcher, end)).expect("bounded watch endings");
        }
    }
    for (_, &(watcher, end)) in &pending {
        append(super::routes::route_view(core, env, views::Event::EndWatch { watcher, end }), &mut out);
    }
    crate::routing::stopping::drained(core, &mut out);
    out
}

fn append(requests: Queue<Request>, out: &mut Queue<Request>) {
    let mut requests = requests;
    for _ in 0..requests.len() {
        let request = requests.pop().expect("watch route outputs");
        out.push(request);
    }
}

fn visibility_end(core: &Core, watcher: Token, binding: &crate::WatchBinding) -> Option<views::End> {
    let subject = &core.view_names.get(binding.subject).expect("bound subject").subject;
    match subject {
        Subject::Application { scope, .. } => {
            if *scope == ApplicationScope::Own || core.people.role(binding.party, scope.project()).is_some() {
                None
            } else {
                Some(views::End::NotAllowed)
            }
        }
        Subject::Task { task } => {
            let Some(row) = core.tasks.task(*task) else {
                return if core.views.is_making(watcher) { None } else { Some(views::End::Gone) };
            };
            match row.phase {
                tasks::Phase::Ended(_) if core.views.is_making(watcher) => {
                    if core.open_project(row.project) && core.people.role(binding.party, row.project).is_some()
                        || documents::task_tree::person_funded(row, binding.party)
                    {
                        None
                    } else {
                        Some(views::End::NotAllowed)
                    }
                }
                tasks::Phase::Ended(_) => Some(views::End::Gone),
                tasks::Phase::Waiting
                | tasks::Phase::Active(_)
                | tasks::Phase::Closing(_)
                | tasks::Phase::Held { .. } => {
                    if core.open_project(row.project) && core.people.role(binding.party, row.project).is_some()
                        || documents::task_tree::person_funded(row, binding.party)
                    {
                        None
                    } else {
                        Some(views::End::NotAllowed)
                    }
                }
            }
        }
        Subject::Tree { task } | Subject::Run { task } => {
            let Some(row) = core.tasks.task(*task) else {
                return if core.views.is_making(watcher) { None } else { Some(views::End::Gone) };
            };
            match row.phase {
                tasks::Phase::Ended(_) if core.views.is_making(watcher) => {
                    if core.people.role(binding.party, row.project).is_some() {
                        None
                    } else {
                        Some(views::End::NotAllowed)
                    }
                }
                tasks::Phase::Ended(_) => Some(views::End::Gone),
                tasks::Phase::Waiting
                | tasks::Phase::Active(_)
                | tasks::Phase::Closing(_)
                | tasks::Phase::Held { .. } => {
                    if core.people.role(binding.party, row.project).is_some() {
                        None
                    } else {
                        Some(views::End::NotAllowed)
                    }
                }
            }
        }
        Subject::Chips { tasks: named } => {
            for task in named {
                if let Some(row) = core.tasks.task(*task)
                    && core.people.role(binding.party, row.project).is_none()
                {
                    return Some(views::End::NotAllowed);
                }
            }
            None
        }
        Subject::Query { query } => match documents::task_query::admission(core, binding.party, query) {
            Ok(()) => None,
            Err(_) => Some(views::End::NotAllowed),
        },
        Subject::Settings | Subject::Deployment => {
            if core.people.role(binding.party, people::DEPLOYMENT).is_some() {
                None
            } else {
                Some(views::End::NotAllowed)
            }
        }
        Subject::Frame { .. } | Subject::Inbox { .. } | Subject::Terminal { .. } | Subject::Accounts { .. } => None,
        Subject::Proposal { .. }
        | Subject::Escalation { .. }
        | Subject::Question { .. }
        | Subject::PersonTask { .. }
        | Subject::Result { .. }
        | Subject::Project { .. }
        | Subject::Note { .. } => Some(views::End::Gone),
    }
}

/// A revision moves the settings watch; all documents derived from templates or
/// future-project policies see one mark.
pub(crate) fn settings_revised(core: &mut Core, _work: &mut Queue<Work>, templates_changed: bool) {
    core.touch_settings();
    if templates_changed {
        core.touch_frames();
    }
}
