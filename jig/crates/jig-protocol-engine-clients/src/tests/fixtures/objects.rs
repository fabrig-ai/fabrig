//! Independent core literals for the single-read proof.
use super::{authority, chip, task_with_hold};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::{documents as d, reads};
use jig_domain_tasks as t;
use skein_lib::{List, Wall};
pub fn pending_proposal() -> d::Proposal {
    let mut funding = List::with_capacity(2);
    for (source, needed, left, after) in
        [(t::Funder::Task(17), 19, None, Some(23)), (t::Funder::Period { project: 7, period: 29 }, 31, Some(0), None)]
    {
        funding.push(d::FundingOffer { source, needed, left, after }).expect("two offers");
    }
    d::Proposal {
        projects: List::with_capacity(0),
        acts: List::with_capacity(0),
        proposer: d::Proposer::Party(11),
        number: 13,
        project: 7,
        reason: d::Text { bytes: Box::from(b"pending".as_slice()), cut: false },
        action: d::ProposalAction::Release { task: d::TaskLink::Stored { number: 37, status: None }, hold: None },
        as_holder: true,
        waiting: d::ProposalWaiting {
            holder: t::ProposalHolder::Policy { project: 7, kind: t::ProposalKind::Amend },
            since: Wall::from_nanos(41),
        },
        funding,
    }
}
pub fn pending_amendment() -> d::Proposal {
    let mut proposal = pending_proposal();
    let task = task_with_hold();
    let before = d::AmendmentState {
        addressed: Some(t::PersonAddress::Person(11)),
        taken_by: Some(11),
        instructions: Some(d::Text { bytes: b"before instructions".as_slice().into(), cut: false }),
        spec: task.spec,
        authority: authority(),
        wake: task.wake,
        dependencies: List::from_boxed(Box::new([43])),
        cut: false,
    };
    let mut after = before.clone();
    after.authority.ceiling.spend = 71;
    after.addressed = Some(t::PersonAddress::Role(5));
    after.taken_by = None;
    proposal.action = d::ProposalAction::Amend {
        task: d::TaskLink::Stored { number: 37, status: None },
        addressed: Some(t::AddressChange { revision: 47, address: t::PersonAddress::Role(5) }),
        before: Box::new(before),
        after: Box::new(after),
        reason: d::Text { bytes: b"widen authority".as_slice().into(), cut: false },
        release: Some(53),
    };
    proposal
}

pub fn proposed_task() -> d::ProposedTask {
    let task = task_with_hold();
    d::ProposedTask {
        number: 37,
        deadline: t::Deadline::Named,
        executor: task.executor,
        spec: task.spec,
        contract: d::ContractLine::Report { words: 59 },
        authority: task.authority,
        budget: 61,
        funder: t::Funder::Task(17),
        dependencies: List::from_boxed(Box::new([43])),
        resources: List::with_capacity(0),
        wake: task.wake,
        priority: Some(3),
        cut: false,
    }
}

pub fn person_goal() -> t::New {
    t::New {
        charter_scope: jig_domain_tasks::CharterScope::Project,
        label: 0,
        flags: t::Flags::PLAIN,
        number: 37,
        project: 7,
        deadline: t::Deadline::Named,
        executor: t::Executor::Person(t::PersonAddress::Person(11)),
        llm: None,
        spec: t::Spec {
            job: None,
            asks: false,
            words: b"a goal".as_slice().into(),
            parameters: Box::new([]),
            inputs: Box::new([]),
        },
        contract: t::Contract::Report { words: 59 },
        authority: t::Authority {
            models: t::Models::ALL,
            tools: t::Tools(7),
            grants: Box::new([]),
            delegation: t::Delegation { kinds: Box::new([]), tasks: 7, depth: 7 },
            budget: t::Budget { spend: 61, deadline: None },
            notes: t::Scopes(7),
            note_resources: Box::new([]),
        },
        numbers: t::Numbers { budget: 61, spent: 0, spent_below: 0, reserved: 0 },
        funder: t::Funder::Task(17),
        dependencies: Box::new([43]),
        holdings: Box::new([]),
        wake: super::wake(),
        recurring: None,
        tracked: Some(3),
    }
}

pub fn failed_result() -> d::ResultDoc {
    d::ResultDoc {
        acts: List::with_capacity(0),
        task: chip(),
        contract: d::ContractLine::Report { words: 53 },
        ending: d::Ending::Failed(d::Text { bytes: Box::from(b"failed".as_slice()), cut: false }),
        at: Some(Wall::from_nanos(47)),
        requester: t::Party::Person(59),
        executor: t::Executor::Agent { charter: 7 },
        chosen_by: Some(61),
        followups: Some(List::with_capacity(0)),
        delegates: List::with_capacity(0),
        proposal: None,
        cut: true,
    }
}
pub fn ended_task() -> reads::Ended {
    reads::Ended {
        value: reads::EndedValue::Task { document: Box::new(task_with_hold()), result: Box::new(failed_result()) },
        at: Some(Wall::from_nanos(67)),
        by: Some(t::Party::Person(71)),
    }
}
pub fn decided_escalation(decision: jig_domain_people::EscalationDecision) -> reads::Ended {
    reads::Ended {
        value: reads::EndedValue::Escalation {
            task: Box::new(task_with_hold()),
            decision: jig_domain_core_vocabulary::EscalationDecisionRecord {
                project: 7,
                requester: 73,
                task: 7,
                revision: 79,
                by: 83,
                at: Wall::from_nanos(89),
                decision,
            },
            hold: None,
        },
        at: Some(Wall::from_nanos(97)),
        by: Some(t::Party::Person(101)),
    }
}
pub fn project_document() -> d::Project {
    d::Project {
        acts: List::with_capacity(0),
        number: 7,
        name: jig_domain_core_vocabulary::documents::Text { bytes: Box::from(*b"Tally"), cut: false },
        resources: List::with_capacity(0),
        policy: d::Policy {
            templates: List::with_capacity(0),
            ceiling: authority(),
            period_spend: 7,
            escalation_role: None,
            projections: List::with_capacity(0),
            roles: List::with_capacity(0),
            requirements: List::with_capacity(0),
            permissions: None,
            permissions_cut: false,
            cut: false,
        },
        members: List::with_capacity(0),
        current_period: None,
        pools: List::with_capacity(0),
        configured_period: 7,
        configured_pools: List::with_capacity(0),
        configured_pools_cut: false,
        spend: List::with_capacity(0),
        recurring: List::with_capacity(0),
        cut: false,
    }
}

pub fn template() -> jig_domain_core_vocabulary::Template {
    jig_domain_core_vocabulary::Template {
        name: None,
        resources: Box::new([]),
        number: 1,
        shown: b"task".as_slice().into(),
        executor: jig_domain_people::TemplateExecutor::Charter(1),
        contract: t::Contract::Report { words: 32 },
        shallower: 1,
        budget: jig_domain_people::TemplateBudget::Amount(1),
        deadline: jig_domain_people::TemplateDeadline::Given,
        tracked: None,
        flags: t::Flags::PLAIN,
        label: 0,
        grace: None,
        llm: None,
    }
}
