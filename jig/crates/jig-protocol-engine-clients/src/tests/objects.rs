use super::fixtures;
use crate::translate;
use alloc::boxed::Box;
use jig_client_protocol_wire::v20 as w;
use jig_client_test_records as records;
use jig_domain_core::{documents as d, reads};
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
                w::EscalationDecisionReject::new(
                    &limits,
                    w::EscalationDecisionRejectParts { reason: Box::from(b"reason".as_slice()) },
                )
                .expect("literal reject"),
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

fn with_decision(source: &reads::Ended, decision: p::EscalationDecision) -> reads::Answer {
    let mut source = source.clone();
    let reads::EndedValue::Escalation { decision: target, .. } = &mut source.value else {
        unreachable!("fixture is an escalation")
    };
    target.decision = decision;
    reads::Answer::Ended(Box::new(source))
}
fn record_with_decision(source: &w::Ended, decision: w::EscalationDecision) -> w::ReadValue {
    let limits = fixtures::wire_limits();
    let mut parts = source.clone().into_parts();
    let w::EndedValue::Escalation(value) = parts.value else { unreachable!("wire literal is an escalation") };
    let mut value = value.into_parts();
    let mut kept = value.decision.into_parts();
    kept.decision = decision;
    value.decision = w::EscalationDecisionRecord::new(&limits, kept).expect("literal decision");
    parts.value = w::EndedValue::Escalation(w::EndedValueEscalation::new(&limits, value).expect("literal escalation"));
    w::ReadValue::Ended(Box::new(w::Ended::new(&limits, parts).expect("literal ended")))
}

#[test]
fn a_person_answer_notice_encodes_its_question_number() {
    let limits = fixtures::wire_limits();
    for question in [0, 1, u64::MAX] {
        let value = jig_domain_tasks::MessageKind::QuestionAnswered { question };
        let encoded = translate::objects::question_answer::message_kind(&value, &limits).expect("notice");
        let expected = w::MessageKind::QuestionAnswered(
            w::MessageKindAnswer::new(&limits, w::MessageKindAnswerParts { question }).expect("number"),
        );
        assert_eq!(encoded, expected);
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
    let mut out = Writer::new(usize::try_from(record.measure()).unwrap());
    record.encode(&mut out).unwrap();
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
