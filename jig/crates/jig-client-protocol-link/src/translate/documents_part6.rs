//! Field-for-field document values (client/domain/contract.md, 5–7).
use super::{
    Limits, List, Reader, Wall, account_state_fields, c, charge, chip_piece, host_piece, host_run_piece, piece_count,
    store_fill_fields, too_large, w,
};
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn account_fields(value: &w::Account, limits: &Limits, room: &mut u64) -> Result<c::Account, w::Problem> {
    super::charge_account(value.uses().as_ref(), value.display(), room)?;
    // An account's runs and its waiting tasks stay lists bounded by the rows.
    let chips = List::<c::Chip>::price(limits.client().rows);
    charge(room, chips)?;
    charge(room, chips)?;
    let opt0 = match value.generation() {
        Some(item) => Some(*item),
        None => None,
    };
    let opt1 = match value.spent_until() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    Ok(c::Account {
        number: value.number(),
        place: value.place(),
        uses: super::account_uses(value.uses().as_ref()),
        plan: value.plan(),
        provider: *value.provider(),
        provider_name: match value.provider_name() {
            Some(name) => Some(crate::translate::facts::document_text(name, limits)?),
            None => None,
        },
        manage_usage: super::checked_manage_usage(value.manage_usage().as_deref(), w::Path::AccountManageUsage)?,
        display: crate::translate::facts::document_text(value.display(), limits)?,
        holder: super::account_holder(value.holder()),
        sources: super::account_source_piece(value.sources(), limits, room)?,
        generation: opt0,
        state: account_state_fields(value.state()),
        usable: value.usable(),
        expiring: value.expiring(),
        attention: value.attention(),
        spent_until: opt1,
        runs: chip_piece(value.runs(), limits, limits.client().rows)?,
        waiting: chip_piece(value.waiting(), limits, limits.client().rows)?,
        cut: value.cut(),
    })
}

pub(crate) fn account_piece(bytes: &[u8], limits: &Limits, room: &mut u64) -> Result<List<c::Account>, w::Problem> {
    let count = piece_count(bytes)?;
    if count > limits.client().accounts.accounts {
        return Err(too_large());
    }
    charge(room, List::<c::Account>::price(count))?;
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(count);
    for _ in 0..count {
        let Ok(Some(item)) = skein_codec::piece::split(&mut reader) else { unreachable!("counted pieces split") };
        let row = w::Account::decode(limits.receiving(), &mut Reader::new(&item))?;
        rows.push(account_fields(&row, limits, room)?).expect("counted pieces fit");
    }
    Ok(rows)
}

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
        w::AgentShortfall::Turns => c::AgentShortfall::Turns,
        w::AgentShortfall::Spend => c::AgentShortfall::Spend,
        w::AgentShortfall::Time => c::AgentShortfall::Time,
        w::AgentShortfall::Waiting => c::AgentShortfall::Waiting,
        w::AgentShortfall::Checks => c::AgentShortfall::Checks,
        w::AgentShortfall::Message => c::AgentShortfall::Message,
        w::AgentShortfall::Inbox => c::AgentShortfall::Inbox,
        w::AgentShortfall::Unacknowledged => c::AgentShortfall::Unacknowledged,
    }
}

pub(crate) fn hello_refusal_fields(value: &w::HelloRefusal) -> c::HelloRefusal {
    match value {
        w::HelloRefusal::Agent(row) => c::HelloRefusal::Agent { why: agent_shortfall_fields(row.why()) },
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

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
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

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
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

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
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

pub(crate) fn deployment_fields(value: &w::Deployment, limits: &Limits) -> Result<c::Deployment, w::Problem> {
    let mut room = super::document_room(limits);
    charge(&mut room, u64::try_from(size_of::<c::Deployment>()).ok())?;
    if value.projects().len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.projects().len());
    for item in value.projects() {
        list0.push(project_capacity_fields(item)).expect("reserved supplied list");
    }
    if value.hello_refusals().len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list1 = List::with_capacity(value.hello_refusals().len());
    for item in value.hello_refusals() {
        list1.push(hello_refusal_line_fields(item)).expect("reserved supplied list");
    }
    if value.person_room_refusals().len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list2 = List::with_capacity(value.person_room_refusals().len());
    for item in value.person_room_refusals() {
        list2.push(person_capacity_refusal_fields(item)).expect("reserved supplied list");
    }
    if value.task_room_refusals().len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list3 = List::with_capacity(value.task_room_refusals().len());
    for item in value.task_room_refusals() {
        list3.push(task_capacity_refusal_fields(item)).expect("reserved supplied list");
    }
    if value.connectors().len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list4 = List::with_capacity(value.connectors().len());
    for item in value.connectors() {
        list4.push(*item).expect("reserved supplied list");
    }
    Ok(c::Deployment {
        providers: super::catalogue::offers(value.providers(), limits, &mut room)?,
        providers_cut: value.providers_cut(),
        key_bytes: value.key_bytes(),
        readiness: super::catalogue::readiness(value.readiness(), limits, &mut room)?,
        readiness_cut: value.readiness_cut(),
        store: store_fill_fields(value.store()),
        hosts: host_piece(value.hosts(), limits, limits.client().rows, &mut room)?,
        attempts: host_run_piece(value.attempts(), limits, limits.client().rows)?,
        accounts: account_piece(value.accounts(), limits, &mut room)?,
        tasks: capacity_fields(value.tasks()),
        workers: capacity_fields(value.workers()),
        attempts_capacity: capacity_fields(value.attempts_capacity()),
        waiting_attempts: value.waiting_attempts(),
        proposals: capacity_fields(value.proposals()),
        projects: list0,
        grace_bound: value.grace_bound(),
        hello_refusals: list1,
        hosts_forgotten: value.hosts_forgotten(),
        hello_refusals_forgotten: value.hello_refusals_forgotten(),
        person_room_refusals: list2,
        person_room_refusals_forgotten: value.person_room_refusals_forgotten(),
        person_answer_retention: value.person_answer_retention(),
        task_room_refusals: list3,
        task_room_refusals_forgotten: value.task_room_refusals_forgotten(),
        cut: value.cut(),
        connectors: list4,
        connectors_cut: value.connectors_cut(),
    })
}

pub(crate) fn project_resource_role_fields(value: &w::ProjectResourceRole) -> c::ProjectResourceRole {
    match value {
        w::ProjectResourceRole::Owned => c::ProjectResourceRole::Owned,
        w::ProjectResourceRole::Participant => c::ProjectResourceRole::Participant,
        w::ProjectResourceRole::Unavailable => c::ProjectResourceRole::Unavailable,
        w::ProjectResourceRole::Context => c::ProjectResourceRole::Context,
    }
}

pub(crate) fn project_resource_fields(value: &w::ProjectResource) -> c::ProjectResource {
    c::ProjectResource {
        part: crate::translate::facts::part(value.part()),
        role: project_resource_role_fields(value.role()),
    }
}
