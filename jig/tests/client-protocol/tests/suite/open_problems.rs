//! The client part of P77 (jig/docs/design/domain/testing.md, 7.3).
use jig_client::{Answered, Ask, Asker, Handle, Key, Outcome, Readable, Readout, contract as c};
use jig_client_protocol_world::{client::Story, fixtures};
use skein_lib::{Duration, List};

const SEED: u64 = 778_510;
const WAIT: Duration = Duration::from_secs(1);
const TASK: u64 = 1;
const PROPOSAL: Handle = Handle { asker: Asker::JIG, number: 90 };

fn send(story: &mut Story, number: u64, ask: c::Ask) -> c::DocumentOutcome {
    let key = Key { asker: Asker::JIG, number };
    story.send(key, Ask::Jig(Box::new(ask)));
    assert!(story.run_until(|s| !s.answered(key).is_empty(), WAIT), "request {number} answered");
    let answers = story.answered(key);
    let [Outcome::Answered(Answered::Jig(c::Answer::Done(c::Outcome::Document(answer))))] = answers.as_slice() else {
        panic!("request {number} applies: {answers:?}")
    };
    answer.clone()
}

/// P77's repaired client part: a member's widening proposal is read through
/// `Read::Proposal` and accepted through `Ask::Decide` by an owner who covers it.
/// The amendment must apply; a broken proposal translation or decision refuses
/// the request or leaves the old budget instead of the accepted widening.
#[test]
fn an_owners_widening_is_never_proposed_to_itself() {
    let mut story = Story::live_amendment_proposal(SEED);
    let member = story.client.link().party.expect("member signed in");
    assert_ne!(member, fixtures::FIRST_OWNER);
    let mut task = fixtures::create_tasks().get(0).expect("first fixture task").clone();
    task.budget = c::NewBudget::Named;
    task.authority.spend = 20;
    task.tracked = Some(1);
    task.label = 2;
    let mut authority = task.authority.clone();
    let mut tasks = List::with_capacity(1);
    tasks.push(task).expect("one goal");
    assert_eq!(
        send(&mut story, 81, c::Ask::Create { project: 1, tasks }),
        c::DocumentOutcome::Created { first: TASK, last: TASK, given: None }
    );
    authority.spend = 150;
    let change = c::Amendment {
        addressed: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: Some(authority),
        instructions: None,
        tracked: None,
        llm: None,
    };
    let proposed = send(
        &mut story,
        82,
        c::Ask::Propose {
            project: 1,
            action: c::ProposeAction::Amendment {
                task: TASK,
                revision: c::Revision { value: 0 },
                change: Box::new(change),
            },
            reason: c::Text::new(b"please widen".as_slice().into(), 32).expect("bounded reason"),
        },
    );
    let c::DocumentOutcome::Proposed { proposal, to: 0 } = proposed else {
        panic!("proposed to the owner: {proposed:?}")
    };
    assert_eq!(story.task_budget(TASK), Some(20), "proposing alone grants no authority");
    story.sign_in_as(fixtures::identity(fixtures::FIRST_OWNER));
    let key = c::ProposalKey { proposer: c::Proposer::Party(member), number: proposal };
    story.read(PROPOSAL, Readable::Jig(c::Read::Proposal { proposal: key }));
    assert!(story.run_until(|s| !s.pages(PROPOSAL).is_empty(), WAIT));
    let pages = story.pages(PROPOSAL);
    let [Readout::Jig(c::ReadResult::Value(c::ReadValue::Proposal(value)))] = pages.as_slice() else {
        panic!("the accepting owner reads the proposal: {pages:?}")
    };
    assert_eq!(value.key, key);
    assert!(matches!(value.action, c::ProposalAction::Amend { .. }), "the proposal carries its amendment");
    send(
        &mut story,
        83,
        c::Ask::Decide {
            project: 1,
            on: c::DecideOn::Proposal { proposal: key, decision: c::Decision::Accepted, running: None },
        },
    );
    assert_eq!(story.task_budget(TASK), Some(150), "accepting the proposal applies the widened budget");
    assert!(matches!(
        story.party_proposal(proposal).expect("retained proposal").state,
        jig_domain_tasks::PersonProposalState::Accepted { .. }
    ));
}
