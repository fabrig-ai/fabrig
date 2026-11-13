//! Whole party-specific offers (`client/domain/contract/documents.md`, 3;
//! `client/domain/objects.md`, 11). The acts owner supplies every judgment.

pub use jig_domain_core_vocabulary::documents::offers::*;

use crate::{
    Core, Limits,
    acts::{self, Act, Admission, Offered},
    documents::{Missing, Subject},
};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::List;

fn list(
    core: &Core,
    limits: &Limits,
    party: u64,
    subject: &Subject,
    kinds: &[Act],
    decision: skein_lib::Wall,
) -> Result<List<Offered>, Missing> {
    let mut result = List::with_capacity(u32::try_from(kinds.len()).expect("closed act list"));
    for act in kinds {
        let value = acts::admit(core, limits, party, *act, subject, decision);
        let admission = receiving(value, &limits.documents)?;
        result
            .push(Offered { effect: effect(core, limits, *act, subject), act: *act, admission })
            .expect("one offer per listed act");
    }
    if heap_bytes(&result).ok_or(Missing::TooLarge)? > heap_bound(&limits.documents).ok_or(Missing::TooLarge)? {
        return Err(Missing::TooLarge);
    }
    Ok(result)
}

pub(super) fn task(
    core: &Core,
    limits: &Limits,
    party: u64,
    row: &tasks::TaskRecord,
    decision: skein_lib::Wall,
) -> Result<List<Offered>, Missing> {
    let mut kinds = List::with_capacity(11);
    let mutable = match row.phase {
        tasks::Phase::Waiting
        | tasks::Phase::Active(_)
        | tasks::Phase::Held { was: tasks::Was::Waiting | tasks::Was::Active(_), .. } => true,
        tasks::Phase::Closing(_) | tasks::Phase::Held { was: tasks::Was::Closing(_), .. } | tasks::Phase::Ended(_) => {
            false
        }
    };
    if mutable {
        if let tasks::Executor::Agent { .. } = row.executor {
            kinds.push(Act::Message).expect("task act bound");
            if row.flags.ended_by == tasks::EndedBy::Standing {
                kinds.push(Act::Talk).expect("task act bound");
                kinds.push(Act::Leave).expect("task act bound");
            }
        }
        kinds.push(Act::Amend).expect("task act bound");
        kinds.push(Act::Move).expect("task act bound");
    }
    match row.phase {
        tasks::Phase::Held { .. } => {
            kinds.push(Act::Release).expect("task act bound");
        }
        tasks::Phase::Active(_) => match row.executor {
            tasks::Executor::Agent { .. } | tasks::Executor::Procedure { .. } => {
                kinds.push(Act::Stop).expect("task act bound");
            }
            tasks::Executor::Person(_) | tasks::Executor::Job { .. } => {}
        },
        tasks::Phase::Waiting | tasks::Phase::Closing(_) | tasks::Phase::Ended(_) => {}
    }
    match row.phase {
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
            kinds.push(Act::Cancel).expect("task act bound");
            if core.conversation_requester(row).is_some() {
                kinds.push(Act::Seen).expect("task act bound");
            }
        }
        tasks::Phase::Ended(_) => {}
    }
    list(core, limits, party, &Subject::Task { task: row.number }, kinds.as_slice(), decision)
}

pub(super) fn subject(
    core: &Core,
    limits: &Limits,
    party: u64,
    subject: &Subject,
    decision: skein_lib::Wall,
) -> Result<List<Offered>, Missing> {
    match subject {
        Subject::Project { .. } => list(
            core,
            limits,
            party,
            subject,
            &[Act::Adopt, Act::Reorder, Act::ChangePolicy, Act::Rename, Act::SetRoles, Act::MakeService, Act::SetPool],
            decision,
        ),
        Subject::Proposal { proposer: super::Proposer::Party(person), number }
            if match core.tasks.person_proposal(*person, *number) {
                Some(row) => row.action.is_settings(),
                None => false,
            } =>
        {
            let owner = core.people.role(party, people::DEPLOYMENT) == Some(people::Role::Owner);
            let kinds = if owner && *person == party {
                &[Act::Accept, Act::Reject, Act::Withdraw][..]
            } else if owner {
                &[Act::Accept, Act::Reject][..]
            } else if *person == party {
                &[Act::Withdraw][..]
            } else {
                &[][..]
            };
            list(core, limits, party, subject, kinds, decision)
        }
        Subject::Proposal { proposer, .. } => {
            let kinds = if *proposer == super::Proposer::Party(party) {
                &[Act::Accept, Act::Reject, Act::PassUp, Act::Withdraw][..]
            } else {
                &[Act::Accept, Act::Reject, Act::PassUp][..]
            };
            list(core, limits, party, subject, kinds, decision)
        }
        Subject::Escalation { task } => {
            let row = core.tasks.task(*task).ok_or(Missing::Unknown)?;
            let kinds = match row.phase {
                tasks::Phase::Held { was: tasks::Was::Closing(_), .. } => {
                    &[Act::Release, Act::LeaveHeld, Act::PassUp, Act::Cancel][..]
                }
                tasks::Phase::Waiting
                | tasks::Phase::Active(_)
                | tasks::Phase::Closing(_)
                | tasks::Phase::Held { was: tasks::Was::Waiting | tasks::Was::Active(_), .. }
                | tasks::Phase::Ended(_) => &[Act::Release, Act::LeaveHeld, Act::PassUp, Act::Amend, Act::Cancel][..],
            };
            list(core, limits, party, subject, kinds, decision)
        }
        Subject::Question { .. } => list(core, limits, party, subject, &[Act::Answer], decision),
        Subject::Result { .. } => list(core, limits, party, &Subject::Inbox { party }, &[Act::Seen], decision),
        Subject::Frame { .. }
        | Subject::Task { .. }
        | Subject::Tree { .. }
        | Subject::Chips { .. }
        | Subject::Inbox { .. }
        | Subject::Terminal { .. }
        | Subject::Accounts { .. }
        | Subject::Query { .. }
        | Subject::Run { .. }
        | Subject::Settings
        | Subject::Deployment
        | Subject::PersonTask { .. }
        | Subject::Note { .. }
        | Subject::Application { .. } => Ok(List::with_capacity(0)),
    }
}

pub(super) fn person(
    core: &Core,
    limits: &Limits,
    party: u64,
    row: &tasks::TaskRecord,
    decision: skein_lib::Wall,
) -> Result<List<Offered>, Missing> {
    match row.phase {
        tasks::Phase::Active(tasks::Active::Due | tasks::Active::Idle) => {
            if !row.waiting_on.is_empty() {
                let kinds = match row.executor {
                    tasks::Executor::Person(tasks::PersonAddress::Role(_)) => &[Act::Take, Act::HandBack][..],
                    tasks::Executor::Job { .. }
                    | tasks::Executor::Person(tasks::PersonAddress::Person(_))
                    | tasks::Executor::Agent { .. }
                    | tasks::Executor::Procedure { .. } => &[][..],
                };
                return list(core, limits, party, &Subject::PersonTask { task: row.number }, kinds, decision);
            }
            let kinds = match row.executor {
                tasks::Executor::Job { .. } => &[][..],
                tasks::Executor::Person(tasks::PersonAddress::Role(_)) => &[Act::Answer, Act::Take, Act::HandBack][..],
                tasks::Executor::Person(tasks::PersonAddress::Person(_))
                | tasks::Executor::Agent { .. }
                | tasks::Executor::Procedure { .. } => &[Act::Answer][..],
            };
            list(core, limits, party, &Subject::PersonTask { task: row.number }, kinds, decision)
        }
        tasks::Phase::Waiting
        | tasks::Phase::Active(
            tasks::Active::Preparing
            | tasks::Active::Claimed { .. }
            | tasks::Active::Running { .. }
            | tasks::Active::BackingOff { .. },
        )
        | tasks::Phase::Closing(_)
        | tasks::Phase::Held { .. }
        | tasks::Phase::Ended(_) => Ok(List::with_capacity(0)),
    }
}

pub(super) fn note(core: &Core, party: u64, project: u32, scope: &people::NoteScope) -> List<Offered> {
    let mut result = List::with_capacity(2);
    for act in [Act::EditNote, Act::DeleteNote] {
        let admission = match acts::note(core, party, project, scope, act) {
            Ok(()) => Admission::Allowed,
            Err(why) => Admission::Refused(why),
        };
        result.push(Offered { effect: None, act, admission }).expect("two note acts");
    }
    result
}

fn effect(core: &Core, limits: &Limits, act: Act, subject: &Subject) -> Option<acts::Effect> {
    let number = match subject {
        Subject::Task { task } | Subject::Escalation { task } => *task,
        Subject::Application { .. }
        | Subject::Frame { .. }
        | Subject::Tree { .. }
        | Subject::Chips { .. }
        | Subject::Inbox { .. }
        | Subject::Terminal { .. }
        | Subject::Accounts { .. }
        | Subject::Query { .. }
        | Subject::Run { .. }
        | Subject::Settings
        | Subject::Deployment
        | Subject::Proposal { .. }
        | Subject::Question { .. }
        | Subject::PersonTask { .. }
        | Subject::Result { .. }
        | Subject::Project { .. }
        | Subject::Note { .. } => return None,
    };
    let row = core.tasks.task(number)?;
    match act {
        Act::Release => {
            let tasks::Phase::Held { why, .. } = row.phase else { return None };
            let what = match why {
                tasks::Hold::Uncertain { .. } => acts::ReleaseEffect::Uncertain,
                tasks::Hold::Exceeded(_)
                | tasks::Hold::Failures(_)
                | tasks::Hold::Stalled
                | tasks::Hold::Stopped
                | tasks::Hold::Placed { .. } => acts::ReleaseEffect::RunAgain,
                tasks::Hold::Dependency(_)
                | tasks::Hold::Drift
                | tasks::Hold::Effects
                | tasks::Hold::EffectFailed
                | tasks::Hold::Budget
                | tasks::Hold::Deadline
                | tasks::Hold::HoldsWaited
                | tasks::Hold::ResourceUnavailable
                | tasks::Hold::ResourceAccess
                | tasks::Hold::PoolLost { .. }
                | tasks::Hold::Circle
                | tasks::Hold::Procedure => acts::ReleaseEffect::Judged,
            };
            Some(acts::Effect::Release(what))
        }
        Act::Cancel => {
            let mut impact = acts::Impact { tasks: super::PhaseCounts::ZERO, live_runs: 0, proposals: 0 };
            for candidate in core.tasks.view_tasks().as_ref() {
                if candidate.number != number
                    && !super::task_tree::descendant(core, candidate.number, number, limits.tasks.depth)
                {
                    continue;
                }
                let child = core.tasks.task(candidate.number).expect("live census task");
                if candidate.number != number {
                    impact.tasks.add(&child.phase);
                }
                if tasks::run_attempt(&child.phase).is_some() {
                    impact.live_runs = impact.live_runs.checked_add(1).expect("live task bound");
                }
                if let Some(proposal) = &child.proposal
                    && let tasks::ProposalState::Pending { .. } = proposal.state
                {
                    impact.proposals = impact.proposals.checked_add(1).expect("live task bound");
                }
            }
            Some(acts::Effect::Cancel(impact))
        }
        Act::Move => Some(acts::Effect::Move { reservation: core.tasks.takeover_budget(number)? }),
        Act::Talk
        | Act::Leave
        | Act::Message
        | Act::Answer
        | Act::Take
        | Act::HandBack
        | Act::Accept
        | Act::Reject
        | Act::PassUp
        | Act::LeaveHeld
        | Act::Amend
        | Act::Stop
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
        | Act::SetPool
        | Act::Withdraw => None,
    }
}
