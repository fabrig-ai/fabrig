/// A test's presentation configuration and the client's receiving bounds.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Profile {
    pub view: crate::Config,
    pub client: jig_client::DomainLimits,
}
impl core::ops::Deref for Profile {
    type Target = crate::Config;
    fn deref(&self) -> &Self::Target {
        &self.view
    }
}
impl core::ops::DerefMut for Profile {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.view
    }
}
pub(crate) const CLIENT: jig_client::DomainLimits = jig_client::DomainLimits {
    owners: &[],
    requests: skein_lib::RequestLimits {
        requests: 2,
        bytes: 1 << 20,
        out: 256,
        first: skein_lib::Duration::from_nanos(10),
        most: skein_lib::Duration::from_nanos(80),
        margin: skein_lib::Duration::from_secs(1),
    },
    ask_bytes: 65536,
    handles: 256,
    watches: 8,
    reads: 8,
    turn_bytes: 65536,
    document_bytes: 65536,
    backoff: jig_client::Backoff {
        first: skein_lib::Duration::from_nanos(10),
        most: skein_lib::Duration::from_nanos(80),
    },
    rows: 2,
    segments: 4,
    authority: 8,
    text: 128,
    turn_text: 128,
    projects: 2,
    fields: 8,
    delegates: 2,
    records: 2,
    kept: 4,
    parts: 8,
    accounts: jig_client::AccountCounts { accounts: 8, sources: 8, sign_ins: 8 },
    heartbeat: skein_lib::Duration::from_secs(1000),
};
use crate::state::address::Address;
use crate::state::records::{RECORDS_VERSION, Record};
use jig_client::contract::Text;
pub const PARTY: u64 = u64::MAX;
pub const TASK: u64 = u64::MAX;
pub fn place() -> Record {
    Record::Place { version: RECORDS_VERSION, party: PARTY, address: Address::Deployment }
}
pub fn draft() -> Record {
    Record::Draft {
        version: RECORDS_VERSION,
        party: PARTY,
        task: TASK,
        words: Text::new(Box::from(b"words".as_slice()), 65536).unwrap(),
        touched: u64::MAX,
    }
}
pub fn places() -> skein_lib::List<Record> {
    use crate::state::address::{Address, ProjectSection, TaskSection};
    let mut addresses = skein_lib::List::with_capacity(21);
    for address in [
        Address::Inbox { project: None },
        Address::Inbox { project: Some(u32::MAX) },
        Address::Conversations { project: None },
        Address::Conversations { project: Some(u32::MAX) },
        Address::Deployment,
    ] {
        addresses.push(address).unwrap();
    }
    for section in [
        TaskSection::Overview,
        TaskSection::Activity,
        TaskSection::Plan,
        TaskSection::History,
        TaskSection::Budget,
        TaskSection::Panels,
    ] {
        addresses.push(Address::Task { task: u64::MAX, section }).unwrap();
    }
    for section in [
        ProjectSection::Overview,
        ProjectSection::Goals,
        ProjectSection::Tasks,
        ProjectSection::Notes,
        ProjectSection::Settings,
        ProjectSection::Resources,
        ProjectSection::Policy,
        ProjectSection::Roles,
        ProjectSection::Spend,
        ProjectSection::Recurring,
    ] {
        addresses.push(Address::Project { project: u32::MAX, section }).unwrap();
    }
    let mut saved = skein_lib::List::with_capacity(addresses.len());
    for address in addresses.into_boxed() {
        saved.push(Record::Place { version: RECORDS_VERSION, party: u64::MAX, address }).unwrap();
    }
    saved
}

/// Four opaque segments expose a display cut while preserving every byte.
#[must_use]
pub(crate) fn long_resource_path<const N: usize>() -> skein_lib::List<Box<[u8]>> {
    let mut path = skein_lib::List::with_capacity(4);
    for byte in [0xF0_u8, 0xF1, 0xF2, 0xF3] {
        path.push(Box::from([byte; N])).unwrap();
    }
    path
}

/// The application's part effects, preserving their exact fields.
pub(crate) fn parts(output: &[crate::Request], capacity: u32) -> Box<[crate::Request]> {
    let mut parts = skein_lib::List::with_capacity(capacity);
    for request in output {
        if let crate::Request::ShowPart { part, scope } = request {
            parts.push(crate::Request::ShowPart { part: part.clone(), scope: *scope }).unwrap();
        }
    }
    parts.into_boxed()
}

pub(crate) mod client;
pub(crate) use client::MOST;
pub(crate) use client::{
    AnsweredByEngine, CardAsk, Ended, FireClient, Fixture, Input, ReadPage, Reported, SendIntent, Served, Tick,
    Watched, intent, kept,
};

pub(crate) fn one<T>(value: T) -> skein_lib::List<T> {
    crate::tests::kit::list([value])
}

/// A send and the release the displayed hold names share one request.
pub(crate) fn releasing(
    ask: jig_client::contract::Ask,
    revision: Option<jig_client::contract::Revision>,
) -> jig_client::contract::Ask {
    use jig_client::contract::{Ask, Operation};
    let Some(revision) = revision else {
        return ask;
    };
    let (project, task, first) = if let Ask::Message { project, task, words } = ask {
        (project, task, Operation::Message { project, task, words })
    } else {
        let Ask::Amend { project, task, revision, change, reason } = ask else {
            panic!("message or amendment");
        };
        (project, task, Operation::Amend { project, task, revision, change, reason })
    };
    let operations = crate::tests::kit::list([first, Operation::Release { project, task, revision, again: None }]);
    Ask::Compound { project, operations }
}
pub(crate) fn selected_change(ask: &jig_client::contract::Ask) -> Option<&jig_client::contract::Amendment> {
    use jig_client::contract::{Ask, Operation};
    if let Ask::Amend { change, .. } = ask {
        return Some(change);
    }
    let Ask::Compound { operations, .. } = ask else {
        return None;
    };
    let [Operation::Amend { change, .. }, Operation::Release { .. }] = operations.as_slice() else {
        return None;
    };
    Some(change)
}

pub(crate) fn refusal_answer(refusal: jig_client::contract::Refusal) -> jig_client::contract::Answer {
    jig_client::contract::Answer::Refused { refusal, at: None }
}

/// A supplied query row, independent of the reference UI's selection.
pub(crate) fn task_line(
    chip: jig_client::contract::Chip,
    project: u32,
    requester: jig_client::contract::Requester,
    changed: skein_lib::Wall,
) -> jig_client::contract::TaskLine {
    use jig_client::contract as c;
    c::TaskLine {
        chip,
        project,
        requester,
        changed,
        tracked: None,
        label: 0,
        flags: c::TaskFlags {
            ended_by: c::EndedBy::Result,
            conversation: false,
            listed: true,
            retention: c::Retention::Kept,
        },
        last_activity: changed,
        ended_at: None,
        budget: c::Budget {
            numbers: c::Numbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 },
            left: 0,
            funder: c::Funder::Task(8),
            deadline: None,
        },
        waits_on: c::WaitsOn::Nothing,
        phases: c::PhaseCounts {
            waiting: 0,
            idle: 0,
            starting: 0,
            running: 1,
            backing_off: 0,
            closing: 0,
            held: 0,
            done: 0,
            failed: 0,
            cancelled: 0,
        },
        held_below: 0,
        conversation: None,
    }
}

pub(crate) fn chat_templates() -> skein_lib::List<jig_client::contract::Template> {
    use jig_client::contract as c;
    let mut values = crate::tests::kit::list_in(
        2,
        [c::Template {
            llm: None,
            number: jig_client::shapes::CHAT,
            name: c::DocumentText::complete(Text::new(b"Chat".as_slice().into(), 32).unwrap()),
            executor: c::TemplateExecutor::Charter(1),
            contract: c::TaskContractDocument::Report { words: 32 },
            shallower: 1,
            budget: c::TemplateBudget::AsAllowed { cap: None },
            deadline: c::TemplateDeadline::None,
            tracked: None,
            flags: c::TaskFlags {
                ended_by: c::EndedBy::Result,
                conversation: true,
                listed: false,
                retention: c::Retention::Kept,
            },
            label: 1,
            grace: None,
        }],
    );
    let mut goal = values.get(0).unwrap().clone();
    goal.number = jig_client::shapes::GOAL;
    goal.executor = c::TemplateExecutor::Chosen;
    goal.budget = c::TemplateBudget::Given;
    goal.deadline = c::TemplateDeadline::Given;
    goal.label = 2;
    goal.tracked = Some(0);
    goal.flags.conversation = false;
    goal.flags.listed = true;
    values.push(goal).unwrap();
    values
}

pub(crate) fn creating() -> jig_client::contract::Creating {
    use jig_client::contract::*;
    let charters = crate::tests::kit::list([1, 2]);
    Creating {
        within: CreateBounds {
            authority: Box::new(crate::tests::tree::cards_objects::owner_authority()),
            pool_left: 20,
            charters,
            procedures: skein_lib::List::with_capacity(0),
            latest: None,
            budget: 20,
            maximum_budget: 100,
            maximum_deadline: None,
        },
        beyond: Beyond::Proposed { to: Recipient::Party(9) },
    }
}

pub(crate) fn set_creating(fixture: &mut Fixture, project: u32, offer: &jig_client::contract::Admission) {
    use jig_client::contract::*;
    let mut me = fixture.frame().document.unwrap().clone();
    let mut membership = None;
    for index in 0..me.projects.len() {
        if me.projects.get(index).unwrap().project == project {
            membership = Some(index);
            break;
        }
    }
    let membership = membership.expect("fixture has the project");
    me.projects.get_mut(membership).unwrap().creating = match offer {
        Admission::Bounded { within: Bounds::Create(within), beyond } => {
            Some(Creating { within: within.clone(), beyond: beyond.clone() })
        }
        Admission::Refused(why) => {
            let mut creating = me.projects.get(membership).unwrap().creating.clone().unwrap();
            creating.within.budget = 0;
            creating.within.maximum_deadline = None;
            creating.beyond = Beyond::Refused(why.clone());
            Some(creating)
        }
        Admission::Allowed | Admission::Proposed { .. } | Admission::Bounded { .. } => None,
    };
    let watch = fixture.frame_watch.unwrap();
    fixture.drive(fixture.now, Watched { watch, event: WatchEvent::Delivered(Delivery::Me(Box::new(me))) });
}
pub(crate) fn chat_request(project: u32, words: Text) -> jig_client::contract::Ask {
    use jig_client::contract::*;
    let mut authority = crate::tests::tree::cards_objects::owner_authority().authority;
    authority.spend = 0;
    let tasks = crate::tests::kit::list([NewTask {
        llm: None,
        deadline: NewDeadline::Named,
        executor: Executor::Agent { charter: 1 },
        spec: Specification {
            asks: false,
            words,
            parameters: skein_lib::List::with_capacity(0),
            inputs: skein_lib::List::with_capacity(0),
        },
        contract: NewContract::Report { words: 32 },
        authority,
        budget: NewBudget::AsAllowed { cap: None },
        wake: None,
        dependencies: skein_lib::List::with_capacity(0),
        named: skein_lib::List::with_capacity(0),
        tracked: None,
        label: 1,
        flags: TaskFlags { ended_by: EndedBy::Result, conversation: true, listed: false, retention: Retention::Kept },
    }]);
    Ask::Create { project, tasks }
}
pub(crate) fn goal_request(
    form: crate::state::page::project::goals::GoalForm,
    words: Text,
) -> jig_client::contract::Ask {
    use jig_client::contract::*;
    let mut authority = crate::tests::tree::cards_objects::owner_authority().authority;
    authority.spend = form.budget;
    authority.deadline = form.deadline;
    let tasks = crate::tests::kit::list([NewTask {
        llm: None,
        deadline: NewDeadline::Named,
        executor: form.executor,
        spec: Specification {
            asks: false,
            words,
            parameters: skein_lib::List::with_capacity(0),
            inputs: skein_lib::List::with_capacity(0),
        },
        contract: NewContract::Report { words: 32 },
        authority,
        budget: NewBudget::Named,
        wake: None,
        dependencies: skein_lib::List::with_capacity(0),
        named: skein_lib::List::with_capacity(0),
        tracked: Some(form.priority),
        label: 2,
        flags: TaskFlags { ended_by: EndedBy::Result, conversation: false, listed: true, retention: Retention::Kept },
    }]);
    Ask::Propose {
        project: form.project,
        action: ProposeAction::Batch { tasks },
        reason: Text::new(Box::from([]), 32).unwrap(),
    }
}

/// Shape stories start from the same delivered membership document.
pub(crate) fn shape_fixture(me: jig_client::contract::Me) -> Fixture {
    let (mut fixture, watch) = crate::tests::tree::cards::start();
    crate::tests::tree::cards::drive(
        &mut fixture,
        Watched {
            watch,
            event: jig_client::contract::WatchEvent::Delivered(jig_client::contract::Delivery::Me(Box::new(me))),
        },
    );
    fixture
}
/// A chat with its creating offer withdrawn, ready to press Begin.
pub(crate) fn unoffered_chat() -> Fixture {
    let mut me = crate::tests::tree::cards::document();
    me.projects.get_mut(0).unwrap().creating = None;
    let mut fixture = shape_fixture(me);
    crate::tests::tree::cards::drive(
        &mut fixture,
        crate::Event::Went { place: crate::state::address::Place::Jig(Address::Conversations { project: Some(4) }) },
    );
    crate::tests::tree::cards::drive(
        &mut fixture,
        crate::Event::Act {
            action: crate::state::action::Act::Edit {
                field: crate::state::fields::Purpose::NewChat,
                text: crate::tests::tree::cards::text(b"not offered"),
            },
        },
    );
    fixture
}

/// The goal form's renderer needs the project document as well as its membership.
pub(crate) fn shape_goal_fixture(me: jig_client::contract::Me) -> Fixture {
    use jig_client::contract::{Read, ReadResult, ReadValue};
    let mut fixture = shape_fixture(me);
    let out = crate::tests::tree::cards::drive(
        &mut fixture,
        crate::Event::Went {
            place: crate::state::address::Place::Jig(Address::Project {
                project: 4,
                section: crate::state::address::ProjectSection::Goals,
            }),
        },
    );
    for request in out {
        if let crate::Request::Client {
            asked: jig_client::Event::Read { handle: read, read: jig_client::Readable::Jig(Read::Project { .. }) },
        } = request
        {
            crate::tests::tree::cards::drive(
                &mut fixture,
                ReadPage {
                    read,
                    result: ReadResult::Value(ReadValue::Project(Box::new(
                        crate::tests::state::domain::owner_project::source(4),
                    ))),
                },
            );
            return fixture;
        }
    }
    panic!("project read")
}

pub(crate) fn chip_phase(phase: jig_client::contract::ChipPhase) -> jig_client::contract::ChipPhaseFacts {
    use jig_client::contract::*;
    let mut details = crate::tests::tree::fragments::chip::details();
    details.project = 4;
    details.executor = Executor::Person(PersonAddress::Party(1));
    details.attempt = None;
    details.run_since = None;
    details.retry = None;
    details.live_delegates = 0;
    ChipPhaseFacts { phase, details: Box::new(details) }
}
pub(crate) fn hold() -> jig_client::contract::ChipHold {
    use jig_client::contract::*;
    ChipHold {
        words: None,
        placed: None,
        circle: None,
        account: None,
        source: None,
        reason: HoldCategory::Budget,
        revision: 4,
        before: PriorPhase::Waiting,
        escalation: None,
        tries: FailureTries { transient: 0, permanent: 0, run: 0, agent: 0, lost: 0, invalid: 0 },
        hold_wait_since: Some(skein_lib::Wall::EPOCH),
    }
}
pub(crate) fn inbox_entry(key: jig_client::contract::ObjectKey) -> jig_client::contract::InboxEntry {
    use jig_client::contract::*;
    let (task, object) = match key {
        ObjectKey::Task(task) => (task, InboxObject::Reply { message: 1 }),
        ObjectKey::Proposal(key) => (8, InboxObject::Proposal { proposer: key.proposer, number: key.number }),
        ObjectKey::Escalation(task) => (task, InboxObject::Escalation { revision: 4 }),
        ObjectKey::Question { task, message } => (task, InboxObject::Question { message }),
        ObjectKey::PersonTask(task) => (task, InboxObject::PersonTask),
        ObjectKey::Result(task) => (task, InboxObject::Result { position: 1 }),
        ObjectKey::Note { .. } => panic!("notes have their own source"),
    };
    InboxEntry {
        reply: None,
        task: TaskLink::Stored { number: task, status: None },
        project: Some(4),
        object,
        activity: Some(1),
        at: Some(skein_lib::Wall::EPOCH),
        unread: None,
        cursor: InboxCursor::Number { number: 1 },
    }
}
pub(crate) fn inbox(rows: skein_lib::List<jig_client::contract::InboxEntry>) -> jig_client::contract::Inbox {
    use jig_client::contract::*;
    Inbox {
        party: 1,
        cached_needs: rows.len(),
        needs: rows,
        updates: skein_lib::List::with_capacity(0),
        next: None,
        cached_updates: Count { value: 0, limited: false },
        results_seen: 0,
        updates_time_complete: true,
        selection: None,
    }
}

pub(crate) fn run(value: jig_client::contract::RunText) -> jig_client::contract::Delivery {
    jig_client::contract::Delivery::Run(Box::new(value))
}

pub(crate) fn archive(
    task: u64,
    position: u64,
    at: skein_lib::Wall,
    name: jig_client::contract::TurnName,
    turn: jig_client::contract::TurnRead,
) -> jig_client::contract::DecodedActivityLine {
    use jig_client::contract::*;
    DecodedActivityLine {
        source: Box::new(ActivityLine {
            task,
            position,
            at,
            line: Activity::Turn { reply: position, attempt: name.attempt, turn: name.number, read: None },
        }),
        decoded: Some(ArchivedTurn { task, name, cumulative: 0, read: None, at, turn }),
    }
}

pub(crate) fn proposal(key: jig_client::contract::ProposalKey, reason: Text) -> jig_client::contract::Proposal {
    use jig_client::contract::*;
    Proposal {
        key,
        project: 4,
        reason: DocumentText::complete(reason),
        action: ProposalAction::Batch(skein_lib::List::with_capacity(0)),
        as_holder: false,
        waiting: ProposalWaiting { holder: HistoryHolder::Person(1), since: skein_lib::Wall::EPOCH },
        funding: skein_lib::List::with_capacity(0),
        acts: skein_lib::List::with_capacity(0),
    }
}
