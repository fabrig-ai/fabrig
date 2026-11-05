//! Every act retains its proposal and escalation admission partition.
use super::*;

use crate::acts::Decision;

const PROPOSAL: u64 = 10;
const TASK: u64 = 1;
const HELD: u64 = 3;
const SETTINGS: u64 = 11;

#[test]
fn every_act_keeps_its_decision_admission() {
    let (mut core, limits) = make_tree();
    sign_in(&mut core, &limits);
    let mut next = new_row(&core, 5, tasks::Funder::Task(TASK), None);
    next.authority.delegation.tasks = 1;
    next.authority.delegation.depth = 1;
    send_task(
        &mut core,
        &limits,
        tasks::Event::Propose {
            reply_to: ReplyTo::new(Token::new(102)),
            proposal: tasks::Proposal {
                number: PROPOSAL,
                proposer: TASK,
                project: 1,
                action: tasks::ProposalAction::Batch(Box::new([next])),
                reason: Box::new([]),
                as_holder: true,
                state: tasks::ProposalState::Pending { holder: tasks::ProposalHolder::Person(1), since: Wall::EPOCH },
            },
        },
    );
    let mut goal = new_row(&core, 5, tasks::Funder::Pool { project: 1, person: 1, period: 1 }, Some(4));
    goal.authority.delegation.tasks = 1;
    goal.authority.delegation.depth = 1;
    send_task(
        &mut core,
        &limits,
        tasks::Event::ProposePerson {
            reply_to: ReplyTo::new(Token::new(103)),
            proposal: tasks::PersonProposal {
                number: PROPOSAL,
                proposer: 1,
                project: 1,
                action: tasks::PersonAction::Batch(Box::new([goal])),
                reason: Box::new([]),
                state: tasks::PersonProposalState::Pending { since: Wall::EPOCH },
            },
        },
    );
    send_task(&mut core, &limits, tasks::Event::Hold { task: HELD, why: tasks::Hold::Uncertain { entry: 1 } });
    let revision = core.tasks.escalation(HELD).unwrap().escalation.revision();
    send_task(
        &mut core,
        &limits,
        tasks::Event::RoutedEscalation { task: HELD, revision, holder: tasks::EscalationHolder::Person(1), entry: 1 },
    );
    let (mut settings_core, _) = make_tree();
    let mut out = Queue::uncapped();
    people::step(
        &mut settings_core.people,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people },
        people::Event::Roles {
            project: people::DEPLOYMENT,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
        },
        &mut out,
    );
    send_task(
        &mut settings_core,
        &limits,
        tasks::Event::Propose {
            reply_to: ReplyTo::new(Token::new(104)),
            proposal: tasks::Proposal {
                number: SETTINGS,
                proposer: 2,
                project: people::DEPLOYMENT,
                action: tasks::ProposalAction::Settings { against: 1, items: Box::new([]) },
                reason: Box::new([]),
                as_holder: false,
                state: tasks::ProposalState::Pending {
                    holder: tasks::ProposalHolder::Policy {
                        project: people::DEPLOYMENT,
                        kind: tasks::ProposalKind::Settings,
                    },
                    since: Wall::EPOCH,
                },
            },
        },
    );
    send_task(
        &mut settings_core,
        &limits,
        tasks::Event::ProposePerson {
            reply_to: ReplyTo::new(Token::new(105)),
            proposal: tasks::PersonProposal {
                number: SETTINGS,
                proposer: 1,
                project: people::DEPLOYMENT,
                action: tasks::PersonAction::Settings { against: 1, items: Box::new([]) },
                reason: Box::new([]),
                state: tasks::PersonProposalState::Pending { since: Wall::EPOCH },
            },
        },
    );
    let no = Err(people::Refusal::NotOffered);
    let further = Err(people::Refusal::NoFurther);
    let check = |act: Act, decision, task, party: Result<(), people::Refusal>, escalation, settings| {
        assert_eq!(act.decision(), decision, "{act:?}");
        assert_eq!(
            crate::acts::proposal(&core, &limits, 1, act, Proposer::Task(TASK), PROPOSAL, Wall::EPOCH),
            task,
            "{act:?} {decision:?}"
        );
        assert_eq!(
            crate::acts::proposal(&core, &limits, 1, act, Proposer::Party(1), PROPOSAL, Wall::EPOCH),
            party.clone(),
            "{act:?}"
        );
        assert_eq!(
            crate::acts::proposal(&settings_core, &limits, 1, act, Proposer::Task(2), SETTINGS, Wall::EPOCH),
            settings,
            "settings {act:?}"
        );
        assert_eq!(
            crate::acts::proposal(&settings_core, &limits, 1, act, Proposer::Party(1), SETTINGS, Wall::EPOCH),
            party,
            "party settings {act:?}"
        );
        assert_eq!(crate::acts::escalation(&core, &limits, 1, act, HELD), escalation, "{act:?}");
    };
    for act in [
        Act::RemoveProject,
        Act::Message,
        Act::Answer,
        Act::Take,
        Act::HandBack,
        Act::Amend,
        Act::Stop,
        Act::Cancel,
        Act::Move,
        Act::Reorder,
        Act::Seen,
        Act::EditNote,
        Act::DeleteNote,
        Act::Adopt,
        Act::ChangePolicy,
        Act::Rename,
        Act::SetRoles,
        Act::MakeService,
        Act::SetPool,
    ] {
        check(act, None, no.clone(), no.clone(), no.clone(), no.clone());
    }
    for (act, decision, task, party, escalation, settings) in [
        (Act::Accept, Some(Decision::Accept), Ok(()), Ok(()), no.clone(), Ok(())),
        (Act::Reject, Some(Decision::Reject), Ok(()), Ok(()), no.clone(), Ok(())),
        (Act::PassUp, Some(Decision::PassUp), Ok(()), further, Ok(()), Err(people::Refusal::NoFurther)),
        (Act::Release, Some(Decision::Release), no.clone(), no.clone(), Ok(()), no.clone()),
        (Act::LeaveHeld, Some(Decision::LeaveHeld), no.clone(), no.clone(), Ok(()), no.clone()),
        (Act::Withdraw, None, Err(people::Refusal::NotOffered), Ok(()), Err(people::Refusal::NotOffered), no.clone()),
    ] {
        check(act, decision, task, party, escalation, settings);
    }
}
