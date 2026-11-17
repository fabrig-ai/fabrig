//! Memory-only changes provenance and archive continuations
//! (temper/docs/design/client/forge/records.md, Where each row came from).
#[cfg(test)]
mod tests;
use crate::{
    Decision, Domain, Limits,
    boundary::{EndedRead, Read, Work},
};
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Map, Token};
use temper_engine_domain_forge as forge;

const KEPT: u32 = 4096;

#[derive(Clone, Hash, Debug)]
struct From {
    goal: Option<forge::ChangeFrom>,
    chat: Option<forge::ChangeFrom>,
    milestone: Option<forge::ChangeFrom>,
}

impl From {
    fn empty() -> Self {
        Self { goal: None, chat: None, milestone: None }
    }
}

#[derive(Hash, Debug)]
struct Entry {
    project: u32,
    used: u64,
    next: Option<u64>,
    milestone: Option<u64>,
    from: From,
}

impl Entry {
    fn new(project: u32, task: u64) -> Self {
        Self { project, used: 0, next: Some(task), milestone: None, from: From::empty() }
    }

    fn accept(&mut self, domain: &Domain, row: &tasks::TaskRecord) {
        if row.project != self.project {
            self.fail();
            return;
        }
        let title = jig_domain_core::documents::chip_title(row, domain.limits.documents.text_bytes.min(256));
        let named = forge::ChangeFrom { task: row.number, title: title.bytes };
        if self.milestone == Some(row.number) && tasks::gathering::executor(row.executor) {
            self.from.milestone = Some(named.clone());
        }
        if self.from.goal.is_none() && row.tracked.is_some() {
            self.from.goal = Some(named.clone());
            self.milestone = match row.requester {
                tasks::Party::Task(parent) => Some(parent),
                tasks::Party::Person(_) | tasks::Party::Deployment { .. } => None,
            };
        }
        self.next = match row.requester {
            tasks::Party::Task(parent) => Some(parent),
            tasks::Party::Person(_) | tasks::Party::Deployment { .. } => {
                for template in domain.core.project_templates(self.project) {
                    if template.number == 1 {
                        match template.executor {
                            jig_domain_people::TemplateExecutor::Charter(charter)
                                if row.executor == (tasks::Executor::Agent { charter }) =>
                            {
                                self.from.chat = Some(named.clone());
                            }
                            jig_domain_people::TemplateExecutor::Charter(_)
                            | jig_domain_people::TemplateExecutor::Chosen => {}
                        }
                    }
                }
                None
            }
        };
    }

    fn fail(&mut self) {
        self.next = None;
        self.from = From::empty();
    }
}

/// At most 4,096 change tasks, touched in document order; eviction discards no durable fact.
#[derive(Hash, Debug)]
pub(crate) struct Memory {
    rows: Map<u64, Entry>,
    clock: u64,
}

impl Memory {
    pub(crate) fn new() -> Self {
        Self { rows: Map::with_capacity(KEPT), clock: 0 }
    }

    fn touch(&mut self, task: u64) {
        self.clock = self.clock.saturating_add(1);
        if let Some(entry) = self.rows.get_mut(&task) {
            entry.used = self.clock;
            return;
        }
        if self.rows.len() < KEPT {
            return;
        }
        let mut oldest = None;
        for (task, entry) in &self.rows {
            let candidate = (entry.used, *task);
            oldest = Some(match oldest {
                Some(previous) => candidate.min(previous),
                None => candidate,
            });
        }
        let (_, task) = oldest.expect("full provenance table");
        self.rows.remove(&task);
    }
}

fn fit(bytes: &[u8]) -> Box<[u8]> {
    skein_lib::bytes::copy_of(skein_lib::bytes::utf8_prefix(bytes, 256))
}

fn live(domain: &Domain, project: u32, task: u64) -> Option<Entry> {
    let mut entry = Entry::new(project, task);
    for _ in 0..=domain.limits.tasks.depth {
        let Some(next) = entry.next else { return Some(entry) };
        entry.accept(domain, domain.core.tasks.task(next)?);
    }
    if entry.next.is_none() { Some(entry) } else { None }
}

fn get(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, project: u32, task: u64) -> From {
    domain.changes_from.touch(task);
    if let Some(entry) = live(domain, project, task) {
        domain.changes_from.rows.insert(task, entry).expect("touched provenance slot");
    }
    if !domain.changes_from.rows.contains_key(&task) {
        domain.changes_from.rows.insert(task, Entry::new(project, task)).expect("touched provenance slot");
    }
    let mut entry = domain.changes_from.rows.remove(&task).expect("provenance inserted");
    entry.used = domain.changes_from.clock;
    advance(domain, env, decision, &mut entry);
    let from = if entry.next.is_none() { entry.from.clone() } else { From::empty() };
    domain.changes_from.rows.insert(task, entry).expect("removed provenance slot");
    from
}

fn advance(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, entry: &mut Entry) {
    for _ in 0..=env.limits.tasks.depth {
        let Some(task) = entry.next else { return };
        if let Some(row) = domain.core.tasks.task(task) {
            entry.accept(domain, row);
            continue;
        }
        for (_, read) in &domain.archive_reads {
            if let Some(Read::Ended(EndedRead::ChangeFrom { task: pending })) = read
                && *pending == task
            {
                return;
            }
        }
        super::route::now::ended(domain, env, decision, EndedRead::ChangeFrom { task });
        return;
    }
    entry.fail();
}

/// Resume every change waiting for this ancestor; an absent or failed row leaves no names.
pub(crate) fn loaded(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    task: u64,
    row: Option<Box<tasks::TaskRecord>>,
) {
    let mut waiting = List::with_capacity(KEPT);
    for (change, entry) in &domain.changes_from.rows {
        if entry.next == Some(task) {
            waiting.push(*change).expect("one continuation per held change");
        }
    }
    let mut subjects = List::with_capacity(KEPT);
    for change in waiting.into_boxed() {
        let mut entry = domain.changes_from.rows.remove(&change).expect("held continuation");
        match &row {
            Some(row) if row.number == task => {
                entry.accept(domain, row);
                advance(domain, env, decision, &mut entry);
            }
            Some(_) | None => entry.fail(),
        }
        let subject = Token::new(forge::changes_subject(entry.project));
        if !subjects.as_slice().contains(&subject) {
            subjects.push(subject).expect("no more projects than changes");
        }
        domain.changes_from.rows.insert(change, entry).expect("removed continuation slot");
    }
    if !subjects.is_empty() {
        domain
            .work
            .push(Work::Core(jig_domain_core::Event::Marked { connector: domain.config.forge_connector, subjects }));
    }
}

/// The forge child knows neither task ancestry nor the project's people.
pub(crate) fn fill(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, view: &mut forge::ChangesView) {
    for queue in &mut view.queues {
        for row in &mut queue.changes {
            let from = get(domain, env, decision, view.project, row.task);
            row.goal = from.goal;
            row.chat = from.chat;
            row.milestone = from.milestone;
        }
        if let Some(row) = &mut queue.in_flight {
            let from = get(domain, env, decision, view.project, row.member);
            row.goal = from.goal;
            row.chat = from.chat;
            row.milestone = from.milestone;
        }
    }
    for rows in [&mut view.landed, &mut view.landed_default] {
        for row in rows.iter_mut() {
            let from = get(domain, env, decision, view.project, row.task);
            row.goal = from.goal;
            row.chat = from.chat;
            row.milestone = from.milestone;
            for sign in &mut row.signed {
                sign.name = match domain.core.people.member_name(view.project, sign.person) {
                    Some(name) => Some(fit(name)),
                    None => None,
                };
            }
        }
    }
}
