use super::*;
fn case_1() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::NoteEdited { name: 8 }),
        owner(w::DocumentOutcome::NoteEdited(
            w::OutcomeNoteEdited { name: 8 },
        )),
    )
}
fn case_2() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::Adopted { project: 3 }),
        owner(w::DocumentOutcome::Adopted(
            w::OutcomeAdopted { project: 3 },
        )),
    )
}
fn case_5() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::Stopped { task: 8 }),
        owner(w::DocumentOutcome::Stopped(
            w::OutcomeStopped { task: 8 },
        )),
    )
}
fn case_6() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::Cancelled { task: 8 }),
        owner(w::DocumentOutcome::Cancelled(
            w::OutcomeCancelled { task: 8 },
        )),
    )
}
fn case_7() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::Released { task: 8 }),
        owner(w::DocumentOutcome::Released(
            w::OutcomeReleased { task: 8 },
        )),
    )
}
fn case_8() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::PersonTaken { task: 8 }),
        owner(w::DocumentOutcome::PersonTaken(
            w::OutcomePersonTaken { task: 8 },
        )),
    )
}
fn case_9() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::PersonHandedBack { task: 8 }),
        owner(w::DocumentOutcome::PersonHandedBack(
            w::OutcomePersonHandedBack { task: 8 },
        )),
    )
}
fn case_10() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::PersonAnswered { task: 8 }),
        owner(w::DocumentOutcome::PersonAnswered(
            w::OutcomePersonAnswered { task: 8 },
        )),
    )
}
fn case_11() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::Moved { task: 8 }),
        owner(w::DocumentOutcome::Moved(
            w::OutcomeMoved { task: 8 },
        )),
    )
}
fn case_12() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::Said { task: 8, message: 8 }),
        owner(w::DocumentOutcome::Said(
            w::OutcomeSaid { task: 8, message: 8 },
        )),
    )
}
fn case_13() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::QuestionAnswered { task: 8, question: 8, message: 8 }),
        owner(w::DocumentOutcome::QuestionAnswered(
            w::OutcomeQuestionAnswered { task: 8, question: 8, message: 8 },
        )),
    )
}
fn case_15() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::Amended { task: 8, released: true }),
        owner(w::DocumentOutcome::Amended(
            w::OutcomeAmended { task: 8 },
        )),
    )
}
fn case_16() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::RolesSet { project: 3 }),
        owner(w::DocumentOutcome::RolesSet(
            w::OutcomeRolesSet { project: 3 },
        )),
    )
}
fn case_18() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::PolicyChanged { project: 3 }),
        owner(w::DocumentOutcome::PolicyChanged(
            w::OutcomePolicyChanged { project: 3 },
        )),
    )
}
fn case_19() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::PoolSet { project: 3, person: 8 }),
        owner(w::DocumentOutcome::PoolSet(
            w::OutcomePoolSet { project: 3, person: 8 },
        )),
    )
}
pub fn outcomes() -> [(p::Reply, w::Final); 15] {
    [
        case_1(),
        case_2(),
        case_5(),
        case_6(),
        case_7(),
        case_8(),
        case_9(),
        case_10(),
        case_11(),
        case_12(),
        case_13(),
        case_15(),
        case_16(),
        case_18(),
        case_19(),
    ]
}
pub fn decision_outcomes() -> [(p::Reply, w::Final); 2] {
    let proposal = w::ProposalKey {
            proposer: w::Proposer::Party(
                w::ProposerParty { value: 7 },
            ),
            number: 9,
        };
    [
        (
            p::Reply::Outcome(p::Outcome::ProposalDecided {
                proposer: p::Proposer::Party(7),
                proposal: 9,
                by: 11,
                at: Wall::from_nanos(13),
                choice: p::ProposalChoice::Passed,
                repeated: false,
            }),
            owner(w::DocumentOutcome::ProposalDecided(
                w::OutcomeProposalDecided {
                        proposal,
                        by: 11,
                        at: 13,
                        choice: w::ProposalChoice::Passed,
                        repeated: false,
                    },
            )),
        ),
        (
            p::Reply::Outcome(p::Outcome::EscalationDecided {
                task: 8,
                revision: 6,
                by: 11,
                at: Wall::from_nanos(13),
                choice: p::EscalationChoice::Released,
            }),
            owner(w::DocumentOutcome::EscalationDecided(
                w::OutcomeEscalationDecided {
                        task: 8,
                        revision: revision(6),
                        by: 11,
                        at: 13,
                        choice: w::EscalationChoice::Released,
                    },
            )),
        ),
    ]
}
pub fn seen_outcomes() -> [(p::Reply, w::Final); 2] {
    [
        (
            p::Reply::Outcome(p::Outcome::MarkedSeen { seen: p::Seen::Results { through: 15 } }),
            owner(w::DocumentOutcome::MarkedSeen(
                w::OutcomeMarkedSeen {
                        seen: w::SeenThrough::Results(
                            w::SeenResults { through: 15 },
                        ),
                    },
            )),
        ),
        (
            p::Reply::Outcome(p::Outcome::MarkedSeen { seen: p::Seen::Conversation { task: 8, through: 15 } }),
            owner(w::DocumentOutcome::MarkedSeen(
                w::OutcomeMarkedSeen {
                        seen: w::SeenThrough::Conversation(
                            w::SeenConversation { task: 8, through: 15 },
                        ),
                    },
            )),
        ),
    ]
}
