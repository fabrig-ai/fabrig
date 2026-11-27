//! Task steering and answer eligibility, without a retained request.
use crate::{
    Core, Limits,
    acts::{Act, Admission, allowed, bounds, objects::mutable, standing},
    admission::compound::PartyTask,
    routing::party::dispatch::Control,
};
use jig_domain_authority as authority;
use jig_domain_people as people;
use jig_domain_tasks as tasks;

pub(super) fn admit(core: &Core, limits: &Limits, party: u64, act: Act, number: u64) -> Admission {
    let Some(row) = core.tasks.task(number) else { return Admission::Refused(people::Refusal::Unknown) };
    let held = core.people.role(party, row.project);
    if let Err(why) = standing::role(core, row.project, held, act) {
        return Admission::Refused(why);
    }
    let who = PartyTask { person: party, project: row.project, role: held, task: number };
    match act {
        Act::Reorder => allowed(core.reorder_allowed(row.project, held)),
        Act::Talk | Act::Leave => {
            if let Err(why) = write(core, limits, party, number) {
                return Admission::Refused(why);
            }
            if row.flags.ended_by == tasks::EndedBy::Standing {
                Admission::Allowed
            } else {
                Admission::Refused(people::Refusal::NotOffered)
            }
        }
        Act::Message => allowed(write(core, limits, party, number)),
        Act::Amend => {
            let admission = bounds::amend(core, limits, party, number, held);
            if let Admission::Refused(_) = admission {
                return admission;
            }
            if let Err(why) = mutable(row) {
                return Admission::Refused(why);
            }

            admission
        }
        Act::Stop => allowed(control(core, limits, who, row, Control::Stop)),
        Act::Cancel => allowed(control(core, limits, who, row, Control::Cancel { reason_len: 0, grace: None })),
        Act::Release => allowed(control(core, limits, who, row, Control::Release { again: None })),
        Act::Move => allowed(core.move_admit(
            who,
            limits.tasks.tree_tasks,
            limits.tasks.depth,
            &tasks::CompoundAdmission::new(&core.tasks, 0),
        )),
        Act::Seen => {
            if core.conversation_requester(row) == Some(party) && held.is_some() {
                Admission::Allowed
            } else {
                Admission::Refused(people::Refusal::Unknown)
            }
        }
        Act::Answer
        | Act::Take
        | Act::HandBack
        | Act::Withdraw
        | Act::Accept
        | Act::Reject
        | Act::PassUp
        | Act::LeaveHeld
        | Act::EditNote
        | Act::DeleteNote
        | Act::Adopt
        | Act::RemoveProject
        | Act::Rename
        | Act::ChangePolicy
        | Act::SetRoles
        | Act::MakeService
        | Act::SetPool => Admission::Refused(people::Refusal::NotOffered),
    }
}

fn control(
    core: &Core,
    limits: &Limits,
    who: PartyTask,
    row: &tasks::TaskRecord,
    control: Control,
) -> Result<(), people::Refusal> {
    core.control_admit(who, control, &limits.tasks, &tasks::CompoundAdmission::new(&core.tasks, 0))?;
    match control {
        Control::Stop => mutable(row),
        Control::Cancel { .. } => match &row.phase {
            tasks::Phase::Ended(ending) => Err(crate::routing::refusals::ended(ending)),
            tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
                Ok(())
            }
        },
        Control::Release { .. } | Control::Measure => match &row.phase {
            tasks::Phase::Held { .. } => Ok(()),
            tasks::Phase::Closing(closing) => Err(crate::routing::refusals::ended(&closing.ending)),
            tasks::Phase::Ended(ending) => Err(crate::routing::refusals::ended(ending)),
            tasks::Phase::Waiting | tasks::Phase::Active(_) => Err(people::Refusal::NotWaiting),
        },
    }
}

pub(crate) fn write(core: &Core, limits: &Limits, party: u64, number: u64) -> Result<(), people::Refusal> {
    write_in(core, limits, party, number, &tasks::CompoundAdmission::new(&core.tasks, 0))
}
pub(crate) fn write_in(
    core: &Core,
    limits: &Limits,
    party: u64,
    number: u64,
    admission: &tasks::CompoundAdmission<'_>,
) -> Result<(), people::Refusal> {
    writable_in(core, limits, party, number, admission, false)
}
fn writable_in(
    core: &Core,
    limits: &Limits,
    party: u64,
    number: u64,
    admission: &tasks::CompoundAdmission<'_>,
    question: bool,
) -> Result<(), people::Refusal> {
    let row = admission.task(number).ok_or(people::Refusal::Unknown)?;
    let held = core.people.role(party, row.project);
    // Project 0's task is the application's (jig/docs/design/domain/authority.md, 6.1).
    if row.project == people::DEPLOYMENT {
        return Err(people::Refusal::Role { held });
    }
    standing::role(core, row.project, held, Act::Message)?;
    let any = match held {
        Some(people::Role::Owner | people::Role::Maintainer) => true,
        Some(people::Role::Policy { role: number }) => match core.authority.role(row.project, number) {
            Some(role) => role.requests.allows(authority::RequestKind::Amend),
            None => false,
        },
        Some(people::Role::Member | people::Role::Observer) | None => false,
    };
    if !any && !Core::person_tree(party, number, limits.tasks.depth, admission) {
        return Err(people::Refusal::Unknown);
    }
    match row.executor {
        tasks::Executor::Agent { .. } => {}
        tasks::Executor::Procedure { connector, code }
            if question && core.person_answers && connector == crate::RECURRING && code == crate::GATHERING => {}
        tasks::Executor::Procedure { .. } | tasks::Executor::Person(_) | tasks::Executor::Job { .. } => {
            return Err(people::Refusal::NotOffered);
        }
    }
    super::mutable_phase(&row.phase)?;
    match row.phase {
        tasks::Phase::Active(_) | tasks::Phase::Held { was: tasks::Was::Active(_), .. } => Ok(()),
        tasks::Phase::Waiting
        | tasks::Phase::Closing(_)
        | tasks::Phase::Held { was: tasks::Was::Waiting | tasks::Was::Closing(_), .. }
        | tasks::Phase::Ended(_) => Err(people::Refusal::NotWaiting),
    }
}

pub(crate) fn person(
    core: &Core,
    party: u64,
    act: Act,
    number: u64,
    revision: Option<u64>,
) -> Result<(), people::Refusal> {
    use crate::admission::person_task::{address_matches, addresses};
    use tasks::PersonAddress::{Person, Role};

    let row = core.tasks.task(number).ok_or(people::Refusal::Unknown)?;
    let (first, offer) = addresses(row).ok_or(people::Refusal::Unknown)?;
    let direct = first == Person(party) || offer == Some(Person(party));
    let held = core.people.role(party, row.project);
    let mut role_addressed = false;
    let mut through_role = false;
    for address in [Some(first), offer] {
        if let Some(address @ Role(_)) = address {
            role_addressed = true;
            through_role |= address_matches(address, party, held);
        }
    }
    let addressed = through_role || (direct && act == Act::Answer) || (role_addressed && act == Act::HandBack);
    if !addressed {
        return Err(people::Refusal::Unknown);
    }
    if let Some(answer) = row.person_answer {
        return Err(people::Refusal::Ended { how: people::Ending::Decided { by: answer.by, at: answer.at } });
    }
    if let Some(by) = row.taken_by
        && by != party
    {
        return Err(people::Refusal::Taken { by });
    }
    if let Some(revision) = revision {
        core.person_revision(number, revision)?;
    }
    mutable(row)?;
    match row.phase {
        tasks::Phase::Active(tasks::Active::Due | tasks::Active::Idle) => {}
        tasks::Phase::Active(
            tasks::Active::Preparing
            | tasks::Active::Claimed { .. }
            | tasks::Active::Running { .. }
            | tasks::Active::BackingOff { .. },
        )
        | tasks::Phase::Waiting
        | tasks::Phase::Closing(_)
        | tasks::Phase::Held { .. }
        | tasks::Phase::Ended(_) => return Err(people::Refusal::NotWaiting),
    }
    match act {
        Act::Take => {
            if row.taken_by.is_none() {
                Ok(())
            } else {
                Err(people::Refusal::NotWaiting)
            }
        }
        Act::HandBack => {
            if row.taken_by == Some(party) {
                Ok(())
            } else {
                Err(people::Refusal::NotWaiting)
            }
        }
        Act::Answer => {
            if !direct && row.taken_by.is_none() {
                return Err(people::Refusal::NotWaiting);
            }
            if row.delegates.is_empty() {
                Ok(())
            } else {
                Err(people::Refusal::Limit { which: people::LimitKind::LiveDelegates })
            }
        }
        Act::Talk
        | Act::Leave
        | Act::Message
        | Act::Withdraw
        | Act::Accept
        | Act::Reject
        | Act::PassUp
        | Act::Release
        | Act::LeaveHeld
        | Act::Amend
        | Act::Stop
        | Act::Cancel
        | Act::Move
        | Act::Reorder
        | Act::Seen
        | Act::EditNote
        | Act::DeleteNote
        | Act::Adopt
        | Act::RemoveProject
        | Act::Rename
        | Act::ChangePolicy
        | Act::SetRoles
        | Act::MakeService
        | Act::SetPool => Err(people::Refusal::NotOffered),
    }
}

pub(crate) fn question(
    core: &Core,
    limits: &Limits,
    party: u64,
    act: Act,
    number: u64,
    message: u64,
) -> Result<(), people::Refusal> {
    if act != Act::Answer {
        return Err(people::Refusal::NotOffered);
    }
    let row = core.tasks.task(number).ok_or(people::Refusal::Unknown)?;
    let role = core.people.role(party, row.project);
    let ticket = core.tasks.question_ticket(number, message);
    if let Some(ticket) = ticket
        && at_question_offer(ticket, number, row.project, party, role)
    {
        if let Some(by) = ticket.by {
            return Err(crate::routing::refusals::answered(message, by));
        }
        return super::mutable_phase(&row.phase);
    }
    standing::role(core, row.project, role, act)?;
    if row.requester != tasks::Party::Person(party) {
        return Err(people::Refusal::Unknown);
    }
    if let Some(ticket) = ticket {
        if ticket.asked != number
            || (core.person_answers
                && ticket.first != tasks::QuestionAnswerer::Person(party)
                && core.question_requester(row) != Some(party))
        {
            return Err(people::Refusal::Unknown);
        }
        if let Some(by) = ticket.by {
            return Err(crate::routing::refusals::answered(message, by));
        }
    }
    writable_in(core, limits, party, number, &tasks::CompoundAdmission::new(&core.tasks, 0), true)?;
    for line in &row.question_lines {
        if line.message == message && line.answer.is_none() {
            return Ok(());
        }
    }
    Err(people::Refusal::Authority { proposable: None, lacking: people::Lack::Reference })
}

/// The offered inbox entry grants its named party the answer, including a
/// role holder acting as a person, not other writes to the task.
pub(crate) fn at_question_offer(
    ticket: tasks::QuestionTicket,
    link: u64,
    project: u32,
    party: u64,
    role: Option<people::Role>,
) -> bool {
    if role.is_none() || ticket.asker != link {
        return false;
    }
    let address = match ticket.holder {
        Some(tasks::EscalationHolder::Person(person)) => tasks::PersonAddress::Person(person),
        Some(tasks::EscalationHolder::Role { project: held_project, role }) if held_project == project => {
            tasks::PersonAddress::Role(role)
        }
        Some(tasks::EscalationHolder::Role { .. } | tasks::EscalationHolder::Task(_)) | None => return false,
    };
    crate::admission::person_task::address_matches(address, party, role)
}
