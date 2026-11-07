//! Field-for-field document values (client/domain/contract.md, 5–7).
use super::{Wall, c, w};

pub(crate) fn capacity_fields(value: &w::Capacity) -> c::Capacity {
    c::Capacity { used: value.used(), limit: value.limit() }
}

pub(crate) fn project_capacity_fields(value: &w::ProjectCapacity) -> c::ProjectCapacity {
    c::ProjectCapacity { project: value.project(), tasks: capacity_fields(value.tasks()), proposals: value.proposals() }
}

pub(crate) fn agent_shortfall_fields(value: &w::AgentShortfall) -> c::AgentShortfall {
    match value {
        w::AgentShortfall::Model(row) => c::AgentShortfall::Model { charter: row.charter(), model: row.model() },
        w::AgentShortfall::Window(row) => c::AgentShortfall::Window { charter: row.charter(), model: row.model() },
        w::AgentShortfall::Output(row) => c::AgentShortfall::Output { charter: row.charter(), model: row.model() },
        w::AgentShortfall::Turns(row) => c::AgentShortfall::Turns { charter: row.charter() },
        w::AgentShortfall::Spend => c::AgentShortfall::Spend,
        w::AgentShortfall::Time(row) => c::AgentShortfall::Time { charter: row.charter() },
        w::AgentShortfall::Waiting(row) => c::AgentShortfall::Waiting { charter: row.charter() },
        w::AgentShortfall::Checks(row) => c::AgentShortfall::Checks { charter: row.charter() },
        w::AgentShortfall::Message => c::AgentShortfall::Message,
        w::AgentShortfall::Inbox => c::AgentShortfall::Inbox,
        w::AgentShortfall::Unacknowledged => c::AgentShortfall::Unacknowledged,
    }
}

pub(crate) fn hello_refusal_fields(value: &w::HelloRefusal) -> c::HelloRefusal {
    match value {
        w::HelloRefusal::Agent(row) => c::HelloRefusal::Agent { why: agent_shortfall_fields(row.why()) },
        w::HelloRefusal::JobKinds => c::HelloRefusal::JobKinds,
        w::HelloRefusal::ReservedChannel => c::HelloRefusal::ReservedChannel,
        w::HelloRefusal::Workers(row) => c::HelloRefusal::Workers { limit: row.limit() },
        w::HelloRefusal::Position(row) => c::HelloRefusal::Position { worker: row.worker() },
        w::HelloRefusal::StopBound(row) => c::HelloRefusal::StopBound { grace: row.grace() },
    }
}

pub(crate) fn hello_refusal_line_fields(value: &w::HelloRefusalLine) -> c::HelloRefusalLine {
    c::HelloRefusalLine {
        channel: value.channel(),
        at: Wall::from_nanos(value.at()),
        slots: value.slots(),
        stop_bound: value.stop_bound(),
        why: hello_refusal_fields(value.why()),
    }
}

pub(crate) fn request_key_fields(value: &w::RequestKey) -> c::RequestKey {
    c::RequestKey { person: value.person(), key: *value.key() }
}

pub(crate) fn person_capacity_action_fields(value: &w::PersonCapacityAction) -> c::PersonCapacityAction {
    match value {
        w::PersonCapacityAction::AccountSignIn => c::PersonCapacityAction::AccountSignIn,
        w::PersonCapacityAction::AccountSignOut => c::PersonCapacityAction::AccountSignOut,
        w::PersonCapacityAction::AccountChange => c::PersonCapacityAction::AccountChange,
        w::PersonCapacityAction::Create => c::PersonCapacityAction::Create,
        w::PersonCapacityAction::Compound => c::PersonCapacityAction::Compound,
        w::PersonCapacityAction::Application(row) => {
            c::PersonCapacityAction::Application { connector: row.connector() }
        }
        w::PersonCapacityAction::Note(row) => c::PersonCapacityAction::Note { name: row.name() },
        w::PersonCapacityAction::Service => c::PersonCapacityAction::Service,
        w::PersonCapacityAction::Adoption => c::PersonCapacityAction::Adoption,
        w::PersonCapacityAction::Roles => c::PersonCapacityAction::Roles,
        w::PersonCapacityAction::Policy => c::PersonCapacityAction::Policy,
        w::PersonCapacityAction::Pool(row) => c::PersonCapacityAction::Pool { person: row.person() },
        w::PersonCapacityAction::Escalation(row) => c::PersonCapacityAction::Escalation { task: row.task() },
        w::PersonCapacityAction::Proposal(row) => c::PersonCapacityAction::Proposal {
            proposer: crate::translate::answers::proposer(row.proposer()),
            number: row.number(),
        },
        w::PersonCapacityAction::Words(row) => c::PersonCapacityAction::Words { task: row.task() },
        w::PersonCapacityAction::Question(row) => {
            c::PersonCapacityAction::Question { task: row.task(), question: row.question() }
        }
        w::PersonCapacityAction::Amend(row) => c::PersonCapacityAction::Amend { task: row.task() },
        w::PersonCapacityAction::MoveTask(row) => c::PersonCapacityAction::Move { task: row.task() },
        w::PersonCapacityAction::TakePerson(row) => c::PersonCapacityAction::TakePerson { task: row.task() },
        w::PersonCapacityAction::HandBackPerson(row) => c::PersonCapacityAction::HandBackPerson { task: row.task() },
        w::PersonCapacityAction::AnswerPerson(row) => c::PersonCapacityAction::AnswerPerson { task: row.task() },
        w::PersonCapacityAction::Stop(row) => c::PersonCapacityAction::Stop { task: row.task() },
        w::PersonCapacityAction::Cancel(row) => c::PersonCapacityAction::Cancel { task: row.task() },
        w::PersonCapacityAction::Lift(row) => c::PersonCapacityAction::Lift { hold: row.hold() },
        w::PersonCapacityAction::Release(row) => c::PersonCapacityAction::Release { task: row.task() },
        w::PersonCapacityAction::MarkSeen(row) => {
            c::PersonCapacityAction::MarkSeen { seen: crate::translate::answers::seen(row.seen()) }
        }
    }
}

pub(crate) fn person_capacity_cause_fields(value: &w::PersonCapacityCause) -> c::PersonCapacityCause {
    match value {
        w::PersonCapacityCause::Requests(row) => c::PersonCapacityCause::Requests { limit: row.limit() },
        w::PersonCapacityCause::Pending(row) => c::PersonCapacityCause::Pending { limit: row.limit() },
        w::PersonCapacityCause::Waiters(row) => c::PersonCapacityCause::Waiters { limit: row.limit() },
        w::PersonCapacityCause::Limit(row) => {
            c::PersonCapacityCause::Limit { which: crate::translate::answers::admission_limit(row.which()) }
        }
    }
}

pub(crate) fn person_capacity_refusal_fields(value: &w::PersonCapacityRefusal) -> c::PersonCapacityRefusal {
    let opt0 = match value.project() {
        Some(item) => Some(*item),
        None => None,
    };
    c::PersonCapacityRefusal {
        key: request_key_fields(value.key()),
        project: opt0,
        action: person_capacity_action_fields(value.action()),
        at: Wall::from_nanos(value.at()),
        cause: person_capacity_cause_fields(value.cause()),
    }
}

pub(crate) fn task_capacity_action_fields(value: &w::TaskCapacityAction) -> c::TaskCapacityAction {
    match value {
        w::TaskCapacityAction::Batch(row) => {
            let opt0 = match row.first() {
                Some(item) => Some(*item),
                None => None,
            };
            c::TaskCapacityAction::Batch { first: opt0, members: row.members() }
        }
        w::TaskCapacityAction::Proposal(row) => c::TaskCapacityAction::Proposal { number: row.number() },
        w::TaskCapacityAction::PersonProposal(row) => c::TaskCapacityAction::PersonProposal { number: row.number() },
        w::TaskCapacityAction::Message(row) => {
            c::TaskCapacityAction::Message { task: row.task(), number: row.number() }
        }
        w::TaskCapacityAction::Source(row) => {
            c::TaskCapacityAction::Source { funder: crate::translate::facts::funder(row.funder()) }
        }
        w::TaskCapacityAction::Introduce(row) => {
            c::TaskCapacityAction::Introduce { left: row.left(), right: row.right() }
        }
        w::TaskCapacityAction::Subscription(row) => c::TaskCapacityAction::Subscription { number: row.number() },
        w::TaskCapacityAction::Effect(row) => c::TaskCapacityAction::Effect { entry: row.entry() },
        w::TaskCapacityAction::Claim(row) => c::TaskCapacityAction::Claim { attempt: row.attempt() },
        w::TaskCapacityAction::Place(row) => c::TaskCapacityAction::Place { hold: row.hold() },
        w::TaskCapacityAction::Release => c::TaskCapacityAction::Release,
        w::TaskCapacityAction::MoveTask => c::TaskCapacityAction::Move,
    }
}

pub(crate) fn task_capacity_cause_fields(value: &w::TaskCapacityCause) -> c::TaskCapacityCause {
    match value {
        w::TaskCapacityCause::PlacedHolds(row) => c::TaskCapacityCause::PlacedHolds { limit: row.limit() },
        w::TaskCapacityCause::ProjectPlacedHolds(row) => {
            c::TaskCapacityCause::ProjectPlacedHolds { limit: row.limit() }
        }
        w::TaskCapacityCause::PartyPlacedHolds(row) => c::TaskCapacityCause::PartyPlacedHolds { limit: row.limit() },
        w::TaskCapacityCause::Exemptions(row) => c::TaskCapacityCause::Exemptions { limit: row.limit() },
        w::TaskCapacityCause::Tasks(row) => c::TaskCapacityCause::Tasks { limit: row.limit() },
        w::TaskCapacityCause::CreationFlights(row) => c::TaskCapacityCause::CreationFlights { limit: row.limit() },
        w::TaskCapacityCause::ResourceReports(row) => c::TaskCapacityCause::ResourceReports { limit: row.limit() },
        w::TaskCapacityCause::ProjectTasks(row) => c::TaskCapacityCause::ProjectTasks { limit: row.limit() },
        w::TaskCapacityCause::Tree(row) => c::TaskCapacityCause::Tree { limit: row.limit() },
        w::TaskCapacityCause::Delegates(row) => c::TaskCapacityCause::Delegates { limit: row.limit() },
        w::TaskCapacityCause::Inbox(row) => {
            c::TaskCapacityCause::Inbox { task: row.task(), messages: row.messages(), bytes: row.bytes() }
        }
        w::TaskCapacityCause::Holdings(row) => c::TaskCapacityCause::Holdings { limit: row.limit() },
        w::TaskCapacityCause::HoldWaiters(row) => c::TaskCapacityCause::HoldWaiters { limit: row.limit() },
        w::TaskCapacityCause::Proposals(row) => c::TaskCapacityCause::Proposals { limit: row.limit() },
        w::TaskCapacityCause::References(row) => c::TaskCapacityCause::References { limit: row.limit() },
        w::TaskCapacityCause::Subscriptions(row) => c::TaskCapacityCause::Subscriptions { limit: row.limit() },
        w::TaskCapacityCause::Funders(row) => c::TaskCapacityCause::Funders { limit: row.limit() },
        w::TaskCapacityCause::Writers(row) => c::TaskCapacityCause::Writers { limit: row.limit() },
    }
}

pub(crate) fn task_capacity_request_fields(value: &w::TaskCapacityRequest) -> c::TaskCapacityRequest {
    let opt0 = match value.project() {
        Some(item) => Some(*item),
        None => None,
    };
    c::TaskCapacityRequest {
        project: opt0,
        by: crate::translate::facts::requester(value.by()),
        action: task_capacity_action_fields(value.action()),
        cause: task_capacity_cause_fields(value.cause()),
    }
}

pub(crate) fn task_capacity_refusal_fields(value: &w::TaskCapacityRefusal) -> c::TaskCapacityRefusal {
    c::TaskCapacityRefusal { request: task_capacity_request_fields(value.request()), at: Wall::from_nanos(value.at()) }
}
