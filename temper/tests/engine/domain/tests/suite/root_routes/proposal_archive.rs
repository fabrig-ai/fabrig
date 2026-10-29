use super::*;
const DELEGATE: u64 = 81;
const PROPOSE: u64 = 82;
const WITHDRAW: u64 = 83;
const WITHDRAW_AGAIN: u64 = 84;
#[test]
fn a_named_withdrawal_answers_from_the_loaded_decided_proposal() {
    let (mut driver, parent) = batch_fixture_with(3, 2);
    let child = call_batch(&mut driver, &parent, DELEGATE, Box::new([report_delegate(b"child", Box::new([]))]))[0];
    let run = assigned_task(&driver, child);
    tool_call(&mut driver, &run, PROPOSE, propose(b"grandchild", 10, b"need a helper", false));
    let proposal = proposal_of(&driver, PROPOSE);
    tool_call(&mut driver, &run, WITHDRAW, engine::Tool::Withdraw { proposal });
    assert_eq!(
        driver.answered(WITHDRAW),
        Some(&engine::CallAnswer::ProposalDecided { proposal, outcome: tasks::ProposalOutcome::Withdrawn })
    );
    assert!(live(&driver, child).proposal.is_none());
    driver.loads.clear();
    tool_call(&mut driver, &run, WITHDRAW_AGAIN, engine::Tool::Withdraw { proposal });
    assert!(
        driver
            .loads
            .iter()
            .any(|(_, range)| *range == engine::Range::Core(engine::CoreRange::TaskProposal { task: child, proposal }))
    );
    assert_eq!(
        driver.answered(WITHDRAW_AGAIN),
        Some(&engine::CallAnswer::ProposalDecided { proposal, outcome: tasks::ProposalOutcome::Withdrawn })
    );
}
