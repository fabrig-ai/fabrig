use jig_client_protocol_wire::Record as _;
use super::fixtures;
use crate::translate;
use alloc::boxed::Box;
use jig_client_protocol_wire as w;
use jig_client_test_records as records;
use jig_domain_core_vocabulary::{documents as d, reads};
use jig_domain_people as p;
#[test]
fn a_pending_proposal_becomes_its_owner_record() {
    let answer = reads::Answer::Document(Box::new(d::Document::Proposal(Box::new(fixtures::pending_proposal()))));
    assert_eq!(
        translate::document(&answer, &[], &crate::Private::none(), &fixtures::wire_limits()),
        Ok(w::ReadValue::Proposal(records::pending_proposal_record()))
    );
}
#[test]
#[expect(clippy::too_many_lines, reason = "whole proposal facts are checked for each maker, holder and action")]
fn pending_party_and_task_actions_cross_whole_for_maker_and_holder() {
    use jig_client_domain::contract as c;
    use jig_domain_core_vocabulary::acts;
    use skein_lib::{List, Wall};
    let mut batch = fixtures::pending_proposal();
    batch.action = d::ProposalAction::Batch(List::from_boxed(Box::new([fixtures::proposed_task()])));
    for (kind, source) in
        [("amend", fixtures::pending_amendment()), ("release", fixtures::pending_proposal()), ("batch", batch)]
    {
        for (proposer, expected) in
            [(d::Proposer::Party(11), c::Proposer::Party(11)), (d::Proposer::Task(17), c::Proposer::Task(17))]
        {
            for maker in [true, false] {
                let mut source = source.clone();
                source.proposer = proposer;
                let offers: &[acts::Act] = if maker {
                    &[acts::Act::Withdraw]
                } else {
                    &[acts::Act::Accept, acts::Act::Reject, acts::Act::PassUp]
                };
                source.acts = List::with_capacity(3);
                for act in offers {
                    source
                        .acts
                        .push(acts::Offered { act: *act, admission: acts::Admission::Allowed, effect: None })
                        .unwrap();
                }
                let value = translate::document(
                    &reads::Answer::Document(Box::new(d::Document::Proposal(Box::new(source.clone())))),
                    &[],
                    &crate::Private::none(),
                    &fixtures::wire_limits(),
                )
                .unwrap();
                let w::ReadValue::Proposal(wire) = &value else { panic!("proposal wire") };
                assert_eq!(wire.key.number, 13);
                match &wire.key.proposer {
                    w::Proposer::Party(party) => {
                        assert_eq!(party.value, 11);
                        assert_eq!(proposer, d::Proposer::Party(11));
                    }
                    w::Proposer::Task(task) => {
                        assert_eq!(task.value, 17);
                        assert_eq!(proposer, d::Proposer::Task(17));
                    }
                }
                let c::ReadValue::Proposal(value) = client_value(&value) else { panic!("proposal") };
                assert_eq!(value.key, c::ProposalKey { proposer: expected, number: 13 });
                assert_eq!(value.project, 7);
                assert_eq!(value.reason.bytes(), b"pending");
                assert!(!value.reason.cut);
                assert!(value.as_holder);
                assert!(value.projects.is_empty());
                assert_eq!(
                    value.waiting,
                    c::ProposalWaiting {
                        holder: c::HistoryHolder::Policy { project: 7, kind: c::MessageProposalKind::Amend },
                        since: Wall::from_nanos(41),
                    }
                );
                assert_eq!(
                    value.funding.as_slice(),
                    &[
                        c::FundingOffer { source: c::Funder::Task(17), needed: 19, left: None, after: Some(23) },
                        c::FundingOffer {
                            source: c::Funder::Period { project: 7, period: 29 },
                            needed: 31,
                            left: Some(0),
                            after: None
                        },
                    ]
                );
                let expected_acts: &[c::ActKind] = if maker {
                    &[c::ActKind::Withdraw]
                } else {
                    &[c::ActKind::Accept, c::ActKind::Reject, c::ActKind::PassUp]
                };
                assert_eq!(usize::try_from(value.acts.len()).unwrap(), expected_acts.len());
                for (offer, expected) in value.acts.iter().zip(expected_acts) {
                    assert_eq!(offer.act, *expected);
                    assert_eq!(offer.admission, c::Admission::Allowed);
                    assert!(offer.effect.is_none());
                }
                match value.action {
                    c::ProposalAction::Amend { task, addressed, before, after, reason, release } => {
                        assert_eq!(kind, "amend");
                        assert_eq!(task, c::TaskLink::Stored { number: 37, status: None });
                        assert_eq!(
                            addressed,
                            Some(c::AddressChange {
                                revision: c::Revision { value: 47 },
                                address: c::PersonAddress::Role(5)
                            })
                        );
                        assert_eq!(before.authority.authority.spend, 7);
                        assert_eq!(after.authority.authority.spend, 71);
                        assert_eq!(before.authority.authority.tools, 7);
                        assert_eq!(before.spec, after.spec);
                        assert_eq!(before.instructions.as_ref().unwrap().bytes(), b"before instructions");
                        assert_eq!(before.dependencies.as_slice(), &[43]);
                        assert_eq!(after.dependencies, before.dependencies);
                        assert_eq!(before.addressed, Some(c::PersonAddress::Party(11)));
                        assert_eq!(before.taken_by, Some(11));
                        assert_eq!(after.addressed, Some(c::PersonAddress::Role(5)));
                        assert_eq!(after.taken_by, None);
                        assert_eq!(reason.bytes(), b"widen authority");
                        assert!(!reason.cut);
                        assert_eq!(release, Some(53));
                    }
                    c::ProposalAction::Batch(batch) => {
                        assert_eq!(kind, "batch");
                        let [task] = batch.as_slice() else { panic!("one batch member") };
                        assert_eq!(task.number, 37);
                        assert_eq!(task.budget, 61);
                        assert_eq!(task.dependencies.as_slice(), &[43]);
                    }
                    c::ProposalAction::Release { task, hold } => {
                        assert_eq!(kind, "release");
                        assert_eq!(task, c::TaskLink::Stored { number: 37, status: None });
                        assert!(hold.is_none());
                    }
                    action @ (c::ProposalAction::Settings(_)
                    | c::ProposalAction::Effect { .. }
                    | c::ProposalAction::ProcedureEffect { .. }
                    | c::ProposalAction::Widen { .. }) => panic!("unexpected action {action:?}"),
                }
            }
        }
    }
}

fn client_value(value: &w::ReadValue) -> jig_client_domain::contract::ReadValue {
    let mut out = skein_lib::Writer::new(usize::try_from(value.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap());
    value.write(&jig_client_protocol_wire::CEILINGS, &mut out).unwrap();
    jig_client_protocol_link::translate::reads::value(&out.finish(), &fixtures::client_limits()).unwrap()
}

#[test]
fn a_decided_escalation_keeps_its_decider_and_wall() {
    let limits = fixtures::wire_limits();
    let source = fixtures::decided_escalation(p::EscalationDecision::Release);
    let expected_source = records::decided_escalation_record(w::EscalationDecision::Release);
    let rows = [
        ("released", p::EscalationDecision::Release, w::EscalationDecision::Release),
        ("passed", p::EscalationDecision::Pass, w::EscalationDecision::Pass),
        (
            "rejected",
            p::EscalationDecision::Reject { reason: Box::from(b"reason".as_slice()) },
            w::EscalationDecision::Reject(
                w::EscalationDecisionReject { reason: Box::from(b"reason".as_slice()) },
            ),
        ),
    ];
    for (label, decision, expected) in rows {
        let answer = with_decision(&source, decision);
        assert_eq!(
            translate::document(&answer, &[], &crate::Private::none(), &limits),
            Ok(record_with_decision(&expected_source, expected)),
            "{label}"
        );
    }
}
#[test]
fn an_ended_task_keeps_its_ending() {
    let answer = reads::Answer::Ended(Box::new(fixtures::ended_task()));
    assert_eq!(
        translate::document(&answer, &[], &crate::Private::none(), &fixtures::wire_limits()),
        Ok(w::ReadValue::Ended(Box::new(records::ended_task_record())))
    );
}
#[test]
fn a_project_document_becomes_its_record() {
    let answer = reads::Answer::Document(Box::new(d::Document::Project(Box::new(fixtures::project_document()))));
    assert_eq!(
        translate::document(&answer, &[], &crate::Private::none(), &fixtures::wire_limits()),
        Ok(w::ReadValue::Project(records::owner_project_record()))
    );
}

#[test]
fn an_escalations_hold_keeps_its_lifecycle_facts() {
    use jig_domain_tasks as t;
    use skein_lib::Wall;
    let mut source = fixtures::decided_escalation(p::EscalationDecision::Release);
    let reads::EndedValue::Escalation { hold, .. } = &mut source.value else { panic!("escalation") };
    *hold = Some(d::HoldLine {
        task: 7,
        position: 31,
        at: Wall::from_nanos(37),
        before: d::PriorPhase::Active(t::Active::Running { attempt: 3 }),
        why: t::Hold::Stopped,
        source: None,
        ending: None,
    });
    let limits = fixtures::wire_limits();
    let w::ReadValue::Ended(value) =
        translate::document(&reads::Answer::Ended(Box::new(source)), &[], &crate::Private::none(), &limits).unwrap()
    else {
        panic!("ended")
    };
    let w::EndedValue::Escalation(value) = &value.value else { panic!("escalation") };
    let hold = value.hold.as_ref().unwrap();
    assert_eq!(hold.task, 7);
    assert_eq!(hold.position, 31);
    assert_eq!(hold.at, 37);
    assert_eq!((&hold.why), &w::HoldCategory::Stopped);
    let w::PriorPhase::Active(active) = &hold.before else { panic!("running before hold") };
    assert_eq!(
        (&active.value),
        &w::RunStage::Running(w::RunStageRunning { attempt: 3 })
    );
}

fn with_decision(source: &reads::Ended, decision: p::EscalationDecision) -> reads::Answer {
    let mut source = source.clone();
    let reads::EndedValue::Escalation { decision: target, .. } = &mut source.value else {
        unreachable!("fixture is an escalation")
    };
    target.decision = decision;
    reads::Answer::Ended(Box::new(source))
}
fn record_with_decision(source: &w::Ended, decision: w::EscalationDecision) -> w::ReadValue {
    let mut parts = source.clone();
    let w::EndedValue::Escalation(value) = parts.value else { unreachable!("wire literal is an escalation") };
    let mut value = value;
    let mut kept = value.decision;
    kept.decision = decision;
    value.decision = kept;
    parts.value = w::EndedValue::Escalation(value);
    w::ReadValue::Ended(Box::new(parts))
}

#[test]
fn a_person_answer_notice_encodes_its_question_number() {
    let limits = fixtures::wire_limits();
    for question in [0, 1, u64::MAX] {
        let value = jig_domain_tasks::MessageKind::QuestionAnswered { question };
        let encoded = translate::objects::question_answer::message_kind(&value, &limits).expect("notice");
        let expected = w::MessageKind::QuestionAnswered(
            w::MessageKindAnswer { question },
        );
        assert_eq!(encoded, expected);
    }
}

#[test]
#[expect(clippy::too_many_lines, reason = "retained identity, state and action facts cross both translations together")]
fn retained_party_histories_cross_under_the_party_key() {
    use jig_client_domain::contract as c;
    use jig_domain_tasks as t;
    use skein_lib::{List, Wall};
    let limits = fixtures::wire_limits();
    let amendment = d::RetainedAction::Amend {
        task: 37,
        change: Box::new(d::AmendmentChange {
            addressed: Some(t::AddressChange { revision: 47, address: t::PersonAddress::Role(5) }),
            instructions: None,
            release: Some(53),
            spec: None,
            authority: Some(fixtures::authority()),
            wake: None,
            dependencies: Some(List::from_boxed(Box::new([43]))),
            reason: d::Text { bytes: b"widen authority".as_slice().into(), cut: false },
            cut: false,
        }),
    };
    for (state, expected) in [
        (
            d::ProposalHistoryState::Pending(d::ProposalWaiting {
                holder: t::ProposalHolder::Policy { project: 7, kind: t::ProposalKind::Amend },
                since: Wall::from_nanos(41),
            }),
            c::ProposalHistoryState::Pending {
                holder: c::HistoryHolder::Policy { project: 7, kind: c::MessageProposalKind::Amend },
                since: Wall::from_nanos(41),
            },
        ),
        (
            d::ProposalHistoryState::Accepted { by: t::Party::Person(19) },
            c::ProposalHistoryState::Accepted { by: c::Requester::Person(19) },
        ),
        (
            d::ProposalHistoryState::Rejected {
                by: t::Party::Person(19),
                reason: d::Text { bytes: b"no".as_slice().into(), cut: false },
            },
            c::ProposalHistoryState::Rejected {
                by: c::Requester::Person(19),
                reason: c::DocumentText { bytes: c::Text::new(b"no".as_slice().into(), 128).unwrap(), cut: false },
            },
        ),
        (d::ProposalHistoryState::Withdrawn, c::ProposalHistoryState::Withdrawn),
    ] {
        let mut batch = List::with_capacity(2);
        batch.push(fixtures::proposed_task()).unwrap();
        let goals = if state == d::ProposalHistoryState::Withdrawn { 1 } else { 2 };
        if goals == 2 {
            let mut second = fixtures::proposed_task();
            second.number = 38;
            batch.push(second).unwrap();
        }
        for (kind, action) in [("amend", amendment.clone()), ("batch", d::RetainedAction::Batch(batch))] {
            let source = reads::Answer::Ended(Box::new(reads::Ended {
                value: reads::EndedValue::PartyProposal(d::ProposalHistory {
                    proposer: 11,
                    number: 13,
                    reason: d::Text { bytes: b"pending".as_slice().into(), cut: false },
                    as_holder: true,
                    action,
                    state: state.clone(),
                }),
                at: Some(Wall::from_nanos(67)),
                by: Some(t::Party::Person(19)),
            }));
            let wire = translate::document(&source, &[], &crate::Private::none(), &limits).unwrap();
            let w::ReadValue::Ended(ended) = &wire else { panic!("ended wire") };
            let w::ObjectKey::Proposal(key) = &ended.key else { panic!("proposal wire key") };
            assert_eq!(key.value.number, 13);
            let w::Proposer::Party(party) = &key.value.proposer else { panic!("party wire key") };
            assert_eq!(party.value, 11);
            let w::EndedValue::Proposal(_) = &ended.value else { panic!("retained wire proposal") };
            let c::ReadValue::Ended(ended) = client_value(&wire) else { panic!("ended") };
            let key = c::ObjectKey::Proposal(c::ProposalKey { proposer: c::Proposer::Party(11), number: 13 });
            assert_eq!(ended.key, key);
            assert!(ended.matches(key));
            assert_eq!(ended.at, Some(Wall::from_nanos(67)));
            assert_eq!(ended.by, Some(c::Requester::Person(19)));
            let c::EndedValue::Proposal(history) = ended.value else { panic!("retained proposal") };
            assert_eq!(history.proposer, 11);
            assert_eq!(history.number, 13);
            assert_eq!(history.reason.bytes(), b"pending");
            assert!(history.as_holder);
            assert_eq!(history.state, expected);
            match history.action {
                c::RetainedAction::Amend { task, change } => {
                    assert_eq!(kind, "amend");
                    assert_eq!(task, 37);
                    assert_eq!(
                        change.addressed,
                        Some(c::AddressChange {
                            revision: c::Revision { value: 47 },
                            address: c::PersonAddress::Role(5)
                        })
                    );
                    assert_eq!(change.release, Some(53));
                    assert_eq!(change.authority.as_ref().unwrap().authority.spend, 7);
                    assert_eq!(change.dependencies.as_ref().unwrap().as_slice(), &[43]);
                    assert_eq!(change.reason.bytes(), b"widen authority");
                }
                c::RetainedAction::Batch(batch) => {
                    assert_eq!(kind, "batch");
                    assert_eq!(batch.len(), goals);
                    let task = batch.get(0).unwrap();
                    assert_eq!(task.number, 37);
                    assert_eq!(task.budget, 61);
                }
                action @ (c::RetainedAction::Settings(_)
                | c::RetainedAction::Effect { .. }
                | c::RetainedAction::ProcedureEffect { .. }
                | c::RetainedAction::Widen { .. }
                | c::RetainedAction::Release { .. }) => panic!("retained action {action:?}"),
            }
        }
    }
}

#[test]
fn decided_single_goal_party_batches_keep_the_person_form() {
    use jig_domain_tasks as t;
    use skein_lib::Wall;
    for state in [
        t::PersonProposalState::Accepted { by: t::Party::Person(19) },
        t::PersonProposalState::Rejected {
            by: t::Party::Person(19),
            reason: b"no".as_slice().into(),
            message: 59,
            at: Wall::from_nanos(61),
        },
    ] {
        let proposal = t::PersonProposal {
            proposer: 11,
            number: 13,
            project: 7,
            action: t::PersonAction::Batch(Box::new([fixtures::person_goal()])),
            reason: b"pending".as_slice().into(),
            state: state.clone(),
        };
        let wire = translate::document(
            &reads::Answer::Ended(Box::new(reads::Ended {
                value: reads::EndedValue::PersonProposal { proposal: Box::new(proposal), decision: None },
                at: None,
                by: None,
            })),
            &[],
            &crate::Private::none(),
            &fixtures::wire_limits(),
        )
        .unwrap();
        let w::ReadValue::Ended(ended) = wire else { panic!("ended") };
        let w::EndedValue::PersonProposal(value) = &ended.value else { panic!("person proposal") };
        assert_eq!(value.proposal.proposer, 11);
        assert_eq!(value.proposal.number, 13);
        assert_eq!(value.proposal.goal.number, 37);
        match state {
            t::PersonProposalState::Accepted { .. } => {
                let w::PersonProposalState::Accepted(_) = &value.proposal.state else { panic!("accepted") };
            }
            t::PersonProposalState::Rejected { .. } => {
                let w::PersonProposalState::Rejected(_) = &value.proposal.state else { panic!("rejected") };
            }
            state @ (t::PersonProposalState::Pending { .. }
            | t::PersonProposalState::Withdrawn { .. }
            | t::PersonProposalState::SettingsAccepted { .. }) => panic!("state {state:?}"),
        }
    }
}

#[test]
fn ended_settings_facts_reach_the_client_whole() {
    use jig_client_domain::contract as c;
    use skein_lib::{List, Writer};
    let limits = fixtures::wire_limits();
    let mut projects = List::with_capacity(1);
    projects.push(23).unwrap();
    let source = reads::Answer::Ended(Box::new(reads::Ended {
        at: None,
        by: None,
        value: reads::EndedValue::Proposal(d::ProposalHistory {
            reason: d::Text { bytes: b"why".as_slice().into(), cut: true },
            number: 19,
            proposer: 7,
            as_holder: false,
            state: d::ProposalHistoryState::Accepted { by: jig_domain_tasks::Party::Person(11) },
            action: d::RetainedAction::Settings {
                against: 3,
                items: List::with_capacity(0),
                made: Some(5),
                projects: projects.clone(),
            },
        }),
    }));
    let record = translate::document(&source, &[], &crate::Private::none(), &limits).unwrap();
    let mut out = Writer::new(usize::try_from(record.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap());
    record.write(&jig_client_protocol_wire::CEILINGS, &mut out).unwrap();
    let c::ReadValue::Ended(ended) =
        jig_client_protocol_link::translate::reads::value(&out.finish(), &fixtures::client_limits()).unwrap()
    else {
        panic!("ended")
    };
    let c::EndedValue::Proposal(proposal) = ended.value else { panic!("proposal") };
    assert_eq!(proposal.reason.bytes(), b"why");
    assert!(proposal.reason.cut);
    let c::RetainedAction::Settings(action) = proposal.action else { panic!("settings") };
    assert_eq!(action.projects, projects);
    assert_eq!(action.made, Some(5));
}
