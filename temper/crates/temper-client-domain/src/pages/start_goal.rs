//! Starting a goal from a chat's draft (client/README.md, 5.14).
use super::{Pages, Request, goal, needs::Toasted};
use crate::{Address, Limits, owner::Owner};
use alloc::boxed::Box;
use jig_client::{Client, Key, Outcome, Told, contract as c, shapes};
use skein_lib::{Env, Queue, Writer};

#[derive(Debug)]
#[expect(clippy::partial_pub_fields, reason = "fields are sheet state; the sent judgement is private correlation")]
pub struct StartGoal {
    pub project: u32,
    pub given: Option<Box<[u8]>>,
    pub chat: Option<u64>,
    pub draft: Box<[u8]>,
    pub name: Box<[u8]>,
    pub way: Option<u32>,
    pub pending: Option<Key>,
    pub error: Option<Failure>,
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

/// The goal's title and draft provenance (client/README.md, 5.14).
#[must_use]
pub fn words(name: &[u8], path: &[u8], chat: Option<u64>) -> Box<[u8]> {
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

pub(super) fn open(pages: &mut Pages, client: &Client, draft: Option<Box<[u8]>>, chat: Option<u64>) {
    if pages.start.is_some() {
        return;
    }
    let (Some(me), Some(draft), Some(chat)) = (pages.me(client), draft, chat) else { return };
    let project = if let Some(page) = &pages.chat {
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
    let draft: Box<[u8]> = draft.trim_ascii().into();
    if draft.is_empty() {
        return;
    }
    let file = goal::file_of(&draft);
    let name = file.strip_suffix(b".md").unwrap_or(file).into();
    let way = default_way(me, project);
    pages.start = Some(StartGoal {
        project,
        given: Some(draft.clone()),
        chat: Some(chat),
        draft,
        name,
        way,
        pending: None,
        error: None,
        sent: None,
    });
}

impl StartGoal {
    pub(super) fn name(&mut self, words: Box<[u8]>) {
        if self.pending.is_none() {
            self.name = words;
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
        if self.pending.is_some() || self.name.trim_ascii().is_empty() || self.draft.is_empty() {
            return;
        }
        let Some(me) = me else { return };
        let Some(way) = self.way else {
            self.error = Some(Failure::NotOffered(shapes::NotOffered::Template));
            return;
        };
        let bound = env.limits.client.domain.text;
        if c::Text::new(self.name.clone(), bound).is_err() || c::Text::new(self.draft.clone(), bound).is_err() {
            self.error = Some(Failure::TooLong);
            return;
        }
        let Ok(words) = c::Text::new(words(self.name.trim_ascii(), &self.draft, self.chat), bound) else {
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
                    out.push(Request::Show { address: Address::Task { task: *first } });
                    pages.start = None;
                    return true;
                }
            }
            Some(Sent::Propose(to)) => {
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
