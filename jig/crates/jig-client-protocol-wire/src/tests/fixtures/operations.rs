use super::*;
pub fn merged_targets() -> [(&'static str, Ask); 4] {
    let limits = limits();
    let proposal = ProposalKey { proposer: Proposer::Party(ProposerParty { value: u64::MAX }), number: u64::MAX };
    let decide = [
        ("proposal", DecideOn::Proposal(DecideProposal { proposal, decision: Decision::Accepted, running: None })),
        (
            "escalation",
            DecideOn::Escalation(DecideEscalation {
                task: u64::MAX,
                revision: revision(&limits),
                decision: Decision::PassedUp,
            }),
        ),
    ];
    let answer = [
        (
            "question",
            AnswerOn::Question(AnswerQuestion {
                task: u64::MAX,
                message: u64::MAX,
                words: b"answer".as_slice().into(),
            }),
        ),
        (
            "person",
            AnswerOn::Person(AnswerPerson {
                task: u64::MAX,
                revision: revision(&limits),
                answer: PersonAnswer::Words(PersonAnswerWords { value: b"answer".as_slice().into() }),
            }),
        ),
    ];
    let [(proposal_name, proposal), (escalation_name, escalation)] = decide;
    let [(question_name, question), (person_name, person)] = answer;
    [
        (proposal_name, Ask::Decide(Decide { project: u32::MAX, on: proposal })),
        (escalation_name, Ask::Decide(Decide { project: u32::MAX, on: escalation })),
        (question_name, Ask::Answer(Answer { project: u32::MAX, on: question })),
        (person_name, Ask::Answer(Answer { project: u32::MAX, on: person })),
    ]
}
