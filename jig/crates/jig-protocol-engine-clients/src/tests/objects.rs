use super::fixtures;
use crate::translate;
use alloc::boxed::Box;
use jig_client_protocol_wire::v19 as w;
use jig_client_test_records as records;
use jig_domain_core::{documents as d, reads};
use jig_domain_people as p;
#[test]
fn a_pending_proposal_becomes_its_owner_record() {
    let answer = reads::Answer::Document(Box::new(d::Document::Proposal(Box::new(fixtures::pending_proposal()))));
    assert_eq!(
        translate::document(&answer, &[], &fixtures::wire_limits()),
        Ok(w::ReadValue::OwnerProposal(records::pending_proposal_record()))
    );
}
#[test]
fn a_decided_escalation_keeps_its_decider_and_wall() {
    let limits = fixtures::wire_limits();
    let source = fixtures::decided_escalation(p::EscalationDecision::Release);
    let expected_source = records::decided_escalation_record(w::OwnerEscalationDecision::Release);
    let rows = [
        ("released", p::EscalationDecision::Release, w::OwnerEscalationDecision::Release),
        ("passed", p::EscalationDecision::Pass, w::OwnerEscalationDecision::Pass),
        (
            "rejected",
            p::EscalationDecision::Reject { reason: Box::from(b"reason".as_slice()) },
            w::OwnerEscalationDecision::Reject(
                w::OwnerEscalationDecisionReject::new(
                    &limits,
                    w::OwnerEscalationDecisionRejectParts { reason: Box::from(b"reason".as_slice()) },
                )
                .expect("literal reject"),
            ),
        ),
    ];
    for (label, decision, expected) in rows {
        let answer = with_decision(&source, decision);
        assert_eq!(
            translate::document(&answer, &[], &limits),
            Ok(record_with_decision(&expected_source, expected)),
            "{label}"
        );
    }
}
#[test]
fn a_historical_task_keeps_its_ending() {
    let answer = reads::Answer::Historical(Box::new(fixtures::historical_task()));
    assert_eq!(
        translate::document(&answer, &[], &fixtures::wire_limits()),
        Ok(w::ReadValue::Historical(Box::new(records::historical_task_record())))
    );
}
#[test]
fn a_project_document_becomes_its_record() {
    let answer = reads::Answer::Document(Box::new(d::Document::Project(Box::new(fixtures::project_document()))));
    assert_eq!(
        translate::document(&answer, &[], &fixtures::wire_limits()),
        Ok(w::ReadValue::OwnerProject(records::owner_project_record()))
    );
}

fn with_decision(source: &reads::Historical, decision: p::EscalationDecision) -> reads::Answer {
    let mut source = source.clone();
    let reads::Past::Escalation { decision: target, .. } = &mut source.value else {
        unreachable!("fixture is an escalation")
    };
    target.decision = decision;
    reads::Answer::Historical(Box::new(source))
}
fn record_with_decision(source: &w::OwnerHistorical, decision: w::OwnerEscalationDecision) -> w::ReadValue {
    let limits = fixtures::wire_limits();
    let mut parts = source.clone().into_parts();
    let w::OwnerHistoricalValue::Escalation(value) = parts.value else { unreachable!("wire literal is an escalation") };
    let mut value = value.into_parts();
    let mut kept = value.decision.into_parts();
    kept.decision = decision;
    value.decision = w::OwnerEscalationDecisionRecord::new(&limits, kept).expect("literal decision");
    parts.value = w::OwnerHistoricalValue::Escalation(
        w::OwnerHistoricalValueEscalation::new(&limits, value).expect("literal escalation"),
    );
    w::ReadValue::Historical(Box::new(w::OwnerHistorical::new(&limits, parts).expect("literal historical")))
}

#[test]
fn a_person_answer_notice_encodes_its_question_number() {
    let limits = fixtures::wire_limits();
    for question in [0, 1, u64::MAX] {
        let value = jig_domain_tasks::MessageKind::QuestionAnswered { question };
        let encoded = translate::objects::question_answer::message_kind(&value, &limits).expect("notice");
        let expected = w::OwnerMessageKind::QuestionAnswered(
            w::OwnerMessageKindAnswer::new(&limits, w::OwnerMessageKindAnswerParts { question }).expect("number"),
        );
        assert_eq!(encoded, expected);
    }
}
