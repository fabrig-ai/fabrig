use super::*;
fn case_1() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::NoteEdited { name: 8 }),
        owner(w::DocumentOutcome::NoteEdited(
            w::OutcomeNoteEdited::new(&wire(), w::OutcomeNoteEditedParts { name: 8 }).expect("NoteEdited"),
        )),
    )
}
fn case_2() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::Adopted { project: 3 }),
        owner(w::DocumentOutcome::Adopted(
            w::OutcomeAdopted::new(&wire(), w::OutcomeAdoptedParts { project: 3 }).expect("Adopted"),
        )),
    )
}
fn case_5() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::Stopped { task: 8 }),
        owner(w::DocumentOutcome::Stopped(
            w::OutcomeStopped::new(&wire(), w::OutcomeStoppedParts { task: 8 }).expect("Stopped"),
        )),
    )
}
fn case_6() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::Cancelled { task: 8 }),
        owner(w::DocumentOutcome::Cancelled(
            w::OutcomeCancelled::new(&wire(), w::OutcomeCancelledParts { task: 8 }).expect("Cancelled"),
        )),
    )
}
fn case_7() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::Released { task: 8 }),
        owner(w::DocumentOutcome::Released(
            w::OutcomeReleased::new(&wire(), w::OutcomeReleasedParts { task: 8 }).expect("Released"),
        )),
    )
}
fn case_8() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::PersonTaken { task: 8 }),
        owner(w::DocumentOutcome::PersonTaken(
            w::OutcomePersonTaken::new(&wire(), w::OutcomePersonTakenParts { task: 8 }).expect("PersonTaken"),
        )),
    )
}
fn case_9() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::PersonHandedBack { task: 8 }),
        owner(w::DocumentOutcome::PersonHandedBack(
            w::OutcomePersonHandedBack::new(&wire(), w::OutcomePersonHandedBackParts { task: 8 })
                .expect("PersonHandedBack"),
        )),
    )
}
fn case_10() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::PersonAnswered { task: 8 }),
        owner(w::DocumentOutcome::PersonAnswered(
            w::OutcomePersonAnswered::new(&wire(), w::OutcomePersonAnsweredParts { task: 8 }).expect("PersonAnswered"),
        )),
    )
}
fn case_11() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::Moved { task: 8 }),
        owner(w::DocumentOutcome::Moved(
            w::OutcomeMoved::new(&wire(), w::OutcomeMovedParts { task: 8 }).expect("Moved"),
        )),
    )
}
fn case_12() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::Said { task: 8, message: 8 }),
        owner(w::DocumentOutcome::Said(
            w::OutcomeSaid::new(&wire(), w::OutcomeSaidParts { task: 8, message: 8 }).expect("Said"),
        )),
    )
}
fn case_13() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::QuestionAnswered { task: 8, question: 8, message: 8 }),
        owner(w::DocumentOutcome::QuestionAnswered(
            w::OutcomeQuestionAnswered::new(
                &wire(),
                w::OutcomeQuestionAnsweredParts { task: 8, question: 8, message: 8 },
            )
            .expect("QuestionAnswered"),
        )),
    )
}
fn case_15() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::Amended { task: 8, released: true }),
        owner(w::DocumentOutcome::Amended(
            w::OutcomeAmended::new(&wire(), w::OutcomeAmendedParts { task: 8 }).expect("Amended"),
        )),
    )
}
fn case_16() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::RolesSet { project: 3 }),
        owner(w::DocumentOutcome::RolesSet(
            w::OutcomeRolesSet::new(&wire(), w::OutcomeRolesSetParts { project: 3 }).expect("RolesSet"),
        )),
    )
}
fn case_18() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::PolicyChanged { project: 3 }),
        owner(w::DocumentOutcome::PolicyChanged(
            w::OutcomePolicyChanged::new(&wire(), w::OutcomePolicyChangedParts { project: 3 }).expect("PolicyChanged"),
        )),
    )
}
fn case_19() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::PoolSet { project: 3, person: 8 }),
        owner(w::DocumentOutcome::PoolSet(
            w::OutcomePoolSet::new(&wire(), w::OutcomePoolSetParts { project: 3, person: 8 }).expect("PoolSet"),
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
    let limits = wire();
    let proposal = w::ProposalKey::new(
        &limits,
        w::ProposalKeyParts {
            proposer: w::Proposer::Party(
                w::ProposerParty::new(&limits, w::ProposerPartyParts { value: 7 }).expect("party"),
            ),
            number: 9,
        },
    )
    .expect("proposal");
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
                w::OutcomeProposalDecided::new(
                    &limits,
                    w::OutcomeProposalDecidedParts {
                        proposal,
                        by: 11,
                        at: 13,
                        choice: w::ProposalChoice::Passed,
                        repeated: false,
                    },
                )
                .expect("decision"),
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
                w::OutcomeEscalationDecided::new(
                    &limits,
                    w::OutcomeEscalationDecidedParts {
                        task: 8,
                        revision: revision(6),
                        by: 11,
                        at: 13,
                        choice: w::EscalationChoice::Released,
                    },
                )
                .expect("decision"),
            )),
        ),
    ]
}
pub fn seen_outcomes() -> [(p::Reply, w::Final); 2] {
    let limits = wire();
    [
        (
            p::Reply::Outcome(p::Outcome::MarkedSeen { seen: p::Seen::Results { through: 15 } }),
            owner(w::DocumentOutcome::MarkedSeen(
                w::OutcomeMarkedSeen::new(
                    &limits,
                    w::OutcomeMarkedSeenParts {
                        seen: w::SeenThrough::Results(
                            w::SeenResults::new(&limits, w::SeenResultsParts { through: 15 }).expect("results"),
                        ),
                    },
                )
                .expect("seen"),
            )),
        ),
        (
            p::Reply::Outcome(p::Outcome::MarkedSeen { seen: p::Seen::Conversation { task: 8, through: 15 } }),
            owner(w::DocumentOutcome::MarkedSeen(
                w::OutcomeMarkedSeen::new(
                    &limits,
                    w::OutcomeMarkedSeenParts {
                        seen: w::SeenThrough::Conversation(
                            w::SeenConversation::new(&limits, w::SeenConversationParts { task: 8, through: 15 })
                                .expect("replies"),
                        ),
                    },
                )
                .expect("seen"),
            )),
        ),
    ]
}
