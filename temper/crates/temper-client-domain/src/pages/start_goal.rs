//! Starting a goal from its draft (client/pages/goal.md, 3).
use super::{Pages, Request, goal, needs::Toasted};
use crate::{Address, Limits, owner::Owner};
use alloc::boxed::Box;
use jig_client::{Client, Key, Outcome, Told, contract as c, shapes};
use skein_lib::{Env, Queue, Writer};

#[derive(Debug)]
#[expect(
    clippy::partial_pub_fields,
    reason = "fields are sheet state; correlation and name-following state stay private"
)]
pub struct StartGoal {
    pub project: u32,
    pub given: Option<Box<[u8]>>,
    pub chat: Option<u64>,
    pub landed: Option<super::work::LandedDraft>,
    pub draft: Box<[u8]>,
    pub name: Box<[u8]>,
    pub way: Option<u32>,
    pub default: Option<u32>,
    pub body: Option<Box<[u8]>>,
    pub always: bool,
    pub pending: Option<Key>,
    pub error: Option<Failure>,
    named: bool,
    sent: Option<Sent>,
}
#[derive(Debug)]
pub enum Failure {
    Refused(c::Refusal),
    NotOffered(shapes::NotOffered),
    Outcome(Outcome),
    TooLong,
}
#[derive(Debug)]
enum Sent {
    Create,
    Propose(c::Recipient),
}

/// The goal's title and draft provenance (client/pages/goal.md, 3).
#[must_use]
pub fn words(name: &[u8], path: &[u8], chat: Option<u64>, always: bool) -> Box<[u8]> {
    let path = path.trim_ascii();
    let mut size = name.len().checked_add(path.len().checked_mul(2).expect("draft words fit")).expect("goal words fit");
    size = size.checked_add(b"\n\nBuild .\nDraft: ".len()).expect("goal words fit");
    let number = match chat {
        Some(number) => Some(skein_lib::Decimal::of(number)),
        None => None,
    };
    if let Some(number) = &number {
        size = size.checked_add(b"\nChat: T".len()).expect("chat words fit");
        size = size.checked_add(number.as_bytes().len()).expect("chat words fit");
    }
    if always {
        size = size.checked_add(b"\nAsks: always".len()).expect("asks words fit");
    }
    let mut words = Writer::new(size);
    words.put(name).expect("measured name");
    words.put(b"\n\nBuild ").expect("measured build");
    words.put(path).expect("measured path");
    words.put(b".\nDraft: ").expect("measured draft");
    words.put(path).expect("measured path");
    if let Some(number) = number {
        words.put(b"\nChat: T").expect("measured chat");
        words.put(number.as_bytes()).expect("measured chat number");
    }
    if always {
        words.put(b"\nAsks: always").expect("measured asks");
    }
    words.finish()
}

/// The first tracked way in template-number order when no project setting is supplied.
#[must_use]
pub fn default_way(me: &c::Me, project: u32) -> Option<u32> {
    let mut first = None;
    for member in &me.projects {
        if member.project == project {
            for template in &member.templates {
                if template.tracked.is_some() {
                    first = Some(match first {
                        Some(number) => template.number.min(number),
                        None => template.number,
                    });
                }
            }
        }
    }
    first
}

/// Nothing to fund a goal, and why (client/pages/goal.md, 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unfunded {
    Held,
    Empty,
}

#[must_use]
pub fn unfunded(me: &c::Me, project: u32, way: Option<u32>) -> Option<Unfunded> {
    if let Some(way) = way
        && let Some(template) = shapes::template(me, project, way)
    {
        match template.budget {
            c::TemplateBudget::Amount(_) => return None,
            c::TemplateBudget::AsAllowed { .. } | c::TemplateBudget::Given => {}
        }
    }
    for member in &me.projects {
        if member.project == project
            && let Some(creating) = &member.creating
        {
            if creating.within.budget != 0 {
                return None;
            }
            if let Some(pool) = &member.pool
                && pool.numbers.reserved > 0
            {
                return Some(Unfunded::Held);
            }
            return Some(Unfunded::Empty);
        }
    }
    None
}

pub(crate) fn preferred_way(me: &c::Me, project: u32, configured: Option<u32>) -> Option<u32> {
    if let Some(number) = configured
        && let Some(template) = shapes::template(me, project, number)
        && template.tracked.is_some()
    {
        return Some(number);
    }
    default_way(me, project)
}

pub(super) fn open(pages: &mut Pages, client: &Client, draft: Option<Box<[u8]>>, chat: Option<u64>) {
    if pages.start.is_some() {
        return;
    }
    let Some(me) = pages.me(client) else { return };
    let (mut draft, given) = match (draft, chat) {
        (Some(path), Some(_)) => (path, true),
        (None, None) => (Box::default(), false),
        (Some(_), None) | (None, Some(_)) => return,
    };
    let project = if !given {
        super::conversation_project(pages)
    } else if let Some(page) = &pages.chat {
        match page.task(client) {
            Some(task) => Some(task.project),
            None => None,
        }
    } else if let Some(page) = &pages.task {
        match page.document(client) {
            Some(task) => Some(task.project),
            None => None,
        }
    } else {
        super::conversation_project(pages)
    };
    let Some(project) = project else { return };
    let body = if given && let Some(page) = &pages.chat {
        let Some(task) = page.task(client) else { return };
        let Some(document) = &page.draft else { return };
        if Some(page.task) != chat || document.body.is_empty() {
            return;
        }
        let mut creating = false;
        for member in &me.projects {
            if member.project == task.project && member.creating.is_some() {
                creating = true;
            }
        }
        if !creating {
            return;
        }
        draft = super::chat::sheet::path(&super::chat::document::slug(&document.title));
        Some(document.body.clone())
    } else {
        None
    };
    let draft: Box<[u8]> = draft.trim_ascii().into();
    if given && draft.is_empty() {
        return;
    }
    let file = goal::file_of(&draft);
    let name = file.strip_suffix(b".md").unwrap_or(file).into();
    let way = default_way(me, project);
    pages.start = Some(StartGoal {
        project,
        given: if given { Some(draft.clone()) } else { None },
        chat,
        landed: None,
        draft,
        name,
        way,
        default: way,
        body,
        always: false,
        pending: None,
        error: None,
        named: false,
        sent: None,
    });
}

pub(super) fn open_landed(
    pages: &mut Pages,
    env: &Env<Limits>,
    client: &Client,
    draft: super::work::LandedDraft,
    run: bool,
    out: &mut Queue<Request>,
) {
    if pages.start.is_some() || !draft.listed {
        return;
    }
    let Some(project) = super::work_project(&pages.address) else { return };
    open(pages, client, None, None);
    let me = pages.me(client);
    let Some(start) = &mut pages.start else { return };
    let file = goal::file_of(&draft.path);
    start.project = project;
    start.name = file.strip_suffix(b".md").unwrap_or(file).into();
    start.given = Some(draft.path.clone());
    start.draft.clone_from(&draft.path);
    start.landed = Some(draft);
    if run {
        start.send(env, me, &mut pages.serial, out);
    }
}
impl StartGoal {
    pub(super) fn draft_field(&mut self, words: Box<[u8]>) {
        if self.given.is_some() || self.pending.is_some() {
            return;
        }
        if !self.named {
            let file = goal::file_of(words.trim_ascii());
            self.name = file.strip_suffix(b".md").unwrap_or(file).into();
        }
        self.draft = words;
        self.error = None;
    }

    pub(super) fn name(&mut self, words: Box<[u8]>) {
        if self.pending.is_none() {
            self.named = true;
            self.name = words;
            self.error = None;
        }
    }
    pub(super) fn asks(&mut self, always: bool) {
        if self.pending.is_none() {
            self.always = always;
            self.error = None;
        }
    }
    pub(super) fn way(&mut self, me: Option<&c::Me>, number: u32) {
        if self.pending.is_none()
            && let Some(me) = me
            && let Some(template) = shapes::template(me, self.project, number)
            && template.tracked.is_some()
        {
            self.way = Some(number);
            self.error = None;
        }
    }
    pub(super) fn send(&mut self, env: &Env<Limits>, me: Option<&c::Me>, serial: &mut u64, out: &mut Queue<Request>) {
        if self.pending.is_some() || self.name.trim_ascii().is_empty() || self.draft.trim_ascii().is_empty() {
            return;
        }
        let Some(me) = me else { return };
        if unfunded(me, self.project, self.way).is_some() {
            return;
        }
        let Some(way) = self.way else {
            self.error = Some(Failure::NotOffered(shapes::NotOffered::Template));
            return;
        };
        self.draft = self.draft.trim_ascii().into();
        let bound =
            if self.body.is_some() { env.limits.client.domain.long_text } else { env.limits.client.domain.text };
        if c::Text::new(self.name.clone(), bound).is_err() || c::Text::new(self.draft.clone(), bound).is_err() {
            self.error = Some(Failure::TooLong);
            return;
        }
        let mut bytes = if let Some(body) = &self.body {
            super::chat::sheet::snapshot(self.name.trim_ascii(), &self.draft, body, self.chat)
        } else {
            words(self.name.trim_ascii(), &self.draft, self.chat, self.always)
        };
        if self.body.is_some() && self.always {
            let mut writer = Writer::new(bytes.len().checked_add(b"\nAsks: always".len()).expect("asks words fit"));
            writer.put(&bytes).expect("measured snapshot");
            writer.put(b"\nAsks: always").expect("measured asks");
            bytes = writer.finish();
        }
        let Ok(words) = c::Text::new(bytes, bound) else {
            self.error = Some(Failure::TooLong);
            return;
        };
        let executor = match shapes::template(me, self.project, way) {
            Some(template) => match template.executor {
                c::TemplateExecutor::Charter(charter) => c::Executor::Agent { charter },
                c::TemplateExecutor::Chosen => c::Executor::Agent { charter: 1 },
            },
            None => c::Executor::Agent { charter: 1 },
        };
        let mut budget = 0;
        for member in &me.projects {
            if member.project == self.project
                && let Some(creating) = &member.creating
            {
                budget = creating.within.budget;
            }
        }
        let draft = shapes::GoalDraft { words, executor, budget, deadline: None, priority: None, llm: None };
        let reason = c::Text::new(b"Started from its draft".as_slice().into(), bound);
        let Ok(reason) = reason else {
            self.error = Some(Failure::TooLong);
            return;
        };
        let (ask, sent) = match shapes::set_goal_from(me, self.project, way, draft, reason) {
            shapes::Judged::Create(ask) => (ask, Sent::Create),
            shapes::Judged::Propose { ask, to } => (ask, Sent::Propose(to)),
            shapes::Judged::Refused(why) => {
                self.error = Some(Failure::Refused(why));
                return;
            }
            shapes::Judged::NotOffered(why) => {
                self.error = Some(Failure::NotOffered(why));
                return;
            }
        };
        *serial = serial.checked_add(1).expect("page keys fit");
        let key = Key { asker: Owner::Pages.asker(), number: *serial };
        self.pending = Some(key);
        self.sent = Some(sent);
        self.error = None;
        out.push(Request::Client(jig_client::Event::Send {
            key,
            name: None,
            request: jig_client::Ask::Jig(Box::new(ask)),
        }));
    }
}

pub(super) fn told(
    pages: &mut Pages,
    env: &Env<Limits>,
    client: &Client,
    told: &Told,
    out: &mut Queue<Request>,
) -> bool {
    if let Told::Answered { key, .. } = told
        && pages.draft_starts.remove(key)
    {
        return true;
    }
    let Some(start) = &mut pages.start else { return false };
    let Some(pending) = start.pending else { return false };
    let outcome = match told {
        Told::Answered { key, outcome } if *key == pending => outcome,
        Told::Pending { key } if *key == pending => {
            match client.pending(pending) {
                Some(pending) if pending.progress == jig_client::Progress::Unknown => {}
                Some(_) | None => return true,
            }
            &Outcome::Unknown
        }
        Told::Link { .. }
        | Told::Pending { .. }
        | Told::Answered { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. }
        | Told::Changed { .. }
        | Told::Text { .. }
        | Told::Ended { .. }
        | Told::Page { .. } => return false,
    };
    start.pending = None;
    let sent = start.sent.take();
    if let Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(done))) = outcome {
        match sent {
            Some(Sent::Create) => {
                if let c::Outcome::Document(c::DocumentOutcome::Created { first, .. }) = done {
                    if let Some(chat) = start.chat {
                        let what = crate::forge::dismissed::PersonFact::GoalFrom {
                            project: start.project,
                            chat,
                            goal: *first,
                            at: env.wall.as_nanos(),
                        };
                        if let Some(request) = crate::forge::dismissed::ask(what, &env.limits.forge) {
                            pages.serial = pages.serial.checked_add(1).expect("page keys fit");
                            let key = Key { asker: Owner::Pages.asker(), number: pages.serial };
                            out.push(Request::Client(jig_client::Event::Send { key, name: None, request }));
                        }
                    }
                    if let Some(draft) = &start.landed {
                        mark_started(
                            start.project,
                            draft,
                            *first,
                            env,
                            &mut pages.serial,
                            &mut pages.draft_starts,
                            out,
                        );
                    }
                    out.push(Request::Show { address: Address::Task { task: *first } });
                    pages.start = None;
                    return true;
                }
            }
            Some(Sent::Propose(to)) => {
                if let Some(chat) = &mut pages.chat {
                    chat.proposed = Some(to.clone());
                }
                pages.needs.say(Toasted::Proposed { to }, env.now);
                pages.start = None;
                return true;
            }
            None => {}
        }
    }
    start.error = Some(Failure::Outcome(outcome.clone()));
    true
}

fn mark_started(
    project: u32,
    draft: &super::work::LandedDraft,
    goal: u64,
    env: &Env<Limits>,
    serial: &mut u64,
    pending: &mut skein_lib::Set<Key>,
    out: &mut Queue<Request>,
) {
    use temper_client_protocol_wire::v1 as w;
    let Ok(repository) = w::DraftRepository::new(
        &w::CEILINGS,
        w::DraftRepositoryParts { owner: draft.owner.clone(), name: draft.name.clone() },
    ) else {
        return;
    };
    let change = w::DraftChange::Started(
        w::StartedDraft::new(&w::CEILINGS, w::StartedDraftParts { goal }).expect("goal number fits"),
    );
    if let Some(request) =
        crate::forge::drafts::ask(project, Some(repository), draft.path.clone(), change, &env.limits.forge)
    {
        *serial = serial.checked_add(1).expect("page keys fit");
        let key = Key { asker: Owner::Pages.asker(), number: *serial };
        pending.insert(key).expect("one key per draft start");
        out.push(Request::Client(jig_client::Event::Send { key, name: None, request }));
    }
}
