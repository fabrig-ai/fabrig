//! Sequential reads as the chat's requester (domain/agent/ask-temper.md, 1; domain/routing.md, 1).
use super::{
    Address, Decision, Domain, Env, FlightRender, Limits, ReplyTo, SeeAnswer, SeeFlight, SeeNames, SeeNotRead,
    SeePerson, SeeRead, SeeReading, SeeWords, Work,
};
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
pub(crate) struct Gathered {
    pub connector: u16,
    pub first: Option<Part>,
    pub second: Option<Part>,
    pub words: List<SeeWords>,
}

#[derive(Hash, Debug)]
pub(crate) enum Part {
    Jig(Answer),
    Forge(Option<Box<forge::Document>>),
}

#[derive(PartialEq, Eq, Debug)]
#[expect(clippy::large_enum_variant, reason = "the transient step moves jig's read directly into its named action")]
pub(crate) enum Step {
    Read(Read),
    Deliver(Option<SeeNotRead>),
}

impl Gathered {
    pub(crate) fn new(connector: u16) -> Self {
        Self { connector, first: None, second: None, words: List::uncapped() }
    }

    pub(crate) fn push(&mut self, address: &Address, answer: Part) {
        if let Some((task, message)) = question(address, self) {
            self.words.push(SeeWords { task, message, words: jig(Some(answer)) }).expect("needs-page question");
            return;
        }
        if self.first.is_none() {
            self.first = Some(answer);
        } else {
            assert!(self.second.is_none(), "an address reads at most two parts");
            self.second = Some(answer);
        }
    }
}

/// The next read depends only on the address, its project and the answers gathered so far.
pub(crate) fn next(address: &Address, project: u32, gathered: &Gathered) -> Step {
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
            | core::reads::PageRow::Milestone(_)
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
            | Answer::Page(_)
            | Answer::None(_),
        )
        | Part::Forge(_) => false,
    }
}

fn query(project: u32, order: documents::QueryOrder) -> documents::TaskQuery {
    documents::TaskQuery {
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

#[expect(clippy::too_many_arguments, reason = "the person read carries its call and parsed address")]
pub(super) fn read(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    to: ReplyTo,
    key: crate::CallKey,
    line: Box<[u8]>,
    from: u32,
    address: Address,
) {
    let context = domain.core.tasks.delegation(key.task).expect("authorized chat delegation");
    match context.requester {
        jig_domain_tasks::Party::Person(_) => {}
        jig_domain_tasks::Party::Task(_) | jig_domain_tasks::Party::Deployment { .. } => {
            super::deliver(env, decision, to, line, from, SeeAnswer::NotRead(SeeNotRead::NoPerson));
            return;
        }
    }
    let gathered = Gathered::new(domain.config.forge_connector);
    let read = match next(&address, context.project, &gathered) {
        Step::Read(read) => read,
        Step::Deliver(_) => unreachable!("first person read"),
    };
    let right = to.into_token();
    domain.call_renders.get_mut(&right).expect("decoded see flight").render = FlightRender::Settings {
        see: Some(Box::new(SeeFlight { line, reading: SeeReading::Person(Box::new(gathered)) })),
        from,
        requested: None,
        asked: None,
    };
    ask(domain, ReplyTo::new(right), key, read);
}

fn ask(domain: &mut Domain, to: ReplyTo, key: crate::CallKey, read: Read) {
    domain.work.push(Work::Core(core::Event::NamedAction { to, key, action: core::NamedAction::PersonRead(read) }));
}

pub(crate) fn is_reading(domain: &Domain, right: Token) -> bool {
    let Some(flight) = domain.call_renders.get(&right) else { return false };
    match &flight.render {
        FlightRender::Settings { see: Some(see), .. } => match see.reading {
            SeeReading::Person(_) => true,
            SeeReading::Settings | SeeReading::Head | SeeReading::File | SeeReading::Proposal => false,
        },
        FlightRender::Plain
        | FlightRender::FindingRead
        | FlightRender::TaskRead
        | FlightRender::ForgeRead
        | FlightRender::Settings { see: None, .. }
        | FlightRender::Amended(_)
        | FlightRender::Change(_)
        | FlightRender::PersonRoles(_)
        | FlightRender::Document(_)
        | FlightRender::Remember(_) => false,
    }
}

pub(crate) fn answered(domain: &mut Domain, to: ReplyTo, answer: Answer) {
    answered_part(domain, to, Part::Jig(answer));
}

pub(crate) fn document(domain: &mut Domain, to: ReplyTo, document: Option<forge::Document>) {
    let document = match document {
        Some(document) => Some(Box::new(document)),
        None => None,
    };
    answered_part(domain, to, Part::Forge(document));
}

fn flight_context(render: &mut FlightRender) -> (&mut SeeFlight, u32) {
    match render {
        FlightRender::Settings { see: Some(see), from, .. } => (see, *from),
        FlightRender::Plain
        | FlightRender::FindingRead
        | FlightRender::TaskRead
        | FlightRender::ForgeRead
        | FlightRender::Settings { see: None, .. }
        | FlightRender::Amended(_)
        | FlightRender::Change(_)
        | FlightRender::PersonRoles(_)
        | FlightRender::Document(_)
        | FlightRender::Remember(_) => unreachable!("see person context"),
    }
}

fn answered_part(domain: &mut Domain, to: ReplyTo, answer: Part) {
    let right = to.into_token();
    let Some(flight) = domain.call_renders.get_mut(&right) else { return };
    let key = flight.key.expect("decoded person read");
    let context = domain.core.tasks.delegation(key.task).expect("person read chat");
    let project = context.project;
    let person = match context.requester {
        jig_domain_tasks::Party::Person(person) => person,
        jig_domain_tasks::Party::Task(_) | jig_domain_tasks::Party::Deployment { .. } => {
            unreachable!("person read requester")
        }
    };
    let (see, _) = flight_context(&mut flight.render);
    let address = super::address(&see.line, project);
    let gathered = match &mut see.reading {
        SeeReading::Person(gathered) => gathered.as_mut(),
        SeeReading::Settings | SeeReading::Head | SeeReading::File | SeeReading::Proposal => {
            unreachable!("see person flight")
        }
    };
    gathered.push(&address, answer);
    let answer = match next(&address, project, gathered) {
        Step::Read(read) => {
            ask(domain, ReplyTo::new(right), key, read);
            return;
        }
        Step::Deliver(Some(why)) => SeeAnswer::NotRead(why),
        Step::Deliver(None) => {
            let gathered = ::core::mem::replace(gathered, Gathered::new(domain.config.forge_connector));
            let names = names::make(domain, project, &gathered);
            SeeAnswer::Person(Box::new(SeePerson { read: record(domain, address, person, project, gathered), names }))
        }
    };
    let flight = domain.call_renders.get_mut(&right).expect("see person flight");
    let (see, from) = flight_context(&mut flight.render);
    let request = crate::boundary::delivery_output(super::Delivery::Host(Box::new(crate::HostDelivery::See {
        to: ReplyTo::new(right),
        line: see.line.clone(),
        from,
        answer: Box::new(answer),
    })));
    assert!(crate::domain::now(domain, request), "see person render door reserved");
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
            | Answer::Document(_)
            | Answer::Ended(_)
            | Answer::Page(_),
        ) => unreachable!("forge read answer"),
    }
}

fn record(domain: &Domain, address: Address, person: u64, project: u32, gathered: Gathered) -> SeeRead {
    let Gathered { first, second, words, .. } = gathered;
    match address {
        Address::Me => SeeRead::Me { person, project: jig(first), needs: jig(second), words: words.into_boxed() },
        Address::Project => SeeRead::Project {
            project: jig(first),
            held: jig(second),
            repositories: names::repositories(domain, project),
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

mod names;
