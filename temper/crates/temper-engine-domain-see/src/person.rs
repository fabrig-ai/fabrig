//! Sequential reads as the chat's requester (domain/agent/ask-temper.md, 1; domain/routing.md, 1).
use crate::{Address, SeeNotRead, SeePerson, SeeRead, SeeWords};
use alloc::boxed::Box;
use jig_domain_core::{
    self as core, documents,
    reads::{Answer, Read, Why},
};
use jig_domain_people::ApplicationScope;
use skein_lib::{List, Token};
use temper_engine_domain_forge as forge;

/// An address's primary reads and its needs-page question words, kept whole until delivery.
#[derive(Hash, Debug)]
pub struct Gathered {
    pub connector: u16,
    pub first: Option<Part>,
    pub second: Option<Part>,
    pub words: List<SeeWords>,
}

#[derive(Hash, Debug)]
pub enum Part {
    Jig(Answer),
    Forge(Option<Box<forge::Document>>),
}

#[derive(PartialEq, Eq, Debug)]
#[expect(clippy::large_enum_variant, reason = "the transient step moves jig's read directly into its named action")]
pub enum Step {
    Read(Read),
    Deliver(Option<SeeNotRead>),
}

impl Gathered {
    #[must_use]
    pub fn new(connector: u16) -> Self {
        Self { connector, first: None, second: None, words: List::uncapped() }
    }

    pub fn push(&mut self, address: &Address, answer: Part) {
        if let Some((task, message)) = question(address, self) {
            self.words.push(SeeWords { task, message, words: jig(Some(answer)) }).expect("needs-page question");
            return;
        }
        if self.first.is_none() {
            let answer = match address {
                Address::Goals { .. } => goal_rows(answer),
                Address::Me
                | Address::Project
                | Address::Tasks
                | Address::Task { .. }
                | Address::Tree { .. }
                | Address::Activity { .. }
                | Address::Turn { .. }
                | Address::Change { .. }
                | Address::Changes
                | Address::Inbox
                | Address::Ways
                | Address::Inventory
                | Address::Setting(_)
                | Address::Landing
                | Address::Gates { .. }
                | Address::Revisions
                | Address::Revision { .. }
                | Address::Proposal { .. }
                | Address::Page { .. }
                | Address::Search { .. }
                | Address::File { .. }
                | Address::Unknown => answer,
            };
            self.first = Some(answer);
        } else {
            assert!(self.second.is_none(), "an address reads at most two parts");
            self.second = Some(answer);
        }
    }
}

/// Milestones use jig's tracked slots, but are not temper's goals (domain/milestones.md, 10).
fn goal_rows(mut part: Part) -> Part {
    match &mut part {
        Part::Jig(Answer::Tasks(page)) => {
            keep_goals(&mut page.live);
            match &mut page.ended {
                core::reads::EndedGroup::Rows { rows, .. } => {
                    keep_goals(rows);
                }
                core::reads::EndedGroup::None => {}
            }
        }
        Part::Jig(
            Answer::TaskText(_)
            | Answer::Waits(_)
            | Answer::Setting(_)
            | Answer::Revision(_)
            | Answer::Revisions(_)
            | Answer::Document(_)
            | Answer::Ended(_)
            | Answer::Page(_)
            | Answer::None(_),
        )
        | Part::Forge(_) => {}
    }
    part
}

fn keep_goals(rows: &mut List<documents::TaskLine>) {
    let all = ::core::mem::replace(rows, List::uncapped());
    for row in all.into_boxed() {
        if !jig_domain_tasks::gathering::executor(row.chip.details.executor) {
            rows.push(row).expect("filtered goal rows fit");
        }
    }
}

/// The next read depends only on the address, its project and the answers gathered so far.
#[must_use]
pub fn next(address: &Address, project: u32, gathered: &Gathered) -> Step {
    if let Some(first) = &gathered.first {
        if let Some(why) = refused(first) {
            return Step::Deliver(Some(why));
        }
        if let Some((task, message)) = question(address, gathered) {
            return Step::Read(Read::Question { task, message });
        }
        if gathered.second.is_some() {
            return Step::Deliver(None);
        }
        return match address {
            Address::Me => Step::Read(needs(project)),
            Address::Project => {
                let mut query = query(project, documents::QueryOrder::Number);
                query.phase = Some(documents::PhaseClass::Held);
                Step::Read(tasks(query))
            }
            Address::Goals { .. } => Step::Read(Read::Notes { project, from: None }),
            Address::Task { number } if tracked(first) => Step::Read(Read::ScopedNotes {
                scope: jig_domain_notes::Scope::Goal { project, goal: *number },
                description: None,
                from: None,
            }),
            Address::Tree { number } => Step::Read(tree(project, *number, true)),
            Address::Changes => {
                Step::Read(application(project, gathered.connector, forge::landings_subject(project, false)))
            }
            Address::Task { .. }
            | Address::Tasks
            | Address::Activity { .. }
            | Address::Turn { .. }
            | Address::Change { .. }
            | Address::Inbox => Step::Deliver(None),
            Address::Ways
            | Address::Inventory
            | Address::Setting(_)
            | Address::Landing
            | Address::Gates { .. }
            | Address::Revisions
            | Address::Revision { .. }
            | Address::Proposal { .. }
            | Address::Page { .. }
            | Address::Search { .. }
            | Address::File { .. }
            | Address::Unknown => unreachable!("a person's address"),
        };
    }
    Step::Read(match address {
        Address::Me | Address::Project => Read::Project { project },
        Address::Goals { ended } => {
            let mut query =
                query(project, if *ended { documents::QueryOrder::Ended } else { documents::QueryOrder::Priority });
            query.tracked = Some(true);
            if *ended {
                query.phase = Some(documents::PhaseClass::Ended);
            }
            tasks(query)
        }
        Address::Tasks => tasks(query(project, documents::QueryOrder::Activity)),
        Address::Task { number } => Read::Task { task: *number },
        Address::Tree { number } => tree(project, *number, false),
        Address::Activity { number } => Read::Activity { task: *number, before: None },
        Address::Turn { number, attempt, turn } => Read::Turn { task: *number, attempt: *attempt, turn: *turn },
        Address::Change { number } => application(project, gathered.connector, *number),
        Address::Changes => application(project, gathered.connector, forge::changes_subject(project)),
        Address::Inbox => needs(project),
        Address::Ways
        | Address::Inventory
        | Address::Setting(_)
        | Address::Landing
        | Address::Gates { .. }
        | Address::Revisions
        | Address::Revision { .. }
        | Address::Proposal { .. }
        | Address::Page { .. }
        | Address::Search { .. }
        | Address::File { .. }
        | Address::Unknown => unreachable!("a person's address"),
    })
}

/// Select only questions from the answered page, skipping the word reads already gathered.
fn question(address: &Address, gathered: &Gathered) -> Option<(u64, u64)> {
    let part = match address {
        Address::Me => gathered.second.as_ref(),
        Address::Inbox => gathered.first.as_ref(),
        Address::Project
        | Address::Goals { .. }
        | Address::Tasks
        | Address::Task { .. }
        | Address::Tree { .. }
        | Address::Activity { .. }
        | Address::Turn { .. }
        | Address::Change { .. }
        | Address::Changes
        | Address::Ways
        | Address::Inventory
        | Address::Setting(_)
        | Address::Landing
        | Address::Gates { .. }
        | Address::Revisions
        | Address::Revision { .. }
        | Address::Proposal { .. }
        | Address::Page { .. }
        | Address::Search { .. }
        | Address::File { .. }
        | Address::Unknown => None,
    };
    let page = match part? {
        Part::Jig(Answer::Page(page)) => page,
        Part::Jig(
            Answer::TaskText(_)
            | Answer::Setting(_)
            | Answer::Revision(_)
            | Answer::Revisions(_)
            | Answer::Tasks(_)
            | Answer::Waits(_)
            | Answer::Document(_)
            | Answer::Ended(_)
            | Answer::None(_),
        )
        | Part::Forge(_) => return None,
    };
    let mut skip = gathered.words.len();
    for row in &page.rows {
        let entry = match row {
            core::reads::PageRow::Inbox(entry) => entry,
            core::reads::PageRow::Line(_)
            | core::reads::PageRow::EndedDelegate(_)
            | core::reads::PageRow::History(_)
            | core::reads::PageRow::Attempt(_)
            | core::reads::PageRow::Hold(_)
            | core::reads::PageRow::Activity { .. }
            | core::reads::PageRow::Note(_)
            | core::reads::PageRow::Spend(_) => continue,
        };
        let message = match entry.object {
            documents::InboxObject::Question { message } => message,
            documents::InboxObject::Proposal { .. }
            | documents::InboxObject::Escalation { .. }
            | documents::InboxObject::PersonTask
            | documents::InboxObject::Reply { .. }
            | documents::InboxObject::Result { .. } => continue,
        };
        if skip != 0 {
            skip = skip.checked_sub(1).expect("remaining answered questions");
            continue;
        }
        let task = match &entry.task {
            documents::TaskLink::Live(chip) | documents::TaskLink::Retained { chip, .. } => chip.number,
            documents::TaskLink::Stored { number, .. } => *number,
        };
        return Some((task, message));
    }
    None
}

fn refused(part: &Part) -> Option<SeeNotRead> {
    match part {
        Part::Jig(Answer::None(Why::Unknown)) | Part::Forge(None) => Some(SeeNotRead::Unknown),
        Part::Jig(Answer::None(Why::Busy)) => Some(SeeNotRead::Busy),
        Part::Jig(
            Answer::None(Why::Gone | Why::TooLarge { .. } | Why::TooLinked { .. } | Why::Malformed | Why::SignedOut)
            | Answer::TaskText(_)
            | Answer::Setting(_)
            | Answer::Revision(_)
            | Answer::Revisions(_)
            | Answer::Tasks(_)
            | Answer::Waits(_)
            | Answer::Document(_)
            | Answer::Ended(_)
            | Answer::Page(_),
        )
        | Part::Forge(Some(_)) => None,
    }
}

fn tracked(part: &Part) -> bool {
    match part {
        Part::Jig(Answer::Document(document)) => match document.as_ref() {
            documents::Document::Task(task) => task.priority.is_some(),
            documents::Document::Frame(_)
            | documents::Document::Chips(_)
            | documents::Document::Run(_)
            | documents::Document::Tree(_)
            | documents::Document::Lines(_)
            | documents::Document::Proposal(_)
            | documents::Document::Escalation(_)
            | documents::Document::Question(_)
            | documents::Document::PersonTask(_)
            | documents::Document::Result(_)
            | documents::Document::Project(_)
            | documents::Document::Note(_)
            | documents::Document::Inbox(_)
            | documents::Document::Terminal { .. }
            | documents::Document::Accounts(_)
            | documents::Document::Deployment(_)
            | documents::Document::Settings(_) => false,
        },
        Part::Jig(Answer::Ended(ended)) => match &ended.value {
            core::reads::EndedValue::Task { document, .. } => document.priority.is_some(),
            core::reads::EndedValue::Proposal(_)
            | core::reads::EndedValue::PartyProposal(_)
            | core::reads::EndedValue::PersonProposal { .. }
            | core::reads::EndedValue::Escalation { .. }
            | core::reads::EndedValue::Question { .. }
            | core::reads::EndedValue::PersonTask { .. }
            | core::reads::EndedValue::Reply(_)
            | core::reads::EndedValue::Turn(_)
            | core::reads::EndedValue::WithheldTurn(_) => false,
        },
        Part::Jig(
            Answer::TaskText(_)
            | Answer::Setting(_)
            | Answer::Revision(_)
            | Answer::Revisions(_)
            | Answer::Tasks(_)
            | Answer::Waits(_)
            | Answer::Page(_)
            | Answer::None(_),
        )
        | Part::Forge(_) => false,
    }
}

fn query(project: u32, order: documents::QueryOrder) -> documents::TaskQuery {
    documents::TaskQuery {
        parts: List::with_capacity(0),
        project: Some(project),
        requester: None,
        root: None,
        parent: None,
        label: None,
        phase: None,
        executor: None,
        tracked: None,
        order,
    }
}

fn tasks(query: documents::TaskQuery) -> Read {
    Read::Tasks { query, live: None, ended: None }
}

fn tree(project: u32, number: u64, ended: bool) -> Read {
    let mut query = query(project, documents::QueryOrder::Number);
    query.parent = Some(number);
    if ended {
        query.phase = Some(documents::PhaseClass::Ended);
    }
    tasks(query)
}

fn needs(project: u32) -> Read {
    Read::Inbox {
        group: core::reads::Group::Needs,
        filter: core::reads::Filter { project: Some(project), goal: None },
        from: None,
    }
}

fn application(project: u32, connector: u16, subject: u64) -> Read {
    Read::Application {
        scope: ApplicationScope::Project(project),
        connector,
        read: Token::new(subject),
        window: documents::Window { rows: 0, after: None },
    }
}

fn jig(part: Option<Part>) -> Answer {
    match part.expect("answered jig read") {
        Part::Jig(answer) => answer,
        Part::Forge(_) => unreachable!("jig read answer"),
    }
}

fn forge(part: Option<Part>) -> Option<Box<forge::Document>> {
    match part.expect("answered application read") {
        Part::Forge(document) => document,
        Part::Jig(Answer::None(_)) => None,
        Part::Jig(
            Answer::TaskText(_)
            | Answer::Setting(_)
            | Answer::Revision(_)
            | Answer::Revisions(_)
            | Answer::Tasks(_)
            | Answer::Waits(_)
            | Answer::Document(_)
            | Answer::Ended(_)
            | Answer::Page(_),
        ) => unreachable!("forge read answer"),
    }
}

fn record(connector: &forge::Domain, address: Address, person: u64, project: u32, gathered: Gathered) -> SeeRead {
    let Gathered { first, second, words, .. } = gathered;
    match address {
        Address::Me => SeeRead::Me { person, project: jig(first), needs: jig(second), words: words.into_boxed() },
        Address::Project => SeeRead::Project {
            project: jig(first),
            held: jig(second),
            repositories: crate::forge::repositories(connector, project),
        },
        Address::Goals { ended } => SeeRead::Goals { ended, goals: jig(first), reports: jig(second) },
        Address::Tasks => SeeRead::Tasks { tasks: jig(first) },
        Address::Task { number } => SeeRead::Task {
            number,
            task: jig(first),
            report: match second {
                Some(part) => Some(jig(Some(part))),
                None => None,
            },
        },
        Address::Tree { number } => SeeRead::Tree { number, live: jig(first), ended: jig(second) },
        Address::Activity { number } => SeeRead::Activity { number, activity: jig(first) },
        Address::Turn { number, attempt, turn } => SeeRead::Turn { number, attempt, turn, read: jig(first) },
        Address::Change { number } => SeeRead::Change { number, panel: forge(first).expect("read change panel") },
        Address::Changes => SeeRead::Changes { changes: forge(first), landings: forge(second) },
        Address::Inbox => SeeRead::Inbox { needs: jig(first), words: words.into_boxed() },
        Address::Ways
        | Address::Inventory
        | Address::Setting(_)
        | Address::Landing
        | Address::Gates { .. }
        | Address::Revisions
        | Address::Revision { .. }
        | Address::Proposal { .. }
        | Address::Page { .. }
        | Address::Search { .. }
        | Address::File { .. }
        | Address::Unknown => unreachable!("a person's address"),
    }
}

/// Finish a person read with its answers and the display names they need.
#[must_use]
pub fn answer(
    core: &core::Core,
    connector: &forge::Domain,
    address: Address,
    person: u64,
    project: u32,
    gathered: Gathered,
) -> SeePerson {
    let names = names::make(core, project, &gathered);
    SeePerson { read: record(connector, address, person, project, gathered), names }
}

mod names;
