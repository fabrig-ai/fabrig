use crate::tests::kit::Goldens;
use crate::{
    tree::fragments::outcome as view,
    words::{Words, outcome_words as phrases},
};
use skein_ui::Limits;
use skein_ui::phrases::{Hole, HoleKind, Invalid, Language, PhraseLimits, Phrasebook, Segment};
use skein_ui::{Builder, Tree};

use jig_client::contract::{
    DocumentOutcome, EscalationChoice, ProposalChoice, ProposalKey, Proposer, Revision, SeenThrough,
};
use skein_lib::{Wall, bytes::copy_of};

fn phrase_limits() -> PhraseLimits {
    PhraseLimits {
        languages: 2,
        phrases: u32::try_from(phrases::ALL.len()).unwrap(),
        holes: 1,
        segments: 200,
        text_bytes: 8192,
        hole_bytes: 0,
    }
}
fn words() -> Words {
    Words::new(Phrasebook::load(phrases::shapes(), Box::new([phrases::english()]), &phrase_limits()).unwrap(), 0, 0)
}
fn render(value: &DocumentOutcome, words: &Words) -> Tree {
    let required = crate::sizes::limits::of(
        &crate::tests::tree::cards::CLIENT,
        &(crate::tests::tree::cards::CLIENT).client,
        &crate::state::page::Arrangement::none(),
    )
    .expect("closed fragment fits");
    let limits = Limits {
        nodes: required.nodes,
        depth: required.depth,
        text_per_node: required.text_per_node,
        text_bytes: required.text_bytes,
        drawing_bytes: 0,
        styles_per_node: 0,
        patches: 0,
        bindings: 0,
        slots: 0,
        markdown_depth: 0,
        scene_marks: 0,
        scene_bytes: 0,
    };
    let mut b = Builder::new(&limits);
    view::render(&mut b, value, words);
    let tree = b.finish();
    assert!(tree.len() <= required.nodes);
    let mut bytes = 0_usize;
    for node in tree.nodes() {
        if let Some(name) = &node.name {
            bytes = bytes.checked_add(name.len()).unwrap();
        }
        for run in &node.text {
            bytes = bytes.checked_add(run.text.len()).unwrap();
        }
    }
    assert!(bytes <= usize::try_from(required.text_bytes).unwrap());
    tree
}

fn max_cases() -> [DocumentOutcome; 23] {
    [
        created_case(),
        DocumentOutcome::Withdrawn { proposal: u64::MAX },
        DocumentOutcome::MarkedSeen { seen: SeenThrough::Results { through: u64::MAX } },
        DocumentOutcome::MarkedSeen { seen: SeenThrough::Conversation { task: u64::MAX, through: u64::MAX } },
        DocumentOutcome::NoteEdited { name: u64::MAX },
        DocumentOutcome::ServiceMade { person: u64::MAX },
        DocumentOutcome::Adopted { project: u32::MAX },
        DocumentOutcome::Stopped { task: u64::MAX },
        DocumentOutcome::Cancelled { task: u64::MAX },
        DocumentOutcome::Released { task: u64::MAX },
        DocumentOutcome::PersonTaken { task: u64::MAX },
        DocumentOutcome::PersonHandedBack { task: u64::MAX },
        DocumentOutcome::PersonAnswered { task: u64::MAX },
        DocumentOutcome::Moved { task: u64::MAX },
        DocumentOutcome::ProposalDecided {
            proposal: ProposalKey { proposer: Proposer::Task(u64::MAX), number: u64::MAX },
            by: u64::MAX,
            at: Wall::from_nanos(u64::MAX),
            choice: ProposalChoice::Accepted,
            repeated: false,
        },
        DocumentOutcome::Said { task: u64::MAX, message: u64::MAX },
        DocumentOutcome::QuestionAnswered { task: u64::MAX, question: u64::MAX, message: u64::MAX },
        DocumentOutcome::Amended { task: u64::MAX },
        DocumentOutcome::Proposed { proposal: u64::MAX, to: u32::MAX },
        DocumentOutcome::RolesSet { project: u32::MAX },
        DocumentOutcome::PolicyChanged { project: u32::MAX },
        DocumentOutcome::PoolSet { project: u32::MAX, person: u64::MAX },
        DocumentOutcome::EscalationDecided {
            task: u64::MAX,
            revision: Revision { value: u64::MAX },
            by: u64::MAX,
            at: Wall::from_nanos(u64::MAX),
            choice: EscalationChoice::Released,
        },
    ]
}
#[test]
fn all_committed_owner_outcomes_render_supplied_maximum_numbers() {
    let mut goldens = Goldens::new("tree/fragments/outcome/all_committed_owner");
    let words = words();
    for value in max_cases() {
        let tree = render(&value, &words);
        goldens.check(&tree);
    }
}
#[test]
fn proposal_decisions_keep_both_makers_and_every_closed_choice_without_explanation() {
    let mut goldens = Goldens::new("tree/fragments/outcome/proposal_decisions_keep");
    let words = words();
    for proposer in [Proposer::Task(9), Proposer::Party(9)] {
        for choice in [
            ProposalChoice::Accepted,
            ProposalChoice::Rejected,
            ProposalChoice::Passed,
            ProposalChoice::Withdrawn,
            ProposalChoice::Stale,
        ] {
            let value = DocumentOutcome::ProposalDecided {
                proposal: ProposalKey { proposer, number: 17 },
                by: 23,
                at: Wall::from_nanos(31),
                choice,
                repeated: false,
            };
            let tree = render(&value, &words);

            goldens.check(&tree);
            assert_eq!(tree.len(), 8, "all proposal facts remain visible");
        }
    }
}
#[test]
fn escalation_decisions_keep_revision_winner_clock_and_every_closed_choice() {
    let mut goldens = Goldens::new("tree/fragments/outcome/escalation_decisions_keep");
    let words = words();
    for choice in [EscalationChoice::Released, EscalationChoice::Rejected, EscalationChoice::Passed] {
        let value = DocumentOutcome::EscalationDecided {
            task: 7,
            revision: Revision { value: 13 },
            by: 19,
            at: Wall::from_nanos(29),
            choice,
        };
        let tree = render(&value, &words);
        goldens.check(&tree);
    }
}
#[test]
fn released_flags_and_question_message_identities_remain_independent() {
    let mut goldens = Goldens::new("tree/fragments/outcome/released_flags_and");
    let words = words();
    {
        let cases: [DocumentOutcome; 3] = [
            DocumentOutcome::Said { task: 7, message: 11 },
            DocumentOutcome::Amended { task: 13 },
            DocumentOutcome::Proposed { proposal: 23, to: 0 },
        ];
        for value in cases {
            goldens.check(&render(&value, &words));
        }
    }
    let question = render(&DocumentOutcome::QuestionAnswered { task: 7, question: 11, message: 19 }, &words);
    goldens.check(&question);
    let seen =
        render(&DocumentOutcome::MarkedSeen { seen: SeenThrough::Conversation { task: 13, through: 17 } }, &words);
    goldens.check(&seen);
}
#[test]
fn pool_person_remains_distinct_from_the_project() {
    let mut goldens = Goldens::new("tree/fragments/outcome/pool_person_remains");
    let words = words();
    let pool = render(&DocumentOutcome::PoolSet { project: 13, person: 23 }, &words);
    goldens.check(&pool);
}
fn selected_language() -> Language {
    let mut language = phrases::english();
    for phrase in &mut language.phrases {
        for form in &mut phrase.forms {
            for segment in &mut form.segments {
                match segment {
                    Segment::Text(bytes) => *bytes = copy_of(b"selected committed outcome language: "),
                    Segment::Hole { .. } => {}
                }
            }
        }
    }
    language
}
#[test]
fn selected_outcome_language_is_checked_and_maximum_fragment_fits() {
    let mut goldens = Goldens::new("tree/fragments/outcome/selected_outcome_language");
    let book =
        Phrasebook::load(phrases::shapes(), Box::new([phrases::english(), selected_language()]), &phrase_limits())
            .unwrap();
    let words = Words::new(book, 1, 1);
    let value = DocumentOutcome::ProposalDecided {
        proposal: ProposalKey { proposer: Proposer::Party(u64::MAX), number: u64::MAX },
        by: u64::MAX,
        at: Wall::EPOCH,
        choice: ProposalChoice::Rejected,
        repeated: false,
    };
    let tree = render(&value, &words);

    goldens.check(&tree);
    assert_eq!(words.book().say(2, phrases::OutcomePhrase::Rejected.id(), &[]).unwrap_err(), Invalid::Language);
    assert_eq!(
        words.book().say(1, phrases::OutcomePhrase::At.id(), &[Hole::Number(7)]).unwrap_err(),
        Invalid::Values { id: phrases::OutcomePhrase::At.id() }
    );
    let mut missing = phrases::english();
    missing.phrases = Box::new([]);
    assert_eq!(
        Phrasebook::load(phrases::shapes(), Box::new([missing]), &phrase_limits()).unwrap_err(),
        Invalid::Missing { id: phrases::OutcomePhrase::MarkedSeen.id() }
    );
    let mut wrong = phrases::english();
    for phrase in &mut wrong.phrases {
        if phrase.id == phrases::OutcomePhrase::Revision.id() {
            phrase.forms[0].segments[1] = Segment::Hole { index: 0, kind: HoleKind::Wall };
        }
    }
    assert_eq!(
        Phrasebook::load(phrases::shapes(), Box::new([wrong]), &phrase_limits()).unwrap_err(),
        Invalid::Holes { id: phrases::OutcomePhrase::Revision.id() }
    );
}

#[test]
fn normal_words_register_every_outcome_once_and_bound_the_selected_language() {
    use crate::words::phrases::{english, jig};
    use skein_lib::List;
    let mut goldens = Goldens::new("tree/fragments/outcome/normal_words_register");

    let normal_shapes = jig::shapes(crate::tests::tree::cards::CLIENT.client.text);
    assert_eq!(normal_shapes.len(), usize::try_from(crate::words::document_words::phrase_count()).unwrap());
    for outcome in phrases::ALL {
        let mut copies = 0_u32;
        for shape in &normal_shapes {
            if shape.id == outcome.id() {
                copies = copies.checked_add(1).unwrap();
            }
        }
        assert_eq!(copies, 1, "each owner outcome shape is registered once");
    }
    let limits = PhraseLimits {
        languages: 2,
        phrases: crate::words::document_words::phrase_count(),
        holes: 4,
        segments: crate::tests::english_phrase_segments(2),
        text_bytes: 200_000,
        hole_bytes: crate::tests::tree::cards::CLIENT.client.text,
    };
    let mut selected = english::language();
    for phrase in &mut selected.phrases {
        if phrase.id == phrases::OutcomePhrase::Rejected.id() {
            phrase.forms[0].segments[0] = Segment::Text(Box::new([b'R'; 2048]));
        }
    }
    let book = Phrasebook::load(normal_shapes, Box::new([english::language(), selected]), &limits).unwrap();
    for outcome in phrases::ALL {
        assert!(book.capacity(outcome.id()).is_some());
    }
    let words = Words::new(book, 1, 1);
    let fragment = crate::sizes::limits::of(
        &crate::tests::tree::cards::CLIENT,
        &(crate::tests::tree::cards::CLIENT).client,
        &crate::state::page::Arrangement::none(),
    )
    .unwrap();
    assert!(fragment.text_per_node >= 2048);
    // This positive language proof needs one row, not every full page window.
    let client = crate::tests::fixtures::Profile {
        view: crate::Config { ..(crate::tests::tree::cards::CLIENT).view },
        client: jig_client::DomainLimits { rows: 1, ..(crate::tests::tree::cards::CLIENT).client },
    };
    assert!(crate::tests::render_limits(&client, &words).unwrap().text_per_node >= 2048);
    let value = DocumentOutcome::ProposalDecided {
        proposal: ProposalKey { proposer: Proposer::Party(u64::MAX), number: 17 },
        by: 23,
        at: Wall::EPOCH,
        choice: ProposalChoice::Rejected,
        repeated: false,
    };
    let tree = render(&value, &words);
    assert_eq!(tree.len(), 8);
    goldens.check(&tree);

    let mut incomplete = english::language();
    let mut kept = List::with_capacity(crate::words::document_words::phrase_count());
    for phrase in incomplete.phrases {
        if phrase.id != phrases::OutcomePhrase::Compound.id() {
            kept.push(phrase).unwrap();
        }
    }
    incomplete.phrases = kept.into_boxed();
    assert_eq!(
        Phrasebook::load(jig::shapes(crate::tests::tree::cards::CLIENT.client.text), Box::new([incomplete]), &limits)
            .unwrap_err(),
        Invalid::Missing { id: phrases::OutcomePhrase::Compound.id() }
    );
}

#[test]
fn durable_answer_rendering_keeps_every_owner_fact_within_the_existing_notice_envelope() {
    let mut goldens = Goldens::new("tree/fragments/outcome/durable_answer_rendering");
    let words = words();
    let required = crate::sizes::limits::of(
        &crate::tests::tree::cards::CLIENT,
        &(crate::tests::tree::cards::CLIENT).client,
        &crate::state::page::Arrangement::none(),
    )
    .unwrap();
    let limits = Limits {
        nodes: required.nodes,
        depth: required.depth,
        text_per_node: required.text_per_node,
        text_bytes: required.text_bytes,
        drawing_bytes: 0,
        styles_per_node: 0,
        patches: 0,
        bindings: 0,
        slots: 0,
        markdown_depth: 0,
        scene_marks: 0,
        scene_bytes: 0,
    };
    for value in max_cases() {
        let mut builder = Builder::new(&limits);
        words.outcome(&mut builder, &jig_client::contract::Outcome::Document(value));
        let tree = builder.finish();
        goldens.check(&tree);
        assert!(tree.len() <= required.nodes);
    }
    // The enclosing notice includes the largest whole per-host outcome.
}

fn created_case() -> DocumentOutcome {
    DocumentOutcome::Created {
        first: 1,
        last: u64::MAX,
        given: Some(jig_client::contract::CreatedGiven { task: 2, amount: u64::MAX }),
    }
}

#[test]
fn the_account_sign_in_outcomes_say_their_phrases() {
    let mut goldens = Goldens::new("tree/fragments/outcome/the_account_sign");
    let words = words();
    goldens.check(&render(
        &DocumentOutcome::AccountSignInStarted { source: 7, address: b"http://issuer/visit".as_slice().into() },
        &words,
    ));
    for confirmed in [false, true] {
        goldens.check(&render(&DocumentOutcome::AccountSignedOut { confirmed, failed_hosts: Box::new([]) }, &words));
    }
}

#[test]
fn a_program_outcome_shows_its_declared_name() {
    let mut goldens = Goldens::new("tree/fragments/outcome/a_program_outcome");
    let tree =
        render(&DocumentOutcome::AccountProgramDeclared { source: 0, name: b"personal".as_slice().into() }, &words());
    goldens.check(&tree);
}
#[test]
fn a_sign_out_outcome_shows_every_whole_host_name() {
    use jig_client::contract::{AccountHostFailure, AccountHostWhy};

    let mut goldens = Goldens::new("tree/fragments/outcome/a_sign_out");

    let mut rows = skein_lib::List::with_capacity(128);
    for _ in 0_u32..128 {
        rows.push(AccountHostFailure { name: Box::from([b'a'; 4096]), why: AccountHostWhy::NotInContact }).unwrap();
    }
    let rows = rows.into_boxed();
    let tree = render(&DocumentOutcome::AccountSignedOut { confirmed: false, failed_hosts: rows }, &words());

    goldens.check(&tree);
}

#[test]
fn project_outcomes_say_what_was_made_or_renamed_and_keep_the_number() {
    let mut goldens = Goldens::new("tree/fragments/outcome/project_outcomes_say");
    let words = words();
    goldens.check(&render(&DocumentOutcome::Renamed { project: 17 }, &words));
    goldens.check(&render(&DocumentOutcome::ProjectMade { project: 19 }, &words));
}

#[test]
fn moved_in_outcomes_show_whether_the_account_was_renewed() {
    let mut goldens = Goldens::new("tree/fragments/outcome/moved_in_outcomes");
    let words = words();
    for renewed in [false, true] {
        let tree = render(&DocumentOutcome::AccountMovedIn { account: u32::MAX, renewed }, &words);
        goldens.check(&tree);
    }
}

#[test]
fn the_removed_outcome_has_its_words() {
    let mut goldens = Goldens::new("tree/fragments/outcome/the_removed_outcome");
    let words = words();
    let outcome = DocumentOutcome::ProjectRemoved { project: 17 };
    goldens.check(&render(&outcome, &words));
    assert_eq!(
        crate::state::outcome::route(&jig_client::contract::Outcome::Document(outcome)),
        crate::state::outcome::Route::Other
    );
}
